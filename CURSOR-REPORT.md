# Cursor agent report — Preflight step 2

**Repository:** Alarm2024/deploy-lint  
**Branch:** `cursor/deploy-lint-cli-f9ab`  
**Commit:** `7226cc3` (after push)

## What changed

| Area | Change |
|------|--------|
| Binary / CLI | `deploy-lint` → **`preflight`** (`Cargo.toml` `[[bin]]`, Clap `name = "preflight"`) |
| README | Title **Preflight**; repo name remains **deploy-lint** |
| Output | `--json` (existing), new **`--sarif`**, new **`--ci`** (line + `::error` annotation per finding) |
| CI | `.github/workflows/preflight.yml` — `cargo test`, scan PR changed deploy files, SARIF upload with `continue-on-error`, PR inline comments |
| Gitar | `.gitar/review/preflight.md` — six rules in plain English |
| Demo | `demo/bad/` vs `demo/fixed/` for fictional **schedule-app** |

No runtime network use. DL003 never prints secret **values** (verified in JSON, SARIF, and `--ci` tests/output).

## Rule IDs (exact)

1. `DL001` — non-root / runAsNonRoot (high)  
2. `DL002` — pinned image tag (high)  
3. `DL003` — no secret literals in env (high)  
4. `DL004` — Kubernetes resource limits (medium)  
5. `DL005` — health probes / HEALTHCHECK (medium)  
6. `DL006` — systemd restart backoff (medium)  

## Test output (`cargo test`)

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
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
test ci_flag_emits_line_and_annotation ... ok
test json_flag_emits_json ... ok
test exits_one_when_high_severity ... ok
test exits_zero_when_no_high_severity ... ok
test scans_directory_recursively ... ok
test sarif_flag_emits_sarif ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/rule_fixtures.rs (target/debug/deps/rule_fixtures-5e924337605702f3)

running 23 tests
test dl001_docker_root_good ... ok
test dl001_docker_root_bad ... ok
test dl002_docker_latest_bad ... ok
test dl002_docker_latest_good ... ok
test dl002_docker_untagged_bad ... ok
test dl001_k8s_root_good ... ok
test dl001_k8s_root_bad ... ok
test dl002_k8s_latest_bad ... ok
test dl003_docker_secret_bad ... ok
test dl003_docker_secret_good ... ok
test dl003_k8s_secret_bad ... ok
test dl002_k8s_latest_good ... ok
test dl003_k8s_secret_good ... ok
test dl004_k8s_limits_good ... ok
test dl005_docker_health_bad ... ok
test dl005_docker_health_good ... ok
test dl003_docker_secret_bad_masks_value_in_json ... ok
test dl004_k8s_limits_bad ... ok
test dl005_k8s_probe_bad ... ok
test dl006_systemd_restart_bad ... ok
test dl006_systemd_restart_good ... ok
test dl005_k8s_probe_good ... ok
test dl003_k8s_secret_bad_masks_value_in_json ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests deploy_lint

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Total automated tests:** 31 passed (2 lib unit + 6 CLI + 23 fixtures).

### Demo smoke

```bash
$ cargo run --quiet -- demo/fixed
No findings.
# exit 0

$ cargo run --quiet -- demo/bad
# exit 1 (high findings present)
```

## Pull request

**Draft PR** (per instructions: no merge, no ready-for-review).

Create / open from:

https://github.com/Alarm2024/deploy-lint/compare/main...cursor/deploy-lint-cli-f9ab?expand=1

Suggested title: *Preflight CLI: SARIF, CI annotations, demo, and GitHub Actions* (mark as **draft**).

If your Cursor PR automation already queued this branch, use that draft PR URL once it appears.

## Spec source

Checklist from uploaded preflight step-2 brief (`cursor-preflight-step2_2_fc7c.md`).
