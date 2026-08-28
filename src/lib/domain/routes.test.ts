import { describe, expect, it } from 'vitest';
import { routeMeta } from './routes';

describe('route metadata', () => {
  it('sets a plain title for every M1 route type', () => {
    const paths = ['/', '/demo', '/demo/purchase-orders/po-nb-1047', '/demo/receive/po-nb-1047', '/demo/receipts/receipt-nb-1047-001', '/demo/discrepancies/case-nb-1047-001', '/privacy', '/terms', '/missing'];
    for (const path of paths) {
      const meta = routeMeta(path);
      expect(meta.title).toContain('Intake Desk');
      expect(meta.title.length).toBeLessThanOrEqual(60);
      expect(meta.description.length).toBeLessThanOrEqual(155);
    }
  });
});
