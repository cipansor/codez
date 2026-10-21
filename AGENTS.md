# AGENTS.md

Guidance for AI agents and contributors working in this repository.

## What this repository is

**Codez** — a Rust reimplementation of Gitea (Axum backend, Leptos/WASM SPA,
`shared` crate of typed DTOs).

The repository currently contains the **complete upstream Gitea source tree**
(6,375 files, Go module `gitea.dev`) at the repository root. That tree is the
migration baseline and the specification: every Rust module is a port of a
specific upstream Go file. Do not delete upstream files until their
replacements exist and are tested.

`MIGRATION.md` describes the porting method and the current per-area status.

## Layout

```
crates/backend     Axum API server (Rust)
crates/frontend    Leptos SPA (Rust/WASM)
crates/shared      DTOs and helpers shared by both

modules/ services/ models/ routers/ ...   upstream Gitea (Go) — the baseline
templates/  web_src/                      upstream Gitea (HTML/TS/CSS)

scripts/migration_status.py   regenerates the per-area progress table
```

Rust module paths mirror the upstream ones, so a ported file is easy to find
from its original: `modules/util/string.go` -> `crates/backend/src/util/string.rs`.

## Build, test, lint

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
python3 scripts/migration_status.py
```

Keep clippy warning-free with `-D warnings`, and do not add
`#[allow(dead_code)]` to silence it — delete the unused helper instead.

There is no Go toolchain in this environment, so the upstream tree cannot be
built or tested here. It is reference material; read it, do not try to run it.

## How to port a file

1. Read the upstream Go file and its `_test.go` sibling.
2. Create the Rust module at the mirrored path.
3. Port each function, giving it a doc comment that names its Go counterpart,
   for example `/// util.ToSnakeCase`.
4. **Transcribe the upstream test cases verbatim** into a `#[cfg(test)]` module.
   Do not invent tests — the upstream table-driven cases are the specification,
   and they are what catch port-specific bugs.
5. Run `cargo test`, `cargo clippy`, and `cargo fmt`.

### Porting rules

- Reproduce upstream behaviour, including its quirks. If a test pins something
  surprising, keep it and add a comment explaining why. A behaviour change
  belongs in its own commit with its own justification.
- Never delete an upstream file in the same commit that adds its port.
- Update `MIGRATION.md` when an area is finished.

### Traps already hit (do not rediscover these)

- **Go indexes strings by byte, not rune.** `truncate_string_bytes("aé", 3)`
  returns `"aé"`, not `"a"`. Never build a `String` by mapping bytes to `char`
  — that corrupts multi-byte UTF-8 (this was a real bug in the first slice).
- **`path.Clean` / `path.Join` semantics must match exactly**, because the
  results are used as storage keys. `util::path::clean_posix_path` reproduces
  them; do not swap it for `std::path`.
- **`go-humanize.ParseBytes` rejects a bare number** with no unit suffix, and
  `FormatByteSize` uses binary units (`KiB`) for a value like `512 * 1024`.
- **`unicode.Is(M/Cf/Cs/Z)` has no std equivalent.** `util/truncate.rs` carries
  a hand-rolled category table for the display-width guess.
- **When a ported test fails, check the expectation against the Go source
  first.** In the first slice, four of five failures were wrong expectations
  and only one was a genuine port bug.

### Areas that are replacements, not translations

These have no line-by-line Rust equivalent; port them by design and track them
separately:

| Upstream | Replaced by |
| --- | --- |
| `templates/` (601 `.tmpl`) | Leptos pages — the templates say what to render |
| `web_src/` (411 TS/CSS) | Leptos components + `style.css` |
| `modules/git/` (149 files) | a git library + smart-HTTP endpoints; all new code |
| `models/db` (xorm ORM) | an explicit storage layer |
| `modules/indexer` | Bleve/Elasticsearch wrappers |

## Repository notes

- This repository is **public**. Never commit `.openhands/` — it holds
  unreviewed security findings and a cross-project memory index. Stage with
  `git add -A -- ':!.openhands'`.
- CI (`.github/workflows/ci.yml`) runs `cargo fmt --check`, `cargo clippy`,
  `cargo test`, and a frontend build. Actions must be pinned to a full-length
  commit SHA or the job fails at "Set up job".
- A missing entity is a `404` with a Gitea-shaped error body, not `200` with a
  `null` body. Follow the upstream convention when adding handlers.
