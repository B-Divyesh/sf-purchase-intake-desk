import type { DemoState } from './model';
import { subtractDecimal } from './decimal';

const escapeCell = (value: string | number): string => {
  const text = String(value);
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
};

export function receiptCsv(state: DemoState): string {
  const headers = [
    'schema_version', 'receipt_id', 'purchase_order', 'supplier', 'packing_list',
    'received_at', 'received_by', 'item_code', 'description', 'expected_each',
    'received_each', 'difference_each', 'condition', 'damaged_each', 'note',
  ];
  const rows = state.lines.map((line) => [
    '1', state.receiptId, state.poNumber, state.supplier, state.packingList,
    state.receivedAt, state.receivedBy, line.code, line.description, line.expectedEach,
    line.receivedEach, subtractDecimal(line.receivedEach, line.expectedEach),
    line.condition, line.damagedEach, line.note,
  ]);
  return `${[headers, ...rows].map((row) => row.map(escapeCell).join(',')).join('\r\n')}\r\n`;
}
