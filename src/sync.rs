use crate::models::context::Context;
use crate::models::paths::Paths;
use crate::scanner::{self, ScanResult};
use anyhow::{Context as _, Result};
use colored::Colorize;
use std::path::Path;

/// Summary of what changed during a sync.
#[derive(Debug, Default)]
pub struct SyncReport {
    pub changes: Vec<String>,
}

impl SyncReport {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Run sync: re-scan the project and update .dex/ files that changed.
/// Returns a report of what changed.
pub fn sync(root: &Path) -> Result<SyncReport> {
    let dex_dir = root.join(".dex");
    if !dex_dir.exists() {
        anyhow::bail!(
            "No .dex/ directory found in {}. Run `dex init` first.",
            root.display()
        );
    }

    // Read existing .dex/ files
    let old_context: Context = {
        let content = std::fs::read_to_string(dex_dir.join("context.toml"))
            .context("Failed to read .dex/context.toml")?;
        toml::from_str(&content).context("Failed to parse .dex/context.toml")?
    };

    let old_paths: Paths = {
        let path = dex_dir.join("paths.toml");
        if path.exists() {
            let content =
                std::fs::read_to_string(&path).context("Failed to read .dex/paths.toml")?;
            toml::from_str(&content).context("Failed to parse .dex/paths.toml")?
        } else {
            Paths::default()
        }
    };

    // Re-scan the project
    let new_result = scanner::scan(root)?;

    // Diff and collect changes
    let mut report = SyncReport::default();
    diff_context(&old_context, &new_result.context, &mut report);
    diff_paths(&old_paths, &new_result.paths, &mut report);

    // Write updated files only if something changed
    if !report.is_empty() {
        write_updated_files(&dex_dir, &new_result)?;
    } else {
        // Still update last_sync timestamp
        update_timestamp(&dex_dir, &new_result)?;
    }

    Ok(report)
}

/// Compare old and new context, appending human-readable change descriptions.
fn diff_context(old: &Context, new: &Context, report: &mut SyncReport) {
    // Languages
    let added_langs: Vec<&String> = new
        .project
        .languages
        .iter()
        .filter(|l| !old.project.languages.contains(l))
        .collect();
    let removed_langs: Vec<&String> = old
        .project
        .languages
        .iter()
        .filter(|l| !new.project.languages.contains(l))
        .collect();
    for lang in &added_langs {
        report.changes.push(format!("languages: added {}", lang));
    }
    for lang in &removed_langs {
        report.changes.push(format!("languages: removed {}", lang));
    }

    // Frameworks
    let added_fw: Vec<&String> = new
        .project
        .frameworks
        .iter()
        .filter(|f| !old.project.frameworks.contains(f))
        .collect();
    let removed_fw: Vec<&String> = old
        .project
        .frameworks
        .iter()
        .filter(|f| !new.project.frameworks.contains(f))
        .collect();
    for fw in &added_fw {
        report.changes.push(format!("frameworks: added {}", fw));
    }
    for fw in &removed_fw {
        report.changes.push(format!("frameworks: removed {}", fw));
    }

    // Build systems
    let added_bs: Vec<&String> = new
        .project
        .build_systems
        .iter()
        .filter(|b| !old.project.build_systems.contains(b))
        .collect();
    let removed_bs: Vec<&String> = old
        .project
        .build_systems
        .iter()
        .filter(|b| !new.project.build_systems.contains(b))
        .collect();
    for bs in &added_bs {
        report.changes.push(format!("build systems: added {}", bs));
    }
    for bs in &removed_bs {
        report
            .changes
            .push(format!("build systems: removed {}", bs));
    }

    // Project type
    if old.project.project_type != new.project.project_type {
        report.changes.push(format!(
            "project type: {} -> {}",
            old.project.project_type, new.project.project_type
        ));
    }

    // Structure style
    if old.structure.style != new.structure.style {
        report.changes.push(format!(
            "structure style: {} -> {}",
            old.structure.style, new.structure.style
        ));
    }

    // Source roots
    let added_sr: Vec<&String> = new
        .structure
        .source_roots
        .iter()
        .filter(|s| !old.structure.source_roots.contains(s))
        .collect();
    let removed_sr: Vec<&String> = old
        .structure
        .source_roots
        .iter()
        .filter(|s| !new.structure.source_roots.contains(s))
        .collect();
    for sr in &added_sr {
        report.changes.push(format!("source roots: added {}", sr));
    }
    for sr in &removed_sr {
        report.changes.push(format!("source roots: removed {}", sr));
    }

    // Test roots
    let added_tr: Vec<&String> = new
        .structure
        .test_roots
        .iter()
        .filter(|t| !old.structure.test_roots.contains(t))
        .collect();
    let removed_tr: Vec<&String> = old
        .structure
        .test_roots
        .iter()
        .filter(|t| !new.structure.test_roots.contains(t))
        .collect();
    for tr in &added_tr {
        report.changes.push(format!("test roots: added {}", tr));
    }
    for tr in &removed_tr {
        report.changes.push(format!("test roots: removed {}", tr));
    }
}

/// Compare old and new paths, appending human-readable change descriptions.
fn diff_paths(old: &Paths, new: &Paths, report: &mut SyncReport) {
    // Entry points
    let old_ep_files: Vec<&str> = old.entry_points.iter().map(|e| e.file.as_str()).collect();
    let new_ep_files: Vec<&str> = new.entry_points.iter().map(|e| e.file.as_str()).collect();

    for ep in &new.entry_points {
        if !old_ep_files.contains(&ep.file.as_str()) {
            report.changes.push(format!("new entry point: {}", ep.file));
        }
    }
    for ep in &old.entry_points {
        if !new_ep_files.contains(&ep.file.as_str()) {
            report
                .changes
                .push(format!("removed entry point: {}", ep.file));
        }
    }

    // Public API
    let old_api_defs: Vec<&str> = old
        .public_api
        .iter()
        .map(|a| a.definition.as_str())
        .collect();
    let new_api_defs: Vec<&str> = new
        .public_api
        .iter()
        .map(|a| a.definition.as_str())
        .collect();

    for api in &new.public_api {
        if !old_api_defs.contains(&api.definition.as_str()) {
            report
                .changes
                .push(format!("new public API: {}", api.definition));
        }
    }
    for api in &old.public_api {
        if !new_api_defs.contains(&api.definition.as_str()) {
            report
                .changes
                .push(format!("removed public API: {}", api.definition));
        }
    }
}

/// Write all .dex/ files with the new scan result.
fn write_updated_files(dex_dir: &Path, result: &ScanResult) -> Result<()> {
    let context_toml = toml::to_string_pretty(&result.context)?;
    std::fs::write(dex_dir.join("context.toml"), context_toml)?;

    let paths_toml = toml::to_string_pretty(&result.paths)?;
    std::fs::write(dex_dir.join("paths.toml"), paths_toml)?;

    Ok(())
}

/// Update only the timestamp in context.toml (when nothing else changed).
fn update_timestamp(dex_dir: &Path, result: &ScanResult) -> Result<()> {
    let context_toml = toml::to_string_pretty(&result.context)?;
    std::fs::write(dex_dir.join("context.toml"), context_toml)?;
    Ok(())
}

/// Print the sync report to stdout.
pub fn print_report(report: &SyncReport) {
    if report.is_empty() {
        println!("  {} up to date", "status:".dimmed());
    } else {
        println!(
            "  {} {} change(s)",
            "updated:".dimmed(),
            report.changes.len()
        );
        for change in &report.changes {
            println!("    {} {}", "-".dimmed(), change);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::context::*;
    use crate::models::paths::*;
    use crate::output;
    use crate::scanner::ScanResult;

    fn make_scan_result(languages: Vec<&str>, entry_files: Vec<&str>) -> ScanResult {
        ScanResult {
            context: Context {
                project: Project {
                    name: "test".into(),
                    project_type: "cli".into(),
                    languages: languages.into_iter().map(String::from).collect(),
                    frameworks: vec![],
                    build_systems: vec!["cargo".into()],
                    description: None,
                    repository: None,
                },
                structure: Structure {
                    style: "modular".into(),
                    source_roots: vec!["src/".into()],
                    test_roots: vec![],
                    config_root: None,
                    workspaces: None,
                },
                status: Status {
                    schema_version: 1,
                    dex_version: "0.1.0".into(),
                    last_sync: "2026-03-14T00:00:00Z".into(),
                },
            },
            paths: Paths {
                entry_points: entry_files
                    .into_iter()
                    .map(|f| EntryPoint {
                        name: "main".into(),
                        file: f.into(),
                        description: Some("entry point".into()),
                    })
                    .collect(),
                critical_paths: vec![],
                public_api: vec![],
            },
        }
    }

    #[test]
    fn test_sync_no_dex_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = sync(dir.path());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No .dex/ directory")
        );
    }

    #[test]
    fn test_sync_nothing_changed() {
        let dir = tempfile::tempdir().unwrap();
        let result = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        output::write_dex_dir(dir.path(), &result).unwrap();

        // Diff against identical data
        let mut report = SyncReport::default();
        diff_context(&result.context, &result.context, &mut report);
        diff_paths(&result.paths, &result.paths, &mut report);

        assert!(report.is_empty());
        assert_eq!(report.changes.len(), 0);
    }

    #[test]
    fn test_sync_detects_added_language() {
        let old = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        let new = make_scan_result(vec!["rust", "typescript"], vec!["src/main.rs"]);

        let mut report = SyncReport::default();
        diff_context(&old.context, &new.context, &mut report);

        assert!(!report.is_empty());
        assert!(
            report
                .changes
                .iter()
                .any(|c| c == "languages: added typescript")
        );
    }

    #[test]
    fn test_sync_detects_removed_language() {
        let old = make_scan_result(vec!["rust", "python"], vec!["src/main.rs"]);
        let new = make_scan_result(vec!["rust"], vec!["src/main.rs"]);

        let mut report = SyncReport::default();
        diff_context(&old.context, &new.context, &mut report);

        assert!(
            report
                .changes
                .iter()
                .any(|c| c == "languages: removed python")
        );
    }

    #[test]
    fn test_sync_detects_new_entry_point() {
        let old = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        let new = make_scan_result(vec!["rust"], vec!["src/main.rs", "src/api/server.rs"]);

        let mut report = SyncReport::default();
        diff_paths(&old.paths, &new.paths, &mut report);

        assert!(
            report
                .changes
                .iter()
                .any(|c| c == "new entry point: src/api/server.rs")
        );
    }

    #[test]
    fn test_sync_detects_removed_entry_point() {
        let old = make_scan_result(vec!["rust"], vec!["src/main.rs", "src/worker.rs"]);
        let new = make_scan_result(vec!["rust"], vec!["src/main.rs"]);

        let mut report = SyncReport::default();
        diff_paths(&old.paths, &new.paths, &mut report);

        assert!(
            report
                .changes
                .iter()
                .any(|c| c == "removed entry point: src/worker.rs")
        );
    }

    #[test]
    fn test_sync_detects_project_type_change() {
        let mut old = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        let mut new = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        old.context.project.project_type = "cli".into();
        new.context.project.project_type = "web-service".into();

        let mut report = SyncReport::default();
        diff_context(&old.context, &new.context, &mut report);

        assert!(
            report
                .changes
                .iter()
                .any(|c| c == "project type: cli -> web-service")
        );
    }

    #[test]
    fn test_sync_multiple_changes() {
        let old = make_scan_result(vec!["rust"], vec!["src/main.rs"]);
        let new = make_scan_result(
            vec!["rust", "typescript"],
            vec!["src/main.rs", "src/index.ts"],
        );

        let mut report = SyncReport::default();
        diff_context(&old.context, &new.context, &mut report);
        diff_paths(&old.paths, &new.paths, &mut report);

        assert!(report.changes.len() >= 2);
    }
}
