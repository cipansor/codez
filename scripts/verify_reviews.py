import requests
import json
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1"

def test_reviews():
    print("Verifying Review Endpoints...")
    repo_owner = "codez-org"
    repo_name = "codez"

    try:
        # 1. Create PR
        print("\nTesting POST /repos/:owner/:repo/pulls...")
        pr_payload = {"title": "Review PR", "body": "Please review", "head": "feature", "base": "main"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/pulls", json=pr_payload)
        r.raise_for_status()
        pr = r.json()
        print(f"Created PR: {pr['number']} - {pr['title']}")
        pr_number = pr['number']

        # 2. Post Review
        print(f"\nTesting POST /repos/:owner/:repo/pulls/{pr_number}/reviews...")
        review_payload = {"body": "LGTM", "event": "APPROVE"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/pulls/{pr_number}/reviews", json=review_payload)
        r.raise_for_status()
        print(f"POST reviews -> {r.status_code}")

        # 3. List Reviews
        print(f"\nTesting GET /repos/:owner/:repo/pulls/{pr_number}/reviews...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/pulls/{pr_number}/reviews")
        r.raise_for_status()
        reviews = r.json()
        assert isinstance(reviews, list), f"expected a list, got {type(reviews).__name__}"
        print(f"SUCCESS: reviews endpoint returned a list ({len(reviews)} item(s))")

    except Exception as e:
        print(f"Error testing reviews: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_reviews()
