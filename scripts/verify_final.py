import requests
import json
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1"

import time

def test_final():
    print("Verifying Final Enhancements...")
    repo_owner = "codez-org"
    repo_name = f"final-repo-{int(time.time())}"

    try:
        # 1. Create Repo
        print("\nTesting POST /user/repos...")
        repo_payload = {"name": repo_name, "auto_init": True, "private": False}
        r = requests.post(f"{BASE_URL}/user/repos", json=repo_payload)
        r.raise_for_status()
        print("Repo created")

        # 2. Update Topics
        print(f"\nTesting PUT /repos/{repo_owner}/{repo_name}/topics...")
        topics_payload = {"topics": ["rust", "demo", "final"]}
        r = requests.put(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/topics", json=topics_payload)
        r.raise_for_status()
        print("Topics updated")

        # 3. List Topics
        print(f"\nTesting GET /repos/{repo_owner}/{repo_name}/topics...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/topics")
        r.raise_for_status()
        topics = r.json()
        print(f"Topics: {[t['name'] for t in topics]}")
        if len(topics) == 3:
            print("SUCCESS: Topics listed")
        else:
            print("FAILURE: Wrong topic count")
            sys.exit(1)

        # 4. Create Issue (Frontend logic does this via API)
        print(f"\nTesting POST /repos/{repo_owner}/{repo_name}/issues...")
        issue_payload = {"title": "Final Check", "body": "Everything looks good"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/issues", json=issue_payload)
        r.raise_for_status()
        issue = r.json()
        print(f"Created Issue: {issue['number']}")

    except Exception as e:
        print(f"Error testing final: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_final()
