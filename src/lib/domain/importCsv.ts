import { multiplyDecimal, parseDecimal } from './decimal';
import type { DemoState, PurchaseOrderLine } from './model';

const REQUIRED = ['purchase_order', 'supplier', 'packing_list', 'item_code', 'description', 'ordered', 'order_unit', 'units_per_case'];

function parseRows(text: string): string[][] {
  const rows: string[][] = [];
  let row: string[] = [];
  let cell = '';
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (quoted) {
      if (character === '"' && text[index + 1] === '"') { cell += '"'; index += 1; }
      else if (character === '"') quoted = false;
      else cell += character;
    } else if (character === '"') quoted = true;
    else if (character === ',') { row.push(cell.trim()); cell = ''; }
    else if (character === '\n') { row.push(cell.trim()); if (row.some(Boolean)) rows.push(row); row = []; cell = ''; }
    else if (character !== '\r') cell += character;
  }
  row.push(cell.trim());
  if (row.some(Boolean)) rows.push(row);
  if (quoted) throw new Error('The CSV has an unclosed quoted value.');
  return rows;
}

const slug = (value: string) => value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 48) || 'record';

export function importPurchaseOrderCsv(text: string): DemoState {
  const rows = parseRows(text.replace(/^\uFEFF/, ''));
  if (rows.length < 2) throw new Error('Add a header row and at least one purchase-order line.');
  const headers = rows[0].map((value) => value.toLowerCase());
  const missing = REQUIRED.filter((header) => !headers.includes(header));
  if (missing.length) throw new Error(`The CSV is missing: ${missing.join(', ')}.`);
  const value = (row: string[], key: string) => row[headers.indexOf(key)]?.trim() ?? '';
  const first = rows[1];
  const poNumber = value(first, 'purchase_order');
  const supplier = value(first, 'supplier');
  const packingList = value(first, 'packing_list');
  if (!poNumber || !supplier || !packingList) throw new Error('Purchase order, supplier, and packing list are required.');
  const seen = new Set<string>();
  const lines: PurchaseOrderLine[] = rows.slice(1).map((row, index) => {
    if (value(row, 'purchase_order') !== poNumber || value(row, 'supplier') !== supplier || value(row, 'packing_list') !== packingList) {
      throw new Error(`Row ${index + 2} belongs to a different purchase order, supplier, or packing list.`);
    }
    const code = value(row, 'item_code').toUpperCase();
    const description = value(row, 'description');
    const ordered = value(row, 'ordered');
    const orderUnit = value(row, 'order_unit').toLowerCase();
    const unitsPerCase = value(row, 'units_per_case');
    if (!code || !description || !parseDecimal(ordered) || !parseDecimal(unitsPerCase)) throw new Error(`Row ${index + 2} has an invalid item or quantity.`);
    if (orderUnit !== 'each' && orderUnit !== 'case') throw new Error(`Row ${index + 2} order_unit must be each or case.`);
    if (seen.has(code)) throw new Error(`Row ${index + 2} repeats item ${code}.`);
    seen.add(code);
    const expectedEach = orderUnit === 'case' ? multiplyDecimal(ordered, unitsPerCase) : ordered;
    return { id: `line-${slug(code)}`, code, description, ordered, orderUnit, unitsPerCase, expectedEach, receivedEach: expectedEach, condition: 'good', damagedEach: '0', note: '' };
  });
  const id = slug(poNumber);
  return {
    schemaVersion: 1, site: 'This device', supplier, poNumber, packingList,
    purchaseOrderId: `po-${id}`, receiptId: `receipt-${id}-${Date.now()}`,
    discrepancyId: `case-${id}-${Date.now()}`, status: 'open', revision: 1,
    receivedAt: new Date().toISOString().slice(0, 19), receivedBy: 'Receiver',
    lines, events: [], attachments: [], source: 'csv',
  };
}

export const PURCHASE_ORDER_TEMPLATE = `${REQUIRED.join(',')}\nPO-1001,Example Supplier,PL-1001,ITEM-1,Example item,10,each,1\n`;
