//! End-to-end tests that run the compiled `cn` binary against the fixtures.

use assert_cmd::Command;
use predicates::prelude::*;

fn cn() -> Command {
    Command::cargo_bin("cn").expect("binary `cn` should build")
}

#[test]
fn finds_definition_across_languages() {
    cn().args(["--path", "tests/fixtures", "def", "parse_config"])
        .assert()
        .success()
        // Defined once in sample.rs and once in sample.py.
        .stderr(predicate::str::contains("2 match"));
}

#[test]
fn finds_callers_but_not_comments() {
    cn().args(["--path", "tests/fixtures", "callers", "parse_config"])
        .assert()
        .success()
        // Called once in each file's main/entry; the Rust comment mention
        // must not be counted.
        .stderr(predicate::str::contains("2 match"));
}

#[test]
fn json_output_is_valid_json() {
    let out = cn()
        .args(["--path", "tests/fixtures", "--json", "def", "read_file"])
        .output()
        .expect("run");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 2);
}

#[test]
fn stats_reports_files_parsed() {
    cn().args(["--path", "tests/fixtures", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("files parsed: 2"));
}

#[test]
fn unknown_symbol_is_graceful() {
    cn().args(["--path", "tests/fixtures", "def", "does_not_exist"])
        .assert()
        .success()
        .stderr(predicate::str::contains("no definitions found"));
}

#[test]
fn graph_tree_shows_callees() {
    // parse_config calls read_file and validate; the tree output should
    // contain both callees under the root.
    cn().args(["--path", "tests/fixtures", "graph", "parse_config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("parse_config"))
        .stdout(predicate::str::contains("read_file"));
}

#[test]
fn graph_mermaid_emits_arrows() {
    cn().args([
        "--path",
        "tests/fixtures",
        "graph",
        "parse_config",
        "--format",
        "mermaid",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("graph LR"))
    .stdout(predicate::str::contains("-->"));
}

#[test]
fn graph_callers_direction() {
    // Who calls read_file? parse_config does.
    cn().args([
        "--path",
        "tests/fixtures",
        "graph",
        "read_file",
        "--callers",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("parse_config"));
}
