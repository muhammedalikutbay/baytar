<script lang="ts">
  import { Syringe, TriangleAlert, CalendarDays } from '@lucide/svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { openWhatsapp } from '$lib/api/settings';
  import { PageHeader, Card, Button, Badge, Dialog, Select, StatTile } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, dateAfterDays, dateAfterMonths, daysUntil, formatDate, whatsappPhone, parseDate } from '$lib/format';
  import { mergedDue } from '$lib/api/vaccinations';
  import * as vaccineTypes from '$lib/api/vaccine_types';
  import type { VaccineDue, VaccineType, Species } from '$lib/api/types';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  import VaccineTypes from '$lib/components/vaccines/VaccineTypes.svelte';
  import PostponeForm from '$lib/components/vaccines/PostponeForm.svelte';
  import VaccineCalendar from '$lib/components/vaccines/VaccineCalendar.svelte';
  import SpeciesIcon from '$lib/components/patients/SpeciesIcon.svelte';
  let rows = $state<VaccineDue[]>([]);
  let types = $state<VaccineType[]>([]);
  let species = $state('');
  let typeId = $state('');
  let period = $state('month');
  let loading = $state(true);
  let failed = $state(false);
  let reminderError = $state(false);
  let refresh = $state(0);
  let dialog = $state<{ mode: 'give' | 'plan'; date: string; row?: VaccineDue } | null>(null);
  let postponing = $state<VaccineDue | null>(null);
  const today = todayDate();
  const weekEnd = dateAfterDays(today, 6 - ((parseDate(today)!.getUTCDay() + 6) % 7));
  const monthEnd = dateAfterDays(dateAfterMonths(today.slice(0, 7) + '-01', 1), -1);
  const filtered = $derived(rows.filter(row => (!species || row.species === species) && (!typeId || row.vaccineTypeId === Number(typeId)) && (period === 'late' ? row.dueOn < today : row.dueOn <= (period === 'week' ? weekEnd : monthEnd))));
  const overdue = $derived(filtered.filter(row => row.dueOn < today));
  const upcoming = $derived(filtered.filter(row => row.dueOn >= today));
  $effect(() => {
    refresh; let cancelled = false;
    loading = true; failed = false;
    Promise.all([mergedDue(), vaccineTypes.list()]).then(([due, catalog]) => { if (!cancelled) { rows = due; types = catalog; } })
      .catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
  async function reminder(row: VaccineDue) {
    const phone = whatsappPhone(row.phone); if (!phone) return;
    reminderError = false;
    const message = tr.vaccineReminder.replace('{owner}', row.ownerName).replace('{patient}', row.patientName).replace('{vaccine}', row.vaccineName).replace('{date}', formatDate(row.dueOn));
    const url = `https://wa.me/${phone}?text=${encodeURIComponent(message)}`;
    try { if (isTauri()) await openWhatsapp(url); else window.open(url, '_blank', 'noopener,noreferrer'); }
    catch { reminderError = true; }
  }
  function saved() { dialog = null; postponing = null; refresh++; }
</script>
<div class="grid gap-6">
  <PageHeader title={tr.vaccines} subtitle={tr.vaccinesDescription} icon={Syringe}>
    {#snippet actions()}<Button variant="secondary" onclick={() => dialog = { mode: 'plan', date: today }}>{tr.vaccinePlan}</Button><Button onclick={() => dialog = { mode: 'give', date: today }}>{tr.vaccineGive}</Button>{/snippet}
  </PageHeader>
  <div class="grid gap-4 sm:grid-cols-2">
    <StatTile icon={TriangleAlert} value={rows.filter(r => r.dueOn < today).length} label={tr.overdue} tone="highlight" />
    <StatTile icon={CalendarDays} value={rows.filter(r => r.dueOn >= today && r.dueOn <= dateAfterDays(today, 7)).length} label={tr.vaccineUpcoming} hint={tr.vaccineThisWeek} />
  </div>
  <VaccineCalendar {refresh} onplan={date => dialog = { mode: 'plan', date }} />
  <div class="grid gap-3 sm:grid-cols-3">
    <Select label={tr.species} bind:value={species} options={[{ value: '', label: tr.anySpecies }, ...(['cat','dog','bird','rabbit','rodent','ferret','reptile','other'] as Species[]).map(s => ({ value: s, label: tr[s] }))]} />
    <Select label={tr.vaccineName} bind:value={typeId} options={[{ value: '', label: tr.vaccineAll }, ...types.map(t => ({ value: String(t.id), label: t.name }))]} />
    <Select label={tr.duePeriod} bind:value={period} options={[{ value: 'week', label: tr.vaccineThisWeek }, { value: 'month', label: tr.vaccineThisMonth }, { value: 'late', label: tr.overdue }]} />
  </div>
  {#if reminderError}<p role="alert" class="text-danger">{tr.vaccineReminderError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else}
    {#each [{ title: tr.overdue, rows: overdue, late: true }, { title: tr.upcoming, rows: upcoming, late: false }] as section}
      {#if period !== 'late' || section.late}
      <Card title={section.title} icon={section.late ? TriangleAlert : Syringe} tone={section.late ? 'highlight' : 'accent'} count={section.rows.length}>
        {#if !section.rows.length}<div class="grid justify-items-start gap-3"><p class="text-fg-muted">{tr.vaccineNoDue}</p><Button size="sm" variant="secondary" onclick={() => dialog = { mode: 'plan', date: today }}>{tr.vaccinePlan}</Button></div>
        {:else}<ul class="divide-y divide-border">{#each section.rows as row (`${row.patientId}-${row.vaccineTypeId ?? row.vaccineName}`)}
          {@const days = daysUntil(row.dueOn)!}
          <li class="flex flex-wrap items-center gap-3 py-4">
            <a href={`/patients/pet?id=${row.patientId}`} class="flex min-w-40 flex-1 items-center gap-3 hover:underline"><SpeciesIcon species={row.species} /><span><span class="block font-semibold">{row.patientName}</span><span class="type-caption text-fg-muted">{row.ownerName}</span></span></a>
            <div class="min-w-36 flex-1"><p>{row.vaccineName}</p><p class="type-caption text-fg-muted">{formatDate(row.dueOn)}</p></div>
            <Badge variant={days <= 14 ? 'highlight' : 'neutral'}>{days < 0 ? tr.vaccineDaysLate.replace('{days}', String(-days)) : days === 0 ? tr.dueToday : tr.vaccineDaysAfter.replace('{days}', String(days))}</Badge>
            {#if row.appointmentId}<Badge variant="accent">{tr.vaccinePlanned}</Badge>{/if}
            <div class="flex flex-wrap gap-2">
              <Button size="sm" variant="secondary" onclick={() => dialog = { mode: 'give', date: today, row }}>{tr.vaccinationDone}</Button>
              <Button size="sm" variant="ghost" onclick={() => row.vaccineTypeId ? postponing = row : dialog = { mode: 'plan', date: row.dueOn, row }}>{tr.vaccinePostpone}</Button>
              <Button size="sm" variant="ghost" disabled={!whatsappPhone(row.phone)} onclick={() => reminder(row)}>{tr.vaccineWhatsApp}</Button>
            </div>
          </li>
        {/each}</ul>{/if}
      </Card>
      {/if}
    {/each}
  {/if}
  <VaccineTypes />
</div>
<Dialog bind:open={() => dialog !== null, value => { if (!value) dialog = null; }} title={dialog?.mode === 'plan' ? tr.vaccinePlan : tr.vaccineGive} description={tr.vaccineWorkflowDescription}>
  {#if dialog}<VaccinationForm mode={dialog.mode} initialDate={dialog.date} patientId={dialog.row?.patientId} vaccineTypeId={dialog.row?.vaccineTypeId} vaccineName={dialog.row?.vaccineName} appointmentId={dialog.mode === 'give' ? dialog.row?.appointmentId : null} onsaved={saved} oncancel={() => dialog = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={() => postponing !== null, value => { if (!value) postponing = null; }} title={tr.vaccinePostpone} description={tr.vaccinePostponeDescription}>
  {#if postponing}<PostponeForm row={postponing} onsaved={saved} oncancel={() => postponing = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
