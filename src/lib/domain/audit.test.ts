import { describe, expect, it } from 'vitest';
import { appendEvent, verifyEventChain } from './audit';

describe('receipt event chain', () => {
  it('appends a correction without changing the finalized event', async () => {
    const finalized = await appendEvent([], { type: 'finalized', at: '2026-08-28T09:40:00', actor: 'Sam', summary: 'Finalized.' });
    const original = structuredClone(finalized[0]);
    const corrected = await appendEvent(finalized, { type: 'corrected', at: '2026-08-28T10:06:00', actor: 'Sam', summary: 'Supplier confirmed.' });
    expect(corrected[0]).toEqual(original);
    expect(corrected[1].previousHash).toBe(original.hash);
    expect(await verifyEventChain(corrected)).toBe(true);
  });

  it('detects a rewritten event', async () => {
    const events = await appendEvent([], { type: 'finalized', at: '2026-08-28T09:40:00', actor: 'Sam', summary: 'Finalized.' });
    events[0].summary = 'Rewritten.';
    expect(await verifyEventChain(events)).toBe(false);
  });
});
