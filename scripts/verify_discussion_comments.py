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

def test_discussion_comments():
    print("Verifying Discussion Comments Endpoints...")
    repo_owner = "codez-org"
    repo_name = "codez"

    try:
        # 1. Create Discussion
        print("\nTesting POST /repos/:owner/:repo/discussions...")
        payload = {
            "title": "Discussion for Comments",
            "body": "This is a discussion to test comments",
            "category": "Ideas"
        }
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/discussions", json=payload)
        r.raise_for_status()
        discussion = r.json()
        print(f"Created Discussion: {discussion['id']}")
        disc_id = discussion['id']

        # 2. Create Comment
        print(f"\nTesting POST /repos/:owner/:repo/discussions/{disc_id}/comments...")
        comment_payload = {
            "body": "This is a comment"
        }
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/discussions/{disc_id}/comments", json=comment_payload)
        r.raise_for_status()
        comment = r.json()
        print(f"Created Comment: {comment['id']} - {comment['body']}")

        if comment['body'] != "This is a comment":
            print("FAILURE: Wrong comment body")
            sys.exit(1)

        # 3. List Comments
        print(f"\nTesting GET /repos/:owner/:repo/discussions/{disc_id}/comments...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/discussions/{disc_id}/comments")
        r.raise_for_status()
        comments = r.json()
        print(f"Comments: {[c['body'] for c in comments]}")

        found = any(c['id'] == comment['id'] for c in comments)
        if found:
            print("SUCCESS: Comment found in list")
        else:
            print("FAILURE: Comment not found in list")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing discussion comments: {e}")
        sys.exit(1)

if __name__ == "__main__":
    wait_for_server()
    test_discussion_comments()
