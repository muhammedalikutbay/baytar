<script lang="ts">
  import { untrack } from 'svelte';
  import { Button, Input, Select, NumberInput, MoneyInput, DateInput, Textarea, Checkbox } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { localToday, formatNumber } from '$lib/format';
  import { validateForm } from '$lib/validation';
  import * as inventory from '$lib/api/inventory';
  import type { InventoryItem, InventoryLot, ApiError } from '$lib/api/types';
  let { item, lots, mode, editing, onsaved, oncancel }: { item: InventoryItem; lots: InventoryLot[]; mode: 'receive' | 'adjust' | 'lot' | 'consume'; editing?: InventoryLot; onsaved: () => void; oncancel: () => void } = $props();
  const initial = untrack(() => editing);
  let selected = $state(initial ? String(initial.id) : untrack(() => mode) === 'adjust' ? String(untrack(() => lots[0]?.id) ?? '') : '');
  let lotNo = $state(initial?.lotNo ?? ''), expiresOn = $state(initial?.expiresOn ?? ''), receivedOn = $state(initial?.receivedOn ?? localToday());
  let quantity = $state<number | null>(initial?.quantity ?? null), cost = $state<number | null>(initial?.costMinor ?? null);
  let movedOn = $state(localToday()), notes = $state(''), reason = $state('use'), allowNegative = $state(false), saving = $state(false), error = $state('');
  const options = $derived(lots.filter(l => !l.deletedAt).map(l => ({ value: String(l.id), label: `${l.lotNo ?? tr.inventoryUnspecified} · ${l.expiresOn ?? tr.inventoryUnspecified} · ${formatNumber(l.quantity)} ${item.unit}` })));
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving || !validateForm(event.currentTarget as HTMLFormElement) || quantity === null) return;
    saving = true; error = '';
    try {
      if (mode === 'adjust') await inventory.adjust({ lotId: Number(selected), delta: quantity, movedOn, notes });
      else if (mode === 'consume') await inventory.consume({ itemId: item.id, quantity, reason: reason as 'use' | 'sale', movedOn, allowNegative, refTable: null, refId: null, notes: notes.trim() || null });
      else {
        const input = { itemId: item.id, lotNo: lotNo.trim() || null, expiresOn: expiresOn || null, quantity, costMinor: cost, receivedOn: receivedOn || null, notes: mode === 'lot' ? initial?.notes ?? null : notes.trim() || null };
        if (mode === 'lot' && initial) await inventory.updateLot(initial.id, input, movedOn, notes);
        else await inventory.receive({ lotId: selected ? Number(selected) : null, lot: input, movedOn });
      }
      onsaved();
    } catch (e) { const err = e as ApiError; error = mode === 'consume' && err?.field === 'quantity' ? tr.inventoryInsufficient : tr.inventorySaveError; }
    finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4">
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    {#if mode === 'receive'}<Select label={tr.inventoryLot} bind:value={selected} options={[{ value: '', label: tr.inventoryNewLot }, ...options]} />{/if}
    {#if mode === 'adjust'}<Select label={tr.inventoryLot} bind:value={selected} options={options} required />{/if}
    {#if mode === 'lot' || mode === 'receive' && !selected}
      <Input label={tr.inventoryLotNo} bind:value={lotNo} />
      <DateInput label={tr.inventoryExpiry} bind:value={expiresOn} />
      <DateInput label={tr.inventoryReceivedOn} bind:value={receivedOn} />
      <MoneyInput label={tr.inventoryCost} bind:value={cost} />
    {/if}
    {#if mode === 'consume'}<Select label={tr.inventoryMovementReason} bind:value={reason} options={[{ value: 'use', label: tr.inventoryUse }, { value: 'sale', label: tr.inventorySale }]} />{/if}
    <NumberInput label={mode === 'adjust' ? tr.inventoryDelta : tr.inventoryQuantity} bind:value={quantity} required min={mode === 'adjust' ? undefined : mode === 'lot' ? 0 : Number.MIN_VALUE} unit={item.unit} />
    <DateInput label={tr.inventoryMovedOn} bind:value={movedOn} required />
    <Textarea label={mode === 'adjust' || mode === 'lot' ? tr.inventoryReason : tr.inventoryNotes} bind:value={notes} required={mode === 'adjust' || mode === 'lot' && initial?.quantity !== quantity} />
    {#if mode === 'consume'}<Checkbox label={tr.inventoryAllowNegative} bind:checked={allowNegative} /><p class="type-caption text-fg-muted">{tr.inventoryDeductHint}</p>{/if}
  </fieldset>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || mode === 'adjust' && !selected}>{saving ? tr.saving : tr.save}</Button></div>
</form>
