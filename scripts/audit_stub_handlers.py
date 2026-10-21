#!/usr/bin/env python3
"""Find handlers that ignore a store that already holds their data.

The commit-status bug (a hardcoded `[]` next to a populated store) is a class,
not a one-off: a handler can be routed, return 200, and still be wrong because
it answers from a constant instead of the state. This flags every function body
that returns a constant empty collection while the matching store is non-empty
in the seed.

It is a heuristic - it prints candidates for a human to read, and does not fail
the build.
"""
import re
from pathlib import Path

ROOT = Path("/workspace/project/codez")
API = ROOT / "crates/backend/src/api"
STATE = (ROOT / "crates/backend/src/state.rs").read_text()

# Store accessor -> the DTO collection it would answer with.
STORES = {
    "commit_statuses": "CommitStatus",
    "reviews": "PullReview",
    "reactions": "Reaction",
    "attachments": "Attachment",
    "notifications": "NotificationThread",
    "packages": "Package",
    "hooks": "Hook",
    "projects": "Project",
    "badges": "Badge",
    "runners": "ActionRunner",
    "branches": "Branch",
    "commits": "Commit",
    "files": "ContentsResponse",
    "labels": "Label",
    "milestones": "Milestone",
    "releases": "Release",
    "tokens": "AccessToken",
    "keys": "PublicKey",
    "org_blocks": "User",
    "follows": "User",
    "blocks": "User",
    "teams": "Team",
}

# Only stores actually populated by the seed can be "wrongly ignored".
seeded = set()
seed = (ROOT / "crates/backend/src/seed.rs").read_text()
for accessor in STORES:
    singular = accessor.rstrip("s")
    if re.search(rf"{accessor}\b.*push|push.*{accessor}", seed) or re.search(
        rf"\b{singular}\b", seed
    ):
        seeded.add(accessor)

EMPTY_RETURN = re.compile(
    r"respond::ok\(\s*(?:Vec::<[^>]*>::new\(\)|vec!\[\]|Vec::new\(\))\s*\)"
    r"|respond::ok\(\s*serde_json::json!\(\s*\[\s*\]\s*\)\s*\)"
    r"|respond::ok\(\s*Vec::<serde_json::Value>::new\(\)\s*\)"
)

findings = []
for path in sorted(API.glob("*.rs")):
    text = path.read_text()
    lines = text.splitlines()
    # Walk fn bodies by brace depth so a stub is attributed to its own fn.
    fn_re = re.compile(r"^(?:pub(?:\(super\))? )?(?:async )?fn (\w+)")
    current = None
    depth = 0
    body_start = 0
    for i, line in enumerate(lines):
        m = fn_re.match(line)
        if m and depth == 0:
            current = m.group(1)
            body_start = i
        depth += line.count("{") - line.count("}")
        if current and EMPTY_RETURN.search(line):
            # The store this handler should have read, if any.
            body = "\n".join(lines[body_start : i + 1])
            used = {a for a in STORES if f"{a}()" in body or f"{a}_mut()" in body}
            findings.append((path.name, i + 1, current, line.strip(), used))
        if depth <= 0 and current:
            current = None

print(f"{'FILE':<16} {'LINE':>5}  {'FUNCTION':<34} STORE READ")
print("-" * 100)
for f, ln, fn, code, used in findings:
    print(f"{f:<16} {ln:>5}  {fn:<34} {','.join(sorted(used)) or '-'}")
print()
print(f"{len(findings)} constant-empty returns across {len({f for f, *_ in findings})} files")
print("Heuristic only: a constant is correct when no store holds the data yet.")

# Second class: a handler that accepts a write verb and reports success but
# never touches a `*_mut()` store. The reaction handlers did this - POST
# answered `201`/`204` and the following GET still returned `[]` - which no
# "did the write return 2xx" test can see.
WRITE_VERB = re.compile(r"Method::(?:POST|PATCH|PUT|DELETE)")
SUCCESS = re.compile(r"respond::(?:created|ok|no_content)\(")
MUTATION = re.compile(r"\.\w+_mut\(\)")

accepted = []
for path in sorted(API.glob("*.rs")):
    text = path.read_text()
    lines = text.splitlines()
    fn_re = re.compile(r"^(?:pub(?:\(super\))? )?(?:async )?fn (\w+)")
    current = None
    depth = 0
    body_start = 0
    for i, line in enumerate(lines):
        m = fn_re.match(line)
        if m and depth == 0:
            current = m.group(1)
            body_start = i
        depth += line.count("{") - line.count("}")
        if depth <= 0 and current:
            body = "\n".join(lines[body_start : i + 1])
            if (
                WRITE_VERB.search(body)
                and SUCCESS.search(body)
                and not MUTATION.search(body)
            ):
                accepted.append((path.name, body_start + 1, current))
            current = None

print()
print("Writes that report success without mutating a store (heuristic):")
print(f"{'FILE':<16} {'LINE':>5}  FUNCTION")
print("-" * 100)
for f, ln, fn in accepted:
    print(f"{f:<16} {ln:>5}  {fn}")
print()
print(f"{len(accepted)} candidates across {len({f for f, *_ in accepted})} files")
print("A delegating handler is a false positive; check where it forwards to.")
