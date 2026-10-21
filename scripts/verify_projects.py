import requests
import json
import sys

BASE_URL = "http://127.0.0.1:3000/api/v1"

def test_projects():
    print("Verifying Project Endpoints...")
    repo_owner = "codez-org"
    repo_name = "codez"

    try:
        # 1. Create Project
        print("\nTesting POST /repos/:owner/:repo/projects...")
        project_payload = {"title": "Kanban Board", "description": "Tracking work"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/projects", json=project_payload)
        r.raise_for_status()
        project = r.json()
        print(f"Created Project: {project['id']} - {project['title']}")
        project_id = project['id']

        # 2. Create Column
        print("\nTesting POST /repos/:owner/:repo/projects/:id/columns...")
        col_payload = {"title": "To Do"}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/projects/{project_id}/columns", json=col_payload)
        r.raise_for_status()
        column = r.json()
        print(f"Created Column: {column['id']} - {column['title']}")
        col_id = column['id']

        # 3. Create Card
        print("\nTesting POST /repos/:owner/:repo/projects/columns/:id/cards...")
        card_payload = {"content": "Task A", "note": None, "issue_id": None}
        r = requests.post(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/projects/columns/{col_id}/cards", json=card_payload)
        r.raise_for_status()
        card = r.json()
        print(f"Created Card: {card['id']} - {card['content']}")

        # 4. List Projects
        print("\nTesting GET /repos/:owner/:repo/projects...")
        r = requests.get(f"{BASE_URL}/repos/{repo_owner}/{repo_name}/projects")
        r.raise_for_status()
        projects = r.json()
        found = any(p['id'] == project_id for p in projects)
        if found:
            print("SUCCESS: Project found in list")
        else:
            print("FAILURE: Project not found")
            sys.exit(1)

    except Exception as e:
        print(f"Error testing projects: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_projects()
