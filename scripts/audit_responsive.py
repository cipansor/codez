"""Audit every Codez route for horizontal overflow at mobile/tablet/desktop widths.

Usage:
    python scripts/audit_responsive.py

Requires a running backend serving the built frontend at
``http://127.0.0.1:3000`` (override with ``CODEZ_BASE_URL``) and the Playwright
browsers installed. Exits non-zero if any route overflows the viewport.
"""

import os
import sys
from pathlib import Path

from playwright.sync_api import sync_playwright

sys.path.insert(0, str(Path(__file__).resolve().parent))
from capture_screenshots import ROUTES  # noqa: E402

BASE_URL = os.environ.get("CODEZ_BASE_URL", "http://127.0.0.1:3000")
WIDTHS = (390, 768, 1440)

# Report at most this many offending elements per route.
MAX_OFFENDERS = 3

_MEASURE = """() => {
    const de = document.documentElement;
    const vw = de.clientWidth;
    const over = [];
    for (const el of document.querySelectorAll('body *')) {
        const r = el.getBoundingClientRect();
        if (r.width === 0 || r.height === 0) continue;
        if (r.right > vw + 1 || r.left < -1) {
            const cs = getComputedStyle(el);
            if (cs.position === 'fixed' || cs.overflowX === 'auto' ||
                cs.overflowX === 'scroll') continue;
            over.push(el.tagName + '.' + (el.className || '').toString().slice(0, 60)
                      + ' right=' + Math.round(r.right));
            if (over.length >= 3) break;
        }
    }
    return {scrollW: de.scrollWidth, clientW: vw, over};
}"""


def audit() -> int:
    issues = 0
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        for width in WIDTHS:
            ctx = browser.new_context(viewport={"width": width, "height": 900})
            page = ctx.new_page()
            print(f"=== width {width} ===")
            for name, path in ROUTES:
                try:
                    page.goto(BASE_URL + path, wait_until="networkidle", timeout=20000)
                except Exception as exc:  # noqa: BLE001 - report and keep auditing
                    print(f"  nav-error {name} ({path}): {exc}")
                    continue
                page.wait_for_timeout(250)
                res = page.evaluate(_MEASURE)
                if res["scrollW"] > res["clientW"] + 1:
                    issues += 1
                    print(f"  OVERFLOW {name}: scrollW={res['scrollW']} clientW={res['clientW']}")
                    for offender in res["over"][:MAX_OFFENDERS]:
                        print(f"      {offender}")
            ctx.close()
        browser.close()
    print(f"\nroutes with layout issues: {issues}")
    return 1 if issues else 0


if __name__ == "__main__":
    raise SystemExit(audit())
