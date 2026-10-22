import requests
import json
import sys
import time

BASE_URL = "http://127.0.0.1:3000/api/v1"

def test_webhooks():
    print("Verifying Webhook Endpoints...")
    repo_owner = "codez-org"
    repo_name = "codez"

    try:
        # 1. Create Webhook
        print("\nTesting POST /repos/:owner/:repo/hooks...")
        hook_payload = {"url": "http://example.com/webhook", "events": ["issues"], "active": True}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/hooks", json=hook_payload)
        r.raise_for_status()
        hook = r.json()
        print(f"Created Hook: {hook['id']} - {hook['url']}")
        hook_id = hook['id']

        # 2. Trigger Event (Create Issue)
        print("\nTriggering 'issues' event by creating issue...")
        issue_payload = {"title": "Webhook Trigger", "body": "test"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/issues", json=issue_payload)
        r.raise_for_status()
        print("Issue Created")

        time.sleep(2)

        # 3. Check Deliveries
        print(f"\nTesting GET /repos/:owner/:repo/hooks/{hook_id}/deliveries...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/hooks/{hook_id}/deliveries")
        r.raise_for_status()
        deliveries = r.json()
        print(f"Deliveries count: {len(deliveries)}")

        found = any(d['event'] == "issues" for d in deliveries)
        if found:
            print("SUCCESS: Delivery found")
        else:
            print("FAILURE: Delivery not found")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing webhooks: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_webhooks()
