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
  await expect(page.getByRole('heading', { name: '2 each short · 1 each damaged' })).toBeVisible();
  await expect(page.getByRole('heading', { name: '2 each short', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: '1 each damaged', exact: true })).toBeVisible();
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

test('@claim:current-count-finalization records changed matched values everywhere', async ({ page }) => {
  await openReceive(page);
  await page.locator('#received-line-blt-a42').fill('48');
  await page.locator('#condition-line-seal-28').selectOption('good');
  await expect(page.locator('.finalize-bar')).toContainText('All 3 lines matched');
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByRole('dialog')).toContainText('all 3 lines matched');
  await page.getByRole('dialog').getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByText('Complete delivery')).toBeVisible();
  await expect(page.getByText('Finalized receipt with all 3 lines matched.')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Open discrepancy' })).toHaveCount(0);
});

test('@claim:exact-decimal-export exports 46.1 minus 48 as -1.9', async ({ page }) => {
  await openReceive(page);
  await page.locator('#received-line-blt-a42').fill('46.1');
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Finalize receipt' }).click();
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export receipt CSV' }).click();
  const stream = await (await downloadPromise).createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream) chunks.push(Buffer.from(chunk));
  const csv = Buffer.concat(chunks).toString('utf8');
  expect(csv).toContain(',48,46.1,-1.9,good,0,');
  expect(csv).not.toContain('-1.8999999999999986');
});

test('@claim:damage-count-validation blocks damage above received', async ({ page }) => {
  await openReceive(page);
  await page.locator('#received-line-seal-28').fill('1');
  await page.locator('#damaged-line-seal-28').fill('2');
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.getByRole('alert')).toContainText('cannot have more damaged items than received');
  await expect(page.getByRole('alert')).toBeFocused();
});

test('@claim:real-po-workspace imports customer CSV and keeps evidence with the receipt', async ({ page }) => {
  const origins = new Set<string>();
  page.on('request', (request) => origins.add(new URL(request.url()).origin));
  await page.goto('/start');
  await page.getByLabel('Purchase order CSV').setInputFiles({
    name: 'po-220.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('purchase_order,supplier,packing_list,item_code,description,ordered,order_unit,units_per_case\nPO-220,Harbor Fasteners,HF-18,BOLT-8,Stainless bolt,2.5,case,20'),
  });
  await page.getByRole('button', { name: 'Check and import PO' }).click();
  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review PO-220.');
  await expect(page.getByText('50 each')).toBeVisible();
  await page.getByRole('link', { name: 'Count this delivery' }).click();
  await page.getByLabel('Evidence caption').fill('Packing list at dock');
  await page.getByLabel('Photo or PDF').setInputFiles({ name: 'packing-list.png', mimeType: 'image/png', buffer: Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]) });
  await expect(page.getByText(/packing-list\.png attached/)).toBeVisible();
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByText('Complete delivery')).toBeVisible();
  const databases = await page.evaluate(async () => (await indexedDB.databases()).map((database) => database.name));
  expect(databases).toContain('intake-desk:workspace:v1');
  expect(databases).not.toContain('intake-desk:demo:v1');
  expect([...origins]).toEqual([new URL(page.url()).origin]);
});

test('@claim:multi-po-retention importing another PO keeps an earlier finalized receipt available', async ({ page }) => {
  const importPo = async (number: string) => {
    await page.goto('/start');
    await page.getByLabel('Purchase order CSV').setInputFiles({
      name: `${number}.csv`, mimeType: 'text/csv',
      buffer: Buffer.from(`purchase_order,supplier,packing_list,item_code,description,ordered,order_unit,units_per_case\n${number},Harbor Fasteners,${number}-list,BOLT-8,Stainless bolt,2,case,20`),
    });
    await page.getByRole('button', { name: 'Check and import PO' }).click();
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(`Review ${number}.`);
  };
  await importPo('PO-220');
  await page.getByRole('link', { name: 'Count this delivery' }).click();
  await page.getByRole('button', { name: 'Finalize receipt' }).click();
  await page.getByRole('dialog').getByRole('button', { name: 'Finalize receipt' }).click();
  await expect(page.getByText('Complete delivery')).toBeVisible();
  await importPo('PO-221');
  await page.goto('/app');
  await expect(page.getByRole('link', { name: /PO-220/ })).toBeVisible();
  await expect(page.getByRole('link', { name: /PO-221/ })).toBeVisible();
  await page.getByRole('link', { name: /PO-220/ }).click();
  await expect(page.getByRole('link', { name: 'Open finalized receipt' })).toBeVisible();
  await page.getByRole('link', { name: 'Open finalized receipt' }).click();
  await expect(page.getByText('Complete delivery')).toBeVisible();
});

test('@claim:checkout-unavailable states the planned price without offering a broken checkout', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('.pricing')).toContainText('$149');
  await expect(page.locator('.pricing')).toContainText('Checkout is unavailable');
  await expect(page.getByRole('button', { name: 'Checkout unavailable' })).toBeDisabled();
  await expect(page.locator('a[href*="/checkout"]')).toHaveCount(0);
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
  const requests: string[] = [];
  page.on('request', (request) => requests.push(request.url()));
  await openReceive(page);
  await page.locator('#received-line-blt-a42').fill('44');
  const databaseNames = await page.evaluate(async () => (await indexedDB.databases()).map((database) => database.name));
  expect(databaseNames).toEqual(['intake-desk:demo:v1']);
  expect([...new Set(requests.map((url) => new URL(url).origin))]).toEqual([new URL(page.url()).origin]);
  expect(requests.some((url) => new URL(url).pathname.startsWith('/api/'))).toBe(false);
  expect(await page.evaluate(() => Object.keys(localStorage))).toEqual([]);
});

test('@claim:authenticated-cache-bypass never serves a cached tenant response to an authenticated request', async ({ page }) => {
  await page.goto('/?demo=1');
  await page.waitForFunction(() => (window as Window & { __INTAKE_SW_READY__?: boolean }).__INTAKE_SW_READY__ === true);
  const result = await page.evaluate(async () => {
    const cacheName = (await caches.keys()).find((name) => name.startsWith('intake-desk:demo-shell:'))!;
    const cache = await caches.open(cacheName);
    await cache.put('/api/v1/me', new Response(JSON.stringify({ tenant_id: 'tenant-a-secret' }), { headers: { 'Content-Type': 'application/json' } }));
    const response = await fetch('/api/v1/me', { headers: { Authorization: 'Bearer invalid-test-token' } });
    return { body: await response.text(), cached: await (await caches.match('/api/v1/me'))?.text() };
  });
  expect(result.cached).toContain('tenant-a-secret');
  expect(result.body).not.toContain('tenant-a-secret');
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

test('@claim:phone-qr-scanning decodes a phone camera barcode into the matching receipt line', async ({ page }) => {
  await page.addInitScript(() => {
    class TestDetector { async detect() { return [{ rawValue: 'BRG-6204' }]; } }
    Object.defineProperty(window, 'BarcodeDetector', { value: TestDetector, configurable: true, writable: true });
    Object.defineProperty(navigator, 'mediaDevices', { value: { getUserMedia: async () => new MediaStream() }, configurable: true });
    Object.defineProperty(HTMLMediaElement.prototype, 'play', { value: async () => undefined, configurable: true });
  });
  await openReceive(page);
  await page.getByRole('button', { name: 'Scan with phone camera' }).click();
  await expect(page.locator('.live-notice')).toContainText('BRG-6204 scanned. The matching count is ready.');
  await expect(page.locator('#received-line-brg-6204')).toBeFocused();
});
