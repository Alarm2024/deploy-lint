use deploy_lint::lint_path;
use std::path::Path;

fn fixture(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
}

fn has_rule(findings: &[deploy_lint::Finding], rule_id: &str) -> bool {
    findings.iter().any(|f| f.rule_id == rule_id)
}

fn output_contains_secret(output: &str) -> bool {
    output.contains("super-secret-token-value")
        || output.contains("literal-password-here")
        || output.contains("super-secret")
}

macro_rules! rule_bad {
    ($name:ident, $path:expr, $rule:expr) => {
        #[test]
        fn $name() {
            let findings = lint_path(&fixture($path)).expect("lint");
            assert!(
                has_rule(&findings, $rule),
                "expected rule {} for {}",
                $rule,
                $path
            );
        }
    };
}

macro_rules! rule_good {
    ($name:ident, $path:expr, $rule:expr) => {
        #[test]
        fn $name() {
            let findings = lint_path(&fixture($path)).expect("lint");
            assert!(
                !has_rule(&findings, $rule),
                "did not expect rule {} for {}",
                $rule,
                $path
            );
        }
    };
}

rule_bad!(dl001_docker_root_bad, "docker/dl001-root-bad/Dockerfile", "DL001");
rule_good!(dl001_docker_root_good, "docker/dl001-root-good/Dockerfile", "DL001");

rule_bad!(dl002_docker_latest_bad, "docker/dl002-latest-bad/Dockerfile", "DL002");
rule_bad!(dl002_docker_untagged_bad, "docker/dl002-untagged-bad/Dockerfile", "DL002");
rule_good!(dl002_docker_latest_good, "docker/dl002-latest-good/Dockerfile", "DL002");

rule_bad!(dl003_docker_secret_bad, "docker/dl003-secret-bad/Dockerfile", "DL003");
rule_good!(dl003_docker_secret_good, "docker/dl003-secret-good/Dockerfile", "DL003");

rule_bad!(dl005_docker_health_bad, "docker/dl005-health-bad/Dockerfile", "DL005");
rule_good!(dl005_docker_health_good, "docker/dl005-health-good/Dockerfile", "DL005");

rule_bad!(dl001_k8s_root_bad, "k8s/dl001-root-bad.yaml", "DL001");
rule_good!(dl001_k8s_root_good, "k8s/dl001-root-good.yaml", "DL001");

rule_bad!(dl002_k8s_latest_bad, "k8s/dl002-latest-bad.yaml", "DL002");
rule_good!(dl002_k8s_latest_good, "k8s/dl002-latest-good.yaml", "DL002");

rule_bad!(dl003_k8s_secret_bad, "k8s/dl003-secret-bad.yaml", "DL003");
rule_good!(dl003_k8s_secret_good, "k8s/dl003-secret-good.yaml", "DL003");

rule_bad!(dl004_k8s_limits_bad, "k8s/dl004-limits-bad.yaml", "DL004");
rule_good!(dl004_k8s_limits_good, "k8s/dl004-limits-good.yaml", "DL004");

rule_bad!(dl005_k8s_probe_bad, "k8s/dl005-probe-bad.yaml", "DL005");
rule_good!(dl005_k8s_probe_good, "k8s/dl005-probe-good.yaml", "DL005");

rule_bad!(
    dl006_systemd_restart_bad,
    "systemd/dl006-restart-bad.service",
    "DL006"
);
rule_good!(
    dl006_systemd_restart_good,
    "systemd/dl006-restart-good.service",
    "DL006"
);

#[test]
fn dl003_docker_secret_bad_masks_value_in_json() {
    use assert_cmd::Command;

    let path = fixture("docker/dl003-secret-bad/Dockerfile");
    let output = Command::cargo_bin("preflight")
        .unwrap()
        .arg("--json")
        .arg(&path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(has_rule(
        &lint_path(&path).unwrap(),
        "DL003"
    ));
    assert!(!output_contains_secret(&stdout));
    assert!(!output_contains_secret(&String::from_utf8_lossy(&output.stderr)));
}

#[test]
fn dl003_k8s_secret_bad_masks_value_in_json() {
    use assert_cmd::Command;

    let path = fixture("k8s/dl003-secret-bad.yaml");
    let output = Command::cargo_bin("preflight")
        .unwrap()
        .arg("--json")
        .arg(&path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output_contains_secret(&stdout));
}
