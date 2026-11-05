/*
 * Copyright (c) 2016-2026 Legacy Author. All rights reserved.
 */

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("headercraft").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Pluggable file header validator and formatter",
        ));
}

#[test]
fn test_cli_init() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("headercraft.toml");

    let mut cmd = Command::cargo_bin("headercraft").unwrap();
    cmd.current_dir(dir.path())
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created \"headercraft.toml\""));

    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[header]"));
    assert!(content.contains("template ="));
}

#[test]
fn test_cli_check_gcc_and_sarif_formats() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("main.py");
    fs::write(&file_path, "#!/usr/bin/env python3\nprint('hello')\n").unwrap();

    // Check with GCC format
    let mut cmd_gcc = Command::cargo_bin("headercraft").unwrap();
    cmd_gcc
        .current_dir(dir.path())
        .args(["check", "--format", "gcc", "main.py"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "main.py:1:1: error: Missing or outdated file header [headercraft]",
        ));

    // Check with SARIF format
    let mut cmd_sarif = Command::cargo_bin("headercraft").unwrap();
    let assert = cmd_sarif
        .current_dir(dir.path())
        .args(["check", "--format", "sarif", "main.py"])
        .assert()
        .failure();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).expect("Output must be valid JSON");
    assert_eq!(v["version"], "2.1.0");
    assert_eq!(v["runs"][0]["tool"]["driver"]["name"], "headercraft");
}

#[test]
fn test_cli_diff_and_fix_workflow() {
    let dir = tempdir().unwrap();
    let java_file = dir.path().join("App.java");
    let initial = "/*\n * Copyright 2016 Legacy Author.\n */\npublic class App {}\n";
    fs::write(&java_file, initial).unwrap();

    // 1. diff shows unified diff with inception year 2016 preserved
    let mut cmd_diff = Command::cargo_bin("headercraft").unwrap();
    cmd_diff
        .current_dir(dir.path())
        .args(["diff", "App.java"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "- * Copyright 2016 Legacy Author.",
        ))
        .stdout(predicate::str::contains("+ * Copyright (c) 2016-"))
        .stdout(predicate::str::contains("Legacy Author"));

    // 2. fix modifies file
    let mut cmd_fix = Command::cargo_bin("headercraft").unwrap();
    cmd_fix
        .current_dir(dir.path())
        .args(["fix", "App.java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated header: App.java"));

    let fixed_content = fs::read_to_string(&java_file).unwrap();
    assert!(fixed_content.contains("2016-"));
    assert!(fixed_content.contains("Legacy Author"));

    // 3. check now succeeds with code 0
    let mut cmd_check = Command::cargo_bin("headercraft").unwrap();
    cmd_check
        .current_dir(dir.path())
        .args(["check", "App.java"])
        .assert()
        .success()
        .stdout(predicate::str::contains("All files have valid headers."));
}
