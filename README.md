# Preflight

Repository name: **deploy-lint**. The CLI binary is **Preflight** (`preflight`).

Preflight is a Rust CLI that scans deployment artifacts for common security and reliability mistakes. It works **offline** (no network access at runtime) and accepts a single file or a directory (recursive scan).

## Fictional demo app

The [`demo/`](demo/) tree shows a made-up **schedule-app** (not a real product):

- [`demo/bad/`](demo/bad/) — intentionally misconfigured Dockerfile, `deploy.yaml`, and `app.service` (expect exit code **1** due to high findings).
- [`demo/fixed/`](demo/fixed/) — corrected versions that pass with exit code **0**.

```bash
preflight demo/fixed   # exit 0
preflight demo/bad     # exit 1 (high severity present)
```

## Supported inputs

| Kind | Detection |
|------|-----------|
| Dockerfile | File named `Dockerfile`, `dockerfile`, `Containerfile`, or `containerfile` |
| Kubernetes | `.yaml` / `.yml` manifests |
| systemd | `.service` unit files |

## Rules

Each finding includes **rule id**, **severity**, **file**, **line**, **reason**, and a **fix hint**. Secret **values are never printed** in any output format (table, JSON, SARIF, CI lines, or GitHub annotations).

| ID | Severity | What it checks |
|----|----------|----------------|
| DL001 | high | Container runs as root (Dockerfile missing non-root `USER`; Kubernetes missing `runAsNonRoot` / non-zero `runAsUser`) |
| DL002 | high | Image uses `:latest` or has no tag |
| DL003 | high | Secret-like env vars (`KEY`, `TOKEN`, `SECRET`, `PASSWORD` in the name) set to **literal** values — only the variable **name** is shown |
| DL004 | medium | Kubernetes container missing `resources.limits` |
| DL005 | medium | No health check (Dockerfile `HEALTHCHECK`; Kubernetes `livenessProbe` or `readinessProbe`) |
| DL006 | medium | systemd `Restart=always` without `RestartSec` or `StartLimitBurst` backoff |

Exit code **1** if any **high** severity finding is present; **0** otherwise. Missing paths exit **2**.

Gitar review hints for the same six rules: [`.gitar/review/preflight.md`](.gitar/review/preflight.md).

## Install

```bash
cargo install --path .
# installs the `preflight` binary
```

Or run without installing:

```bash
cargo run -- <path>
```

## Usage

```bash
preflight deployment.yaml
preflight ./infra
preflight --json ./k8s
preflight --sarif ./k8s > results.sarif
preflight --ci ./k8s    # file:line rule-id message + ::error annotations for GitHub Actions
```

### CI output (`--ci`)

For each finding, Preflight prints one line:

```text
path/to/file.yaml:19 DL003 Secret-like env var DATABASE_PASSWORD has a literal value (masked)
```

and a GitHub Actions annotation:

```text
::error file=path/to/file.yaml,line=19::DL003: Secret-like env var DATABASE_PASSWORD has a literal value (masked)
```

High-severity findings still yield exit code **1** so the job fails.

## Development

```bash
cargo test
```

Fixture tests live under `tests/fixtures/`. Pull requests run [`.github/workflows/preflight.yml`](.github/workflows/preflight.yml): `cargo test`, Preflight on changed deploy files, optional SARIF upload when code scanning is available, and inline PR review comments.

## License

MIT — see [LICENSE](LICENSE).
