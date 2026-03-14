use anyhow::Result;
use std::path::Path;

/// Data extracted from a project manifest file.
#[derive(Debug, Default)]
pub struct ManifestData {
    pub name: Option<String>,
    pub frameworks: Vec<String>,
    pub build_system: String,
    pub workspaces: Vec<WorkspaceMember>,
    pub has_bin: bool,
    pub has_lib: bool,
    pub bin_targets: Vec<String>,
}

#[derive(Debug)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: String,
}

/// Parse Cargo.toml
pub fn parse_cargo_toml(path: &Path) -> Result<ManifestData> {
    let content = std::fs::read_to_string(path)?;
    let doc: toml::Value = toml::from_str(&content)?;

    let mut data = ManifestData {
        build_system: "cargo".into(),
        ..Default::default()
    };

    // Project name
    if let Some(name) = doc
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
    {
        data.name = Some(name.to_string());
    }

    // Check for [[bin]] targets
    if let Some(bins) = doc.get("bin").and_then(|b| b.as_array()) {
        data.has_bin = true;
        for bin in bins {
            if let Some(name) = bin.get("name").and_then(|n| n.as_str()) {
                data.bin_targets.push(name.to_string());
            }
        }
    }

    // Check for [lib]
    if doc.get("lib").is_some() {
        data.has_lib = true;
    }

    // Detect frameworks from dependencies
    if let Some(toml::Value::Table(deps)) = doc.get("dependencies") {
        for dep in deps.keys() {
            match dep.as_str() {
                "axum" | "actix-web" | "rocket" | "warp" => {
                    data.frameworks.push(dep.to_string());
                }
                "tokio" | "async-std" => {
                    data.frameworks.push(dep.to_string());
                }
                "clap" => {
                    data.frameworks.push("clap".into());
                }
                _ => {}
            }
        }
    }

    // Workspace detection
    if let Some(workspace) = doc.get("workspace") {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            for member in members {
                if let Some(member_path) = member.as_str() {
                    data.workspaces.push(WorkspaceMember {
                        name: member_path
                            .split('/')
                            .last()
                            .unwrap_or(member_path)
                            .to_string(),
                        path: member_path.to_string(),
                    });
                }
            }
        }
    }

    Ok(data)
}

/// Parse package.json
pub fn parse_package_json(path: &Path) -> Result<ManifestData> {
    let content = std::fs::read_to_string(path)?;
    let doc: serde_json::Value = serde_json::from_str(&content)?;

    let mut data = ManifestData {
        build_system: "npm".into(),
        ..Default::default()
    };

    if let Some(name) = doc.get("name").and_then(|n| n.as_str()) {
        data.name = Some(name.to_string());
    }

    // Detect frameworks from dependencies + devDependencies
    for key in &["dependencies", "devDependencies"] {
        if let Some(deps) = doc.get(key).and_then(|d| d.as_object()) {
            for dep in deps.keys() {
                match dep.as_str() {
                    "react" | "vue" | "svelte" | "angular" | "@angular/core" => {
                        data.frameworks.push(dep.clone());
                    }
                    "next" | "nuxt" | "remix" | "astro" | "vite" => {
                        data.frameworks.push(dep.clone());
                    }
                    "express" | "fastify" | "koa" | "hono" => {
                        data.frameworks.push(dep.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    // Workspace detection
    if let Some(workspaces) = doc.get("workspaces").and_then(|w| w.as_array()) {
        for ws in workspaces {
            if let Some(ws_path) = ws.as_str() {
                data.workspaces.push(WorkspaceMember {
                    name: ws_path.split('/').last().unwrap_or(ws_path).to_string(),
                    path: ws_path.to_string(),
                });
            }
        }
    }

    if doc.get("bin").is_some() {
        data.has_bin = true;
    }
    if doc.get("main").is_some() || doc.get("module").is_some() {
        data.has_lib = true;
    }

    Ok(data)
}

/// Parse go.mod
pub fn parse_go_mod(path: &Path) -> Result<ManifestData> {
    let content = std::fs::read_to_string(path)?;
    let mut data = ManifestData {
        build_system: "go".into(),
        ..Default::default()
    };

    for line in content.lines() {
        let line = line.trim();
        if let Some(module) = line.strip_prefix("module ") {
            data.name = Some(module.trim().to_string());
            break;
        }
    }

    let frameworks_to_detect = [
        "github.com/gin-gonic/gin",
        "github.com/labstack/echo",
        "github.com/gofiber/fiber",
        "github.com/gorilla/mux",
    ];
    for fw in &frameworks_to_detect {
        if content.contains(fw) {
            let short = fw.split('/').last().unwrap_or(fw);
            data.frameworks.push(short.to_string());
        }
    }

    Ok(data)
}

/// Parse pyproject.toml
pub fn parse_pyproject_toml(path: &Path) -> Result<ManifestData> {
    let content = std::fs::read_to_string(path)?;
    let doc: toml::Value = toml::from_str(&content)?;

    let mut data = ManifestData {
        build_system: "python".into(),
        ..Default::default()
    };

    if let Some(project) = doc.get("project") {
        if let Some(name) = project.get("name").and_then(|n| n.as_str()) {
            data.name = Some(name.to_string());
        }
        if let Some(deps) = project.get("dependencies").and_then(|d| d.as_array()) {
            detect_python_frameworks(deps, &mut data.frameworks);
        }
    } else if let Some(poetry) = doc.get("tool").and_then(|t| t.get("poetry")) {
        if let Some(name) = poetry.get("name").and_then(|n| n.as_str()) {
            data.name = Some(name.to_string());
        }
    }

    if doc
        .get("project")
        .and_then(|p| p.get("scripts"))
        .is_some()
    {
        data.has_bin = true;
    }

    Ok(data)
}

fn detect_python_frameworks(deps: &[toml::Value], frameworks: &mut Vec<String>) {
    let fw_names = [
        "django",
        "flask",
        "fastapi",
        "starlette",
        "tornado",
        "aiohttp",
    ];
    for dep in deps {
        if let Some(dep_str) = dep.as_str() {
            let dep_name = dep_str
                .split(&['>', '<', '=', '!', '~', ' ', ';'][..])
                .next()
                .unwrap_or("")
                .to_lowercase();
            if fw_names.contains(&dep_name.as_str()) {
                frameworks.push(dep_name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_file(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("manifest");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (dir, path)
    }

    #[test]
    fn test_parse_cargo_toml_basic() {
        let (_dir, path) = write_temp_file(
            r#"
[package]
name = "my-cli"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = "4"
tokio = "1"
"#,
        );
        let data = parse_cargo_toml(&path).unwrap();
        assert_eq!(data.name, Some("my-cli".into()));
        assert_eq!(data.build_system, "cargo");
        assert!(data.frameworks.contains(&"clap".to_string()));
        assert!(data.frameworks.contains(&"tokio".to_string()));
    }

    #[test]
    fn test_parse_cargo_toml_workspace() {
        let (_dir, path) = write_temp_file(
            r#"
[workspace]
members = ["crates/core", "crates/cli"]
"#,
        );
        let data = parse_cargo_toml(&path).unwrap();
        assert_eq!(data.workspaces.len(), 2);
        assert_eq!(data.workspaces[0].path, "crates/core");
    }

    #[test]
    fn test_parse_go_mod() {
        let (_dir, path) = write_temp_file(
            r#"
module github.com/myorg/myservice

go 1.22

require (
    github.com/gin-gonic/gin v1.9.0
)
"#,
        );
        let data = parse_go_mod(&path).unwrap();
        assert_eq!(
            data.name,
            Some("github.com/myorg/myservice".into())
        );
        assert!(data.frameworks.contains(&"gin".to_string()));
    }

    #[test]
    fn test_parse_pyproject_toml() {
        let (_dir, path) = write_temp_file(
            r#"
[project]
name = "my-api"
dependencies = ["fastapi>=0.100", "uvicorn"]

[project.scripts]
my-api = "my_api:main"
"#,
        );
        let data = parse_pyproject_toml(&path).unwrap();
        assert_eq!(data.name, Some("my-api".into()));
        assert!(data.frameworks.contains(&"fastapi".to_string()));
        assert!(data.has_bin);
    }

    #[test]
    fn test_parse_package_json() {
        let (_dir, path) = write_temp_file(
            r#"{
  "name": "my-app",
  "dependencies": {
    "react": "^18.0.0",
    "next": "^14.0.0"
  }
}"#,
        );
        let data = parse_package_json(&path).unwrap();
        assert_eq!(data.name, Some("my-app".into()));
        assert!(data.frameworks.contains(&"react".to_string()));
        assert!(data.frameworks.contains(&"next".to_string()));
    }
}
