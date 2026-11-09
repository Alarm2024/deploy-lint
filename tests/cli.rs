use assert_cmd::Command;
use predicates::prelude::*;

fn fixture(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
}

#[test]
fn exits_zero_when_no_high_severity() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg(fixture("k8s/dl004-limits-good.yaml"))
        .assert()
        .success();
}

#[test]
fn exits_one_when_high_severity() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg(fixture("docker/dl001-root-bad/Dockerfile"))
        .assert()
        .code(1);
}

#[test]
fn scans_directory_recursively() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg(fixture("docker/dl002-latest-good"))
        .assert()
        .success()
        .stdout(predicate::str::contains("No findings."));
}

#[test]
fn json_flag_emits_json() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg("--json")
        .arg(fixture("k8s/dl005-probe-bad.yaml"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"findings\""));
}

#[test]
fn sarif_flag_emits_sarif() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg("--sarif")
        .arg(fixture("k8s/dl005-probe-bad.yaml"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"version\": \"2.1.0\""));
}

#[test]
fn ci_flag_emits_line_and_annotation() {
    Command::cargo_bin("preflight")
        .unwrap()
        .arg("--ci")
        .arg(fixture("docker/dl001-root-bad/Dockerfile"))
        .assert()
        .code(1)
        .stdout(predicate::str::contains("DL001"))
        .stdout(predicate::str::contains("::error file="));
}
