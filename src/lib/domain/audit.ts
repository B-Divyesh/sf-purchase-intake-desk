import type { ReceiptEvent } from './model';

export async function sha256(value: string): Promise<string> {
  const bytes = new TextEncoder().encode(value);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, '0')).join('');
}

export async function appendEvent(
  events: ReceiptEvent[],
  input: Pick<ReceiptEvent, 'type' | 'at' | 'actor' | 'summary'>,
): Promise<ReceiptEvent[]> {
  const previousHash = events.at(-1)?.hash ?? 'GENESIS';
  const sequence = events.length + 1;
  const id = `event-${sequence}`;
  const hash = await sha256(
    JSON.stringify({ id, sequence, ...input, previousHash }),
  );
  return [...events, { id, sequence, ...input, previousHash, hash }];
}

export async function verifyEventChain(events: ReceiptEvent[]): Promise<boolean> {
  let previousHash = 'GENESIS';
  for (const event of events) {
    if (event.previousHash !== previousHash) return false;
    const { hash: _hash, ...contents } = event;
    if ((await sha256(JSON.stringify(contents))) !== event.hash) return false;
    previousHash = event.hash;
  }
  return true;
}
