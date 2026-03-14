use crate::models::paths::{EntryPoint, PublicApi};
use std::path::Path;

const ENTRY_POINT_FILES: &[(&str, &str)] = &[
    ("src/main.rs", "Rust binary entry point"),
    ("src/lib.rs", "Rust library root"),
    ("src/bin/", "Rust binary targets"),
    ("main.go", "Go entry point"),
    ("cmd/", "Go command entry points"),
    ("src/index.ts", "TypeScript entry point"),
    ("src/index.js", "JavaScript entry point"),
    ("src/main.ts", "TypeScript entry point"),
    ("src/main.js", "JavaScript entry point"),
    ("src/app.ts", "TypeScript application entry"),
    ("src/app.js", "JavaScript application entry"),
    ("index.ts", "TypeScript entry point"),
    ("index.js", "JavaScript entry point"),
    ("main.py", "Python entry point"),
    ("app.py", "Python application entry"),
    ("src/__main__.py", "Python package entry point"),
    ("__main__.py", "Python package entry point"),
    ("manage.py", "Django management script"),
];

/// Detect entry points from a list of file paths (relative to project root).
pub fn detect_entry_points(files: &[&Path]) -> Vec<EntryPoint> {
    let mut entries = Vec::new();

    for file in files {
        let path_str = file.to_string_lossy();
        for &(pattern, desc) in ENTRY_POINT_FILES {
            if pattern.ends_with('/') {
                if path_str.starts_with(pattern) {
                    let name = file
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".into());
                    entries.push(EntryPoint {
                        name,
                        file: path_str.to_string(),
                        description: Some(desc.to_string()),
                    });
                }
            } else if path_str == pattern {
                let name = file
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".into());
                entries.push(EntryPoint {
                    name,
                    file: path_str.to_string(),
                    description: Some(desc.to_string()),
                });
            }
        }
    }

    entries.sort_by(|a, b| a.file.cmp(&b.file));
    entries.dedup_by(|a, b| a.file == b.file);
    entries
}

/// Detect public API surface from file paths.
pub fn detect_public_api(files: &[&Path]) -> Vec<PublicApi> {
    let mut apis = Vec::new();
    let api_patterns = [
        ("routes", "Route definitions"),
        ("router", "Router configuration"),
        ("handlers", "Request handlers"),
        ("endpoints", "API endpoints"),
    ];

    for file in files {
        let path_str = file.to_string_lossy();
        if path_str.contains("test") {
            continue;
        }
        for &(pattern, desc) in &api_patterns {
            // Match on the file stem or directory name, not arbitrary substrings
            let parts: Vec<&str> = path_str.split('/').collect();
            let matches = parts.iter().any(|part| {
                let stem = part.strip_suffix(".rs")
                    .or_else(|| part.strip_suffix(".ts"))
                    .or_else(|| part.strip_suffix(".js"))
                    .or_else(|| part.strip_suffix(".py"))
                    .or_else(|| part.strip_suffix(".go"))
                    .unwrap_or(part);
                stem == pattern
            });
            if matches {
                apis.push(PublicApi {
                    name: desc.to_string(),
                    definition: path_str.to_string(),
                });
                break; // Only one match per file
            }
        }
    }

    apis.sort_by(|a, b| a.definition.cmp(&b.definition));
    apis.dedup_by(|a, b| a.definition == b.definition);
    apis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust_entry_point() {
        let files = vec![Path::new("src/main.rs"), Path::new("src/lib.rs")];
        let entries = detect_entry_points(&files);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].file, "src/lib.rs");
        assert_eq!(entries[1].file, "src/main.rs");
    }

    #[test]
    fn test_detect_go_cmd_entries() {
        let files = vec![
            Path::new("cmd/server/main.go"),
            Path::new("cmd/worker/main.go"),
        ];
        let entries = detect_entry_points(&files);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_detect_public_api() {
        let files = vec![
            Path::new("src/api/router.rs"),
            Path::new("src/api/handlers/user.rs"),
            Path::new("tests/api_test.rs"),
        ];
        let apis = detect_public_api(&files);
        assert!(apis.iter().all(|a| !a.definition.contains("test")));
        assert!(apis.len() >= 2);
    }

    #[test]
    fn test_no_entry_points() {
        let files = vec![Path::new("README.md"), Path::new("docs/guide.md")];
        let entries = detect_entry_points(&files);
        assert!(entries.is_empty());
    }
}
