import { addDateDays, localToday, parseDate } from '$lib/format';
import { tr } from '$lib/i18n/tr';
import type { ApiError, Audit, CommandMap, ItemInput, InventoryItem, LotInput, InventoryLot, StockMovement, ReceiveInput, AdjustInput, ConsumeInput, ExpiringLot, Medication, VaccineType, Vaccination, PrescriptionItem } from './types';

interface Context {
  now(): string;
  fold(value: string): string;
  fail(kind: ApiError['kind'], message: string, field?: string): never;
  catalog(command: string, args: unknown): unknown;
}
export function createInventoryMock(ctx: Context) {
  let nextId = 8000;
  let items: InventoryItem[] = [], lots: InventoryLot[] = [], movements: StockMovement[] = [];
  const audit = (): Audit => ({ id: nextId++, createdAt: ctx.now(), updatedAt: ctx.now(), deletedAt: null });
  const invalid = (field: string) => ctx.fail('invalid', 'Invalid inventory value.', field);
  const conflict = () => ctx.fail('conflict', 'Inventory state conflict.');
  const number = (field: string, v: number, min = 0) => { if (!Number.isFinite(v) || v < min) invalid(field); };
  const date = (field: string, v: string) => { if (!parseDate(v)) invalid(field); };
  function item(id: number, active = true): InventoryItem {
    const row = items.find(i => i.id === id && (!active || !i.deletedAt)) ?? ctx.fail('not_found', 'Record not found.');
    const live = lots.filter(l => l.itemId === id && !l.deletedAt);
    row.stock = live.reduce((sum, l) => sum + l.quantity, 0);
    row.nearestExpiry = live.filter(l => l.quantity > 0 && l.expiresOn).map(l => l.expiresOn!).sort()[0] ?? null;
    row.unallocatedDelta = movements.filter(m => m.itemId === id && m.lotId === null).reduce((sum, m) => sum + m.delta, 0);
    return row;
  }
  function lot(id: number, active = true) { const row = lots.find(l => l.id === id && (!active || !l.deletedAt)) ?? ctx.fail('not_found', 'Record not found.'); item(row.itemId); return row; }
  function prepare(input: ItemInput, id?: number) {
    if (!input.name.trim()) invalid('name'); if (!input.unit.trim()) invalid('unit');
    if (!['medication', 'vaccine', 'supply', 'food', 'product', 'other'].includes(input.kind)) invalid('kind');
    if (input.salePriceMinor !== null && (!Number.isSafeInteger(input.salePriceMinor) || input.salePriceMinor < 0)) invalid('salePriceMinor');
    if (input.reorderLevel !== null) number('reorderLevel', input.reorderLevel);
    if (input.medicationId !== null && input.vaccineTypeId !== null) invalid('medicationId');
    if (input.medicationId !== null) {
      const m = ctx.catalog('medication_get', { id: input.medicationId }) as Medication;
      if (input.kind !== 'medication' || m.deletedAt) invalid('medicationId');
    }
    if (input.vaccineTypeId !== null) {
      const v = ctx.catalog('vaccine_type_get', { id: input.vaccineTypeId }) as VaccineType;
      if (input.kind !== 'vaccine' || v.deletedAt || !v.isActive) invalid('vaccineTypeId');
    }
    if (items.some(i => !i.deletedAt && i.id !== id && ((input.medicationId !== null && i.medicationId === input.medicationId) || (input.vaccineTypeId !== null && i.vaccineTypeId === input.vaccineTypeId)))) conflict();
  }
  function create(input: ItemInput) { prepare(input); const row: InventoryItem = { ...input, name: input.name.trim(), unit: input.unit.trim(), ...audit(), stock: 0, nearestExpiry: null, unallocatedDelta: 0 }; items.push(row); return row; }
  function catalog(kind: 'medication' | 'vaccine', id: number) {
    const existing = items.find(i => !i.deletedAt && (kind === 'medication' ? i.medicationId === id : i.vaccineTypeId === id));
    if (existing) return item(existing.id);
    const source = ctx.catalog(kind === 'medication' ? 'medication_get' : 'vaccine_type_get', { id }) as Medication | VaccineType;
    return create({ name: source.name, kind, medicationId: kind === 'medication' ? id : null, vaccineTypeId: kind === 'vaccine' ? id : null, unit: kind === 'medication' ? (source as Medication).packUnit || 'adet' : 'doz', salePriceMinor: null, reorderLevel: null, isActive: true, notes: null });
  }
  function prepareLot(input: LotInput) {
    number('quantity', input.quantity);
    if (input.costMinor !== null && (!Number.isSafeInteger(input.costMinor) || input.costMinor < 0)) invalid('costMinor');
    if (input.expiresOn) date('expiresOn', input.expiresOn); if (input.receivedOn) date('receivedOn', input.receivedOn);
  }
  function insertLot(input: LotInput) { prepareLot(input); if (!item(input.itemId).isActive) conflict(); const row = { ...input, quantity: 0, ...audit() }; lots.push(row); return row; }
  function move(itemId: number, lotId: number | null, delta: number, reason: StockMovement['reason'], movedOn: string, notes: string | null = null, refTable: StockMovement['refTable'] = null, refId: number | null = null) {
    date('movedOn', movedOn); if (!Number.isFinite(delta) || delta === 0) invalid('delta');
    if (lotId !== null) { const l = lot(lotId); if (l.itemId !== itemId) conflict(); number('quantity', l.quantity + delta); if (l.quantity + delta === l.quantity) invalid('delta'); l.quantity += delta; l.updatedAt = ctx.now(); }
    const row: StockMovement = { ...audit(), itemId, lotId, delta, reason, movedOn, notes, refTable, refId }; movements.push(row); item(itemId).updatedAt = ctx.now(); return row;
  }
  const itemLots = (id: number, deleted = false) => { item(id, false); return lots.filter(l => l.itemId === id && (deleted || !l.deletedAt)).sort((a, b) => Number(a.expiresOn === null) - Number(b.expiresOn === null) || (a.expiresOn ?? '').localeCompare(b.expiresOn ?? '') || (a.receivedOn ?? '').localeCompare(b.receivedOn ?? '') || a.id - b.id); };
  const list = (deleted = false) => items.filter(i => deleted || !i.deletedAt).map(i => item(i.id, false)).sort((a, b) => ctx.fold(a.name).localeCompare(ctx.fold(b.name)) || a.id - b.id);
  const low = () => list().filter(i => i.isActive && i.reorderLevel !== null && i.stock <= i.reorderLevel);
  function expiry(today: string, days?: number): ExpiringLot[] {
    date('today', today); if (days !== undefined && (!Number.isInteger(days) || days < 0 || !addDateDays(today, days))) invalid('days');
    return lots.filter(l => !l.deletedAt && l.quantity > 0 && l.expiresOn && !item(l.itemId, false).deletedAt && item(l.itemId).isActive && (days === undefined ? l.expiresOn < today : l.expiresOn >= today && l.expiresOn <= addDateDays(today, days)))
      .sort((a, b) => a.expiresOn!.localeCompare(b.expiresOn!) || a.id - b.id).map(l => ({ item: item(l.itemId), lot: l }));
  }
  function consume(input: ConsumeInput) {
    if (!item(input.itemId).isActive) conflict(); date('movedOn', input.movedOn); number('quantity', input.quantity, Number.MIN_VALUE);
    if (!['use', 'sale'].includes(input.reason)) invalid('reason');
    if ((input.refTable === null) !== (input.refId === null)) invalid('refTable');
    if (input.refTable) {
      const i = item(input.itemId);
      if (input.refTable === 'vaccinations') { const row = ctx.catalog('vaccination_get', { id: input.refId }) as Vaccination; if (row.deletedAt || i.vaccineTypeId === null || row.vaccineTypeId !== i.vaccineTypeId) invalid('refId'); }
      else if (input.refTable === 'prescription_items') { const row = ctx.catalog('prescription_get', { id: input.refId }) as PrescriptionItem; if (row.deletedAt || i.medicationId === null || row.medicationId !== i.medicationId) invalid('refId'); }
      else invalid('refTable');
      if (movements.some(m => m.itemId === input.itemId && m.refTable === input.refTable && m.refId === input.refId)) conflict();
    }
    const eligible = itemLots(input.itemId).filter(l => l.quantity > 0 && (!l.expiresOn || l.expiresOn >= input.movedOn));
    const available = eligible.reduce((sum, l) => sum + l.quantity, 0), tolerance = Math.max(input.quantity, available) * Number.EPSILON * 4;
    if (!input.allowNegative && available < input.quantity && input.quantity - available > tolerance) invalid('quantity');
    let left = input.quantity; const result: StockMovement[] = [];
    for (const l of eligible) { if (left <= 0) break; const take = Math.min(left, l.quantity); result.push(move(input.itemId, l.id, -take, input.reason, input.movedOn, input.notes, input.refTable, input.refId)); left -= take; if (left <= tolerance) left = 0; }
    if (left > 0) result.push(move(input.itemId, null, -left, input.reason, input.movedOn, input.notes, input.refTable, input.refId));
    return result;
  }
  function dispatch(command: string, args: unknown): unknown {
    switch (command) {
      case 'inventory_items': return list((args as CommandMap['inventory_items']['args']).includeDeleted);
      case 'inventory_item_get': return item((args as { id: number }).id, false);
      case 'inventory_item_create': return create((args as { input: ItemInput }).input);
      case 'inventory_item_update': { const { id, input } = args as CommandMap['inventory_item_update']['args']; const row = item(id); prepare(input, id); Object.assign(row, input, { updatedAt: ctx.now() }); return item(id); }
      case 'inventory_item_set_deleted': { const { id, deleted } = args as CommandMap['inventory_item_set_deleted']['args']; const row = item(id, false); if (!!row.deletedAt === deleted || (deleted && (row.stock !== 0 || row.unallocatedDelta !== 0))) conflict(); if (!deleted) prepare(row, id); row.updatedAt = ctx.now(); row.deletedAt = deleted ? row.updatedAt : null; return row; }
      case 'inventory_from_catalog': { const { kind, catalogId } = args as CommandMap['inventory_from_catalog']['args']; return catalog(kind, catalogId); }
      case 'inventory_lots': { const { itemId, includeDeleted } = args as CommandMap['inventory_lots']['args']; return itemLots(itemId, includeDeleted); }
      case 'inventory_lot_create': { const { input, movedOn } = args as CommandMap['inventory_lot_create']['args']; date('movedOn', movedOn); const row = insertLot(input); if (input.quantity) move(input.itemId, row.id, input.quantity, 'adjust', movedOn, input.notes); return row; }
      case 'inventory_lot_update': { const { id, input, movedOn, reason } = args as CommandMap['inventory_lot_update']['args']; prepareLot(input); date('movedOn', movedOn); const row = lot(id); if (row.itemId !== input.itemId) conflict(); if (row.quantity !== input.quantity) { if (!reason.trim()) invalid('notes'); move(row.itemId, id, input.quantity - row.quantity, 'adjust', movedOn, reason); } Object.assign(row, input, { updatedAt: ctx.now() }); return row; }
      case 'inventory_lot_set_deleted': { const { id, deleted } = args as CommandMap['inventory_lot_set_deleted']['args']; const row = lot(id, false); if (!!row.deletedAt === deleted || row.quantity !== 0) conflict(); row.updatedAt = ctx.now(); row.deletedAt = deleted ? row.updatedAt : null; return row; }
      case 'inventory_receive': { const { input } = args as { input: ReceiveInput }; prepareLot(input.lot); date('movedOn', input.movedOn); if (input.lot.quantity <= 0) invalid('quantity'); if (!item(input.lot.itemId).isActive) conflict(); const row = input.lotId === null ? insertLot(input.lot) : lot(input.lotId); if (row.itemId !== input.lot.itemId) conflict(); move(row.itemId, row.id, input.lot.quantity, 'purchase', input.movedOn, input.lot.notes); return row; }
      case 'inventory_adjust': { const { input } = args as { input: AdjustInput }; if (!input.notes.trim()) invalid('notes'); return move(lot(input.lotId).itemId, input.lotId, input.delta, 'adjust', input.movedOn, input.notes); }
      case 'inventory_consume': return consume((args as { input: ConsumeInput }).input);
      case 'inventory_write_off_expired': { const { itemId, today } = args as CommandMap['inventory_write_off_expired']['args']; date('today', today); item(itemId); return itemLots(itemId).filter(l => l.quantity > 0 && l.expiresOn && l.expiresOn < today).map(l => move(itemId, l.id, -l.quantity, 'expired', today)); }
      case 'inventory_history': { const { itemId } = args as { itemId: number }; item(itemId, false); return movements.filter(m => m.itemId === itemId).sort((a, b) => b.movedOn.localeCompare(a.movedOn) || b.id - a.id); }
      case 'inventory_low_stock': return low();
      case 'inventory_expiring': { const { today, days } = args as CommandMap['inventory_expiring']['args']; return expiry(today, days); }
      case 'inventory_expired': return expiry((args as { today: string }).today);
      case 'inventory_alerts': {
        const today = (args as { today?: string }).today ?? localToday();
        const distinct = (rows: ExpiringLot[]) => rows.filter((r, n) => rows.findIndex(i => i.item.id === r.item.id) === n);
        const lowStock = low(), expiring = distinct(expiry(today, 60)), expired = distinct(expiry(today));
        return { itemCount: list().length, lowStockCount: lowStock.length, expiringCount: expiring.length, expiredCount: expired.length, lowStock: lowStock.slice(0, 5), expiring: expiring.slice(0, 5), expired: expired.slice(0, 5) };
      }
      default: return ctx.fail('not_found', 'Unknown inventory command.');
    }
  }
  const today = localToday();
  const med = (ctx.catalog('medications_list', {}) as Medication[])[0];
  const vaccine = (ctx.catalog('vaccine_types_list', {}) as VaccineType[]).find(v => v.isActive);
  const seeds = [med && catalog('medication', med.id), vaccine && catalog('vaccine', vaccine.id), create({ name: tr.inventoryExampleSupply, kind: 'supply', medicationId: null, vaccineTypeId: null, unit: tr.inventoryExampleUnit, salePriceMinor: 4500, reorderLevel: 10, isActive: true, notes: null })].filter((i): i is InventoryItem => !!i);
  for (const [n, i] of seeds.entries()) {
    i.reorderLevel = 5; i.salePriceMinor = 12500 + n * 2000;
    for (const [index, days] of [n === 0 ? -3 : 30, 180].entries()) {
      const l = insertLot({ itemId: i.id, lotNo: `DEMO-${n + 1}-${index + 1}`, expiresOn: addDateDays(today, days), quantity: 0, costMinor: 6500, receivedOn: today, notes: null });
      move(i.id, l.id, n === 2 ? 2 : 4, 'purchase', today);
    }
  }
  return {
    has: (command: string) => command.startsWith('inventory_'),
    call(command: string, args: unknown): unknown {
      const snapshot = structuredClone({ items, lots, movements, nextId });
      try { return dispatch(command, args); }
      catch (error) { ({ items, lots, movements, nextId } = snapshot); throw error; }
    }
  };
}
