import { expect, test, type Page } from '@playwright/test';

async function openDemo(page: Page) {
  await page.goto('/?demo=1');
  await expect(page).toHaveURL(/\/demo\?demo=1$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Find the delivery.');
}

async function openReceive(page: Page) {
  await openDemo(page);
  await page.getByRole('link', { name: /Northline Bearings/ }).click();
  await page.getByRole('link', { name: 'Count this delivery' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Count the delivery.');
}

async function finalize(page: Page) {
  await openReceive(page);
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('dialog').getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page).toHaveURL(/\/demo\/receipts\/receipt-nb-1047-001$/);
}

test('@claim:demo-entry-reset demo opens ready and reset restores the sample', async ({ page }) => {
  await openReceive(page);
  await page.locator('#received-line-blt-a42').fill('41');
  await page.getByRole('button', { name: 'Reset demo' }).first().click();
  await page.getByRole('dialog').getByRole('button', { name: 'Reset demo' }).click();
  await expect(page).toHaveURL(/\/demo\?demo=1$/);
  await page.getByRole('link', { name: /Northline Bearings/ }).click();
  await page.getByRole('link', { name: 'Count this delivery' }).click();
  await expect(page.locator('#received-line-brg-6204')).toHaveValue('120');
  await expect(page.locator('#received-line-blt-a42')).toHaveValue('46');
  await expect(page.locator('#received-line-seal-28')).toHaveValue('24');
});

test('@claim:expected-vs-received compares all received counts', async ({ page }) => {
  await openReceive(page);
  await page.locator('#received-line-brg-6204').fill('120');
  await page.locator('#received-line-blt-a42').fill('46');
  await page.locator('#received-line-seal-28').fill('24');
  await expect(page.locator('#row-line-brg-6204')).toContainText('Matched');
  await expect(page.locator('#row-line-blt-a42')).toContainText('2 each short');
  await expect(page.locator('#row-line-seal-28')).toContainText('1');
});

test('@claim:exact-unit-conversion shows four cases as exactly 48 each', async ({ page }) => {
  await openReceive(page);
  await expect(page.locator('#row-line-blt-a42')).toContainText('4 cases × 12 = 48 each expected');
  await expect(page.locator('#row-line-blt-a42')).not.toContainText('48.000');
});

test('@claim:partial-discrepancy records shortage and damage in one case', async ({ page }) => {
  await finalize(page);
  await page.getByRole('link', { name: 'Open discrepancy' }).click();
  await expect(page.getByRole('heading', { name: '2 each short · 1 seal damaged' })).toBeVisible();
  await expect(page.getByText('Two belts did not arrive')).toBeVisible();
  await expect(page.getByText('One seal is damaged')).toBeVisible();
});

test('@claim:immutable-correction appends a linked event', async ({ page }) => {
  await finalize(page);
  const originalHash = await page.locator('.timeline code').first().textContent();
  await page.getByRole('button', { name: 'Record a correction' }).click();
  await page.getByLabel('Correction reason').fill('Supplier confirmed the two missing belts.');
  await page.getByRole('dialog').getByRole('button', { name: 'Record correction' }).click();
  await expect(page.locator('.timeline li')).toHaveCount(2);
  await expect(page.locator('.timeline code').first()).toHaveText(originalHash ?? '');
  await expect(page.locator('.timeline')).toContainText('Hash chain verified');
});

test('@claim:receipt-csv-export downloads three versioned line rows', async ({ page }) => {
  await finalize(page);
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export receipt CSV' }).click();
  const download = await downloadPromise;
  const stream = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  const csv = Buffer.concat(chunks).toString('utf8');
  expect(csv.trim().split('\r\n')).toHaveLength(4);
  expect(csv).toContain('schema_version,receipt_id,purchase_order');
  expect(csv).toContain('BLT-A42');
  expect(csv).toContain(',-2,good,0,');
});

test('@claim:offline-reload keeps changed counts offline', async ({ page, context }) => {
  await openReceive(page);
  await page.waitForFunction(() => (window as Window & { __INTAKE_SW_READY__?: boolean }).__INTAKE_SW_READY__ === true);
  await page.locator('#received-line-blt-a42').fill('45');
  await expect(page.getByRole('status').filter({ hasText: 'Saved on this device.' }).last()).toBeVisible();
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Count the delivery.');
  await expect(page.locator('#received-line-blt-a42')).toHaveValue('45');
  await page.locator('#received-line-brg-6204').fill('119');
  await page.reload();
  await expect(page.locator('#received-line-brg-6204')).toHaveValue('119');
  await context.setOffline(false);
});

test('@claim:demo-isolation uses only same-origin requests and demo storage', async ({ page }) => {
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await openReceive(page);
  await page.locator('#received-line-blt-a42').fill('44');
  const databaseNames = await page.evaluate(async () => (await indexedDB.databases()).map((database) => database.name));
  expect(databaseNames).toEqual(['intake-desk:demo:v1']);
  expect([...origins]).toEqual([new URL(page.url()).origin]);
});

test('@claim:scanner-manual-fallback accepts scanner and typed codes', async ({ page }) => {
  await openReceive(page);
  const scan = page.getByLabel('Item code');
  await scan.pressSequentially('BRG-6204', { delay: 5 });
  await scan.press('Enter');
  await expect(page.locator('#received-line-brg-6204')).toBeFocused();
  await scan.fill('BLT-A42');
  await scan.press('Enter');
  await expect(page.locator('#received-line-blt-a42')).toBeFocused();
});
