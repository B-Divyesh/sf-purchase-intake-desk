import { describe, expect, it } from 'vitest';
import { componentInventory } from '.';

describe('component inventory contract', () => {
  it('contains 12 to 20 uniquely named components', () => {
    const names = componentInventory.map(({ name }) => name);
    expect(names).toHaveLength(20);
    expect(new Set(names).size).toBe(names.length);
  });
});
