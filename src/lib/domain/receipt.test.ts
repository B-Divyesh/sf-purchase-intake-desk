import { describe, expect, it } from 'vitest';
import { createDemoSeed } from './model';
import { summarizeReceipt } from './receipt';

describe('receipt summary', () => {
  it('derives the seeded discrepancy from current line values', () => {
    const summary = summarizeReceipt(createDemoSeed());
    expect(summary.compact).toBe('2 each short · 1 each damaged');
    expect(summary.issues).toHaveLength(2);
  });

  it('records a changed fully matched receipt without seed discrepancies', () => {
    const state = createDemoSeed();
    state.lines[1].receivedEach = '48';
    state.lines[2].condition = 'good';
    state.lines[2].damagedEach = '0';
    const summary = summarizeReceipt(state);
    expect(summary.compact).toBe('All 3 lines matched');
    expect(summary.stateLabel).toBe('Complete delivery');
    expect(summary.audit).toBe('Finalized receipt with all 3 lines matched.');
  });
});
