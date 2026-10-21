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

`python3 scripts/migration_status.py` prints the live table. It counts upstream
files (Go tests excluded, since ported tests live inline in the Rust module they
cover) against `*.rs` files in the target directory. As of this slice:

```
upstream area       rust target                            upstream  ported    pct
----------------------------------------------------------------------------------
modules/util        crates/backend/src/util                      29      26  89.7%
modules/container   crates/backend/src/container                  2       3 150.0%
modules/consts      crates/backend/src/consts                     2       3 150.0%
modules/cachegroup  crates/backend/src/cachegroup                 1       1 100.0%
modules/regexplru   crates/backend/src/regexplru                  1       1 100.0%
modules/secret      crates/backend/src/secret                     1       1 100.0%
modules/tempdir     crates/backend/src/tempdir                    1       1 100.0%
modules/uri         crates/backend/src/uri                        1       1 100.0%
modules/user        crates/backend/src/user                       1       1 100.0%
modules/structs     crates/shared/src                            58       1   1.7%
modules/setting     crates/backend/src/config                    60       0   0.0%
modules/markup      crates/backend/src/markup                    47       0   0.0%
modules/git         crates/backend/src/git                       92       0   0.0%
models              crates/backend/src/models                   248       0   0.0%
services            crates/backend/src/services                 373       0   0.0%
routers/api         crates/backend/src/api                      173       0   0.0%
routers/web         crates/backend/src/web                      195       0   0.0%
web_src             crates/frontend/src                         219       1   0.5%
templates           crates/frontend/src/pages                   587       0   0.0%
----------------------------------------------------------------------------------
TOTAL                                                          2091      40   1.9%
```

`modules/util` shows 26 of 29 because the count is per file: the port merges
`file_unix.go` and `file_windows.go` into one `util/file_mode.rs`, and
`util/misc.rs` covers `util.go`. A ported area may therefore exceed 100% when a
single Rust module covers more than one upstream file.

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

### Ported: `modules/container`

| Go file | Rust file | Notes |
| --- | --- | --- |
| `set.go` | `container/set.rs` | A `HashSet` newtype; `Union` needs `T: Clone` |
| `filter.go` | `container/filter.rs` | First occurrence wins, matching the map-based dedup |

### Ported: small leaf modules

| Upstream | Rust file | Notes |
| --- | --- | --- |
| `modules/consts/runtime.go` | `consts/is_windows.rs` | `cfg!(windows)` replaces the `runtime.GOOS` check |
| `modules/consts/asymkey.go` | `consts/asymkey.rs` | The RSA/EC bit floors and defaults |
| `modules/cachegroup/cachegroup.go` | `cachegroup/mod.rs` | An enum instead of loose strings, so a typo is a compile error |
| `modules/regexplru/regexplru.go` | `regexplru/mod.rs` | `Arc<Regex>` because `regex::Regex` is not `Clone`; a small LRU replaces `golang-lru` |
| `modules/secret/secret.go` | `secret/mod.rs` | AES-CFB keyed by `SHA-256(key)`; the wire format is pinned by a Go-generated vector test |
| `modules/tempdir/tempdir.go` | `tempdir/mod.rs` | `TempFile` carries the path, since `std::fs::File` has no `Name()` |
| `modules/uri/uri.go` | `uri/mod.rs` | `reqwest::blocking` replaces `net/http`; the caller supplies the client for the SSRF-confinement case |
| `modules/user/user.go` | `user/mod.rs` | The `whoami` crate replaces `os/user`; the `USER`/`USERNAME` fallback is kept |

Two ported details are worth calling out because they were caught by tests
rather than by reading:

- `secret.EncryptSecret` hashes the key with SHA-256 and passes 32 bytes to
  `aes.NewCipher`, which selects **AES-256**; a first cut that truncated the key
  to 16 bytes decrypted to garbage. The Go-generated ciphertext test now pins
  this.
- `tempdir`'s upstream test uses `strings.HasPrefix`, so `foo0` is a valid
  prefix of `foo`; `PathBuf::starts_with` compares path *components* and
  rejects it. The port compares the rendered paths.

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
