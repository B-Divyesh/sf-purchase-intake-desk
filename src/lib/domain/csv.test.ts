import { describe, expect, it } from 'vitest';
import { receiptCsv } from './csv';
import { createDemoSeed } from './model';

describe('receipt CSV', () => {
  it('uses a versioned schema and one escaped row per line', () => {
    const state = createDemoSeed();
    state.lines[0].note = 'Checked, counted and "sealed"';
    const csv = receiptCsv(state);
    const rows = csv.trim().split('\r\n');
    expect(rows).toHaveLength(4);
    expect(rows[0]).toContain('schema_version');
    expect(rows[1]).toContain('"Checked, counted and ""sealed"""');
    expect(rows[2]).toContain('BLT-A42');
    expect(rows[2]).toContain(',-2,');
  });
});
