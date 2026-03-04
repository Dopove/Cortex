use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cortex"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("run"));
}

#[test]
fn test_cli_build_no_args() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cortex"));
    cmd.arg("build")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ));
}

#[test]
fn test_cli_run_invalid_path() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cortex"));
    cmd.arg("run")
        .arg("/non/existent/path.cortex")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Bundle not found"));
}

#[test]
fn test_cli_e2e_mock_build() {
    let dir = tempdir().unwrap();
    let project_path = dir.path();
    let output_path = project_path.join("test_bundle.cortex");

    // Creating a mock project
    fs::write(project_path.join("main.py"), "print('hello')").unwrap();
    fs::write(project_path.join("requirements.txt"), "").unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cortex"));
    cmd.arg("build")
        .arg(project_path.to_str().unwrap())
        .arg(output_path.to_str().unwrap())
        .assert()
        .success();

    assert!(output_path.exists());
}
