#!/usr/bin/env python3
"""Audit how much of Gitea's surface Codez implements.

Gitea is ~510k lines of Go across four independent surfaces. Codez ports one
of them (the v1 REST API). This script measures each surface so the gap is a
number rather than an impression.
"""
import re
import subprocess
from pathlib import Path

ROOT = Path("/workspace/project/codez")
GITEA = ROOT / "gitea"


def count(pattern, path, flags=0):
    total = 0
    for f in Path(path).rglob("*.go"):
        total += len(re.findall(pattern, f.read_text(errors="ignore"), flags))
    return total


def files(path, suffix=None):
    return [f for f in Path(path).rglob("*" + (suffix or "")) if f.is_file()]


def loc(path, suffix):
    return sum(
        len(f.read_text(errors="ignore").splitlines()) for f in files(path, suffix)
    )


rows = []

# 1. v1 REST API — the surface Codez ports.
ops = len(re.findall(r"Op \{", (ROOT / "crates/backend/src/api/ops.rs").read_text()))
v1_routes = count(r"\bm\.(Get|Post|Put|Patch|Delete|Combo)\(", GITEA / "routers/api/v1")
rows.append(("v1 REST API (OpenAPI spec)", f"{ops} ops", "ROUTED", ""))

# 2. Package registry — separate router, NOT in the v1 spec.
pkg_routes = count(r"\br\.(Get|Post|Put|Patch|Delete)\(", GITEA / "routers/api/packages")
pkg_eco = len([d for d in (GITEA / "routers/api/packages").iterdir() if d.is_dir()])
rows.append(
    (
        "Package registry protocol",
        f"{pkg_routes} routes / {pkg_eco} ecosystems",
        "1 ecosystem (npm)",
        "npm only",
    )
)

# 3. Web SSR routes + templates.
web_routes = count(r"\bm\.(Get|Post|Put|Patch|Delete|Combo)\(", GITEA / "routers/web")
tmpl = len(files(GITEA / "templates", ".tmpl"))
rows.append(
    ("Web SSR routes + templates", f"{web_routes} routes / {tmpl} templates", "n/a", "SPA instead")
)

# 4. Git smart HTTP + LFS.
git_proto = count(r'"(?:info/refs|git-upload-pack|git-receive-pack)"', GITEA)
lfs = len(files(GITEA / "services/lfs", ".go"))
rows.append(
    ("Git smart-HTTP protocol", f"{git_proto} refs", "NO", "no real git storage")
)
rows.append(("Git LFS", f"{lfs} files", "NO", "not implemented"))

# 5. Private/internal + cmd.
private = count(r"\bm\.(Get|Post|Put|Patch|Delete|Combo)\(", GITEA / "routers/private")
rows.append(("Private/internal routes", f"{private} routes", "NO", "server-to-server"))

print(f"{'SURFACE':<34} {'GITEA':<34} {'CODEZ':<20} NOTE")
print("-" * 110)
for name, g, c, note in rows:
    print(f"{name:<34} {g:<34} {c:<20} {note}")

print()
go_files = len(files(GITEA, ".go"))
go_loc = loc(GITEA, ".go")
rs_loc = loc(ROOT / "crates", ".rs")
fe_routes = len(re.findall(r"path!?\(\"", (ROOT / "crates/frontend/src/main.rs").read_text()))
print(f"Gitea Go:   {go_files} files / {go_loc:,} lines")
print(f"Codez Rs:  {rs_loc:,} lines across 3 crates, {fe_routes} SPA routes")
print(f"Ratio:      {rs_loc / go_loc:.1%} of Gitea's Go by line count")
