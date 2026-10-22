import { expect, type Page } from '@playwright/test';

/**
 * Collects uncaught page errors so a spec can assert the SPA never threw while
 * rendering. Returns the live array; it fills as the page runs.
 */
export function trackPageErrors(page: Page): string[] {
  const errors: string[] = [];
  page.on('pageerror', (error) => errors.push(String(error)));
  return errors;
}

/**
 * Wait until the Leptos bundle has mounted. The app renders the top navigation
 * on every route, so the brand link is a reliable "hydration done" signal. The
 * timeout is generous because the WASM bundle's first load under parallel
 * workers (notably WebKit) can exceed the default 5s.
 */
export async function waitForApp(page: Page): Promise<void> {
  await expect(page.locator('header.app-header a.brand')).toBeVisible({ timeout: 15_000 });
}

/**
 * A unique value for anything a create spec then asserts on. The backend keeps
 * in-memory state across runs, so a fixed title can already exist and make the
 * assertion pass even when the write (or its refresh) is broken.
 */
export function uniqueLabel(prefix: string): string {
  return `${prefix} ${Date.now()}-${Math.floor(Math.random() * 10_000)}`;
}
