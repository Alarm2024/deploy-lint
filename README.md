# deploy-lint

`deploy-lint` is a Rust CLI that scans deployment artifacts for common security and reliability mistakes. It works offline (no network access at runtime) and accepts a single file or a directory (recursive scan).

## Supported inputs

| Kind | Detection |
|------|-----------|
| Dockerfile | File named `Dockerfile`, `dockerfile`, `Containerfile`, or `containerfile` |
| Kubernetes | `.yaml` / `.yml` manifests |
| systemd | `.service` unit files |

## Rules

Each finding includes **rule id**, **severity**, **file**, **line**, **reason**, and a **fix hint**.

| ID | Severity | What it checks |
|----|----------|----------------|
| DL001 | high | Container runs as root (Dockerfile missing non-root `USER`; Kubernetes missing `runAsNonRoot` / non-zero `runAsUser`) |
| DL002 | high | Image uses `:latest` or has no tag |
| DL003 | high | Secret-like env vars (`KEY`, `TOKEN`, `SECRET`, `PASSWORD` in the name) set to **literal** values — only the variable **name** is shown (value is never printed) |
| DL004 | medium | Kubernetes container missing `resources.limits` |
| DL005 | medium | No health check (Dockerfile `HEALTHCHECK`; Kubernetes `livenessProbe` or `readinessProbe`) |
| DL006 | medium | systemd `Restart=always` without `RestartSec` or `StartLimitBurst` backoff |

Exit code **1** if any **high** severity finding is present; **0** otherwise. Missing paths exit **2**.

## Install

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run -- <path>
```

## Usage

```bash
# Scan one manifest
deploy-lint deployment.yaml

# Scan a tree (e.g. repo root)
deploy-lint ./infra

# Machine-readable output
deploy-lint --json ./k8s
```

### Example (table)

```text
$ deploy-lint tests/fixtures/docker/dl001-root-bad/Dockerfile
RULE  | SEVERITY | FILE                                      | LINE   | REASON                                           | FIX
DL001 | high     | .../dl001-root-bad/Dockerfile             | 1      | Container is configured to run as root           | Add a non-root USER ...
DL005 | medium   | .../dl001-root-bad/Dockerfile             | 3      | Dockerfile has no HEALTHCHECK instruction      | Add HEALTHCHECK ...
```

### Example (JSON)

```bash
deploy-lint --json tests/fixtures/k8s/dl003-secret-bad.yaml
```

```json
{
  "findings": [
    {
      "rule_id": "DL003",
      "severity": "high",
      "file": "...",
      "line": 11,
      "reason": "Secret-like env var DB_PASSWORD has a literal value (masked)",
      "fix_hint": "Use secretKeyRef, envFrom secretRef, or external secret injection"
    }
  ],
  "summary": { "total": 1, "high": 1, "medium": 0, "low": 0 }
}
```

## Development

```bash
cargo test
```

Fixture tests live under `tests/fixtures/` with good and bad cases for every rule.

## License

MIT — see [LICENSE](LICENSE).
