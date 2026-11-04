<script lang="ts">
  import { CalendarDays, ChevronLeft, ChevronRight } from '@lucide/svelte';
  import { Card, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, dateAfterDays, dateAfterMonths, formatMonth, formatDate, parseDate } from '$lib/format';
  import { calendar } from '$lib/api/vaccinations';
  import type { VaccineCalendarEvent } from '$lib/api/types';
  let { refresh, onplan }: { refresh: number; onplan: (date: string) => void } = $props();
  let month = $state(todayDate().slice(0, 7) + '-01');
  let events = $state<VaccineCalendarEvent[]>([]);
  let loading = $state(false);
  let failed = $state(false);
  let retry = $state(0);
  const end = $derived(dateAfterDays(dateAfterMonths(month, 1), -1));
  const days = $derived.by(() => {
    const first = parseDate(month)!;
    const offset = (first.getUTCDay() + 6) % 7;
    return Array.from({ length: Math.ceil((offset + Number(end.slice(-2))) / 7) * 7 }, (_, i) => dateAfterDays(month, i - offset));
  });
  const weekdays = Array.from({ length: 7 }, (_, i) => new Intl.DateTimeFormat('tr-TR', { weekday: 'short', timeZone: 'UTC' }).format(parseDate(dateAfterDays('2026-01-05', i))!));
  $effect(() => {
    const from = month, to = end; refresh; retry;
    let cancelled = false;
    loading = true; failed = false; events = [];
    calendar(from, to).then(rows => { if (!cancelled) events = rows; }).catch(() => { if (!cancelled) failed = true; }).finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
</script>
<Card title={tr.vaccineCalendar} icon={CalendarDays}>
  <div class="mb-4 flex items-center justify-between gap-3">
    <Button size="sm" variant="ghost" aria-label={tr.vaccinePreviousMonth} onclick={() => month = dateAfterMonths(month, -1)}><ChevronLeft size={18} /></Button>
    <h3 class="font-semibold" aria-live="polite">{formatMonth(month)}</h3>
    <Button size="sm" variant="ghost" aria-label={tr.vaccineNextMonth} onclick={() => month = dateAfterMonths(month, 1)}><ChevronRight size={18} /></Button>
  </div>
  <div class="mb-3 flex gap-4 type-caption text-fg-muted"><span>● {tr.vaccineGiven}</span><span class="text-accent">◇ {tr.vaccinePlanned}</span></div>
  {#if loading}<p role="status">{tr.loading}</p>{/if}
  {#if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => retry++}>{tr.retry}</Button>{/if}
  <div class="grid grid-cols-7 gap-px rounded-md border border-border bg-border">
    {#each weekdays as day}<div class="bg-surface-muted p-2 text-center type-caption">{day}</div>{/each}
    {#each days as day}
      {@const entries = events.filter(e => e.on === day)}
      <button type="button" class="min-h-20 min-w-0 bg-surface p-2 text-left hover:bg-surface-muted" class:opacity-40={day.slice(0, 7) !== month.slice(0, 7)}
        aria-label={tr.vaccineDayPlan.replace('{date}', formatDate(day)) + (entries.length ? ` · ${entries.map(e => `${e.planned ? tr.vaccinePlanned : tr.vaccineGiven}: ${e.patientName}, ${e.vaccineName}`).join('; ')}` : '')}
        aria-current={day === todayDate() ? 'date' : undefined} onclick={() => onplan(day)}>
        <span class="block tabular-nums" class:font-bold={day === todayDate()} class:text-accent={day === todayDate()}>{Number(day.slice(-2))}</span>
        {#each entries as event}<span class="mt-1 block truncate type-caption" class:text-accent={event.planned} title={`${event.patientName} · ${event.vaccineName}`}>{event.planned ? '◇' : '●'} {event.patientName} · {event.vaccineName}</span>{/each}
      </button>
    {/each}
  </div>
</Card>
