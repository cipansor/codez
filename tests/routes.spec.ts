import { expect, test } from '@playwright/test';
import { trackPageErrors, waitForApp } from './helpers';

/**
 * Route smoke net.
 *
 * Every client route in `crates/frontend/src/main.rs` is rendered once and
 * asserted to (a) mount the app shell and (b) never raise an uncaught error.
 * Keep this list in sync with the router: a renamed or removed route must fail
 * loudly here rather than silently 404 in the browser.
 *
 * The paths use resources that exist in the demo seed (`codez-org/codez`,
 * user `alice`, issue #1, pull #2, milestone id 300, release id 500).
 */
const ROUTES: string[] = [
  '/',
  '/explore',
  '/search',
  '/notifications',
  '/login',
  '/register',
  '/settings/profile',
  '/admin',
  '/admin/users',
  '/admin/orgs',
  '/admin/emails',
  '/repo/create',
  '/repo/migrate',
  '/packages/codez-org',
  '/org/create',
  '/orgs/codez-org',
  '/users/alice',
  '/users/alice/followers',
  '/users/alice/following',
  '/repos/codez-org/codez',
  '/repos/codez-org/codez/src/main',
  '/repos/codez-org/codez/src/README.md',
  '/repos/codez-org/codez/edit/README.md',
  '/repos/codez-org/codez/search',
  '/repos/codez-org/codez/commits',
  '/repos/codez-org/codez/commits/d47c058533feb571f44aa8fe52ff001227c22d5d',
  '/repos/codez-org/codez/branches',
  '/repos/codez-org/codez/tags',
  '/repos/codez-org/codez/pulse',
  '/repos/codez-org/codez/compare',
  '/repos/codez-org/codez/issues',
  '/repos/codez-org/codez/issues/1',
  '/repos/codez-org/codez/labels',
  '/repos/codez-org/codez/milestones',
  '/repos/codez-org/codez/milestones/300',
  '/repos/codez-org/codez/pulls',
  '/repos/codez-org/codez/pulls/2',
  '/repos/codez-org/codez/releases',
  '/repos/codez-org/codez/releases/new',
  '/repos/codez-org/codez/releases/500',
  '/repos/codez-org/codez/actions',
  '/repos/codez-org/codez/wiki',
  '/repos/codez-org/codez/settings',
];

for (const route of ROUTES) {
  test(`renders ${route} without errors`, async ({ page }) => {
    const errors = trackPageErrors(page);
    const response = await page.goto(route);
    expect(response?.status(), `HTTP status for ${route}`).toBeLessThan(400);

    await waitForApp(page);

    // The router fallback is the only place this copy appears; its presence
    // means the path did not match any route.
    await expect(page.locator('.not-found')).toHaveCount(0);
    expect(errors, `uncaught errors on ${route}`).toEqual([]);
  });
}

test('unknown client route shows the 404 fallback', async ({ page }) => {
  await page.goto('/this/route/does/not/exist');
  await waitForApp(page);
  await expect(page.locator('.not-found')).toContainText('404');
});
