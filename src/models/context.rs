use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub project: Project,
    pub structure: Structure,
    pub status: Status,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub build_systems: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<Repository>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    pub url: String,
    pub default_branch: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Structure {
    pub style: String,
    pub source_roots: Vec<String>,
    pub test_roots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<Vec<Workspace>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub workspace_type: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub schema_version: u32,
    pub dex_version: String,
    pub last_sync: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_round_trip() {
        let ctx = Context {
            project: Project {
                name: "test-project".into(),
                project_type: "cli".into(),
                languages: vec!["rust".into()],
                frameworks: vec![],
                build_systems: vec!["cargo".into()],
                description: None,
                repository: None,
            },
            structure: Structure {
                style: "flat".into(),
                source_roots: vec!["src/".into()],
                test_roots: vec![],
                config_root: None,
                workspaces: None,
            },
            status: Status {
                schema_version: 1,
                dex_version: "0.1.0".into(),
                last_sync: "2026-03-14T20:00:00Z".into(),
            },
        };

        let toml_str = toml::to_string_pretty(&ctx).unwrap();
        let parsed: Context = toml::from_str(&toml_str).unwrap();
        assert_eq!(ctx, parsed);
    }

    #[test]
    fn test_context_with_workspaces() {
        let ctx = Context {
            project: Project {
                name: "mono".into(),
                project_type: "monorepo".into(),
                languages: vec!["rust".into()],
                frameworks: vec![],
                build_systems: vec!["cargo".into()],
                description: None,
                repository: None,
            },
            structure: Structure {
                style: "monorepo".into(),
                source_roots: vec![],
                test_roots: vec![],
                config_root: None,
                workspaces: Some(vec![Workspace {
                    name: "core".into(),
                    path: "crates/core/".into(),
                    workspace_type: "library".into(),
                }]),
            },
            status: Status {
                schema_version: 1,
                dex_version: "0.1.0".into(),
                last_sync: "2026-03-14T20:00:00Z".into(),
            },
        };

        let toml_str = toml::to_string_pretty(&ctx).unwrap();
        assert!(toml_str.contains("[[structure.workspaces]]"));
        let parsed: Context = toml::from_str(&toml_str).unwrap();
        assert_eq!(ctx, parsed);
    }
}
