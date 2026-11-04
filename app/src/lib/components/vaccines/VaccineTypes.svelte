<script lang="ts">
  import { Card, Button, Badge, Dialog, Toast } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as types from '$lib/api/vaccine_types';
  import type { VaccineType } from '$lib/api/types';
  import VaccineTypeForm from './VaccineTypeForm.svelte';
  let rows = $state<VaccineType[]>([]);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let open = $state(false);
  let editing = $state<VaccineType | undefined>();
  let removing = $state<VaccineType | null>(null);
  let busy = $state(false);
  let error = $state('');
  let deleted = $state<VaccineType | null>(null);
  let toast = $state(false);
  $effect(() => {
    refresh; let cancelled = false; loading = true; failed = false;
    types.list().then(value => { if (!cancelled) rows = value; }).catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  async function toggle(row: VaccineType) {
    if (busy) return; busy = true; error = '';
    try { await types.update(row.id, { isActive: !row.isActive }); refresh++; }
    catch { error = tr.actionError; } finally { busy = false; }
  }
  async function remove() {
    if (!removing || busy) return; busy = true; error = '';
    try { deleted = await types.softDelete(removing.id); removing = null; toast = true; refresh++; }
    catch { error = tr.actionError; } finally { busy = false; }
  }
  async function undo() {
    if (!deleted || busy) return; busy = true; error = '';
    try { await types.restore(deleted.id); deleted = null; toast = false; refresh++; }
    catch { error = tr.actionError; } finally { busy = false; }
  }
</script>
<section aria-label={tr.vaccineTypes}><Card>
  <div class="mb-4 flex flex-wrap items-center justify-between gap-3"><h2 class="type-section">{tr.vaccineTypes}</h2><Button variant="secondary" size="sm" onclick={() => { editing = undefined; open = true; }}>{tr.vaccineTypeAdd}</Button></div>
  {#if error && !removing}<p role="alert" class="text-danger">{error}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
  {:else if !rows.length}<p class="text-fg-muted">{tr.vaccineTypesEmpty}</p>
  {:else}<ul class="divide-y divide-border">{#each rows as row (row.id)}<li class="flex flex-wrap items-center justify-between gap-3 py-3">
    <div class="min-w-0"><p class="font-medium break-words">{row.name} <Badge>{row.isActive ? tr.active : tr.inactive}</Badge></p>
      <p class="type-caption text-fg-muted">{row.species ? tr[row.species] : tr.anySpecies}{row.defaultIntervalDays !== null ? ` · ${row.defaultIntervalDays} ${tr.dayUnit}` : ''}</p>
      {#if row.notes}<p class="mt-1 whitespace-pre-wrap break-words text-fg-muted">{row.notes}</p>{/if}</div>
    <div class="flex flex-wrap gap-2"><Button size="sm" variant="secondary" onclick={() => { editing = row; open = true; }}>{tr.edit}</Button>
      <Button size="sm" variant="ghost" disabled={busy} onclick={() => toggle(row)}>{row.isActive ? tr.deactivate : tr.activate}</Button>
      <Button size="sm" variant="danger" disabled={busy} onclick={() => { error = ''; removing = row; }}>{tr.delete}</Button></div>
  </li>{/each}</ul>{/if}
</Card></section>
<Dialog bind:open title={editing ? tr.edit : tr.vaccineTypeAdd} description={tr.vaccineTypeDescription}>
  {#if open}<VaccineTypeForm type={editing} onsaved={() => { open = false; refresh++; }} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={() => removing !== null, value => { if (!value) removing = null; }} title={tr.delete} description={tr.vaccineDeleteDescription}>
  {#if removing}<p class="font-medium">{removing.name}</p>{/if}
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#snippet footer()}<Button variant="secondary" onclick={() => removing = null}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
<Toast bind:open={toast} message={tr.vaccineDeleted}>
  {#snippet action()}{#if deleted}<Button size="sm" variant="secondary" disabled={busy} onclick={undo}>{tr.undo}</Button>{/if}{/snippet}
</Toast>
