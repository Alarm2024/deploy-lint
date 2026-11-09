# Preflight deploy rules

Use these checks on every pull request that touches deployment files (Dockerfiles, Kubernetes YAML, or systemd unit files). The **Preflight** CLI enforces the same rules offline in CI.

## DL001 — Do not run as root (high)

Containers must not run as UID 0. Dockerfiles need a non-root `USER`. Kubernetes workloads need `securityContext.runAsNonRoot: true` and a non-zero `runAsUser` (pod or container level).

## DL002 — Pin image tags (high)

Do not use `:latest` or untagged image references. Use an explicit version tag or digest so deploys are reproducible.

## DL003 — No secret literals in env (high)

Environment variables whose names suggest secrets (`KEY`, `TOKEN`, `SECRET`, `PASSWORD`, etc.) must not be set to literal values in the image or manifest. Use secret stores, `secretKeyRef`, or runtime injection. **Never commit or print actual secret values** — only variable names may appear in logs or reports.

## DL004 — Kubernetes resource limits (medium)

Every container in Kubernetes manifests should declare `resources.limits` (at least memory or CPU) so the scheduler can cap usage.

## DL005 — Health checks (medium)

Dockerfiles should define a `HEALTHCHECK`. Kubernetes workloads should define `livenessProbe` and/or `readinessProbe` so orchestrators can detect unhealthy instances.

## DL006 — systemd restart backoff (medium)

If a unit uses `Restart=always`, set backoff such as `RestartSec` and/or `StartLimitBurst` to avoid tight restart loops.

## Severity and CI

Any **high** finding must fail the Preflight step (non-zero exit) so the build turns red. Medium findings are reported but do not fail the job unless your team policy says otherwise.
