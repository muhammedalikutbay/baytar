<script lang="ts">
  import { Pill } from '@lucide/svelte';
  import { PageHeader, Button, Card, Dialog, EmptyState, Input, Toast } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatNumber } from '$lib/format';
  import * as medications from '$lib/api/medications';
  import type { Medication } from '$lib/api/types';
  import MedicationForm from '$lib/components/medications/MedicationForm.svelte';
  import DoseCalculator from '$lib/components/medications/DoseCalculator.svelte';
  import { formLabels } from '$lib/components/medications/options';
  let rows = $state<Medication[]>([]);
  let visible = $state<Medication[]>([]);
  let query = $state('');
  let loading = $state(true);
  let loadError = $state(false);
  let actionError = $state(false);
  let refresh = $state(0);
  let open = $state(false);
  let editing = $state<Medication | undefined>();
  let busy = $state<number[]>([]);
  let deleted = $state<{ id: number; name: string; open: boolean }[]>([]);
  $effect(() => {
    refresh; let cancelled = false;
    medications.list().then(value => { if (!cancelled) rows = value; })
      .catch(() => { if (!cancelled) actionError = true; });
    return () => { cancelled = true; };
  });
  $effect(() => {
    const text = query.trim(); refresh;
    let cancelled = false; loading = true; loadError = false;
    const timer = setTimeout(async () => {
      try { const value = text ? await medications.search(text) : await medications.list(); if (!cancelled) visible = value; }
      catch { if (!cancelled) loadError = true; }
      finally { if (!cancelled) loading = false; }
    }, text ? 150 : 0);
    return () => { cancelled = true; clearTimeout(timer); };
  });
  function edit(value?: Medication) { editing = value; open = true; }
  function saved() { open = false; refresh++; }
  async function remove(value: Medication) {
    if (busy.includes(value.id)) return;
    busy = [...busy, value.id]; actionError = false;
    try {
      await medications.softDelete(value.id);
      rows = rows.filter(m => m.id !== value.id); visible = visible.filter(m => m.id !== value.id);
      deleted = [...deleted, { id: value.id, name: value.name, open: true }]; refresh++;
    } catch { actionError = true; }
    finally { busy = busy.filter(id => id !== value.id); }
  }
  async function undo(id: number) {
    if (busy.includes(id)) return;
    busy = [...busy, id]; actionError = false;
    try { await medications.restore(id); deleted = deleted.filter(m => m.id !== id); refresh++; }
    catch { actionError = true; }
    finally { busy = busy.filter(value => value !== id); }
  }
  function doseRange(m: Medication): string {
    if (m.doseMgPerKgMin === null && m.doseMgPerKgMax === null) return tr.medicationUnspecified;
    return `${m.doseMgPerKgMin === null ? '…' : formatNumber(m.doseMgPerKgMin)}${m.doseMgPerKgMax === null ? '' : '–' + formatNumber(m.doseMgPerKgMax)} mg/kg`;
  }
</script>

<div class="grid gap-8">
  <PageHeader title={tr.medications} subtitle={tr.medicationsDescription} icon={Pill}>
    {#snippet actions()}<Button onclick={() => edit()}>{tr.medicationsAction}</Button>{/snippet}
  </PageHeader>
  <p class="text-fg-muted">{tr.medicationNoDoseData}</p>
  {#if actionError}<p role="alert" class="text-danger">{tr.actionError}</p>{/if}
  <Input label={tr.medicationSearch} type="search" bind:value={query} />
  <Card title={tr.medications} icon={Pill} count={visible.length} class="overflow-hidden p-0 sm:p-0 [&>div:first-child]:m-4">
    {#if loading}<p role="status" class="p-6 text-fg-muted">{tr.loading}</p>
    {:else if loadError}<div class="p-6"><p role="alert" class="mb-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></div>
    {:else if visible.length === 0}
      <EmptyState icon={Pill} title={query.trim() ? tr.noResults : tr.medicationsEmptyTitle} description={query.trim() ? tr.medicationNoResults : tr.medicationsEmptyDescription}>
        {#snippet action()}{#if !query.trim()}<Button variant="secondary" onclick={() => edit()}>{tr.medicationsAction}</Button>{/if}{/snippet}
      </EmptyState>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-left">
          <thead class="border-b border-border bg-surface-muted type-caption text-fg-muted"><tr>
            <th scope="col" class="px-5 py-3">{tr.medicationName} / {tr.activeIngredient}</th>
            <th scope="col" class="px-5 py-3">{tr.medicationForm}</th>
            <th scope="col" class="px-5 py-3">{tr.medicationConcentration} / {tr.medicationStrength}</th>
            <th scope="col" class="px-5 py-3">{tr.medicationDoseRange}</th>
            <th scope="col" class="px-5 py-3"><span class="sr-only">{tr.medicationEdit} / {tr.medicationDelete}</span></th>
          </tr></thead>
          <tbody class="divide-y divide-border">
            {#each visible as medication (medication.id)}<tr class="hover:bg-surface-muted">
              <th scope="row" class="px-5 py-3 font-medium"><span class="block">{medication.name}</span><span class="block font-normal text-fg-muted">{medication.activeIngredient ?? tr.medicationUnspecified}</span></th>
              <td class="px-5 py-3 text-fg-muted">{medication.form ? formLabels[medication.form] : tr.medicationUnspecified}</td>
              <td class="px-5 py-3 tabular-nums">
                {#if medication.concentrationMgPerMl !== null}<span class="block whitespace-nowrap">{formatNumber(medication.concentrationMgPerMl)} mg/ml</span>{/if}
                {#if medication.strengthMgPerUnit !== null}<span class="block whitespace-nowrap">{formatNumber(medication.strengthMgPerUnit)} mg/{medication.form === 'tablet' ? tr.doseTabletUnit : medication.form === 'capsule' ? tr.doseCapsuleUnit : tr.doseOtherUnit}</span>{/if}
                {#if medication.concentrationMgPerMl === null && medication.strengthMgPerUnit === null}<span class="text-fg-muted">{tr.medicationUnspecified}</span>{/if}
              </td>
              <td class="whitespace-nowrap px-5 py-3 tabular-nums">{doseRange(medication)}</td>
              <td class="px-5 py-3"><div class="flex gap-2">
                <Button size="sm" variant="secondary" disabled={busy.includes(medication.id)} aria-label={`${medication.name}: ${tr.medicationEdit}`} onclick={() => edit(medication)}>{tr.edit}</Button>
                <Button size="sm" variant="danger" disabled={busy.includes(medication.id)} aria-label={`${medication.name}: ${tr.medicationDelete}`} onclick={() => remove(medication)}>{tr.delete}</Button>
              </div></td>
            </tr>{/each}
          </tbody>
        </table>
      </div>
    {/if}
  </Card>
  <DoseCalculator medications={rows} />
</div>

<Dialog bind:open title={editing ? tr.medicationEdit : tr.medicationsDialogTitle} description={tr.medicationFormDescription}>
  {#if open}<MedicationForm medication={editing} onsaved={saved} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<div class="fixed right-6 bottom-6 z-30 grid max-h-[50dvh] max-w-[calc(100vw-3rem)] gap-2 overflow-y-auto p-1">
  {#each deleted as item (item.id)}
    <Toast bind:open={item.open} inline message={`${item.name}: ${tr.medicationDeleted}`}>
      {#snippet action()}<Button size="sm" variant="secondary" disabled={busy.includes(item.id)} onclick={() => undo(item.id)}>{tr.undo}</Button>{/snippet}
    </Toast>
  {/each}
</div>
