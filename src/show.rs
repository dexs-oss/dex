use crate::models::context::Context;
use crate::models::paths::Paths;
use anyhow::{Context as _, Result};
use colored::Colorize;
use std::path::Path;

pub fn show(root: &Path, section: Option<&str>) -> Result<()> {
    let dex_dir = root.join(".dex");
    if !dex_dir.exists() {
        anyhow::bail!(
            "No .dex/ directory found in {}. Run `dex init` first.",
            root.display()
        );
    }

    let context: Context = {
        let content = std::fs::read_to_string(dex_dir.join("context.toml"))
            .context("Failed to read .dex/context.toml")?;
        toml::from_str(&content).context("Failed to parse .dex/context.toml")?
    };

    let paths: Paths = {
        let path = dex_dir.join("paths.toml");
        if path.exists() {
            let content =
                std::fs::read_to_string(&path).context("Failed to read .dex/paths.toml")?;
            toml::from_str(&content).context("Failed to parse .dex/paths.toml")?
        } else {
            Paths::default()
        }
    };

    match section {
        None => {
            show_project(&context);
            println!();
            show_structure(&context);
            println!();
            show_entry_points(&paths);
            show_api(&paths);
        }
        Some("project") => show_project(&context),
        Some("structure") => show_structure(&context),
        Some("entry-points") => show_entry_points(&paths),
        Some("api") => show_api(&paths),
        Some(other) => {
            anyhow::bail!(
                "Unknown section: '{}'. Valid sections: project, structure, entry-points, api",
                other
            );
        }
    }

    Ok(())
}

fn show_project(ctx: &Context) {
    println!("{}", "project".bold().cyan());
    println!("  {} {}", "name:".dimmed(), ctx.project.name);
    println!("  {} {}", "type:".dimmed(), ctx.project.project_type);
    println!(
        "  {} {}",
        "languages:".dimmed(),
        ctx.project.languages.join(", ")
    );
    if !ctx.project.frameworks.is_empty() {
        println!(
            "  {} {}",
            "frameworks:".dimmed(),
            ctx.project.frameworks.join(", ")
        );
    }
    println!(
        "  {} {}",
        "build:".dimmed(),
        ctx.project.build_systems.join(", ")
    );
    println!(
        "  {} v{} (schema {})",
        "dex:".dimmed(),
        ctx.status.dex_version,
        ctx.status.schema_version
    );
}

fn show_structure(ctx: &Context) {
    println!("{}", "structure".bold().cyan());
    println!("  {} {}", "style:".dimmed(), ctx.structure.style);
    if !ctx.structure.source_roots.is_empty() {
        println!(
            "  {} {}",
            "source:".dimmed(),
            ctx.structure.source_roots.join(", ")
        );
    }
    if !ctx.structure.test_roots.is_empty() {
        println!(
            "  {} {}",
            "tests:".dimmed(),
            ctx.structure.test_roots.join(", ")
        );
    }
    if let Some(ref config) = ctx.structure.config_root {
        println!("  {} {}", "config:".dimmed(), config);
    }
    if let Some(ref workspaces) = ctx.structure.workspaces {
        println!(
            "  {} {} workspace(s)",
            "workspaces:".dimmed(),
            workspaces.len()
        );
        for ws in workspaces {
            println!("    {} {}", "-".dimmed(), ws.path);
        }
    }
}

fn show_entry_points(paths: &Paths) {
    if paths.entry_points.is_empty() {
        return;
    }
    println!("{}", "entry points".bold().cyan());
    for ep in &paths.entry_points {
        let desc = ep
            .description
            .as_deref()
            .map(|d| format!(" ({})", d))
            .unwrap_or_default();
        println!("  {} {}{}", "-".dimmed(), ep.file, desc.dimmed());
    }
}

fn show_api(paths: &Paths) {
    if paths.public_api.is_empty() {
        return;
    }
    println!("{}", "public api".bold().cyan());
    for api in &paths.public_api {
        println!("  {} {} — {}", "-".dimmed(), api.definition, api.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::context::*;
    use crate::models::paths::*;

    #[test]
    fn test_show_no_dex_dir() {
        let dir = tempfile::tempdir().unwrap();
        let result = show(dir.path(), None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No .dex/ directory")
        );
    }

    #[test]
    fn test_show_reads_context() {
        let dir = tempfile::tempdir().unwrap();
        let dex_dir = dir.path().join(".dex");
        std::fs::create_dir(&dex_dir).unwrap();

        let ctx = Context {
            project: Project {
                name: "test-proj".into(),
                project_type: "cli".into(),
                languages: vec!["rust".into()],
                frameworks: vec!["clap".into()],
                build_systems: vec!["cargo".into()],
                description: None,
                repository: None,
            },
            structure: Structure {
                style: "modular".into(),
                source_roots: vec!["src/".into()],
                test_roots: vec!["tests/".into()],
                config_root: None,
                workspaces: None,
            },
            status: Status {
                schema_version: 1,
                dex_version: "0.1.0".into(),
                last_sync: "2026-03-14T00:00:00Z".into(),
            },
        };

        let paths = Paths {
            entry_points: vec![EntryPoint {
                name: "main".into(),
                file: "src/main.rs".into(),
                description: Some("Rust binary entry point".into()),
            }],
            critical_paths: vec![],
            public_api: vec![],
        };

        std::fs::write(
            dex_dir.join("context.toml"),
            toml::to_string_pretty(&ctx).unwrap(),
        )
        .unwrap();
        std::fs::write(
            dex_dir.join("paths.toml"),
            toml::to_string_pretty(&paths).unwrap(),
        )
        .unwrap();

        // Should not error
        let result = show(dir.path(), None);
        assert!(result.is_ok());

        // Section filtering should work
        assert!(show(dir.path(), Some("project")).is_ok());
        assert!(show(dir.path(), Some("structure")).is_ok());
        assert!(show(dir.path(), Some("entry-points")).is_ok());
        assert!(show(dir.path(), Some("api")).is_ok());

        // Unknown section should error
        assert!(show(dir.path(), Some("unknown")).is_err());
    }
}
