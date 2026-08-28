import { describe, expect, it } from 'vitest';
import { designTokens } from './tokens';

describe('design token contract', () => {
  it('keeps a 44px minimum target', () => {
    expect(designTokens.minimumTargetPx).toBeGreaterThanOrEqual(44);
  });

  it('defines matching semantic colors for day and night', () => {
    expect(Object.keys(designTokens.palette.day)).toEqual(
      Object.keys(designTokens.palette.night),
    );
  });

  it('uses the agreed 4px-based spacing scale', () => {
    expect(designTokens.spacePx).toEqual([4, 8, 12, 16, 24, 32, 48, 64, 96]);
  });
});
