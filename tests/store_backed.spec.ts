import { expect, test } from '@playwright/test';
import { uniqueLabel, waitForApp } from './helpers';

const REPO = '/repos/codez-org/codez';

/**
 * The wiki, the package registry and the Actions run registry are backed by the
 * in-memory stores, so each spec writes through the UI (or the API) and then
 * asserts the observable follow-up rather than a fixed empty payload.
 */

test('a new wiki page appears in the sidebar and renders its content', async ({ page }) => {
  const title = uniqueLabel('WikiPage').replace(/\s+/g, '-');

  await page.goto(`${REPO}/wiki/pages/${title}/edit`);
  await waitForApp(page);

  await page.locator('textarea').fill('Hello from the wiki spec.');
  await page.getByRole('button', { name: 'Save page' }).click();

  // The save navigates to the page view, which must render the stored content.
  await expect(page.locator('.wiki-view')).toContainText('Hello from the wiki spec.', {
    timeout: 10_000,
  });

  // And the page is listed in the sidebar.
  await expect(page.locator('.wiki-sidebar a', { hasText: title })).toBeVisible();
});

test('the wiki sidebar lists the seeded page', async ({ page }) => {
  await page.goto(`${REPO}/wiki`);
  await waitForApp(page);
  await expect(page.locator('.wiki-sidebar a', { hasText: 'Home' })).toBeVisible();
});

test('the package registry lists the seeded packages', async ({ page }) => {
  await page.goto('/packages/codez-org');
  await waitForApp(page);
  await expect(page.locator('.package-list a', { hasText: 'codez-backend' }).first()).toBeVisible();
});

test('dispatching a workflow records a run without a reload', async ({ page }) => {
  await page.goto(`${REPO}/actions`);
  await waitForApp(page);

  // The seeded workflow list renders with a dispatch control.
  const dispatch = page.getByRole('button', { name: 'Dispatch' }).first();
  await expect(dispatch).toBeVisible();

  const runs = page.locator('.runs-list li');
  const before = await runs.count();

  await dispatch.click();

  // The run list refreshes from the store, so a new run appears in place.
  await expect(runs).toHaveCount(before + 1, { timeout: 10_000 });
});
