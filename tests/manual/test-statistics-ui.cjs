const { chromium } = require('playwright');
const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');
const path = require('node:path');
(async () => {
  const browser = await chromium.launch({ channel: 'chrome', headless: true });
  const page = await browser.newPage({ viewport: { width: process.env.STATS_WIDE ? 2533 : 1440, height: process.env.STATS_WIDE ? 1366 : 900 } });
  const errors = [];
  page.on('pageerror', error => errors.push(String(error)));
  await page.addInitScript(() => {
    const party = { name: 'Firma Eins', city: '', country: 'DE' };
    const workspace = { invoice_id: 0, company_id: 1, customer_id: 0, seller: party, buyer: {}, number: '2026-0001', date: '2026-10-10', items: [{ description: '', quantity: 1, unit: 'Stunden', unit_price_cents: 0, vat_percent: 19 }] };
    window.__TAURI__ = { core: { invoke: async (command) => {
      if (command === 'load_app') return { workspace, companies: [{ id: 1, party, template: 'example' }], invoices: [], customers: [], templates: ['example'], settings: { due_days: 14, recent_count: 5 }, active_company_id: 1, status: 'draft' };
      if (command === 'calculate') return { lines: [{ net_cents: 0 }], taxes: [], net_cents: 0, tax_cents: 0, gross_cents: 0 };
      if (command === 'load_statistics') return { skipped: 0, incomplete: 1, invoices: [
        { id: 1, number: 'R1', date: '2026-10-01', status: 'issued', customer_id: 2, customer: 'Kunde', item_count: 3, net_cents: 10000, tax_cents: 1900, gross_cents: 11900, hours: 2, hourly_net_cents: 10000 },
        { id: 2, number: 'R2', date: '2026-10-02', status: 'draft', customer_id: 0, customer: 'Weiterer Kunde', item_count: 1, net_cents: 5000, tax_cents: 950, gross_cents: 5950, hours: 1, hourly_net_cents: 5000 }
      ] };
      throw Error('Unexpected command: ' + command);
    } } };
  });
  await page.goto(pathToFileURL(path.resolve(__dirname, '../../ui/index.html')).href);
  await page.locator('[data-page=statistics]').click();
  await page.locator('.stats-revenue').waitFor();
  assert(await page.locator('main > header #stats-header-filters').isVisible());
  assert.equal(await page.locator('#statistics-page select').count(), 0);
  assert(await page.locator('#stats-from').isHidden());
  await page.locator('#stats-period').selectOption('custom');
  assert(await page.locator('#stats-from').isVisible());
  await page.locator('#stats-period').selectOption('year');
  assert((await page.locator('.stats-revenue').innerText()).includes('150,00'));
  assert((await page.locator('.stats-priority').first().innerText()).includes('28,50'));
  assert((await page.locator('.stats-priority').last().innerText()).includes('3'));
  assert((await page.locator('#stats-warning').innerText()).includes('1 Rechnung(en)'));
  assert.equal(await page.locator('#stats-content').getByText('Entwürfe').count(), 0);
  assert.equal(await page.locator('.stats-records .stats-record:visible').count(), 6);
  assert.equal(await page.locator('.stats-records .stats-record.featured').count(), 2);
  assert((await page.locator('.stats-gross').innerText()).includes('178,50'));
  assert.equal(await page.locator('.stats-note').count(), 0);
  assert.equal(await page.locator('.stats-records summary').count(), 0);
  assert.equal(await page.locator('.stats-records').getByText('MONATSEND-ABRECHNER').count(), 0);
  assert.equal(await page.locator('.stats-records').getByText('LÄNGSTE MONATSSERIE').count(), 0);
  if (process.env.STATS_SCREENSHOT) await page.screenshot({ path: process.env.STATS_SCREENSHOT, fullPage: true });
  const first = await page.locator('.stats-overview').boundingBox();
  const next = await page.locator('.stats-trend').boundingBox();
  assert(next.y >= first.y + first.height + 19);
  if (process.env.STATS_WIDE) {
    const customers = await page.locator('.stats-customers').boundingBox();
    assert(customers.x > next.x + next.width - 1);
    assert(Math.abs(customers.y - next.y) < 2);
    assert(next.width < 1100);
  }
  await page.locator('#stats-period').selectOption('previous');
  assert(await page.locator('.stats-empty').isVisible());
  await page.locator('[data-page=invoices]').click();
  assert.equal(await page.locator('#invoice-list th').allTextContents().then(labels => labels.some(label => label.includes('Status'))), false);
  assert.equal(await page.locator('#save-status').innerText(), '');
  assert.deepEqual(errors, []);
  await browser.close();
  console.log('Statistics UI passed: all valid invoices, exclusion notice, empty state and layout spacing.');
})().catch(error => { console.error(error); process.exit(1); });
