<script lang="ts">
  import { localToday } from '$lib/format';
  import { Card, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatNumber, todayDate } from '$lib/format';
  import * as visits from '$lib/api/visits';
  import * as appointments from '$lib/api/appointments';
  import { mergedDue } from '$lib/api/vaccinations';
  import type { Appointment, VaccineDue, Visit } from '$lib/api/types';
  import { weightSeries, isPastClinicalRecord } from './clinical';
  let { patientId, revision = 0, onappointments, onvaccines }: { patientId: number; revision?: number; onappointments: () => void; onvaccines: () => void } = $props();
  let next = $state<Appointment | null>(null), vaccine = $state<VaccineDue | null>(null), history = $state<Visit[]>([]);
  let loading = $state(true), failed = $state(false), refresh = $state(0);
  const weights = $derived(weightSeries(history.map(v => ({ date: v.visitOn, time: v.visitTime, id: v.id, kg: v.weightKg ?? NaN }))));
  $effect(() => {
    const id = patientId; revision; refresh; let cancelled = false; loading = true; failed = false;
    Promise.all([visits.listByPatient(id), appointments.between(todayDate(), '9999-12-31'), mergedDue({ patientId: id })]).then(([encounters, calendar, due]) => {
      if (cancelled) return;
      history = encounters.filter(v => isPastClinicalRecord(v.visitOn, v.visitTime));
      next = calendar.map(c => c.appointment).filter(a => a.patientId === id && a.kind !== 'vaccine' && a.status === 'planned' && !a.deletedAt && (a.startsOn > todayDate() || !a.startsTime || !isPastClinicalRecord(a.startsOn, a.startsTime)))
        .sort((a,b) => a.startsOn.localeCompare(b.startsOn) || (a.startsTime ?? '').localeCompare(b.startsTime ?? '') || a.id - b.id)[0] ?? null;
      vaccine = due.slice().sort((a,b) => a.dueOn.localeCompare(b.dueOn))[0] ?? null;
    }).catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
</script>
<div class="grid gap-4 lg:grid-cols-2">
  <Card title={tr.flowUpcoming}>
    {#if loading}<p role="status">{tr.loading}</p>{:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
    {:else}<div class="grid gap-3 text-sm">
      {#if next}<button type="button" class="min-h-10 text-left text-accent underline" onclick={onappointments}>{tr.flowNextAppointment}: {formatDate(next.startsOn)} {next.startsTime ?? ''} · {next.title}</button>{/if}
      {#if vaccine}<button type="button" class={`min-h-10 text-left underline ${vaccine.dueOn < localToday() ? "text-highlight" : "text-accent"}`} onclick={onvaccines}>{vaccine.dueOn < localToday() ? tr.flowOverdueVaccine : tr.flowNextVaccine}: {vaccine.vaccineName} · {formatDate(vaccine.dueOn)}</button>{/if}
      {#if !next && !vaccine}<p class="text-fg-muted">{tr.flowNoUpcoming}</p>{/if}
    </div>{/if}
  </Card>
  <Card title={tr.weightTrend}>
    {#if weights && !loading && !failed}
      <p class="font-semibold tabular-nums">{formatNumber(weights.last.kg)} kg <span class="type-caption text-fg-muted">· {formatDate(weights.last.date)}</span></p>
      {#if weights.change !== null}<p class="type-caption text-fg-muted">{tr.flowWeightChange}: {weights.change > 0 ? '+' : ''}{formatNumber(weights.change)} kg</p>{/if}
      <svg viewBox="0 0 330 150" class="mt-2 h-36 w-full text-fg-muted" role="img" aria-label={tr.weightTrend}>
        <title>{weights.points.map(p => formatDate(p.date) + ': ' + formatNumber(p.kg) + ' kg').join('; ')}</title>
        <path d="M48 16 V115 H300" fill="none" stroke="currentColor" />
        <text x="2" y="12" font-size="10">kg</text>
        <text x="2" y="26" font-size="10">{formatNumber(weights.max)}</text>
        {#if weights.max !== weights.min}<text x="2" y="112" font-size="10">{formatNumber(weights.min)}</text>{/if}
        <polyline points={weights.points.map(p => p.x + ',' + p.y).join(' ')} fill="none" stroke="var(--color-accent)" stroke-width="2" />
        {#each weights.points as point, i}<circle cx={point.x} cy={point.y} r="3" fill="var(--color-accent)"><title>{formatDate(point.date)}: {formatNumber(point.kg)} kg</title></circle>
          {#if i === 0 || i === weights.points.length - 1}<text x={point.x} y={134} text-anchor={weights.points.length === 1 ? 'middle' : i === 0 ? 'start' : 'end'} font-size="10">{formatDate(point.date)}</text>{/if}
        {/each}
      </svg>
    {:else if !loading && !failed}<p class="type-caption text-fg-muted">{tr.noWeight}</p>{/if}
  </Card>
</div>
