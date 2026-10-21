# Codez

[![CI](https://github.com/analisaperlengkapan/codez/actions/workflows/ci.yml/badge.svg)](https://github.com/analisaperlengkapan/codez/actions/workflows/ci.yml)

Codez is a lightweight, Gitea-inspired code hosting platform written in Rust. It pairs an
**Axum** HTTP backend with a **Leptos** (Rust/WASM) single-page frontend, sharing one set of
typed DTOs across both.

> **Note:** persistence is in-memory so the demo runs with zero setup. Data resets when the
> backend restarts.

## Tech stack

| Layer | Technology |
| --- | --- |
| Backend | [Axum](https://github.com/tokio-rs/axum), Tokio, Tower |
| Frontend | [Leptos](https://github.com/leptos-rs/leptos) (Rust/WASM), Trunk |
| Shared | Common DTOs and utilities in `crates/shared` |
| Storage | In-memory (resets on restart) |
| Build | Cargo workspaces + Trunk |
| Tests | `cargo test` (backend unit + integration) + Playwright (end-to-end) |
| CI | GitHub Actions — fmt, clippy, tests, frontend build, e2e |

## Architecture

The API surface is ported from Gitea's OpenAPI 3 specification. `scripts/gen_gitea_api.py`
reads the spec in the `gitea/` checkout and generates `crates/backend/src/api/routes.rs`
(the router) and `ops.rs` (the operation table). A single dispatcher matches each concrete
request against that table and forwards it to a per-family handler module under
`crates/backend/src/api/` (repos, issues, users, orgs, admin, actions, packages, search,
misc). Families with real behaviour are implemented explicitly; the remainder fall through
to a specification-derived default so the whole documented surface answers coherently
instead of returning a bare `404`. All shared state lives behind `RwLock`s in `state.rs`
that are poison-safe (`.unwrap_or_else(|e| e.into_inner())`), so a panicking handler never
takes the whole server down. DTOs live in `crates/shared` (`models.rs`, `wire.rs`),
re-exported from its root so both crates keep a flat `shared::<Type>` import path.

The frontend is a Leptos SPA with one page module per feature area (and a `repo/` directory
module for the larger repository surfaces), all sharing a single `style.css` design system
built on tokens. Every request goes through the typed client in `crates/frontend/src/api.rs`,
which centralises URL building and error handling so pages never hand-roll fetch chains.

Repository pages share a common header and section navigation via the `RepoNav` component,
which highlights the active section and appears on every repository sub-page.

## Screenshots

Every screen below is captured from a running instance by
[`scripts/capture_screenshots.py`](scripts/capture_screenshots.py).

### Discovery & dashboard

| Dashboard | Explore | Repository |
| --- | --- | --- |
| ![Dashboard](docs/screenshots/01_dashboard.png) | ![Explore](docs/screenshots/02_explore.png) | ![Repository](docs/screenshots/03_repo_detail.png) |

### Repository code

| Code browser | Commits | Branches |
| --- | --- | --- |
| ![Code](docs/screenshots/04_repo_code.png) | ![Commits](docs/screenshots/05_commits.png) | ![Branches](docs/screenshots/06_branches.png) |

| Tags | Releases | Pulse |
| --- | --- | --- |
| ![Tags](docs/screenshots/07_tags.png) | ![Releases](docs/screenshots/14_releases.png) | ![Pulse](docs/screenshots/17_pulse.png) |

### Issues & pull requests

| Issues | Issue detail | Pull requests |
| --- | --- | --- |
| ![Issues](docs/screenshots/08_issues.png) | ![Issue detail](docs/screenshots/09_issue_detail.png) | ![Pulls](docs/screenshots/10_pulls.png) |

| Pull request detail | Labels | Milestones |
| --- | --- | --- |
| ![Pull detail](docs/screenshots/11_pull_detail.png) | ![Labels](docs/screenshots/12_labels.png) | ![Milestones](docs/screenshots/13_milestones.png) |

| Milestone detail | Release detail | Wiki |
| --- | --- | --- |
| ![Milestone detail](docs/screenshots/38_milestone_detail.png) | ![Release detail](docs/screenshots/39_release_detail.png) | ![Wiki](docs/screenshots/16_wiki.png) |

| Wiki editor | Packages | Actions |
| --- | --- | --- |
| ![Wiki editor](docs/screenshots/37_wiki_edit.png) | ![Packages](docs/screenshots/18_packages.png) | ![Actions](docs/screenshots/15_actions.png) |

### Compare, search & notifications

| Compare | Code search | Notifications |
| --- | --- | --- |
| ![Compare](docs/screenshots/35_compare.png) | ![Code search](docs/screenshots/36_code_search.png) | ![Notifications](docs/screenshots/19_notifications.png) |

### Account & administration

| Search | Admin | Admin users |
| --- | --- | --- |
| ![Search](docs/screenshots/20_search.png) | ![Admin](docs/screenshots/21_admin.png) | ![Admin users](docs/screenshots/22_admin_users.png) |

| Admin orgs | Admin emails | User profile |
| --- | --- | --- |
| ![Admin orgs](docs/screenshots/23_admin_orgs.png) | ![Admin emails](docs/screenshots/24_admin_emails.png) | ![User profile](docs/screenshots/25_user_profile.png) |

| Followers | Following | Organization |
| --- | --- | --- |
| ![Followers](docs/screenshots/26_user_followers.png) | ![Following](docs/screenshots/27_user_following.png) | ![Organization](docs/screenshots/28_org_profile.png) |

| Settings | Login | Register |
| --- | --- | --- |
| ![Settings](docs/screenshots/29_settings.png) | ![Login](docs/screenshots/30_login.png) | ![Register](docs/screenshots/31_register.png) |

### Repository management

| Create repository | Migrate repository | Create organization |
| --- | --- | --- |
| ![Create repo](docs/screenshots/32_create_repo.png) | ![Migrate repo](docs/screenshots/33_migrate_repo.png) | ![Create org](docs/screenshots/34_create_org.png) |

| Repository settings | File editor |
| --- | --- |
| ![Repo settings](docs/screenshots/40_repo_settings.png) | ![File editor](docs/screenshots/41_file_edit.png) |

## Features

- **Repositories** — create, migrate, and configure, with a code browser, commit history,
  branches, tags, topics, and star/watch.
- **Issues & pull requests** — issues with comments, labels, milestones, and assignees;
  pull requests with reviews, plus a compare view.
- **Releases & packages** — releases with assets plus a package registry.
- **Organizations & teams** — organizations, teams, and memberships.
- **Users** — registration and sign-in, profile settings, SSH/GPG keys, access tokens,
  and follow relationships.
- **Wiki** — integrated per-repository documentation with an editor.
- **Webhooks, secrets & keys** — repository webhooks, Actions secrets, and deploy keys.
- **Actions** — workflow runs with a run list per workflow.
- **Administration** — admin dashboards for users, organizations, emails, and scheduled
  tasks.
- **Search** — global search plus per-repository code search.

## Getting started

### Prerequisites

- Rust (latest stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- `trunk`: `cargo install trunk`

### Build and run

```bash
# 1. Build the frontend into crates/frontend/dist
(cd crates/frontend && trunk build)

# 2. Run the backend, which also serves the built frontend
cargo run -p backend
```

Open <http://127.0.0.1:3000>. The backend serves both the API (`/api/v1/*`) and the SPA, so
client-side deep links work without a separate dev server.

For frontend hot-reload during development, run `trunk serve` in `crates/frontend` instead
and point it at the backend API.

### Configuration

All settings are environment variables with sensible defaults:

| Variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `3000` | Bind port (listens on `0.0.0.0`) |
| `CODEZ_STATIC_DIR` | `crates/frontend/dist` | Compiled frontend assets |

## Development

```bash
cargo fmt --all                        # format
cargo clippy --workspace --all-targets # lint
cargo test --workspace                 # run all tests
```

### Layout

```
crates/
  backend/    Axum API server — api/ (generated routes + ops, dispatcher, one
              handler per family), respond.rs, state.rs, seed.rs, tests/
  frontend/   Leptos SPA (pages/, components/) + api.rs client + style.css
  shared/     DTOs (models.rs) and serde helpers (wire.rs), re-exported from lib.rs
gitea/        Upstream Gitea checkout (no .git) — source of the OpenAPI spec
docs/
  screenshots/  UI reference images used in this README
scripts/        Verification, spec generation, and screenshot tooling
tests/          Playwright end-to-end specs
```

### Responsive design

The layout is audited at 390 px, 768 px, and 1440 px. The global header scrolls
horizontally on narrow screens instead of wrapping, so the document never overflows.
`scripts/capture_screenshots.py` drives the screenshot gallery; `python scripts/audit_responsive.py`
sweeps every route at three widths and reports any element that escapes the viewport.

### Verification & screenshots

With a server running (default `http://127.0.0.1:3000`; override with `CODEZ_BASE_URL`):

```bash
python scripts/capture_screenshots.py docs/screenshots   # refresh README images
python scripts/verify_reviews.py                         # exercise a feature flow
```

The `tests/` directory holds Playwright specs. They target `http://127.0.0.1:8080`
(the `trunk serve` origin, which proxies `/api/v1` to the backend) and honour the
`CODEZ_BASE_URL` override:

```bash
(cd crates/frontend && trunk serve)   # UI + API proxy on :8080
npx playwright test --project=chromium
```

Suites are `tests/routes.spec.ts`, a per-route smoke net that renders every client route
and fails on any uncaught page error; `tests/navigation.spec.ts`, which follows the global
nav into the seeded resources; `tests/write_flows.spec.ts`, which drives the create forms
and asserts the observable follow-up (a new repository navigates to its page, a new
organization appears in the list without a reload); and `tests/rejected_writes.spec.ts`,
which forces a non-2xx response and asserts the user's input survives with a visible
`.form-error[role="alert"]`.

## API

API endpoints are prefixed with `/api/v1` and follow Gitea's REST conventions, ported from
its OpenAPI 3 specification.

## Contributing

Contributions are welcome — see [`CONTRIBUTING.md`](CONTRIBUTING.md) for setup, the
conventions we follow, and the checks CI runs. Security issues should be reported
privately per [`SECURITY.md`](SECURITY.md), not in a public issue.

## License

Released under the [MIT License](LICENSE).

