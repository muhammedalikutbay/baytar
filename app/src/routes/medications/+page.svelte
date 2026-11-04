<script lang="ts">
  import { Pill, Search } from '@lucide/svelte';
  import { PageHeader, Button, Card, Dialog, EmptyState, Input, Toast, Select, Badge } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatNumber } from '$lib/format';
  import { fold } from '$lib/api/client';
  import * as medications from '$lib/api/medications';
  import type { Medication } from '$lib/api/types';
  import MedicationForm from '$lib/components/medications/MedicationForm.svelte';
  import DoseCalculator from '$lib/components/medications/DoseCalculator.svelte';
  import { formLabels, categoryLabels } from '$lib/components/medications/options';
  let rows = $state<Medication[]>([]), query = $state(''), category = $state(''), species = $state(''), statusFilter = $state('active');
  let loading = $state(true), loadError = $state(false), actionError = $state(false), refresh = $state(0);
  let open = $state(false), editing = $state<Medication | undefined>();
  let busy = $state<number[]>([]), deleted = $state<{ id: number; name: string; open: boolean }[]>([]);
  const categories = $derived([...new Set(rows.map(m => m.category).filter(c => c !== null))].sort((a,b) => categoryLabels[a].localeCompare(categoryLabels[b], 'tr')));
  const speciesOptions = $derived([...new Set(rows.flatMap(m => (m.targetSpecies ?? '').split(/[,;|]/).map(s => s.trim()).filter(Boolean)))].sort((a,b) => a.localeCompare(b,'tr')));
  const visible = $derived(rows.filter(m => (!query.trim() || fold(m.name + ' ' + (m.activeIngredient ?? '')).includes(fold(query)))
    && (!category || m.category === category) && (!species || (m.targetSpecies ?? '').split(/[,;|]/).some(s => s.trim() === species))
    && (statusFilter === 'all' || (statusFilter === 'inactive' ? !!m.deletedAt : !m.deletedAt))));
  $effect(() => {
    refresh; let cancelled = false; loading = true; loadError = false;
    medications.list(true).then(value => { if (!cancelled) rows = value; })
      .catch(() => { if (!cancelled) loadError = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  function edit(value?: Medication) { editing = value; open = true; }
  function saved() { open = false; refresh++; }
  async function remove(value: Medication) {
    if (busy.includes(value.id)) return;
    busy = [...busy, value.id]; actionError = false;
    try { await medications.softDelete(value.id); deleted = [...deleted.filter(m => m.id !== value.id), { id:value.id, name:value.name, open:true }]; refresh++; }
    catch { actionError = true; } finally { busy = busy.filter(id => id !== value.id); }
  }
  async function undo(id: number) {
    if (busy.includes(id)) return;
    busy = [...busy, id]; actionError = false;
    try { await medications.restore(id); deleted = deleted.filter(m => m.id !== id); open = false; refresh++; }
    catch { actionError = true; } finally { busy = busy.filter(value => value !== id); }
  }
  function doseRange(m: Medication): string {
    if (m.doseMgPerKgMin === null && m.doseMgPerKgMax === null) return tr.medicationUnspecified;
    return (m.doseMgPerKgMin === null ? '…' : formatNumber(m.doseMgPerKgMin)) + (m.doseMgPerKgMax === null ? '' : '–' + formatNumber(m.doseMgPerKgMax)) + ' mg/kg';
  }
</script>
<div class="grid gap-6">
  <PageHeader title={tr.medications} subtitle={tr.medicationsDescription} icon={Pill}>
    {#snippet actions()}<Button onclick={() => edit()}>{tr.medicationsAction}</Button>{/snippet}
  </PageHeader>
  <p class="text-fg-muted">{tr.medicationNoDoseData}</p>
  {#if actionError}<p role="alert" class="text-danger">{tr.actionError}</p>{/if}
  <div class="sticky top-10 z-20 grid gap-3 rounded-md border border-border bg-surface p-4 shadow-hairline">
    <div class="grid gap-3 sm:grid-cols-[minmax(0,2fr)_1fr_1fr]">
      <Input icon={Search} label={tr.medicationSearch} type="search" bind:value={query} />
      <Select label={tr.species} bind:value={species} options={[{value:'',label:tr.anySpecies}, ...speciesOptions.map(value => ({value,label:value}))]} />
      <Select label={tr.medicationState} bind:value={statusFilter} options={[{value:'active',label:tr.billingActive},{value:'inactive',label:tr.billingInactive},{value:'all',label:tr.medicationAllStates}]} />
    </div>
    <div role="group" aria-label={tr.medicationCategory} class="flex flex-wrap gap-2">
      <Button size="sm" variant="ghost" aria-pressed={!category} class={!category ? 'text-accent underline' : ''} onclick={() => category = ''}>{tr.medicationAllCategories}</Button>
      {#each categories as value}<Button size="sm" variant="ghost" aria-pressed={category === value} class={category === value ? 'text-accent underline' : ''} onclick={() => category = value}>{categoryLabels[value]}</Button>{/each}
    </div>
  </div>
  <Card title={tr.medications} icon={Pill} count={visible.length} class="p-0 sm:p-0 [&>div:first-child]:m-4">
    {#if loading}<p role="status" class="p-6 text-fg-muted">{tr.loading}</p>
    {:else if loadError}<div class="p-6"><p role="alert" class="mb-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></div>
    {:else if !visible.length}<EmptyState icon={Pill} title={rows.length ? tr.noResults : tr.medicationsEmptyTitle} description={rows.length ? tr.medicationNoResults : tr.medicationsEmptyDescription} />
    {:else}<div class="overflow-x-auto"><table class="w-full text-left text-sm">
      <thead class="type-caption border-b border-border bg-surface-muted text-fg-muted"><tr>
        <th scope="col" class="px-4 py-3">{tr.medicationName} / {tr.activeIngredient}</th><th scope="col" class="px-4 py-3">{tr.medicationForm} · {tr.medicationStrength}</th><th scope="col" class="px-4 py-3">{tr.medicationDoseRange}</th><th scope="col" class="px-4 py-3">{tr.medicationState}</th>
      </tr></thead><tbody class="divide-y divide-border">
        {#each visible as medication (medication.id)}<tr class="relative hover:bg-surface-muted focus-within:bg-surface-muted">
          <th scope="row" class="px-4 py-3 text-left font-normal"><button type="button" class="text-left font-medium after:absolute after:inset-0 focus-visible:outline-none focus-visible:after:outline-2 focus-visible:after:outline-accent" onclick={() => edit(medication)}>{medication.name}</button><span class="mt-1 block text-fg-muted">{medication.activeIngredient ?? tr.medicationUnspecified}</span>
            {#if medication.isBuiltin}<span class="mt-1 inline-block"><Badge>{tr.medicationBuiltin}</Badge></span>{/if}
          </th>
          <td class="px-4 py-3 text-fg-muted">{medication.form ? formLabels[medication.form] : tr.medicationUnspecified}
            {#if medication.concentrationMgPerMl !== null} · {formatNumber(medication.concentrationMgPerMl)} mg/ml{/if}
            {#if medication.strengthMgPerUnit !== null} · {formatNumber(medication.strengthMgPerUnit)} mg/{medication.form === 'tablet' ? tr.doseTabletUnit : medication.form === 'capsule' ? tr.doseCapsuleUnit : tr.doseOtherUnit}{/if}
          </td>
          <td class="whitespace-nowrap px-4 py-3 tabular-nums">{doseRange(medication)}</td>
          <td class="px-4 py-3"><div class="flex flex-wrap gap-2">{#if medication.prescriptionRequired}<Badge>{tr.medicationRx}</Badge>{/if}{#if medication.deletedAt}<Badge>{tr.billingInactive}</Badge>{/if}</div></td>
        </tr>{/each}
      </tbody></table></div>{/if}
  </Card>
  <DoseCalculator medications={rows} />
</div>
<Dialog bind:open title={editing ? tr.medicationEdit : tr.medicationsDialogTitle} description={tr.medicationFormDescription}>
  {#if open}
    {#if editing?.deletedAt}<div class="grid gap-4"><p>{editing.name}</p><p class="text-fg-muted">{tr.medicationInactiveHint}</p><Button disabled={busy.includes(editing.id)} onclick={() => editing && undo(editing.id)}>{tr.billingActive}</Button></div>
    {:else}<MedicationForm medication={editing} onsaved={saved} oncancel={() => open = false} />
      {#if editing}<div class="mt-4 border-t border-border pt-4"><Button variant="destructive" disabled={busy.includes(editing.id)} onclick={async () => { if (editing) { await remove(editing); open = false; } }}>{tr.medicationDelete}</Button></div>{/if}
    {/if}
  {/if}
  {#snippet footer()}{/snippet}
</Dialog>
<div class="fixed right-6 bottom-6 z-30 grid max-h-[50dvh] max-w-[calc(100vw-3rem)] gap-2 overflow-y-auto p-1">
  {#each deleted as item (item.id)}<Toast bind:open={item.open} inline message={item.name + ': ' + tr.medicationDeleted}>
    {#snippet action()}<Button size="sm" variant="secondary" disabled={busy.includes(item.id)} onclick={() => undo(item.id)}>{tr.undo}</Button>{/snippet}
  </Toast>{/each}
</div>
