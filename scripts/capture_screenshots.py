"""Capture full-page screenshots of every Codez UI route.

Usage:
    python scripts/capture_screenshots.py [output_dir]

Requires a running backend serving the built frontend (see README) and the
Playwright browsers installed. The routes and the seed resources they point at
mirror ``tests/routes.spec.ts``; keep the two lists in sync.
"""

import os
import sys

from playwright.sync_api import sync_playwright

BASE_URL = os.environ.get("CODEZ_BASE_URL", "http://127.0.0.1:3000")

REPO = "/repos/codez-org/codez"

ROUTES = [
    ("01_dashboard", "/"),
    ("02_explore", "/explore"),
    ("03_repo_detail", REPO),
    ("04_repo_code", f"{REPO}/src/main"),
    ("05_commits", f"{REPO}/commits"),
    ("06_branches", f"{REPO}/branches"),
    ("07_tags", f"{REPO}/tags"),
    ("08_issues", f"{REPO}/issues"),
    ("09_issue_detail", f"{REPO}/issues/1"),
    ("10_pulls", f"{REPO}/pulls"),
    ("11_pull_detail", f"{REPO}/pulls/2"),
    ("12_labels", f"{REPO}/labels"),
    ("13_milestones", f"{REPO}/milestones"),
    ("14_releases", f"{REPO}/releases"),
    ("15_actions", f"{REPO}/actions"),
    ("16_wiki", f"{REPO}/wiki"),
    ("17_pulse", f"{REPO}/pulse"),
    ("18_packages", "/packages/codez-org"),
    ("19_notifications", "/notifications"),
    ("20_search", "/search"),
    ("21_admin", "/admin"),
    ("22_admin_users", "/admin/users"),
    ("23_admin_orgs", "/admin/orgs"),
    ("24_admin_emails", "/admin/emails"),
    ("25_user_profile", "/users/alice"),
    ("26_user_followers", "/users/alice/followers"),
    ("27_user_following", "/users/alice/following"),
    ("28_org_profile", "/orgs/codez-org"),
    ("29_settings", "/settings/profile"),
    ("30_login", "/login"),
    ("31_register", "/register"),
    ("32_create_repo", "/repo/create"),
    ("33_migrate_repo", "/repo/migrate"),
    ("34_create_org", "/org/create"),
    ("35_compare", f"{REPO}/compare"),
    ("36_code_search", f"{REPO}/search"),
    ("37_wiki_edit", f"{REPO}/wiki/pages/Home/edit"),
    ("38_milestone_detail", f"{REPO}/milestones/300"),
    ("39_release_detail", f"{REPO}/releases/500"),
    ("40_repo_settings", f"{REPO}/settings"),
    ("41_file_edit", f"{REPO}/edit/README.md"),
]


def capture(output_dir: str) -> None:
    os.makedirs(output_dir, exist_ok=True)
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_context(viewport={"width": 1440, "height": 1000}).new_page()
        for name, path in ROUTES:
            try:
                page.goto(BASE_URL + path, wait_until="networkidle", timeout=20000)
            except Exception as exc:  # noqa: BLE001 - report and keep capturing
                print(f"navigation error for {path}: {exc}")
            page.wait_for_timeout(600)
            page.screenshot(path=os.path.join(output_dir, f"{name}.png"), full_page=True)
            print(f"saved {name}")
        browser.close()


if __name__ == "__main__":
    capture(sys.argv[1] if len(sys.argv) > 1 else "docs/screenshots")
