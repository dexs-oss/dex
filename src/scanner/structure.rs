use std::collections::HashSet;

/// Detected project structure.
#[derive(Debug)]
pub struct ProjectStructure {
    pub style: String,
    pub source_roots: Vec<String>,
    pub test_roots: Vec<String>,
    pub config_root: Option<String>,
}

const SOURCE_PATTERNS: &[&str] = &["src", "lib", "app", "pkg", "cmd", "internal"];
const TEST_PATTERNS: &[&str] = &["tests", "test", "__tests__", "spec", "specs"];
const CONFIG_PATTERNS: &[&str] = &["config", "conf", ".config", "cfg"];

/// Detect project structure from a list of top-level directory names.
pub fn detect_structure(dirs: &[&str], has_workspaces: bool) -> ProjectStructure {
    let dir_set: HashSet<&str> = dirs.iter().copied().collect();

    let source_roots: Vec<String> = SOURCE_PATTERNS
        .iter()
        .filter(|p| dir_set.contains(*p))
        .map(|p| format!("{}/", p))
        .collect();

    let test_roots: Vec<String> = TEST_PATTERNS
        .iter()
        .filter(|p| dir_set.contains(*p))
        .map(|p| format!("{}/", p))
        .collect();

    let config_root = CONFIG_PATTERNS
        .iter()
        .find(|p| dir_set.contains(*p))
        .map(|p| format!("{}/", p));

    let style = if has_workspaces {
        "monorepo".to_string()
    } else if source_roots.is_empty() {
        "flat".to_string()
    } else {
        "modular".to_string()
    };

    ProjectStructure {
        style,
        source_roots,
        test_roots,
        config_root,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_rust_layout() {
        let dirs = vec!["src", "tests", "docs"];
        let s = detect_structure(&dirs, false);
        assert_eq!(s.style, "modular");
        assert_eq!(s.source_roots, vec!["src/"]);
        assert_eq!(s.test_roots, vec!["tests/"]);
    }

    #[test]
    fn test_monorepo() {
        let dirs = vec!["crates", "src"];
        let s = detect_structure(&dirs, true);
        assert_eq!(s.style, "monorepo");
    }

    #[test]
    fn test_flat_project() {
        let dirs = vec!["docs", "scripts"];
        let s = detect_structure(&dirs, false);
        assert_eq!(s.style, "flat");
        assert!(s.source_roots.is_empty());
    }

    #[test]
    fn test_config_detection() {
        let dirs = vec!["src", "config"];
        let s = detect_structure(&dirs, false);
        assert_eq!(s.config_root, Some("config/".into()));
    }
}
