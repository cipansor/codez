import { expect, test } from '@playwright/test';
import { waitForApp } from './helpers';

/**
 * Follow toggle on a user profile.
 *
 * The backend keeps in-memory state across runs, so the spec drives the toggle
 * to a known state first and then asserts the observable follow-up: the button
 * label flips without a reload and the follower count on the profile reflects
 * the new edge.
 */
test('following a user flips the toggle and updates the count', async ({ page }) => {
  await page.goto('/users/bob');
  await waitForApp(page);

  const toggle = page.locator('.user-links button');
  await expect(toggle).toBeVisible();

  // Normalise to "not following" so the assertions do not depend on leftovers
  // from a previous run of the suite.
  if ((await toggle.textContent())?.trim() === 'Unfollow') {
    await toggle.click();
    await expect(toggle).toHaveText('Follow');
  }

  const followers = page.locator('.user-links a').first();
  const before = await followers.textContent();

  await toggle.click();
  await expect(toggle).toHaveText('Unfollow');

  // The count comes from the same follow edges, so it must move with the toggle.
  await expect(followers).not.toHaveText(before ?? '');

  // Restore the seed state so the other specs see an unfollowed profile.
  await toggle.click();
  await expect(toggle).toHaveText('Follow');
  await expect(followers).toHaveText(before ?? '');
});
