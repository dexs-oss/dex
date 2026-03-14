use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn dex_cmd() -> Command {
    Command::cargo_bin("dex").unwrap()
}

fn cleanup(fixture: &Path) {
    let _ = std::fs::remove_dir_all(fixture.join(".dex"));
}

#[test]
fn test_init_rust_cli() {
    let fixture = Path::new("tests/fixtures/rust_cli");
    cleanup(fixture);

    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("rust"))
        .stdout(predicate::str::contains("cli"));

    assert!(fixture.join(".dex/context.toml").exists());
    assert!(fixture.join(".dex/paths.toml").exists());
    assert!(fixture.join(".dex/README").exists());

    let ctx = std::fs::read_to_string(fixture.join(".dex/context.toml")).unwrap();
    assert!(ctx.contains("name = \"my-cli-tool\""));
    assert!(ctx.contains("\"rust\""));
    assert!(ctx.contains("\"cargo\""));

    let paths = std::fs::read_to_string(fixture.join(".dex/paths.toml")).unwrap();
    assert!(paths.contains("src/main.rs"));

    cleanup(fixture);
}

#[test]
fn test_init_ts_web() {
    let fixture = Path::new("tests/fixtures/ts_web");
    cleanup(fixture);

    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success();

    let ctx = std::fs::read_to_string(fixture.join(".dex/context.toml")).unwrap();
    assert!(ctx.contains("name = \"my-web-app\""));
    assert!(ctx.contains("\"typescript\""));
    assert!(ctx.contains("\"react\"") || ctx.contains("\"next\""));

    cleanup(fixture);
}

#[test]
fn test_init_go_service() {
    let fixture = Path::new("tests/fixtures/go_service");
    cleanup(fixture);

    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success();

    let ctx = std::fs::read_to_string(fixture.join(".dex/context.toml")).unwrap();
    assert!(ctx.contains("\"go\""));
    assert!(ctx.contains("\"gin\""));

    cleanup(fixture);
}

#[test]
fn test_init_python_lib() {
    let fixture = Path::new("tests/fixtures/python_lib");
    cleanup(fixture);

    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success();

    let ctx = std::fs::read_to_string(fixture.join(".dex/context.toml")).unwrap();
    assert!(ctx.contains("name = \"mylib\""));
    assert!(ctx.contains("\"python\""));
    assert!(ctx.contains("\"fastapi\""));

    cleanup(fixture);
}

#[test]
fn test_init_rust_workspace() {
    let fixture = Path::new("tests/fixtures/rust_workspace");
    cleanup(fixture);

    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success();

    let ctx = std::fs::read_to_string(fixture.join(".dex/context.toml")).unwrap();
    assert!(ctx.contains("monorepo"));
    assert!(ctx.contains("[[structure.workspaces]]"));

    cleanup(fixture);
}

#[test]
fn test_init_nonexistent_path() {
    dex_cmd()
        .args(["init", "/nonexistent/path/that/does/not/exist"])
        .assert()
        .failure();
}

#[test]
fn test_sync_without_init_fails() {
    let dir = tempfile::tempdir().unwrap();
    dex_cmd()
        .args(["sync", dir.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("dex init"));
}

#[test]
fn test_sync_after_init_reports_up_to_date() {
    let fixture = Path::new("tests/fixtures/rust_cli");
    cleanup(fixture);

    // First, init
    dex_cmd()
        .args(["init", fixture.to_str().unwrap()])
        .assert()
        .success();

    // Then, sync immediately — nothing should have changed
    dex_cmd()
        .args(["sync", fixture.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("up to date"));

    cleanup(fixture);
}

#[test]
fn test_sync_detects_changes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Create a minimal Rust project
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[package]
name = "test-sync"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    // Init
    dex_cmd()
        .args(["init", root.to_str().unwrap()])
        .assert()
        .success();

    // Add a TypeScript file to simulate code changes
    std::fs::write(root.join("src/index.ts"), "console.log('hello');").unwrap();

    // Sync should detect the new language
    dex_cmd()
        .args(["sync", root.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("languages: added typescript"));
}
