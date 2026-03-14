use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Paths {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub entry_points: Vec<EntryPoint>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub critical_paths: Vec<CriticalPath>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub public_api: Vec<PublicApi>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EntryPoint {
    pub name: String,
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CriticalPath {
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicApi {
    pub name: String,
    pub definition: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_round_trip() {
        let paths = Paths {
            entry_points: vec![EntryPoint {
                name: "main".into(),
                file: "src/main.rs".into(),
                description: Some("Application entry point".into()),
            }],
            critical_paths: vec![],
            public_api: vec![],
        };

        let toml_str = toml::to_string_pretty(&paths).unwrap();
        let parsed: Paths = toml::from_str(&toml_str).unwrap();
        assert_eq!(paths, parsed);
    }

    #[test]
    fn test_empty_paths() {
        let paths = Paths::default();
        let toml_str = toml::to_string_pretty(&paths).unwrap();
        assert!(!toml_str.contains("entry_points"));
    }
}
