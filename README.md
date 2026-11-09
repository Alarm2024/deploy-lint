# deploy-lint

## CI / Preflight check

The [Preflight](.github/workflows/preflight.yml) workflow runs on every pull request and on pushes to `main`. It:

- Installs [yamllint](https://yamllint.readthedocs.io/) and lints all tracked `*.yml` and `*.yaml` files using the relaxed rules in [`.yamllint.yml`](.yamllint.yml).
- Fails if any tracked file contains a literal `password:` or `token:` key with a non-empty inline value (a lightweight guard against accidentally committed secrets).

There are no deploy steps, external services, or repository secrets involved.
