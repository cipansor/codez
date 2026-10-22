# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## Project overview

Codez is a Gitea-like code hosting platform: an **Axum** backend and a **Leptos**
(Rust/WASM) SPA, with shared DTOs in a third crate. Storage is in-memory by default and
optionally PostgreSQL when `DATABASE_URL` is set.

## Repository layout

```
crates/backend     Axum API server
  src/
    main.rs        Entry point + SPA/static fallback
    api/           The `/api/v1` surface, ported from Gitea's OpenAPI spec
      routes.rs    Generated router wiring every documented path to dispatch
      ops.rs       Generated operation table (method, path, response shape)
      dispatch.rs  Matches a request to an op, routes by resource family
      mod.rs       Shared request/response plumbing
      <domain>.rs  Handlers per family (repos, issues, users, orgs, admin,
                   actions, packages, search, misc) + ops.rs
    respond.rs     Response builders + Gitea-shaped errors
    state.rs       Shared AppState (in-memory stores)
    seed.rs        Demo in-memory state
    static_files.rs  SPA/static serving with a path-containment check
    ids.rs         Identifier helpers
    pagination.rs  `X-Total-Count` / `X-HasMore` pagination
    time.rs        Timestamp helpers
    web.rs         SPA-facing helpers
    tests/         Integration tests (contracts, coverage, domains, repos)
crates/frontend    Leptos SPA
  src/
    main.rs        Router definitions
    api.rs         Typed HTTP client helpers (fetch + write wrappers)
    components/    Reusable UI (nav.rs, repo_nav.rs, ui.rs)
    pages/         One module per feature area
  style.css        Design system (tokens + components + utilities)
  index.html       Trunk entry (pulls in style.css)
crates/shared      DTOs and helpers shared by both crates
  src/             models.rs (Gitea DTOs) + wire.rs (serde helpers),
                   re-exported from lib.rs so `shared::<Type>` keeps working
gitea/             Upstream Gitea checkout used as the port's source of truth
                   (no `.git`); `scripts/gen_gitea_api.py` reads its OpenAPI spec
docs/screenshots   README gallery images
scripts/           Verification + screenshot tooling
tests/             Playwright end-to-end specs
```

## Build, test, lint

```bash
# Frontend must be built before the backend can serve it
(cd crates/frontend && trunk build)

cargo run -p backend                    # serve API + SPA on :3000 (PORT overrides)
cargo test --workspace                  # all tests (backend + shared)
cargo clippy --workspace --all-targets  # lint (keep this clean)
cargo fmt --all                         # format

# Regenerate the API router/ops from Gitea's OpenAPI spec (needs the gitea/ checkout)
python scripts/gen_gitea_api.py
```

Test suites:

- **Backend integration** (`crates/backend/src/tests/`, one module per area) drive the real
  router via `tower::ServiceExt::oneshot` against the seeded in-memory state. The modules
  are `contracts` (wire-shape), `coverage` (every documented path is routed), `domains`
  (per-family behaviour), and `repos` (repository/pagination/static serving).
- **Backend unit** tests are colocated with the pure helpers they cover (e.g. `ids.rs`,
  `pagination.rs`, `respond.rs`, and the per-family modules under `api/`).
- **Shared** DTO/serde tests live in `crates/shared`.
- **Frontend e2e** (`tests/`) is Playwright: feature specs plus `routes.spec.ts`, which
  smoke-renders every client route and asserts no uncaught page errors.

CI (`.github/workflows/ci.yml`) runs the same checks on every push/PR: `cargo fmt
--check`, `cargo clippy -- -D warnings`, `cargo test`, a release `trunk build`, and the
Playwright e2e suite against a locally started backend + `trunk serve`. The `rust`,
`frontend`, and `e2e` jobs are independent (each in its own checkout), so the e2e job
builds the bundle it needs via `trunk serve` rather than consuming another job's output.

The backend serves the SPA from `CODEZ_STATIC_DIR` (default `crates/frontend/dist`). Deep
links return `index.html`; unknown `/api/*` paths return `404`.

## Conventions

### Rust

- Keep `cargo clippy --workspace --all-targets` warning-free.
- The API lives under `crates/backend/src/api/`, one module per resource family
  (`repos.rs`, `issues.rs`, `users.rs`, `orgs.rs`, `admin.rs`, `actions.rs`,
  `packages.rs`, `search.rs`, `misc.rs`). `routes.rs` and `ops.rs` are generated from
  Gitea's OpenAPI spec by `scripts/gen_gitea_api.py` — regenerate them, do not hand-edit.
- `api/dispatch.rs` matches a concrete request against the generated operation table and
  forwards it to the family handler; a family that does not handle an op falls through to
  the specification-derived default in `dispatch::fallback`, so every documented path
  answers coherently. Add real behaviour by handling the op in its family module.
- Shared DTOs live in `crates/shared/src/` (`models.rs`, `wire.rs`), all re-exported from
  `lib.rs` so downstream code keeps the flat `shared::<Type>` path. Do not glob-import
  `shared::*` in the backend; the re-exports are the public surface.
- Frontend pages: one file per feature area under `crates/frontend/src/pages/`; large
  features (e.g. `repo`) use a directory module with a re-exporting `mod.rs`.
- All frontend HTTP goes through `crates/frontend/src/api.rs`. Pages must not call
  `gloo_net::http::Request` directly — use `api_url()` plus the `get*`/`post*`/`patch*`/
  `put*`/`delete` helpers so error handling stays uniform.
- Write helpers return `bool`: `true` only when the request was sent *and* the server
  replied `2xx`. Clear a form, navigate, or refresh only on `true`; on `false` keep the
  user's input and render `WRITE_ERROR` in a `.form-error` element with `role="alert"`.
  Never discard the result of a write.
- Any `create_resource` whose data a write can change must list the component's `refresh`
  signal in its source tuple, and the success branch must bump that signal. A resource
  that omits it silently keeps stale data — the classic symptom is a toggle whose label
  never flips or a create form that clears while the list stays unchanged.
- Avoid glob-import ambiguity between `leptos_router::*` and shared types in page modules —
  import concrete items.
- Run `cargo fmt --all` before committing.

### Frontend styling

- `crates/frontend/style.css` is the single design system. Sections in order:
  tokens, reset/base, layout, components, feature sections, utilities, responsive.
- Use design tokens (`var(--color-*)`, `var(--radius*)`) rather than raw hex/px. Component
  rules should contain no literal colors; if one is needed, add a token first. Inverted
  surfaces have their own scales (`--color-header-*`, `--color-log-*`, `--color-on-emphasis`)
  because they do not follow the light-canvas palette.
- Prefer existing utility and component classes over inline `style=` attributes.
- When adding markup with a new class, add its rule to `style.css` (there is no CSS
  framework — unstyled classes are a bug).
- Shared repo chrome (header + section nav) belongs in `components/repo_nav.rs` and should
  appear on every repository sub-page.

### Tests

- Backend integration tests live in `crates/backend/src/tests/`, split by concern
  (`contracts.rs`, `coverage.rs`, `domains.rs`, `repos.rs`) with a shared `mod.rs`, and
  exercise real handler/router code paths.
- Pure backend helpers get colocated `#[cfg(test)] mod tests` unit tests in the file that
  defines them, rather than being tested only through HTTP.
- Playwright specs live in `tests/` and run with `npx playwright test`. They use the
  `baseURL` from `playwright.config.ts` (`CODEZ_BASE_URL`, default
  `http://127.0.0.1:8080`) — never hardcode absolute URLs in specs; pass relative paths
  to `page.goto()` / `page.request.*()`.
- `tests/routes.spec.ts` is the route smoke net: keep its `ROUTES` list in sync with
  `crates/frontend/src/main.rs` so a renamed or removed route fails loudly.
- A spec for a write must assert the *observable* follow-up, not just that the request
  succeeded: a create appears in the list without a reload, a close/reopen toggle flips
  its label, and a rejected write (`page.route` forced non-2xx, see
  `tests/rejected_writes.spec.ts`) keeps the typed value and shows
  `.form-error[role="alert"]`. Never `page.reload()` to make a list assertion pass — that
  hides a missing resource refresh.
- Use a unique value (e.g. `'Label ' + Date.now()`) for anything a create spec then
  asserts on. The backend keeps in-memory state across runs, so a fixed title such as
  `v2.0 Beta` can already exist and the assertion passes even when the write, or the
  refresh it triggers, is broken.
- Ad-hoc API verification scripts live in `scripts/verify_*.py`.

## Verifying changes

```bash
cargo test --workspace
(cd crates/frontend && trunk build)
cargo run -p backend &                       # then:
python scripts/capture_screenshots.py docs/screenshots
python scripts/audit_responsive.py           # 390/768/1440 overflow sweep
```

Screenshot filenames are referenced by `README.md`; if you renumber routes in
`scripts/capture_screenshots.py`, update the README image links to match. Keep the route
list there pointing at resources that actually exist in the demo seed (for example the
seeded org slug is `codez-org`, not `admin`).

## Gotchas

- In Axum, `.fallback()` replaces any previously set `.fallback_service()`. The static file
  handler in `main.rs` is intentionally a single `.fallback()` that resolves assets itself.
- The static handler serves a file only if its canonicalized real path stays inside the
  asset root (`read_within_root`). Normalizing `..` is not enough on its own: a symlink
  placed in `dist/` can point outside it, so keep the resolved-destination check when
  touching that code.
- The header hides the search box at ≤768px and shows a plain Search link (`.search-link`)
  instead; keep both in sync when changing the global nav.
- The frontend build output (`crates/frontend/dist/`) is gitignored — build it locally.
- `node_modules/` is gitignored; run `npm install` for Playwright.
- A missing entity is a `404` with Gitea's error body
  (`{"message": "...", "url": "https://docs.gitea.com/api/..."}`), produced by
  `respond::ApiError::not_found`. Match that shape for a new detail handler rather than
  returning an empty body or a `200`.
- Operations a family has not implemented are answered by `dispatch::fallback`, which
  derives a coherent reply from the spec (lists → `[]`, detail → `null`, other writes →
  `204` or an `implemented: false` echo). When you give an op real behaviour, handle it in
  its family module — do not special-case it in the dispatcher. Every documented operation
  is currently handled, so the fallback is a safety net rather than a routine reply.
- Fallback replies carry `x-codez-fallback: 1`, because the body cannot reveal a stub:
  an unimplemented list returns `[]`, exactly what an empty store returns.
  `tests::coverage::no_documented_operation_is_answered_by_the_fallback` samples all 537
  operations and fails if the marker appears, so a resource that regresses to a stub is
  caught. Do not remove the header to silence it — implement the handler instead.
- At narrow viewports the global header scrolls horizontally (`overflow-x: auto`) rather
  than wrapping; the inner `.nav-links` deliberately overflow their container. Don't
  "fix" this by allowing the header to break out — it is what keeps the document from
  overflowing at 390px.
