import requests
import json
import sys
import time

BASE_URL = "http://127.0.0.1:3000/api/v1"

def wait_for_server():
    print("Waiting for server to start...")
    for _ in range(30):
        try:
            requests.get(f"{BASE_URL}/repos")
            print("Server is up!")
            return
        except requests.exceptions.ConnectionError:
            time.sleep(1)
    print("Server failed to start")
    sys.exit(1)

def test_invalid_discussion_comment():
    print("Verifying Bug Fix 1: Invalid Discussion ID...")
    repo_owner = "codez-org"
    repo_name = "codez"

    # Try to create comment on non-existent discussion ID 9999
    print("\nTesting POST /repos/:owner/:repo/discussions/9999/comments...")
    comment_payload = {
        "body": "This should fail"
    }
    r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/discussions/9999/comments", json=comment_payload)

    if r.status_code == 404:
        print("SUCCESS: Received 404 for invalid discussion ID")
    else:
        print(f"FAILURE: Expected 404, got {r.status_code}")
        sys.exit(1)

if __name__ == "__main__":
    wait_for_server()
    test_invalid_discussion_comment()
