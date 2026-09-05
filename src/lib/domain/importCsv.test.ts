import { describe, expect, it } from 'vitest';
import { importPurchaseOrderCsv } from './importCsv';

const csv = `purchase_order,supplier,packing_list,item_code,description,ordered,order_unit,units_per_case\nPO-22,"Acme, Ltd",PL-9,BELT-1,Drive belt,4,case,12.5`;

describe('purchase order CSV intake', () => {
  it('imports quoted values and exact decimal case quantities', () => {
    const state = importPurchaseOrderCsv(csv);
    expect(state.supplier).toBe('Acme, Ltd');
    expect(state.lines[0].expectedEach).toBe('50');
    expect(state.source).toBe('csv');
  });

  it('rejects duplicate item codes before storing anything', () => {
    expect(() => importPurchaseOrderCsv(`${csv}\nPO-22,"Acme, Ltd",PL-9,BELT-1,Other,1,each,1`)).toThrow('repeats item');
  });

  it('rejects zero ordered and case-conversion quantities', () => {
    expect(() => importPurchaseOrderCsv(csv.replace(',4,case,12.5', ',0,case,12.5'))).toThrow('greater than zero');
    expect(() => importPurchaseOrderCsv(csv.replace(',4,case,12.5', ',4,case,0'))).toThrow('greater than zero');
  });
});
