import { describe, expect, it } from 'vitest';
import { compareDecimal, differenceLabel, multiplyDecimal, subtractDecimal } from './decimal';

describe('decimal quantity rules', () => {
  it('converts cases to each without floating point residue', () => {
    expect(multiplyDecimal('4', '12')).toBe('48');
    expect(multiplyDecimal('1.25', '12')).toBe('15');
  });

  it('classifies matched, short, and extra counts', () => {
    expect(compareDecimal('48.0', '48')).toBe(0);
    expect(differenceLabel('48', '46')).toBe('2 each short');
    expect(differenceLabel('48', '50.5')).toBe('2.5 each extra');
    expect(subtractDecimal('48.00', '46')).toBe('2');
  });
});
