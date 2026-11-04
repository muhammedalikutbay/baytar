<script lang="ts">
  import { Checkbox, NumberInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { localToday } from '$lib/format';
  import * as inventory from '$lib/api/inventory';
  import * as settings from '$lib/api/settings';
  import type { InventoryItem } from '$lib/api/types';
  // Mount in a create form and call consumeAfterSave once with the saved clinical record id.
  // Quantities must be entered in stock units; prescribed dose is deliberately not inferred.
  let { medicationId = null, vaccineTypeId = null }: { medicationId?: number | null; vaccineTypeId?: number | null } = $props();
  let linked = $state<InventoryItem | null>(null), checked = $state(true), quantity = $state<number | null>(1), ready = $state(false), error = $state('');
  let successfulRef: string | null = null;
  $effect(() => {
    const med = medicationId, vaccine = vaccineTypeId; let cancelled = false; ready = false; linked = null; error = '';
    Promise.all([inventory.list(), settings.getAll()]).then(([items, preferences]) => {
      if (cancelled) return;
      linked = items.find(i => i.isActive && (med !== null ? i.medicationId === med : vaccine !== null && i.vaccineTypeId === vaccine)) ?? null;
      checked = preferences.inventory_deduct_default !== '0'; ready = true;
    }).catch(() => { if (!cancelled) error = tr.loadError; });
    return () => { cancelled = true; };
  });
  async function preference(value: boolean) { checked = value; try { await settings.set('inventory_deduct_default', value ? '1' : '0'); } catch { error = tr.inventoryPreferenceError; } }
  export async function consumeAfterSave(refTable: 'vaccinations' | 'prescription_items', refId: number, movedOn = localToday()) {
    if (!ready) throw new Error(tr.loadError);
    if (!linked || !checked) return;
    if (quantity === null || !Number.isFinite(quantity) || quantity <= 0) throw new Error(tr.inputNumberInvalid);
    const ref = `${refTable}:${refId}`; if (successfulRef === ref) return;
    try { await inventory.consume({ itemId: linked.id, quantity, reason: 'use', movedOn, allowNegative: false, refTable, refId, notes: null }); successfulRef = ref; }
    catch { error = tr.inventoryConsumeSavedError; throw new Error(tr.inventoryConsumeSavedError); }
  }
</script>
{#if linked}
  <div class="grid gap-2 rounded-md border border-border p-3">
    <Checkbox label={`${tr.inventoryDeduct} · ${linked.name}`} {checked} onCheckedChange={preference} />
    {#if checked}<NumberInput label={tr.inventoryQuantity} bind:value={quantity} min={Number.MIN_VALUE} required unit={linked.unit} /><p class="type-caption text-fg-muted">{tr.inventoryDeductHint}</p>{/if}
  </div>
{/if}
{#if error}<p role="alert" class="type-caption text-danger">{error}</p>{/if}
