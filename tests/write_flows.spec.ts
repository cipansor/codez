import { expect, test } from '@playwright/test';
import { uniqueLabel, waitForApp } from './helpers';

const REPO = '/repos/codez-org/codez';

/**
 * Write flows assert the observable follow-up, never just that the request was
 * sent: a create must appear in the list without a reload, and a toggle must
 * flip its label. A `page.reload()` here would hide a missing resource refresh,
 * so it is deliberately absent.
 */

test('creating an issue shows it in the list without a reload', async ({ page }) => {
  await page.goto(`${REPO}/issues`);
  await waitForApp(page);

  const title = uniqueLabel('Issue');
  await page.getByPlaceholder('Issue title').fill(title);
  await page.getByPlaceholder('Leave a description').fill('Filed by the e2e suite.');
  await page.getByRole('button', { name: 'New issue' }).click();

  // The new row must appear from the resource refresh alone.
  await expect(page.locator('.item-list, ul').getByText(title, { exact: false })).toBeVisible();

  // The form clears only on a successful write.
  await expect(page.getByPlaceholder('Issue title')).toHaveValue('');
  await expect(page.locator('.form-error')).toHaveCount(0);
});

test('closing and reopening an issue flips the toggle label', async ({ page }) => {
  await page.goto(`${REPO}/issues/1`);
  await waitForApp(page);

  const toggle = page.locator('.pr-actions button');
  await expect(toggle).toBeVisible();

  const initial = (await toggle.textContent())?.trim();
  await toggle.click();

  // The label must flip without a reload, driven by the refresh signal.
  const flipped = initial === 'Close' ? 'Reopen' : 'Close';
  await expect(toggle).toHaveText(flipped);

  // Put it back so the seeded state is stable for the next run.
  await toggle.click();
  await expect(toggle).toHaveText(initial!);
});

test('creating a label adds it to the list without a reload', async ({ page }) => {
  await page.goto(`${REPO}/labels`);
  await waitForApp(page);

  const name = uniqueLabel('label');
  await page.getByPlaceholder('Label name').fill(name);
  await page.getByRole('button', { name: 'New label' }).click();

  await expect(page.locator('.labels-list').getByText(name, { exact: false })).toBeVisible();
  await expect(page.getByPlaceholder('Label name')).toHaveValue('');
});

test('creating a repository navigates to the new repository', async ({ page }) => {
  await page.goto('/repo/create');
  await waitForApp(page);

  const name = uniqueLabel('repo').toLowerCase().replace(/[^a-z0-9-]+/g, '-');
  await page.locator('.form-group').filter({ hasText: 'Repository name' }).locator('input').fill(name);
  await page.getByRole('button', { name: 'Create repository' }).click();

  // A successful create navigates straight to the repository overview.
  await expect(page).toHaveURL(new RegExp(`/repos/alice/${name}$`));
  await expect(page.locator('.repo-header')).toContainText(name);
});

test('creating an organization navigates to its profile', async ({ page }) => {
  await page.goto('/org/create');
  await waitForApp(page);

  const name = uniqueLabel('org').toLowerCase().replace(/[^a-z0-9-]+/g, '-');
  await page
    .locator('.form-group')
    .filter({ hasText: 'Organization name' })
    .locator('input')
    .fill(name);
  await page.getByRole('button', { name: 'Create organization' }).click();

  await expect(page).toHaveURL(new RegExp(`/orgs/${name}$`));
});
