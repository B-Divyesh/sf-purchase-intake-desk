export type Condition = 'good' | 'damaged';

export interface PurchaseOrderLine {
  id: string;
  code: string;
  description: string;
  ordered: string;
  orderUnit: 'each' | 'case';
  unitsPerCase: string;
  expectedEach: string;
  receivedEach: string;
  condition: Condition;
  damagedEach: string;
  note: string;
}

export interface ReceiptEvent {
  id: string;
  sequence: number;
  type: 'opened' | 'finalized' | 'corrected';
  at: string;
  actor: string;
  summary: string;
  previousHash: string;
  hash: string;
}

export interface DemoState {
  schemaVersion: 1;
  site: string;
  supplier: string;
  poNumber: string;
  packingList: string;
  purchaseOrderId: string;
  receiptId: string;
  discrepancyId: string;
  status: 'open' | 'draft' | 'finalized';
  revision: number;
  receivedAt: string;
  receivedBy: string;
  lines: PurchaseOrderLine[];
  events: ReceiptEvent[];
}

export const DEMO_DATABASE = 'intake-desk:demo:v1';
export const STATE_KEY = 'west-yard-nb-1047';

export const createDemoSeed = (): DemoState => ({
  schemaVersion: 1,
  site: 'West Yard',
  supplier: 'Northline Bearings',
  poNumber: 'NB-1047',
  packingList: 'NL-8821',
  purchaseOrderId: 'po-nb-1047',
  receiptId: 'receipt-nb-1047-001',
  discrepancyId: 'case-nb-1047-001',
  status: 'open',
  revision: 1,
  receivedAt: '2026-08-28T09:40:00',
  receivedBy: 'Sam',
  lines: [
    {
      id: 'line-brg-6204',
      code: 'BRG-6204',
      description: 'Deep-groove bearing',
      ordered: '120',
      orderUnit: 'each',
      unitsPerCase: '1',
      expectedEach: '120',
      receivedEach: '120',
      condition: 'good',
      damagedEach: '0',
      note: '',
    },
    {
      id: 'line-blt-a42',
      code: 'BLT-A42',
      description: 'A42 drive belt',
      ordered: '4',
      orderUnit: 'case',
      unitsPerCase: '12',
      expectedEach: '48',
      receivedEach: '46',
      condition: 'good',
      damagedEach: '0',
      note: 'Two belts missing from one opened case.',
    },
    {
      id: 'line-seal-28',
      code: 'SEAL-28',
      description: '28 mm shaft seal',
      ordered: '24',
      orderUnit: 'each',
      unitsPerCase: '1',
      expectedEach: '24',
      receivedEach: '24',
      condition: 'damaged',
      damagedEach: '1',
      note: 'One seal has a split outer lip.',
    },
  ],
  events: [],
});
