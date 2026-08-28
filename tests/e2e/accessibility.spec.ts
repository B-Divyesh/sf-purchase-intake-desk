import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const routes = ['/', '/demo', '/demo/purchase-orders/po-nb-1047', '/demo/receive/po-nb-1047', '/privacy', '/terms', '/404'];

test('M1 routes have one heading, no overflow, and no serious axe findings', async ({ page }) => {
  for (const route of routes) {
    await page.goto(route);
    await expect(page.locator('main h1')).toHaveCount(1);
    await expect(page.locator('main')).toBeVisible();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
    expect(overflow, `${route} must not overflow`).toBe(false);
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations.filter((violation) => ['serious', 'critical'].includes(violation.impact ?? '')), `${route} axe violations`).toEqual([]);
  }
});

test('route changes move focus and dialogs return it', async ({ page }) => {
  await page.goto('/demo/receive/po-nb-1047');
  await page.getByRole('link', { name: 'Privacy' }).first().click();
  await expect(page.getByRole('heading', { level: 1 })).toBeFocused();
  await page.goto('/demo');
  const reset = page.getByRole('button', { name: 'Reset demo' }).first();
  await reset.click();
  await page.getByRole('dialog').getByRole('button', { name: 'Keep changes' }).click();
  await expect(reset).toBeFocused();
});
