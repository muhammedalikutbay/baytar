<script lang="ts">
  import { Package, TriangleAlert, CalendarClock, CalendarX2 } from '@lucide/svelte';
  import { PageHeader, Button, Card, StatTile, Badge, Input, Select, Checkbox, Dialog, EmptyState, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatNumber, formatDate, formatMoney, localToday } from '$lib/format';
  import * as inventory from '$lib/api/inventory';
  import type { InventoryItem, InventoryAlerts } from '$lib/api/types';
  import ItemForm from '$lib/components/inventory/ItemForm.svelte';
  import CatalogForm from '$lib/components/inventory/CatalogForm.svelte';
  import { kindLabels, kindOptions, warnings } from '$lib/components/inventory/options';
  let rows = $state<InventoryItem[]>([]), alerts = $state<InventoryAlerts | null>(null);
  let query = $state(''), kind = $state(''), warningsOnly = $state(false), archived = $state(false), open = $state<'item' | 'catalog' | null>(null);
  let loading = $state(true), error = $state(false), refresh = $state(0);
  $effect(() => {
    refresh; const includeDeleted = archived; let cancelled = false; loading = true; error = false;
    Promise.all([inventory.list(includeDeleted), inventory.alerts(localToday())]).then(([items, counts]) => { if (!cancelled) { rows = items; alerts = counts; } })
      .catch(() => { if (!cancelled) error = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  const visible = $derived(rows.filter(i => (!kind || i.kind === kind) && (!warningsOnly || warnings(i).length > 0) && i.name.toLocaleLowerCase('tr-TR').includes(query.trim().toLocaleLowerCase('tr-TR'))));
  function saved() { open = null; refresh++; }
</script>
<div class="grid gap-8">
  <PageHeader title={tr.inventory} subtitle={tr.inventoryDescription} icon={Package}>
    {#snippet actions()}<Button variant="secondary" onclick={() => open = 'catalog'}>{tr.inventoryCatalog}</Button><Button onclick={() => open = 'item'}>{tr.inventoryNew}</Button>{/snippet}
  </PageHeader>
  <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
    <StatTile icon={Package} value={alerts?.itemCount ?? '—'} label={tr.inventoryItems} tone="neutral" />
    <StatTile icon={TriangleAlert} value={alerts?.lowStockCount ?? '—'} label={tr.inventoryLow} tone="highlight" />
    <StatTile icon={CalendarClock} value={alerts?.expiringCount ?? '—'} label={tr.inventoryExpiring} tone="highlight" />
    <StatTile icon={CalendarX2} value={alerts?.expiredCount ?? '—'} label={tr.inventoryExpired} tone="highlight" />
  </div>
  <Card title={tr.inventory} icon={Package} tone="highlight" count={visible.length}>
    <div class="mb-5 grid gap-4 sm:grid-cols-2">
      <Input label={tr.inventorySearch} type="search" bind:value={query} />
      <Select label={tr.inventoryKind} bind:value={kind} options={[{ value: '', label: tr.inventoryAllKinds }, ...kindOptions]} />
      <Checkbox label={tr.inventoryWarningsOnly} bind:checked={warningsOnly} />
      <Checkbox label={tr.inventoryArchived} bind:checked={archived} />
    </div>
    {#if loading}<p role="status" class="py-8 text-fg-muted">{tr.loading}</p>
    {:else if error}<p role="alert" class="mb-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
    {:else if visible.length === 0}
      <EmptyState icon={Package} title={rows.length ? tr.noResults : tr.inventoryEmpty} description={rows.length ? tr.inventoryNoResults : tr.inventoryEmptyDescription}>
        {#snippet action()}<Button variant="secondary" onclick={() => open = 'catalog'}>{tr.inventoryCatalog}</Button>{/snippet}
      </EmptyState>
    {:else}
      <div class="overflow-x-auto"><table class="w-full text-left">
        <thead class="border-b border-border bg-surface-muted type-caption text-fg-muted"><tr>
          <th scope="col" class="px-3 py-3">{tr.inventoryName}</th><th scope="col" class="px-3 py-3">{tr.inventoryKind}</th>
          <th scope="col" class="px-3 py-3">{tr.inventoryStock}</th><th scope="col" class="px-3 py-3">{tr.inventoryNearestExpiry}</th>
          <th scope="col" class="px-3 py-3">{tr.inventoryPrice}</th><th scope="col" class="px-3 py-3">{tr.inventoryReorder}</th><th scope="col" class="px-3 py-3">{tr.inventoryStatus}</th>
        </tr></thead>
        <tbody class="divide-y divide-border">{#each visible as item (item.id)}
          <tr class="hover:bg-surface-muted"><th scope="row" class="px-3 py-4 font-medium"><a class="text-accent underline underline-offset-4" href={`/inventory/item?id=${item.id}`}>{item.name}</a></th>
            <td class="px-3 py-4"><Badge>{kindLabels[item.kind]}</Badge></td>
            <td class="whitespace-nowrap px-3 py-4 tabular-nums">{formatNumber(item.stock)} {item.unit}</td>
            <td class="whitespace-nowrap px-3 py-4">{formatDate(item.nearestExpiry) || '—'}</td>
            <td class="whitespace-nowrap px-3 py-4 tabular-nums">{formatMoney(item.salePriceMinor)}</td>
            <td class="px-3 py-4 tabular-nums">{item.reorderLevel === null ? '—' : formatNumber(item.reorderLevel)}</td>
            <td class="px-3 py-4"><div class="flex flex-wrap gap-1">
              {#if item.deletedAt}<Badge>{tr.inventoryDeleted}</Badge>{:else if !item.isActive}<Badge>{tr.inventoryInactive}</Badge>
              {:else if warnings(item).length}{#each warnings(item) as warning}<Badge variant="highlight">{warning}</Badge>{/each}
              {:else}<Badge variant="success">{tr.inventoryHealthy}</Badge>{/if}
            </div></td>
          </tr>
        {/each}</tbody>
      </table></div>
      <p class="type-caption mt-4 text-fg-muted">{tr.inventoryPhysicalHint}</p>
    {/if}
  </Card>
</div>
<Dialog bind:open={() => open !== null, (v) => { if (!v) open = null; }} title={open === 'catalog' ? tr.inventoryCatalog : tr.inventoryNew} description={open === 'catalog' ? tr.inventoryCatalogDescription : tr.inventoryItemDescription}>
  {#if open === 'item'}<ItemForm onsaved={saved} oncancel={() => open = null} />{:else if open === 'catalog'}<CatalogForm onsaved={saved} oncancel={() => open = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
