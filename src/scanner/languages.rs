use std::collections::HashMap;
use std::path::Path;

/// Map of file extension to language name.
fn extension_map() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("rs", "rust"),
        ("ts", "typescript"),
        ("tsx", "typescript"),
        ("js", "javascript"),
        ("jsx", "javascript"),
        ("py", "python"),
        ("go", "go"),
        ("java", "java"),
        ("kt", "kotlin"),
        ("swift", "swift"),
        ("c", "c"),
        ("cpp", "c++"),
        ("cc", "c++"),
        ("h", "c"),
        ("hpp", "c++"),
        ("cs", "c#"),
        ("rb", "ruby"),
        ("php", "php"),
        ("scala", "scala"),
        ("zig", "zig"),
        ("lua", "lua"),
        ("dart", "dart"),
        ("ex", "elixir"),
        ("exs", "elixir"),
        ("hs", "haskell"),
        ("ml", "ocaml"),
        ("clj", "clojure"),
    ])
}

/// Detect languages from a list of file paths.
/// Returns languages sorted by file count (descending).
pub fn detect_languages(files: &[&Path]) -> Vec<String> {
    let ext_map = extension_map();
    let mut counts: HashMap<String, usize> = HashMap::new();

    for file in files {
        if let Some(ext) = file.extension().and_then(|e| e.to_str())
            && let Some(&lang) = ext_map.get(ext)
        {
            *counts.entry(lang.to_string()).or_default() += 1;
        }
    }

    let mut langs: Vec<(String, usize)> = counts.into_iter().collect();
    langs.sort_by(|a, b| b.1.cmp(&a.1));
    langs.into_iter().map(|(lang, _)| lang).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        let files = vec![
            Path::new("src/main.rs"),
            Path::new("src/lib.rs"),
            Path::new("Cargo.toml"),
        ];
        let langs = detect_languages(&files);
        assert_eq!(langs, vec!["rust"]);
    }

    #[test]
    fn test_detect_multiple_languages() {
        let files = vec![
            Path::new("src/main.rs"),
            Path::new("src/lib.rs"),
            Path::new("src/util.rs"),
            Path::new("frontend/index.ts"),
            Path::new("frontend/app.tsx"),
        ];
        let langs = detect_languages(&files);
        assert_eq!(langs[0], "rust");
        assert_eq!(langs[1], "typescript");
    }

    #[test]
    fn test_detect_no_languages() {
        let files = vec![Path::new("README.md"), Path::new(".gitignore")];
        let langs = detect_languages(&files);
        assert!(langs.is_empty());
    }
}
