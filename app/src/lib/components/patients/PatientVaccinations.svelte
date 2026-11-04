<script lang="ts">
  import PrintButton from '$lib/components/print/PrintButton.svelte';
  import { page } from '$app/state';
  import { Syringe, CalendarDays } from '@lucide/svelte';
  import { Card, Button, Badge, Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, daysUntil, todayDate } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import { list as listTypes } from '$lib/api/vaccine_types';
  import { get as getPatient } from '$lib/api/patients';
  import type { Vaccination, VaccineDue, VaccineType, Patient } from '$lib/api/types';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  let { revision = 0, onchange = () => {} }: { revision?: number; onchange?: () => void } = $props();
  let records = $state<Vaccination[]>([]);
  let due = $state<VaccineDue[]>([]);
  let types = $state<VaccineType[]>([]);
  let patient = $state<Patient | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let dialog = $state<{ mode: 'give' | 'plan'; typeId?: number | null; name?: string; appointmentId?: number | null } | null>(null);
  const patientId = $derived(Number(page.url.searchParams.get('id')));
  const schedules = $derived.by(() => {
    const entries = types.filter(t => t.isActive && !t.deletedAt && (!t.species || t.species === patient?.species)).map(t => ({ typeId: t.id as number | null, name: t.name }));
    for (const r of [...records, ...due]) {
      if (!entries.some(e => r.vaccineTypeId ? e.typeId === r.vaccineTypeId : e.name.toLocaleLowerCase('tr-TR') === r.vaccineName.toLocaleLowerCase('tr-TR'))) entries.push({ typeId: r.vaccineTypeId, name: r.vaccineName });
    }
    return entries.map(entry => {
      const doses = records.filter(r => entry.typeId ? r.vaccineTypeId === entry.typeId : !r.vaccineTypeId && r.vaccineName === entry.name);
      const next = due.find(r => entry.typeId ? r.vaccineTypeId === entry.typeId : !r.vaccineTypeId && r.vaccineName === entry.name);
      const days = next ? daysUntil(next.dueOn) : null;
      return { ...entry, doses, next, days };
    }).sort((a,b) => (a.next?.dueOn ?? '9999').localeCompare(b.next?.dueOn ?? '9999') || a.name.localeCompare(b.name, 'tr'));
  });
  $effect(() => {
    const id = patientId; refresh; revision;
    let cancelled = false;
    loading = true; failed = false; dialog = null;
    Promise.all([vaccinations.listByPatient(id), vaccinations.mergedDue({ patientId: id }), listTypes(), getPatient(id)]).then(([history, reminders, catalog, pet]) => {
      if (!cancelled) { records = history; due = reminders; types = catalog; patient = pet; }
    }).catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
</script>
<section aria-label={tr.vaccines} class="grid gap-4">
  <div class="flex justify-end"><PrintButton kind="vaccination" id={patientId} /></div>
  <Card title={tr.vaccineSchedule} icon={CalendarDays}>
    <div class="mb-4 flex flex-wrap justify-end gap-2"><Button size="sm" variant="secondary" disabled={!!patient?.deceasedOn || loading} onclick={() => dialog = { mode: 'plan' }}>{tr.vaccinePlan}</Button><Button size="sm" disabled={!!patient?.deceasedOn || loading} onclick={() => dialog = { mode: 'give' }}>{tr.vaccineGive}</Button></div>
    {#if loading}<p role="status">{tr.loading}</p>
    {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
    {:else if !schedules.length}<p class="text-fg-muted">{tr.vaccineScheduleEmpty}</p>
    {:else}
      <ul class="divide-y divide-border">{#each schedules as schedule (`${schedule.typeId ?? schedule.name}`)}
        <li class="grid gap-3 py-4">
          <div class="flex flex-wrap items-center justify-between gap-3"><h3 class="font-semibold">{schedule.name}</h3>
            <Badge variant={schedule.days === null ? 'neutral' : schedule.days <= 14 ? 'highlight' : 'success'}>{schedule.days === null ? tr.vaccineNoPlan : schedule.days < 0 ? tr.vaccineLate : schedule.days <= 14 ? tr.vaccineApproaching : tr.vaccineCurrent}</Badge>
          </div>
          <p class="type-caption text-fg-muted">{schedule.doses.length} {tr.vaccineDoses}{schedule.doses.length ? ` · ${schedule.doses.map(d => formatDate(d.givenOn)).join(' · ')}` : ''}</p>
          <div class="flex flex-wrap items-center justify-between gap-3">
            <p>{schedule.next ? `${schedule.next.appointmentId ? tr.vaccinePlanned : tr.nextDueOn}: ${formatDate(schedule.next.dueOn)}` : tr.vaccineNoPlan}</p>
            <div class="flex gap-2"><Button size="sm" variant="ghost" disabled={!!patient?.deceasedOn} onclick={() => dialog = { mode: 'plan', typeId: schedule.typeId, name: schedule.name }}>{tr.vaccinePlan}</Button>
              <Button size="sm" variant="secondary" disabled={!!patient?.deceasedOn} onclick={() => dialog = { mode: 'give', typeId: schedule.typeId, name: schedule.name, appointmentId: schedule.next?.appointmentId }}>{tr.vaccineGive}</Button></div>
          </div>
        </li>
      {/each}</ul>
    {/if}
  </Card>
  <Card title={tr.vaccineHistory} icon={Syringe} count={records.length}>
    {#if !records.length}<p class="text-fg-muted">{tr.vaccineHistoryEmpty}</p>
    {:else}<ul class="divide-y divide-border">{#each records as record (record.id)}<li class="py-3">
      <p class="font-medium">{record.vaccineName} · {formatDate(record.givenOn)}</p>
      {#if record.batchNo}<p class="type-caption text-fg-muted">{tr.batchNo}: {record.batchNo}</p>{/if}
      {#if record.notes}<p class="whitespace-pre-wrap break-words">{record.notes}</p>{/if}
    </li>{/each}</ul>{/if}
  </Card>
</section>
<Dialog bind:open={() => dialog !== null, value => { if (!value) dialog = null; }} title={dialog?.mode === 'plan' ? tr.vaccinePlan : tr.vaccineGive} description={tr.vaccineWorkflowDescription}>
  {#if dialog}<VaccinationForm {patientId} mode={dialog.mode} vaccineTypeId={dialog.typeId} vaccineName={dialog.name} appointmentId={dialog.appointmentId} initialDate={todayDate()} onsaved={() => { dialog = null; refresh++; onchange(); }} oncancel={() => dialog = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
