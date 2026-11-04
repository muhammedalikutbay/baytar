<script lang="ts">
  import { Search, Package } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import { Button, Input, Select, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { call } from '$lib/api/client';
  import * as inventory from '$lib/api/inventory';
  import type { InventoryItem } from '$lib/api/types';
  let { onsaved, oncancel }: { onsaved: (item: InventoryItem) => void; oncancel: () => void } = $props();
  let rows = $state<{ id: number; name: string; kind: 'medication' | 'vaccine' }[]>([]), query = $state(''), selection = $state('');
  let loading = $state(true), saving = $state(false), error = $state('');
  const options = $derived(rows.filter(r => r.name.toLocaleLowerCase('tr-TR').includes(query.toLocaleLowerCase('tr-TR'))).map(r => ({ value: `${r.kind}:${r.id}`, label: `${r.name} · ${r.kind === 'medication' ? tr.inventoryMedication : tr.inventoryVaccine}` })));
  async function load() {
    loading = true; error = '';
    try { const [meds, vaccines] = await Promise.all([call('medications_list', {}), call('vaccine_types_list', {})]); rows = [...meds.map(m => ({ id: m.id, name: m.name, kind: 'medication' as const })), ...vaccines.filter(v => v.isActive).map(v => ({ id: v.id, name: v.name, kind: 'vaccine' as const }))]; }
    catch { error = tr.loadError; } finally { loading = false; }
  }
  onMount(load);
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (!selection || saving) return; saving = true; error = '';
    const [kind, id] = selection.split(':');
    try { onsaved(await inventory.fromCatalog(kind as 'medication' | 'vaccine', Number(id))); }
    catch { error = tr.inventorySaveError; } finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4">
  <RequiredNote />
  <Input icon={Search} label={tr.inventoryCatalogSearch} type="search" bind:value={query} />
  {#if loading}<p role="status">{tr.loading}</p>{:else}<Select icon={Package} label={tr.inventoryCatalogEntry} bind:value={selection} options={options} placeholder={tr.inventoryCatalogEntry} required />{/if}
  {#if !loading && options.length === 0}<p class="text-fg-muted">{tr.noResults}</p>{/if}
  {#if error}<p role="alert" class="text-danger">{error}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || loading || !selection}>{saving ? tr.saving : tr.inventoryCatalog}</Button></div>
</form>
