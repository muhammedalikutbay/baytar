import { tr } from '$lib/i18n/tr';
import type { InventoryKind, StockReason, InventoryItem } from '$lib/api/types';
import { daysUntil } from '$lib/format';
export const kindLabels: Record<InventoryKind, string> = { medication: tr.inventoryMedication, vaccine: tr.inventoryVaccine, supply: tr.inventorySupply, food: tr.inventoryFood, product: tr.inventoryProduct, other: tr.inventoryOther };
export const kindOptions = Object.entries(kindLabels).map(([value, label]) => ({ value, label }));
export const reasonLabels: Record<StockReason, string> = { purchase: tr.inventoryPurchase, use: tr.inventoryUse, sale: tr.inventorySale, adjust: tr.inventoryAdjustment, expired: tr.inventoryExpiredMovement, return: tr.inventoryReturn };
export function warnings(item: InventoryItem): string[] {
  if (item.deletedAt || !item.isActive) return [];
  const result: string[] = [];
  if (item.reorderLevel !== null && item.stock <= item.reorderLevel) result.push(tr.inventoryLow);
  const days = item.nearestExpiry ? daysUntil(item.nearestExpiry) : null;
  if (days !== null && days < 0) result.push(tr.inventoryExpired);
  else if (days !== null && days <= 60) result.push(tr.inventoryExpiring);
  if (item.unallocatedDelta < 0) result.push(tr.inventoryUnallocated);
  return result;
}
