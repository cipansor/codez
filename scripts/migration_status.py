#!/usr/bin/env python3
"""Report Gitea -> Codez migration progress, area by area.

Grounds MIGRATION.md in real file counts instead of hand-maintained lists, so
the numbers cannot silently drift as the port proceeds.
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# (upstream area, Rust target, upstream file glob)
AREAS = [
    ("modules/util", "crates/backend/src/util", "*.go"),
    ("modules/container", "crates/backend/src/container", "*.go"),
    ("modules/consts", "crates/backend/src/consts", "*.go"),
    ("modules/cachegroup", "crates/backend/src/cachegroup", "*.go"),
    ("modules/regexplru", "crates/backend/src/regexplru", "*.go"),
    ("modules/secret", "crates/backend/src/secret", "*.go"),
    ("modules/tempdir", "crates/backend/src/tempdir", "*.go"),
    ("modules/uri", "crates/backend/src/uri", "*.go"),
    ("modules/user", "crates/backend/src/user", "*.go"),
    ("modules/structs", "crates/shared/src", "*.go"),
    ("modules/setting", "crates/backend/src/config", "*.go"),
    ("modules/markup", "crates/backend/src/markup", "*.go"),
    ("modules/git", "crates/backend/src/git", "*.go"),
    ("models", "crates/backend/src/models", "*.go"),
    ("services", "crates/backend/src/services", "*.go"),
    ("routers/api", "crates/backend/src/api", "*.go"),
    ("routers/web", "crates/backend/src/web", "*.go"),
    ("web_src", "crates/frontend/src", "*.ts"),
    ("templates", "crates/frontend/src/pages", "*.tmpl"),
]


def count_files(base: Path, pattern: str) -> int:
    if not base.is_dir():
        return 0
    return sum(1 for p in base.rglob(pattern) if p.is_file())


def count_upstream(base: Path, pattern: str) -> int:
    """Count upstream files, ignoring Go tests.

    Ported tests live inline in the Rust module they cover, so a `_test.go`
    file has no separate Rust counterpart; counting them would understate
    progress.
    """
    if not base.is_dir():
        return 0
    return sum(
        1 for p in base.rglob(pattern) if p.is_file() and not p.name.endswith("_test.go")
    )


def main() -> int:
    rows = []
    for src_rel, dst_rel, pattern in AREAS:
        upstream = count_upstream(ROOT / src_rel, pattern)
        ported = count_files(ROOT / dst_rel, "*.rs")
        rows.append((src_rel, dst_rel, upstream, ported))

    width = max(len(r[0]) for r in rows) + 2
    print(f"{'upstream area':<{width}}{'rust target':<38}{'upstream':>9}{'ported':>8}{'pct':>7}")
    print("-" * (width + 62))
    for src_rel, dst_rel, upstream, ported in rows:
        pct = f"{100 * ported / upstream:.1f}%" if upstream else "n/a"
        print(f"{src_rel:<{width}}{dst_rel:<38}{upstream:>9}{ported:>8}{pct:>7}")

    total_up = sum(r[2] for r in rows)
    total_port = sum(r[3] for r in rows)
    pct = f"{100 * total_port / total_up:.1f}%" if total_up else "n/a"
    print("-" * (width + 62))
    print(f"{'TOTAL':<{width}}{'':<38}{total_up:>9}{total_port:>8}{pct:>7}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
