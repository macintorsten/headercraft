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
            "main.py:1:1: error: Missing or outdated file header [hash_line]",
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

#[test]
fn test_cli_crlf_file_workflow() {
    let dir = tempdir().unwrap();
    let win_file = dir.path().join("Windows.java");
    let crlf_content = "package com.win;\r\n\r\npublic class Windows {}\r\n";
    fs::write(&win_file, crlf_content).unwrap();

    // 1. fix preserves CRLF
    let mut cmd_fix = Command::cargo_bin("headercraft").unwrap();
    cmd_fix
        .current_dir(dir.path())
        .args(["fix", "Windows.java"])
        .assert()
        .success();

    let fixed = fs::read_to_string(&win_file).unwrap();
    assert!(fixed.contains("\r\n"));
    assert!(!fixed.replace("\r\n", "").contains('\n'));

    // 2. check succeeds without false diff
    let mut cmd_check = Command::cargo_bin("headercraft").unwrap();
    cmd_check
        .current_dir(dir.path())
        .args(["check", "Windows.java"])
        .assert()
        .success();
}

#[test]
fn test_cli_check_only_rule_workflow() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("headercraft.toml");
    let config_toml = r##"
[[rules]]
name = "shell_usage"
extensions = ["sh"]
style = "line"
comment_prefix = "# "
fixable = false
keywords = ["Usage:", "SYNOPSIS"]
"##;
    fs::write(&config_path, config_toml).unwrap();

    let script_file = dir.path().join("deploy.sh");
    fs::write(&script_file, "#!/bin/bash\necho deploy\n").unwrap();

    // 1. check fails with manual fix required note
    let mut cmd_check = Command::cargo_bin("headercraft").unwrap();
    cmd_check
        .current_dir(dir.path())
        .args(["check", "--format", "gcc", "deploy.sh"])
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "deploy.sh:1:1: error: Missing or outdated file header (manual fix required) [shell_usage]",
        ));

    // 2. diff skips check-only rule
    let mut cmd_diff = Command::cargo_bin("headercraft").unwrap();
    cmd_diff
        .current_dir(dir.path())
        .args(["diff", "deploy.sh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    // 3. fix refuses to auto-fix and outputs manual fix required
    let mut cmd_fix = Command::cargo_bin("headercraft").unwrap();
    cmd_fix
        .current_dir(dir.path())
        .args(["fix", "deploy.sh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Manual fix required: deploy.sh (rule 'shell_usage' is check-only)",
        ));

    // 4. Manually add usage header
    fs::write(
        &script_file,
        "#!/bin/bash\n# Usage: deploy.sh --env <env>\n\necho deploy\n",
    )
    .unwrap();

    // 5. check now succeeds
    let mut cmd_check_after = Command::cargo_bin("headercraft").unwrap();
    cmd_check_after
        .current_dir(dir.path())
        .args(["check", "deploy.sh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("All files have valid headers."));
}
