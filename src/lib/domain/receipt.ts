import { addDecimal, compareDecimal, subtractDecimal } from './decimal';
import type { DemoState, PurchaseOrderLine } from './model';

export interface ReceiptIssue {
  kind: 'short' | 'extra' | 'damaged';
  line: PurchaseOrderLine;
  quantity: string;
}

export interface ReceiptSummary {
  issues: ReceiptIssue[];
  shortTotal: string;
  extraTotal: string;
  damagedTotal: string;
  stateLabel: 'Complete delivery' | 'Delivery with discrepancies';
  compact: string;
  audit: string;
}

export function summarizeReceipt(state: DemoState): ReceiptSummary {
  const issues: ReceiptIssue[] = [];
  let shortTotal = '0';
  let extraTotal = '0';
  let damagedTotal = '0';

  for (const line of state.lines) {
    const comparison = compareDecimal(line.receivedEach, line.expectedEach);
    if (comparison < 0) {
      const quantity = subtractDecimal(line.expectedEach, line.receivedEach);
      shortTotal = addDecimal(shortTotal, quantity);
      issues.push({ kind: 'short', line, quantity });
    } else if (comparison > 0) {
      const quantity = subtractDecimal(line.receivedEach, line.expectedEach);
      extraTotal = addDecimal(extraTotal, quantity);
      issues.push({ kind: 'extra', line, quantity });
    }
    if (line.condition === 'damaged' && compareDecimal(line.damagedEach, '0') > 0) {
      damagedTotal = addDecimal(damagedTotal, line.damagedEach);
      issues.push({ kind: 'damaged', line, quantity: line.damagedEach });
    }
  }

  const parts: string[] = [];
  if (compareDecimal(shortTotal, '0') > 0) parts.push(`${shortTotal} each short`);
  if (compareDecimal(extraTotal, '0') > 0) parts.push(`${extraTotal} each extra`);
  if (compareDecimal(damagedTotal, '0') > 0) parts.push(`${damagedTotal} each damaged`);
  const matched = issues.length === 0;
  const compact = matched ? `All ${state.lines.length} lines matched` : parts.join(' · ');
  return {
    issues,
    shortTotal,
    extraTotal,
    damagedTotal,
    stateLabel: matched ? 'Complete delivery' : 'Delivery with discrepancies',
    compact,
    audit: matched
      ? `Finalized receipt with all ${state.lines.length} lines matched.`
      : `Finalized receipt with ${parts.join(' and ')}.`,
  };
}
