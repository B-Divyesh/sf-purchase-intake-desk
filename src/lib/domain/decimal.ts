const DECIMAL = /^(0|[1-9]\d*)(?:\.(\d+))?$/;

interface ParsedDecimal {
  value: bigint;
  scale: number;
}

export function parseDecimal(input: string): ParsedDecimal | null {
  const value = input.trim();
  const match = DECIMAL.exec(value);
  if (!match) return null;
  const fraction = match[2] ?? '';
  return { value: BigInt(value.replace('.', '')), scale: fraction.length };
}

function powerOfTen(power: number): bigint {
  return 10n ** BigInt(power);
}

export function compareDecimal(left: string, right: string): number {
  const a = parseDecimal(left);
  const b = parseDecimal(right);
  if (!a || !b) throw new Error('Quantity must be a positive decimal number.');
  const scale = Math.max(a.scale, b.scale);
  const av = a.value * powerOfTen(scale - a.scale);
  const bv = b.value * powerOfTen(scale - b.scale);
  return av === bv ? 0 : av < bv ? -1 : 1;
}

export function subtractDecimal(left: string, right: string): string {
  const a = parseDecimal(left);
  const b = parseDecimal(right);
  if (!a || !b) throw new Error('Quantity must be a positive decimal number.');
  const scale = Math.max(a.scale, b.scale);
  const result = a.value * powerOfTen(scale - a.scale) - b.value * powerOfTen(scale - b.scale);
  const negative = result < 0n;
  const absolute = negative ? -result : result;
  if (scale === 0) return `${negative ? '-' : ''}${absolute}`;
  const digits = absolute.toString().padStart(scale + 1, '0');
  const whole = digits.slice(0, -scale);
  const fraction = digits.slice(-scale).replace(/0+$/, '');
  return `${negative ? '-' : ''}${whole}${fraction ? `.${fraction}` : ''}`;
}

export function addDecimal(left: string, right: string): string {
  const a = parseDecimal(left);
  const b = parseDecimal(right);
  if (!a || !b) throw new Error('Quantity must be a positive decimal number.');
  const scale = Math.max(a.scale, b.scale);
  const result = a.value * powerOfTen(scale - a.scale) + b.value * powerOfTen(scale - b.scale);
  if (scale === 0) return result.toString();
  const digits = result.toString().padStart(scale + 1, '0');
  const whole = digits.slice(0, -scale);
  const fraction = digits.slice(-scale).replace(/0+$/, '');
  return `${whole}${fraction ? `.${fraction}` : ''}`;
}

export function multiplyDecimal(left: string, right: string): string {
  const a = parseDecimal(left);
  const b = parseDecimal(right);
  if (!a || !b) throw new Error('Quantity must be a positive decimal number.');
  const scale = a.scale + b.scale;
  const raw = a.value * b.value;
  if (scale === 0) return raw.toString();
  const digits = raw.toString().padStart(scale + 1, '0');
  const whole = digits.slice(0, -scale);
  const fraction = digits.slice(-scale).replace(/0+$/, '');
  return `${whole}${fraction ? `.${fraction}` : ''}`;
}

export function differenceLabel(expected: string, received: string): string {
  const comparison = compareDecimal(received, expected);
  if (comparison === 0) return 'Matched';
  if (comparison < 0) return `${subtractDecimal(expected, received)} each short`;
  return `${subtractDecimal(received, expected)} each extra`;
}
