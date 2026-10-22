import requests
import json
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1"

def test_pkgs_actions():
    print("Verifying Packages and Actions Endpoints...")
    repo_owner = "codez-org"
    repo_name = "codez"

    try:
        # 1. Upload Package
        print("\nTesting POST /packages/:owner...")
        pkg_payload = {"name": "test-pkg", "version": "1.0.0", "package_type": "npm"}
        r = requests.post(f"{BASE_URL}/packages/{repo_owner}", json=pkg_payload)
        r.raise_for_status()
        pkg = r.json()
        print(f"Uploaded Package: {pkg['id']} - {pkg['name']} v{pkg['version']}")

        # 2. Get Package Detail
        print(f"\nTesting GET /packages/:owner/:type/:name/:version...")
        r = requests.get(f"{BASE_URL}/packages/{repo_owner}/npm/test-pkg/1.0.0")
        r.raise_for_status()
        pkg_fetched = r.json()
        if pkg_fetched['name'] == "test-pkg":
            print("SUCCESS: Fetched package")
        else:
            print("FAILURE: Fetched wrong package")
            sys.exit(1)

        # 3. Trigger Workflow
        # Use ID 1 (mock workflow from handler)
        workflow_id = 1
        print(f"\nTesting POST /repos/:owner/:repo/actions/workflows/{workflow_id}/runs...")
        run_payload = {"workflow_id": workflow_id, "ref_name": "main"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/actions/workflows/{workflow_id}/runs", json=run_payload)
        r.raise_for_status()
        run = r.json()
        print(f"Triggered Run: {run['id']} - {run['status']}")

        # 4. List Workflow Runs
        print(f"\nTesting GET /repos/:owner/:repo/actions/workflows/{workflow_id}/runs...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/actions/workflows/{workflow_id}/runs")
        r.raise_for_status()
        runs = r.json()
        print(f"Runs count: {len(runs)}")

        found = any(r['id'] == run['id'] for r in runs)
        if found:
            print("SUCCESS: Run found in list")
        else:
            print("FAILURE: Run not found")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing packages/actions: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_pkgs_actions()
