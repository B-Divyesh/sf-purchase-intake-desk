import { describe, expect, it } from 'vitest';
import { createDemoSeed, DEMO_DATABASE } from './model';

describe('deterministic demo seed', () => {
  it('uses only the promised demo namespace and exact sample', () => {
    expect(DEMO_DATABASE).toBe('intake-desk:demo:v1');
    const first = createDemoSeed();
    const second = createDemoSeed();
    expect(first).toEqual(second);
    expect(first.poNumber).toBe('NB-1047');
    expect(first.lines.map((line) => line.receivedEach)).toEqual(['120', '46', '24']);
    expect(first.lines[1].expectedEach).toBe('48');
  });
});
