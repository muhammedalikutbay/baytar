<script lang="ts">
  import { page } from '$app/state';
  import { Syringe } from '@lucide/svelte';
  import { Card, EmptyState, Button, Badge, Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, daysUntil } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import type { Vaccination } from '$lib/api/types';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  let records = $state<Vaccination[]>([]);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let add = $state(false);
  const patientId = $derived(Number(page.url.searchParams.get('id')));
  const latest = $derived.by(() => {
    const names = new Set<string>();
    return records.filter(record => {
      const key = record.vaccineName.toLocaleLowerCase('tr-TR').trim();
      if (names.has(key)) return false;
      names.add(key); return true;
    });
  });
  $effect(() => {
    const id = patientId; refresh;
    let cancelled = false;
    loading = true; failed = false; records = []; add = false;
    vaccinations.listByPatient(id).then(rows => {
      if (!cancelled) records = rows.filter(row => !row.deletedAt).sort((a, b) => b.givenOn.localeCompare(a.givenOn) || b.id - a.id);
    }).catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
</script>
<section aria-label={tr.vaccines}><Card>
  <div class="mb-4 flex items-center justify-between gap-4"><h2 class="type-section">{tr.vaccines}</h2><Button size="sm" onclick={() => add = true}>{tr.vaccinationAdd}</Button></div>
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
  {:else if !records.length}<EmptyState icon={Syringe} title={tr.patientVaccinesEmpty} description={tr.patientVaccinesDescription} />
  {:else}
    <ul class="divide-y divide-border">{#each latest as record (record.id)}
      {@const days = record.nextDueOn ? daysUntil(record.nextDueOn) : null}
      <li class="flex flex-wrap items-center justify-between gap-3 py-3">
        <div><p class="font-medium">{record.vaccineName}</p><p class="type-caption text-fg-muted">{tr.givenOn}: {formatDate(record.givenOn)}</p>
          <p class="mt-1 tabular-nums">{record.nextDueOn ? `${tr.nextDueOn}: ${formatDate(record.nextDueOn)}` : tr.dueUnknown}</p></div>
        {#if days !== null && days < 0}<Badge variant="danger">{tr.overdue} · {-days} {tr.daysLate}</Badge>
        {:else if days !== null && days <= 14}<Badge variant="highlight">{days === 0 ? tr.dueToday : tr.dueSoon}</Badge>{/if}
      </li>
    {/each}</ul>
    <details class="mt-4 border-t border-border pt-4"><summary class="cursor-pointer text-accent underline">{tr.vaccineHistory} ({records.length})</summary>
      <ul class="mt-3 divide-y divide-border">{#each records as record (record.id)}<li class="py-3">
        <p class="font-medium">{record.vaccineName} · {formatDate(record.givenOn)}</p>
        <p class="text-fg-muted">{record.nextDueOn ? `${tr.nextDueOn}: ${formatDate(record.nextDueOn)}` : tr.dueUnknown}</p>
        {#if record.batchNo}<p>{tr.batchNo}: {record.batchNo}</p>{/if}
        {#if record.notes}<p class="whitespace-pre-wrap break-words">{record.notes}</p>{/if}
      </li>{/each}</ul>
    </details>
  {/if}
</Card></section>
<Dialog bind:open={add} title={tr.vaccinationAdd} description={tr.vaccinationDescription}>
  {#if add}<VaccinationForm {patientId} onsaved={() => { add = false; refresh++; }} oncancel={() => add = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
