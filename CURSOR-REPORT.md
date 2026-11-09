# Cursor agent report — Preflight step 3 PR A (DL007)

**Repository:** Alarm2024/deploy-lint  
**Branch:** `cursor/dl007-kubernetes-privileged-52b7`  
**Base:** `cursor/deploy-lint-cli-f9ab`

## Change summary

- **DL007** (high): flags Kubernetes `containers` and `initContainers` with `securityContext.privileged: true` or `allowPrivilegeEscalation: true`.
- Fixture tests: privileged bad, allow-esc bad, good (explicit false), initContainer, multi-doc YAML, malformed YAML (no panic).
- `demo/bad/deploy.yaml` adds `privileged: true`; `demo/fixed` unchanged and still exits 0.

## Test output (`cargo test`)

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.74s
     Running unittests src/lib.rs (target/debug/deps/deploy_lint-0da49bb386932f75)

running 2 tests
test output::tests::ci_lines_and_annotations ... ok
test output::tests::sarif_never_includes_fix_hint_secrets ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/preflight-a532892ec042f21e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/cli.rs (target/debug/deps/cli-bd88d6e718d5b3ae)

running 6 tests
test exits_one_when_high_severity ... ok
test ci_flag_emits_line_and_annotation ... ok
test json_flag_emits_json ... ok
test exits_zero_when_no_high_severity ... ok
test sarif_flag_emits_sarif ... ok
test scans_directory_recursively ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/rule_fixtures.rs (target/debug/deps/rule_fixtures-5e924337605702f3)

running 29 tests
test dl001_docker_root_good ... ok
test dl001_docker_root_bad ... ok
test dl001_k8s_root_bad ... ok
test dl001_k8s_root_good ... ok
test dl002_docker_latest_bad ... ok
test dl002_docker_latest_good ... ok
test dl002_docker_untagged_bad ... ok
test dl002_k8s_latest_bad ... ok
test dl002_k8s_latest_good ... ok
test dl003_docker_secret_bad ... ok
test dl003_docker_secret_good ... ok
test dl003_k8s_secret_bad ... ok
test dl003_k8s_secret_good ... ok
test dl003_docker_secret_bad_masks_value_in_json ... ok
test dl004_k8s_limits_bad ... ok
test dl005_docker_health_good ... ok
test dl005_docker_health_bad ... ok
test dl004_k8s_limits_good ... ok
test dl003_k8s_secret_bad_masks_value_in_json ... ok
test dl006_systemd_restart_bad ... ok
test dl006_systemd_restart_good ... ok
test dl005_k8s_probe_bad ... ok
test dl005_k8s_probe_good ... ok
test dl007_allow_esc_bad ... ok
test dl007_malformed_yaml_does_not_panic ... ok
test dl007_init_bad ... ok
test dl007_privileged_good ... ok
test dl007_privileged_bad ... ok
test dl007_multidoc_bad ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests deploy_lint

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Total automated tests:** 37 passed (2 lib + 6 CLI + 29 fixtures).

### Demo smoke

```bash
$ cargo run --quiet -- demo/fixed
No findings.
# exit 0

$ cargo run --quiet -- demo/bad
# exit 1 (high findings including DL007)
```
