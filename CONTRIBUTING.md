# Contributing to Codez

Thanks for your interest in improving Codez. This document covers the essentials:
how to set up, the conventions we follow, and what CI checks your change must pass.

## Getting set up

```bash
rustup target add wasm32-unknown-unknown   # for the WASM frontend
cargo install trunk                        # frontend bundler
npm install                                # Playwright + tooling
```

Build the frontend, then run the backend which also serves it:

```bash
(cd crates/frontend && trunk build)
cargo run -p backend       # http://127.0.0.1:3000
```

For frontend hot-reload, run `trunk serve` in `crates/frontend` (serves on :8080 and
proxies `/api/v1` to the backend on :3000).

## Before you open a PR

Run the same checks CI runs and make sure they are all green:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
(cd crates/frontend && trunk build)
npx playwright test --project=chromium     # needs a server on :8080
```

If you touched layout or CSS, also run the responsive sweep (needs a server on :3000):

```bash
python scripts/audit_responsive.py
```

## Conventions

- **Backend** — one handler module per domain under `crates/backend/src/handlers/`;
  routing lives in `crates/backend/src/routes/`, one sub-router per area. Shared DTOs live
  in `crates/shared`, one module per domain.
- **Frontend** — one page module per feature area under `crates/frontend/src/pages/`. All
  HTTP goes through `crates/frontend/src/api.rs`; do not call `gloo_net` directly in pages.
- **Styling** — `crates/frontend/style.css` is the single design system. Use the existing
  tokens and component classes. Any new class in markup needs a rule in `style.css`.
- **Commit messages** — imperative mood, scoped where useful (e.g. `feat(repo): …`,
  `fix(ui): …`). Keep the subject under ~72 characters.

`AGENTS.md` has a deeper reference aimed at AI agents and contributors alike — read it
before making structural changes.

## Reporting bugs and requesting features

Use the issue templates under `.github/ISSUE_TEMPLATE/`. For anything that would benefit
from discussion first, open a GitHub Discussion instead of an issue.

## License

By contributing you agree that your contributions are licensed under the MIT License
(see `LICENSE`).