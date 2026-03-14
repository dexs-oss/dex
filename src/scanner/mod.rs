pub mod entry_points;
pub mod languages;
pub mod manifests;
pub mod structure;

use crate::models::context::{Context, Project, Status, Structure, Workspace};
use crate::models::paths::Paths;
use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Result of scanning a project.
pub struct ScanResult {
    pub context: Context,
    pub paths: Paths,
}

/// Scan a project directory and produce context + paths.
pub fn scan(root: &Path) -> Result<ScanResult> {
    if !root.exists() {
        anyhow::bail!("Directory not found: {}", root.display());
    }

    // 1. Walk the file tree
    let (files, dirs) = walk_tree(root)?;
    let file_refs: Vec<&Path> = files.iter().map(|p| p.as_path()).collect();

    // 2. Detect languages
    let detected_languages = languages::detect_languages(&file_refs);

    // 3. Parse manifests
    let manifest_data = parse_manifests(root)?;

    // 4. Determine project name
    let project_name = manifest_data
        .iter()
        .find_map(|m| m.name.clone())
        .unwrap_or_else(|| {
            root.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".into())
        });

    // 5. Collect frameworks and build systems
    let mut frameworks: Vec<String> = manifest_data
        .iter()
        .flat_map(|m| m.frameworks.clone())
        .collect();
    frameworks.sort();
    frameworks.dedup();

    let mut build_systems: Vec<String> = manifest_data
        .iter()
        .map(|m| m.build_system.clone())
        .collect();
    build_systems.sort();
    build_systems.dedup();

    // 6. Detect structure
    let dir_names: Vec<&str> = dirs
        .iter()
        .filter_map(|d| d.file_name().and_then(|n| n.to_str()))
        .collect();
    let has_workspaces = manifest_data.iter().any(|m| !m.workspaces.is_empty());
    let proj_structure = structure::detect_structure(&dir_names, has_workspaces);

    // 7. Determine project type
    let project_type = detect_project_type(&manifest_data, has_workspaces, &file_refs);

    // 8. Collect workspace info
    let workspaces: Option<Vec<Workspace>> = if has_workspaces {
        let ws: Vec<Workspace> = manifest_data
            .iter()
            .flat_map(|m| {
                m.workspaces.iter().map(|w| Workspace {
                    name: w.name.clone(),
                    path: format!("{}/", w.path),
                    workspace_type: "unknown".into(),
                })
            })
            .collect();
        if ws.is_empty() { None } else { Some(ws) }
    } else {
        None
    };

    // 9. Detect entry points and public APIs
    let entry_points = entry_points::detect_entry_points(&file_refs);
    let public_api = entry_points::detect_public_api(&file_refs);

    // 10. Build result
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let context = Context {
        project: Project {
            name: project_name,
            project_type,
            languages: detected_languages,
            frameworks,
            build_systems,
            description: None,
            repository: None,
        },
        structure: Structure {
            style: proj_structure.style,
            source_roots: proj_structure.source_roots,
            test_roots: proj_structure.test_roots,
            config_root: proj_structure.config_root,
            workspaces,
        },
        status: Status {
            schema_version: 1,
            dex_version: env!("CARGO_PKG_VERSION").to_string(),
            last_sync: now,
        },
    };

    let paths = Paths {
        entry_points,
        critical_paths: vec![],
        public_api,
    };

    Ok(ScanResult { context, paths })
}

/// Walk the file tree and return (files, top-level directories).
fn walk_tree(root: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut files = Vec::new();
    let mut top_dirs = Vec::new();

    for entry in WalkBuilder::new(root).hidden(true).build() {
        let entry = entry?;
        let path = entry.path();

        if path == root {
            continue;
        }

        let relative = path.strip_prefix(root).unwrap_or(path);

        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            if relative.components().count() == 1 {
                top_dirs.push(relative.to_path_buf());
            }
        } else {
            files.push(relative.to_path_buf());
        }
    }

    Ok((files, top_dirs))
}

/// Parse all manifest files found in the project root.
fn parse_manifests(root: &Path) -> Result<Vec<manifests::ManifestData>> {
    let mut results = Vec::new();

    let cargo_toml = root.join("Cargo.toml");
    if cargo_toml.exists() {
        results.push(manifests::parse_cargo_toml(&cargo_toml)?);
    }

    let package_json = root.join("package.json");
    if package_json.exists() {
        results.push(manifests::parse_package_json(&package_json)?);
    }

    let go_mod = root.join("go.mod");
    if go_mod.exists() {
        results.push(manifests::parse_go_mod(&go_mod)?);
    }

    let pyproject = root.join("pyproject.toml");
    if pyproject.exists() {
        results.push(manifests::parse_pyproject_toml(&pyproject)?);
    }

    Ok(results)
}

/// Detect project type from manifest data and file list.
fn detect_project_type(
    manifests: &[manifests::ManifestData],
    has_workspaces: bool,
    files: &[&Path],
) -> String {
    if has_workspaces {
        return "monorepo".into();
    }

    let all_frameworks: Vec<&str> = manifests
        .iter()
        .flat_map(|m| m.frameworks.iter().map(|f| f.as_str()))
        .collect();

    let web_frameworks = [
        "axum",
        "actix-web",
        "rocket",
        "warp",
        "express",
        "fastify",
        "koa",
        "hono",
        "next",
        "nuxt",
        "remix",
        "gin",
        "echo",
        "fiber",
        "django",
        "flask",
        "fastapi",
        "react",
        "vue",
        "svelte",
        "angular",
    ];

    if all_frameworks.iter().any(|f| web_frameworks.contains(f)) {
        return "web-service".into();
    }

    let has_bin = manifests.iter().any(|m| m.has_bin);
    let has_clap = all_frameworks.contains(&"clap");
    let has_main_file = files.iter().any(|f| {
        let s = f.to_string_lossy();
        s == "src/main.rs" || s == "main.go" || s == "main.py" || s == "__main__.py"
    });
    if has_bin || has_clap || has_main_file {
        return "cli".into();
    }

    let has_lib = manifests.iter().any(|m| m.has_lib);
    if has_lib {
        return "library".into();
    }

    "library".into()
}
