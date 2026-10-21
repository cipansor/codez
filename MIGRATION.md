# Migrating Gitea to Codez

This repository holds the complete upstream Gitea source tree (6,375 files) as
the migration baseline. The goal is to reimplement it in Rust as **Codez**:
an Axum HTTP backend, a Leptos/WASM SPA frontend, and a `shared` crate of typed
DTOs.

Nothing is deleted from the upstream tree until its replacement exists and is
tested, so every area can be diffed against the original while it is being
ported.

## How the port is organised

Each Rust module corresponds to one upstream file, and each ported function
carries a doc comment naming its Go counterpart (for example
`/// util.ToSnakeCase`). That keeps the mapping searchable in both directions
and makes review mechanical: open the Go file, open the Rust file, compare.

Tests are transcribed from the upstream Go tests rather than invented. Where
upstream has a table-driven case list, the Rust test carries the same cases
verbatim. This is the only reliable way to catch the semantic differences that
show up in a port: byte-versus-rune indexing, `nil` versus empty slices, and
error-as-value versus `Result`.

Run the ported tests with:

```bash
cargo test --workspace
```

## Current status

`python3 scripts/migration_status.py` prints the live table. As of the first
slice:

```
upstream area    rust target                            upstream  ported    pct
-------------------------------------------------------------------------------
modules/util     crates/backend/src/util                      56      10  17.9%
modules/structs  crates/shared/src                            60       1   1.7%
modules/setting  crates/backend/src/config                    86       0   0.0%
modules/markup   crates/backend/src/markup                    76       0   0.0%
modules/git      crates/backend/src/git                      149       0   0.0%
models           crates/backend/src/models                   390       0   0.0%
services         crates/backend/src/services                 560       0   0.0%
routers/api      crates/backend/src/api                      190       0   0.0%
routers/web      crates/backend/src/web                      236       0   0.0%
web_src          crates/frontend/src                         219       1   0.5%
templates        crates/frontend/src/pages                   587       0   0.0%
-------------------------------------------------------------------------------
TOTAL                                                       2609      12   0.5%
```

### Ported: `modules/util`

| Go file | Rust file | Notes |
| --- | --- | --- |
| `path.go` | `util/path.rs` | `path.Clean`/`path.Join` semantics reproduced, since results are storage keys |
| `string.go` | `util/string.rs` | `ToSnakeCase` is byte-oriented upstream, so it is here too |
| `truncate.go` | `util/truncate.rs` | Width guess kept as a guess; the "never zero" invariant is tested |
| `slice.go` | `util/slice.rs` | |
| `paginate.go` | `util/paginate.rs` | |
| `bytesize.go` | `util/bytesize.rs` | Reimplements `go-humanize` formatting and parsing |
| `filename.go` | `util/filename.rs` | |
| `error.go` | `util/error.rs` | `errorForUser` becomes an explicit `ErrorKind` carrying the status |
| `url.go` | `util/url.rs` | |
| `sanitize.go` | `util/error.rs` | Credential masking shares the module |

## Areas that are replacements, not translations

Some upstream areas have no line-by-line equivalent. These are ported by
design, not by transcription, and should be tracked separately:

- **`templates/` (601 files)** — replaced by the Leptos SPA. The `.tmpl` files
  are the reference for what each page must render.
- **`web_src/` (411 files)** — TypeScript/CSS replaced by Leptos components.
  `style.css` is the single design system.
- **`modules/git/` (149 files)** — wraps the `git` binary. The Rust equivalent
  is a fresh implementation over a git library plus smart-HTTP endpoints; there
  is no function-level mapping.
- **`models/db/`** — xorm ORM. Replaced by an explicit storage layer; Codez is
  in-memory by default with optional PostgreSQL.
- **`modules/indexer/`** — Bleve/Elasticsearch wrappers.

## Ground rules

- Keep `cargo clippy --workspace --all-targets` warning-free and run
  `cargo fmt --all` before committing.
- A ported function keeps its upstream name in snake_case and its upstream
  doc comment, including the upstream caveats.
- Do not "improve" behaviour while porting. If upstream has a quirk that a test
  pins, reproduce the quirk and note it in a comment; behaviour changes belong
  in their own commit with their own justification.
- Never delete an upstream file in the same commit that adds its port. Delete
  it only once the port is tested and nothing references the original.
