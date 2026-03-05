<p align="center">
  <img src="assets/logo.svg" width="128" alt="Docs Sentry Logo"/>
</p>

<h1 align="center">Docs Sentry</h1>

<p align="center">
  <strong>Audit GitHub organization READMEs and surface documentation gaps in seconds</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024-000000?logo=rust" alt="Rust 2024">
  <img src="https://img.shields.io/badge/GitHub_CLI-gh-181717?logo=github" alt="GitHub CLI">
  <img src="https://img.shields.io/badge/Output-table%20%7C%20markdown%20%7C%20json-0EA5E9" alt="Output Formats">
  <img src="https://img.shields.io/badge/License-MIT-22C55E" alt="MIT License">
</p>

<p align="center">
  <a href="#quick-start"><strong>Quick Start -></strong></a>
</p>

<p align="center">
  Docs Sentry scans repositories in a GitHub organization, checks README structure against a practical template, and produces actionable reports for maintainers.
</p>

---

<div><img src="https://quickstart-for-agents.vercel.app/api/header.svg?theme=opencode&logo=github&title=Audit+README+quality+across+GitHub+organization+repositories&font=inter" width="100%" /></div>

```
You are an AI agent working on docs-sentry, a Rust 2024 CLI that audits
README quality for GitHub organization repositories using gh CLI data.
Clone https://github.com/Open330/docs-sentry and improve scoring rules,
report formats, and workflow integrations for documentation governance.
```

## Features

**Organization Scan** -- Pull repositories with `gh repo list` and inspect each README through the GitHub API.

**Template-Aware Scoring** -- Score docs coverage with weighted checks for sections like Features, Quick Start, Architecture, and License using heading-intent matching (`#`, `##`, `###`, and heading variants).

**Actionable Gaps** -- Return missing required and recommended sections per repo, plus threshold-based notes.

**Three Output Modes** -- Emit terminal table, markdown report, or machine-consumable JSON.

**Strict CI Mode** -- Use `--strict` to fail with exit code `2` when one or more repos fall below `--min-score`.

**Fetch Error Visibility** -- Network/auth/API failures are reported as `fetch-error` instead of being mislabeled as missing READMEs.

## Score Model

| Check | Weight | Type |
| --- | ---: | --- |
| Features | 18 | Required |
| Quick Start / Getting Started | 16 | Required |
| Architecture | 14 | Required |
| License | 14 | Required |
| Tech Stack | 10 | Recommended |
| Usage / CLI Options / Commands | 8 | Recommended |
| Run Tests / Testing | 8 | Recommended |
| Deploy / Deployment | 6 | Recommended |
| Centered Hero Block | 3 | Recommended |
| Agent Header Block | 3 | Recommended |

Total: 100 points.

## Status Tiers

| Status | Score |
| --- | ---: |
| `strong` | 85-100 |
| `needs-work` | 60-84 |
| `weak` | 0-59 |
| `missing-readme` | README missing |
| `fetch-error` | README could not be fetched |

## CLI Options

| Option | Default | Description |
| --- | --- | --- |
| `--org` | `Open330` | Target GitHub organization |
| `--limit` | `100` | Maximum repositories to scan |
| `--min-score` | `70` | Threshold used to flag low documentation quality |
| `--format` | `table` | Output format: `table`, `markdown`, `json` |
| `--include-private` | `false` | Include private repositories |
| `--strict` | `false` | Exit with code 2 if repos are below threshold |

## Architecture

```text
docs-sentry/
├── assets/
│   └── logo.svg               # README logo
├── src/
│   ├── main.rs                # CLI entry point and exit-code behavior
│   ├── lib.rs                 # Scan orchestration
│   ├── config.rs              # Argument parsing and defaults
│   ├── github.rs              # gh command wrappers and repository/README fetches
│   ├── audit.rs               # Scoring model, status classification, summaries
│   └── output.rs              # Table/markdown/json rendering
├── .github/workflows/ci.yml   # fmt + clippy + test + release build checks
└── Cargo.toml                 # Package metadata
```

## Tech Stack

| Layer | Technology |
| --- | --- |
| Language | Rust (edition 2024) |
| Data Source | GitHub CLI (`gh`) |
| Runtime | Rust standard library |
| CI | GitHub Actions |

## Quick Start

```bash
# Clone
git clone https://github.com/Open330/docs-sentry.git
cd docs-sentry

# Audit default org with table output
cargo run --

# Generate markdown report
cargo run -- --org Open330 --format markdown --min-score 80

# Include private repos and enforce threshold
cargo run -- --org Open330 --include-private --strict
```

### Run Tests

```bash
cargo test
```

### Typecheck and Lint

```bash
cargo check
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Build

```bash
cargo build --release
```

## Deploy

No server deployment is required. This project is a local/CI CLI utility.

## License

MIT

---

<p align="center">
  <sub>Built for maintainers who want fast and consistent README governance across repositories.</sub>
</p>
