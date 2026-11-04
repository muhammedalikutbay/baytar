<script lang="ts">
  import { page } from '$app/state';
  import { Package, History, Boxes } from '@lucide/svelte';
  import { PageHeader, Button, Card, StatTile, Badge, Dialog, Checkbox, EmptyState, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatNumber, formatMoney, localToday } from '$lib/format';
  import * as inventory from '$lib/api/inventory';
  import type { InventoryItem, InventoryLot, StockMovement, ApiError } from '$lib/api/types';
  import { kindLabels, reasonLabels } from '$lib/components/inventory/options';
  import ItemForm from '$lib/components/inventory/ItemForm.svelte';
  import StockForm from '$lib/components/inventory/StockForm.svelte';
  let item = $state<InventoryItem | null>(null), lots = $state<InventoryLot[]>([]), movements = $state<StockMovement[]>([]);
  let loading = $state(true), loadError = $state(false), error = $state(''), refresh = $state(0), busy = $state(false), archived = $state(false);
  let mode = $state<'edit' | 'receive' | 'adjust' | 'lot' | 'consume' | 'writeoff' | null>(null), editing = $state<InventoryLot | undefined>();
  let dialogOpen = $state(false);
  const id = $derived(Number(page.url.searchParams.get('id')));
  $effect(() => {
    refresh; const current = id, includeDeleted = archived; let cancelled = false; loading = true; loadError = false;
    Promise.all([inventory.get(current), inventory.lots(current, includeDeleted), inventory.history(current)]).then(([i, l, m]) => { if (!cancelled) { item = i; lots = l; movements = m; } })
      .catch(() => { if (!cancelled) loadError = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  const expired = $derived(lots.filter(l => !l.deletedAt && l.quantity > 0 && l.expiresOn && l.expiresOn < localToday()));
  const title = $derived(mode === 'edit' ? tr.inventoryEdit : mode === 'receive' ? tr.inventoryReceive : mode === 'adjust' ? tr.inventoryAdjust : mode === 'lot' ? tr.inventoryEditLot : mode === 'consume' ? tr.inventoryConsume : tr.inventoryWriteOff);
  const description = $derived(mode === 'edit' ? tr.inventoryItemDescription : mode === 'receive' ? tr.inventoryReceiveDescription : mode === 'adjust' ? tr.inventoryAdjustDescription : mode === 'lot' ? tr.inventoryEditLotDescription : mode === 'consume' ? tr.inventoryConsumeDescription : tr.inventoryWriteOffDescription);
  function show(value: typeof mode, lot?: InventoryLot) { error = ''; editing = lot; mode = value; dialogOpen = true; }
  function saved() { dialogOpen = false; refresh++; }
  async function action(fn: () => Promise<unknown>) {
    if (busy) return; busy = true; error = '';
    try { await fn(); saved(); } catch (e) { error = (e as ApiError)?.kind === 'conflict' ? tr.inventoryConflict : tr.inventorySaveError; } finally { busy = false; }
  }
</script>
<div class="grid gap-8">
  <a href="/inventory" class="w-fit text-accent underline underline-offset-4">{tr.inventoryBack}</a>
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if loadError || !item}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
  {:else}
    <PageHeader title={item.name} subtitle={`${kindLabels[item.kind]} · ${item.unit}`} icon={Package}>
      {#snippet actions()}
        {#if item!.deletedAt}<Button onclick={() => action(() => inventory.restore(id))} disabled={busy}>{tr.inventoryRestore}</Button>
        {:else}<Button variant="secondary" onclick={() => show('edit')}>{tr.edit}</Button><Button disabled={!item!.isActive} onclick={() => show('receive')}>{tr.inventoryReceive}</Button>{/if}
      {/snippet}
    </PageHeader>
    {#if error && !dialogOpen}<p role="alert" class="text-danger">{error}</p>{/if}
    <div class="grid gap-4 sm:grid-cols-3">
      <StatTile icon={Boxes} value={`${formatNumber(item.stock)} ${item.unit}`} label={tr.inventoryStock} hint={tr.inventoryPhysicalHint} tone="highlight" />
      <StatTile icon={Package} value={formatMoney(item.salePriceMinor)} label={tr.inventoryPrice} tone="neutral" />
      <StatTile icon={History} value={formatNumber(item.unallocatedDelta)} label={tr.inventoryUnallocated} hint={tr.inventoryUnallocatedHint} tone={item.unallocatedDelta < 0 ? 'highlight' : 'neutral'} />
    </div>
    <div class="flex flex-wrap items-center gap-3">
      <Badge>{item.deletedAt ? tr.inventoryDeleted : item.isActive ? tr.inventoryActive : tr.inventoryInactive}</Badge>
      <span class="text-fg-muted">{tr.inventoryReorder}: {item.reorderLevel === null ? '—' : `${formatNumber(item.reorderLevel)} ${item.unit}`}</span>
      {#if item.notes}<p class="whitespace-pre-wrap">{item.notes}</p>{/if}
    </div>
    {#if !item.deletedAt}<div class="flex flex-wrap gap-2">
      <Button variant="secondary" disabled={!lots.some(l => !l.deletedAt)} onclick={() => show('adjust')}>{tr.inventoryAdjust}</Button>
      <Button variant="secondary" disabled={!item.isActive} onclick={() => show('consume')}>{tr.inventoryConsume}</Button>
      <Button variant="destructive" disabled={expired.length === 0} onclick={() => show('writeoff')}>{tr.inventoryWriteOff}</Button>
      <Button variant="ghost" disabled={busy || item.stock !== 0 || item.unallocatedDelta !== 0} onclick={() => action(() => inventory.softDelete(id))}>{tr.delete}</Button>
    </div>{/if}
    <Card title={tr.inventoryLots} icon={Boxes} tone="highlight" count={lots.length}>
      <div class="mb-4"><Checkbox label={tr.inventoryArchived} bind:checked={archived} /></div>
      {#if lots.length === 0}<EmptyState icon={Boxes} title={tr.inventoryNoLots} description={tr.inventoryNoLotsDescription}>
        {#snippet action()}{#if !item!.deletedAt}<Button onclick={() => show('receive')} disabled={!item!.isActive}>{tr.inventoryReceive}</Button>{/if}{/snippet}
      </EmptyState>{:else}
      <div class="overflow-x-auto"><table class="w-full text-left"><thead class="border-b border-border type-caption text-fg-muted"><tr>
        <th scope="col" class="p-3">{tr.inventoryLotNo}</th><th scope="col" class="p-3">{tr.inventoryExpiry}</th><th scope="col" class="p-3">{tr.inventoryQuantity}</th><th scope="col" class="p-3">{tr.inventoryCost}</th><th scope="col" class="p-3">{tr.inventoryReceivedOn}</th><th scope="col" class="p-3"><span class="sr-only">{tr.edit}</span></th>
      </tr></thead><tbody class="divide-y divide-border">{#each lots as lot (lot.id)}<tr>
        <th scope="row" class="p-3 font-medium">{lot.lotNo ?? tr.inventoryUnspecified}{#if lot.deletedAt} <Badge>{tr.inventoryDeleted}</Badge>{/if}</th>
        <td class="p-3">{formatDate(lot.expiresOn) || '—'}{#if !lot.deletedAt && lot.quantity > 0 && lot.expiresOn && lot.expiresOn < localToday()} <Badge variant="highlight">{tr.inventoryExpired}</Badge>{/if}</td>
        <td class="whitespace-nowrap p-3 tabular-nums">{formatNumber(lot.quantity)} {item.unit}</td><td class="p-3 tabular-nums">{formatMoney(lot.costMinor)}</td><td class="p-3">{formatDate(lot.receivedOn) || '—'}</td>
        <td class="p-3">{#if !item.deletedAt}<div class="flex gap-2">{#if lot.deletedAt}<Button size="sm" variant="secondary" disabled={busy} onclick={() => action(() => inventory.restoreLot(lot.id))}>{tr.inventoryRestore}</Button>{:else}<Button size="sm" variant="secondary" onclick={() => show('lot', lot)}>{tr.edit}</Button><Button size="sm" variant="ghost" disabled={busy || lot.quantity !== 0} onclick={() => action(() => inventory.deleteLot(lot.id))}>{tr.delete}</Button>{/if}</div>{/if}</td>
      </tr>{/each}</tbody></table></div>{/if}
    </Card>
    <Card title={tr.inventoryMovements} icon={History} tone="highlight" count={movements.length}>
      {#if movements.length === 0}<p class="text-fg-muted">{tr.inventoryNoMovements}</p>{:else}<div class="overflow-x-auto"><table class="w-full text-left"><thead class="border-b border-border type-caption text-fg-muted"><tr>
        <th scope="col" class="p-3">{tr.inventoryMovedOn}</th><th scope="col" class="p-3">{tr.inventoryMovementReason}</th><th scope="col" class="p-3">{tr.inventoryLot}</th><th scope="col" class="p-3">{tr.inventoryDelta}</th><th scope="col" class="p-3">{tr.inventoryReference}</th><th scope="col" class="p-3">{tr.inventoryNotes}</th>
      </tr></thead><tbody class="divide-y divide-border">{#each movements as movement (movement.id)}<tr>
        <td class="whitespace-nowrap p-3">{formatDate(movement.movedOn)}</td><td class="p-3"><Badge variant={movement.reason === 'expired' ? 'highlight' : 'neutral'}>{reasonLabels[movement.reason]}</Badge></td>
        <td class="p-3">{movement.lotId === null ? tr.inventoryUnallocated : lots.find(l => l.id === movement.lotId)?.lotNo ?? `#${movement.lotId}`}</td>
        <td class="whitespace-nowrap p-3 tabular-nums">{movement.delta > 0 ? '+' : ''}{formatNumber(movement.delta)} {item.unit}</td>
        <td class="p-3">{movement.refId === null ? '—' : `${movement.refTable === 'vaccinations' ? tr.inventoryVaccine : tr.inventoryMedication} #${movement.refId}`}</td><td class="whitespace-pre-wrap p-3 text-fg-muted">{movement.notes ?? '—'}</td>
      </tr>{/each}</tbody></table></div>{/if}
    </Card>
  {/if}
</div>
<Dialog bind:open={dialogOpen} {title} {description}>
  {#if dialogOpen && item}
    {#if mode === 'edit'}<ItemForm {item} onsaved={saved} oncancel={() => dialogOpen = false} />
    {:else if mode === 'writeoff'}
      <ul class="grid gap-2">{#each expired as lot}<li>{lot.lotNo ?? tr.inventoryUnspecified}: {formatNumber(lot.quantity)} {item.unit} · {formatDate(lot.expiresOn)}</li>{/each}</ul>
      {#if error}<p role="alert" class="mt-4 text-danger">{error}</p>{/if}
      <div class="mt-5 flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={() => dialogOpen = false}>{tr.cancel}</Button><Button variant="destructive" disabled={busy} onclick={() => action(() => inventory.writeOffExpired(id, localToday()))}>{tr.inventoryConfirmWriteOff}</Button></div>
    {:else if mode}<StockForm {item} {lots} mode={mode} {editing} onsaved={saved} oncancel={() => dialogOpen = false} />{/if}
  {/if}
  {#snippet footer()}{/snippet}
</Dialog>
