import requests
import json
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1"

def test_backend():
    print("Verifying Backend Endpoints...")

    # 1. List issues across the instance
    # Gitea exposes no /user/issues endpoint; the spec equivalent is the
    # cross-repo issue search at /repos/issues/search.
    print("\nTesting GET /repos/issues/search...")
    try:
        repo_owner = "codez-org"
        repo_name = "codez"
        issue_payload = {"title": "Verification Issue", "body": "To test list"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/issues", json=issue_payload)
        r.raise_for_status()
        issue = r.json()
        print(f"Created Issue: {issue['id']}")

        # Search for the issue we just created.
        r = requests.get(f"{BASE_URL}/repos/issues/search?state=open&type=issues")
        r.raise_for_status()
        issues = r.json()
        print(f"Issue Search Count: {len(issues)}")

        found = any(i['title'] == "Verification Issue" for i in issues)
        if found:
            print("SUCCESS: Found created issue in search results")
        else:
            print("FAILURE: Did not find created issue")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing issue search: {e}")
        sys.exit(1)

    # 2. List pull requests across the instance (search with type=pulls).
    print("\nTesting GET /repos/issues/search?type=pulls...")
    try:
        pr_payload = {"title": "Verification PR", "body": "Body", "head": "feature", "base": "main"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/pulls", json=pr_payload)
        r.raise_for_status()
        pr = r.json()
        print(f"Created PR: {pr['id']}")

        r = requests.get(f"{BASE_URL}/repos/issues/search?state=open&type=pulls")
        r.raise_for_status()
        pulls = r.json()
        print(f"Pull Search Count: {len(pulls)}")

        found = any(p['title'] == "Verification PR" for p in pulls)
        if found:
            print("SUCCESS: Found created PR in search results")
        else:
            print("FAILURE: Did not find created PR")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing pull search: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_backend()
