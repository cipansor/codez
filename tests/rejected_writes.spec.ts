import { expect, test } from '@playwright/test';
import { waitForApp } from './helpers';

const REPO = '/repos/codez-org/codez';

/**
 * A rejected write must keep the user's input and surface an error, rather than
 * clearing the form as if it had succeeded. `page.route` forces a non-2xx reply
 * so the failure path runs against the real handler code.
 */

test('a rejected issue create keeps the typed title and shows an alert', async ({ page }) => {
  await page.route('**/api/v1/repos/*/*/issues', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({ status: 500, body: '{"message":"boom"}' });
    } else {
      await route.continue();
    }
  });

  await page.goto(`${REPO}/issues`);
  await waitForApp(page);

  await page.getByPlaceholder('Issue title').fill('This write will fail');
  await page.getByRole('button', { name: 'New issue' }).click();

  const alert = page.locator('.form-error[role="alert"]');
  await expect(alert).toBeVisible();
  await expect(alert).toContainText('Could not save');

  // The typed value must survive so the write can be retried.
  await expect(page.getByPlaceholder('Issue title')).toHaveValue('This write will fail');
});

test('a rejected label create keeps the typed name and shows an alert', async ({ page }) => {
  await page.route('**/api/v1/repos/*/*/labels', async (route) => {
    if (route.request().method() === 'POST') {
      await route.fulfill({ status: 500, body: '{"message":"boom"}' });
    } else {
      await route.continue();
    }
  });

  await page.goto(`${REPO}/labels`);
  await waitForApp(page);

  await page.getByPlaceholder('Label name').fill('kept-on-failure');
  await page.getByRole('button', { name: 'New label' }).click();

  const alert = page.locator('.form-error[role="alert"]');
  await expect(alert).toBeVisible();
  await expect(page.getByPlaceholder('Label name')).toHaveValue('kept-on-failure');
});
