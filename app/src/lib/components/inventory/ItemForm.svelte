<script lang="ts">
  import { untrack } from 'svelte';
  import { Button, Input, Select, NumberInput, MoneyInput, Textarea, Checkbox } from '$lib/components/ui';
  import { validateForm } from '$lib/validation';
  import { tr } from '$lib/i18n/tr';
  import * as inventory from '$lib/api/inventory';
  import type { InventoryItem, InventoryKind, ApiError } from '$lib/api/types';
  import { kindOptions } from './options';
  let { item, onsaved, oncancel }: { item?: InventoryItem; onsaved: (item: InventoryItem) => void; oncancel: () => void } = $props();
  const initial = untrack(() => item);
  let name = $state(initial?.name ?? ''), kind = $state(initial?.kind ?? 'supply'), unit = $state(initial?.unit ?? 'adet');
  let price = $state<number | null>(initial?.salePriceMinor ?? null), reorder = $state<number | null>(initial?.reorderLevel ?? null);
  let active = $state(initial?.isActive ?? true), notes = $state(initial?.notes ?? ''), saving = $state(false), error = $state('');
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving || !validateForm(event.currentTarget as HTMLFormElement)) return;
    saving = true; error = '';
    const input = { name: name.trim(), kind: kind as InventoryKind, unit: unit.trim(), salePriceMinor: price, reorderLevel: reorder, isActive: active, notes: notes.trim() || null, medicationId: initial?.medicationId ?? null, vaccineTypeId: initial?.vaccineTypeId ?? null };
    try { onsaved(initial ? await inventory.update(initial.id, input) : await inventory.create(input)); }
    catch (e) { error = (e as ApiError)?.kind === 'conflict' ? tr.inventoryConflict : tr.inventorySaveError; }
    finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4">
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    <Input label={tr.inventoryName} bind:value={name} required />
    <Select label={tr.inventoryKind} bind:value={kind} options={kindOptions} disabled={!!initial?.medicationId || !!initial?.vaccineTypeId} />
    <Input label={tr.inventoryUnit} bind:value={unit} required />
    <MoneyInput label={tr.inventoryPrice} bind:value={price} />
    <NumberInput label={tr.inventoryReorder} bind:value={reorder} min={0} unit={unit} />
    <Checkbox label={tr.inventoryActive} bind:checked={active} />
    <Textarea label={tr.inventoryNotes} bind:value={notes} />
  </fieldset>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
