import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

const routes = ['/', '/start', '/demo', '/demo/purchase-orders/po-nb-1047', '/demo/receive/po-nb-1047', '/privacy', '/terms', '/404'];

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

test('keyboard skip link works and visible controls meet 44px touch targets', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link', { name: 'Skip to main content' })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('main')).toBeFocused();

  for (const route of ['/', '/start', '/demo', '/demo/purchase-orders/po-nb-1047', '/demo/receive/po-nb-1047', '/privacy', '/terms']) {
    await page.goto(route);
    const undersized = await page.locator('a, button, input, select, textarea').evaluateAll((elements) => elements
      .filter((element) => {
        const rect = element.getBoundingClientRect();
        const style = getComputedStyle(element);
        return style.visibility !== 'hidden' && style.display !== 'none' && rect.width > 0 && rect.height > 0;
      })
      .map((element) => {
        const rect = element.getBoundingClientRect();
        return { name: element.getAttribute('aria-label') || element.textContent?.trim() || element.getAttribute('name'), width: rect.width, height: rect.height };
      })
      .filter(({ width, height }) => width < 44 || height < 44));
    expect(undersized, `${route} has undersized controls`).toEqual([]);
  }
});

test('service worker update uses the build-versioned cache', async ({ page }) => {
  await page.goto('/?demo=1');
  await page.waitForFunction(() => (window as Window & { __INTAKE_SW_READY__?: boolean }).__INTAKE_SW_READY__ === true);
  const result = await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
    return (await caches.keys()).filter((name) => name.startsWith('intake-desk:demo-shell:'));
  });
  expect(result).toEqual([expect.stringMatching(/^intake-desk:demo-shell:(?!m1$).+/)]);
});
