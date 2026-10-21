import { expect, test } from '@playwright/test';
import { waitForApp } from './helpers';

const REPO = '/repos/codez-org/codez';

/**
 * Read flows over the ported Gitea API surface: each page must render the data
 * the backend actually serves for the demo seed, not a placeholder or an empty
 * state. These are the smoke tests for the API -> SPA wiring.
 */

test('dashboard lists the signed-in user repositories', async ({ page }) => {
  await page.goto('/');
  await waitForApp(page);
  await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
  await expect(page.locator('body')).toContainText('alice/notes');
});

test('repository overview shows description, languages and commits', async ({ page }) => {
  await page.goto(REPO);
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('A Gitea-compatible code hosting platform in Rust');
  await expect(page.locator('body')).toContainText('Rust');
  await expect(page.locator('body')).toContainText('Port the Gitea API surface to Axum');
});

test('issue list renders seeded issues', async ({ page }) => {
  await page.goto(`${REPO}/issues`);
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('Serve the full /api/v1 surface');
});

test('pull request list renders the seeded pull request', async ({ page }) => {
  await page.goto(`${REPO}/pulls`);
  await waitForApp(page);
  await expect(page.locator('.pr-list, body')).toContainText('main');
});

test('release list renders the seeded release', async ({ page }) => {
  await page.goto(`${REPO}/releases`);
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('Initial port');
});

test('organization profile lists members and repositories', async ({ page }) => {
  await page.goto('/orgs/codez-org');
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('codez-org/codez');
  await expect(page.locator('body')).toContainText('alice');
});

test('user profile lists repositories and organizations', async ({ page }) => {
  await page.goto('/users/alice');
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('Alice Liddell');
  await expect(page.locator('body')).toContainText('alice/notes');
});

test('admin dashboard shows instance counters', async ({ page }) => {
  await page.goto('/admin');
  await waitForApp(page);
  await expect(page.getByRole('heading', { name: 'Administration' })).toBeVisible();
  await expect(page.locator('body')).toContainText('Users');
});

test('code browser renders a seeded file', async ({ page }) => {
  await page.goto(`${REPO}/src/README.md`);
  await waitForApp(page);
  await expect(page.locator('body')).toContainText('A Gitea-compatible platform in Rust');
});
