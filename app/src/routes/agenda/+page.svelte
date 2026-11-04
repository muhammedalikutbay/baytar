<script lang="ts">
  import { browser } from '$app/environment';
  import { addMonths } from '$lib/components/agenda/calendar';
  import { onMount } from 'svelte';
  import { CalendarDays, Plus } from '@lucide/svelte';
  import { PageHeader, Button, Card, Dialog, EmptyState, Badge } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { addDateDays, formatAgendaDate, formatDate, localDate, parseDate } from '$lib/format';
  import * as appointments from '$lib/api/appointments';
  import type { Appointment, AppointmentStatus } from '$lib/api/types';
  import AppointmentForm from '$lib/components/agenda/AppointmentForm.svelte';
  import AppointmentItem from '$lib/components/agenda/AppointmentItem.svelte';
  import { compareAppointments, withPhones, isOverdue, statusLabels, type AgendaItem } from '$lib/components/agenda/appointments';
  let date = $state(localDate());
  type View = 'day' | 'week' | 'month' | 'list';
  let view = $state<View>('day');
  let viewReady = $state(false);
  const views = [{value:'day', label:tr.agendaDay}, {value:'week', label:tr.agendaWeek}, {value:'month', label:tr.agendaMonth}, {value:'list', label:tr.agendaList}] as const;
  let now = $state(new Date());
  let items = $state<AgendaItem[]>([]);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let open = $state(false);
  let editing = $state<Appointment>();
  let busy = $state(false);
  let actionError = $state('');
  const monday = $derived(addDateDays(date, -((parseDate(date)!.getUTCDay() + 6) % 7)));
  const monthStart = $derived(date.slice(0, 8) + '01');
  const gridStart = $derived(addDateDays(monthStart, -((parseDate(monthStart)!.getUTCDay() + 6) % 7)));
  const from = $derived(view === 'month' ? gridStart : view === 'week' ? monday : date);
  const to = $derived(addDateDays(from, view === 'month' ? 41 : view === 'list' ? 29 : view === 'week' ? 6 : 0));
  const days = $derived(Array.from({ length: view === 'month' ? 42 : view === 'list' ? 30 : view === 'day' ? 1 : 7 }, (_, index) => addDateDays(from, index)));
  const weekdays = Array.from({length:7}, (_, i) => new Intl.DateTimeFormat('tr-TR', {weekday:'short', timeZone:'UTC'}).format(parseDate(addDateDays('2026-11-02', i))!));
  onMount(() => {
    try { const stored = localStorage.getItem('baytar.agenda.view'); if (views.some(v => v.value === stored)) view = stored as View; } catch { /* Storage may be unavailable. */ }
    viewReady = true;
    const timer = setInterval(() => now = new Date(), 30000); return () => clearInterval(timer); });
  $effect(() => { if (browser && viewReady) { try { localStorage.setItem('baytar.agenda.view', view); } catch { /* Storage may be unavailable. */ } } });
  $effect(() => {
    const start = from, end = to, mode = view; refresh;
    let cancelled = false;
    loading = true; failed = false; items = [];
    (async () => {
      try {
        const rows = await withPhones(mode === 'day' && start === localDate() ? await appointments.today(start) : await appointments.between(start, end));
        if (!cancelled) items = rows.sort(compareAppointments);
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  function move(direction: number) { const next = view === 'month' ? addMonths(date, direction) : addDateDays(date, direction * (view === 'day' ? 1 : view === 'list' ? 30 : 7)); if (next && next >= '0001-01-08' && next <= '9999-12-24') date = next; }
  function edit(appointment: Appointment) { editing = appointment; open = true; }
  async function status(id: number, value: AppointmentStatus) {
    if (busy) return;
    busy = true; actionError = '';
    try { await appointments.setStatus(id, value); refresh++; }
    catch { actionError = tr.actionError; }
    finally { busy = false; }
  }
</script>
<div class="grid gap-8">
  <PageHeader title={tr.agenda} subtitle={tr.agendaDescription} icon={CalendarDays}>
    {#snippet actions()}<Button onclick={() => { editing = undefined; open = true; }}><Plus size={18} aria-hidden="true" />{tr.agendaAction}</Button>{/snippet}
  </PageHeader>
  <div class="flex flex-wrap items-center justify-between gap-4">
    <div class="flex flex-wrap items-center gap-2"><Button variant="secondary" onclick={() => move(-1)}>{tr.previous}</Button><Button variant="secondary" onclick={() => date = localDate()}>{tr.agendaToday}</Button><Button variant="secondary" onclick={() => move(1)}>{tr.next}</Button></div>
    <div role="group" aria-label={tr.agenda} class="flex flex-wrap gap-2">{#each views as mode}<Button variant="secondary" aria-pressed={view === mode.value} class={view === mode.value ? 'text-accent underline' : ''} onclick={() => view = mode.value}>{mode.label}</Button>{/each}</div>
  </div>
  <h2 class="type-title tabular-nums" aria-live="polite">{view === 'month' ? new Intl.DateTimeFormat('tr-TR', {month:'long', year:'numeric', timeZone:'UTC'}).format(parseDate(date)!) : view === 'day' ? formatAgendaDate(date) : `${formatDate(from)} – ${formatDate(to)}`}</h2>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" class="mt-4" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else if view === 'day' && !items.length}<Card title={formatAgendaDate(date)} icon={CalendarDays} count={0}><EmptyState icon={CalendarDays} title={tr.agendaDayEmpty} description={tr.agendaEmptyHint} /></Card>
  {:else if view === 'month'}
    <div class="overflow-x-auto rounded-lg border border-border bg-surface"><div class="grid min-w-[560px] grid-cols-7">
      {#each weekdays as weekday}<div class="type-caption border-b border-border p-2 text-center text-fg-muted">{weekday}</div>{/each}
      {#each days as day (day)}
        {@const rows = items.filter(item => item.appointment.startsOn === day)}
        <button type="button" class="min-h-32 min-w-0 border-r border-b border-border p-2 text-left hover:bg-surface-muted" class:bg-surface-muted={day.slice(0,7) !== date.slice(0,7)} aria-label={formatAgendaDate(day) + ' · ' + rows.length} aria-current={day === localDate() ? 'date' : undefined} onclick={() => { date = day; view = 'day'; }}>
          <span class="mb-2 inline-flex size-7 items-center justify-center rounded-full text-sm" class:text-accent={day === localDate()} class:ring-1={day === localDate()}>{Number(day.slice(-2))}</span>
          {#each rows.slice(0,3) as item}<span class="type-caption mb-2 block"><span class="block truncate">{item.appointment.startsTime ?? tr.agendaAllDay} {item.appointment.title}</span>
            {#if item.appointment.status !== 'planned'}<Badge variant={item.appointment.status === 'done' ? 'success' : 'neutral'}>{statusLabels[item.appointment.status]}</Badge>
            {:else if isOverdue(item.appointment, now)}<Badge variant="highlight">{tr.appointmentOverdue}</Badge>{/if}
          </span>{/each}
          {#if rows.length > 3}<span class="type-caption text-accent">{tr.agendaMore.replace('{count}', String(rows.length - 3))}</span>{/if}
        </button>
      {/each}
    </div></div>
  {:else}<div class="grid gap-4">{#each days.filter(day => view !== 'list' || items.some(item => item.appointment.startsOn === day)) as day (day)}
    {@const rows = items.filter(item => item.appointment.startsOn === day)}
    <Card title={formatAgendaDate(day)} icon={CalendarDays} count={rows.length}>
      {#snippet actions()}{#if view === 'week' || view === 'list'}<Button size="sm" variant="ghost" onclick={() => { date = day; view = 'day'; }}>{tr.agendaDay}</Button>{/if}{/snippet}
      <div class="divide-y divide-border">{#each rows as item (item.appointment.id)}<AppointmentItem {item} {busy} {now} onedit={edit} onstatus={status} />{:else}<p class="py-4 text-fg-muted">{tr.agendaDayEmpty}</p>{/each}</div>
    </Card>
  {/each}{#if view === 'list' && !items.length}<Card><EmptyState icon={CalendarDays} title={tr.agendaDayEmpty} description={tr.agendaEmptyHint} /></Card>{/if}</div>{/if}
</div>
<Dialog bind:open title={editing ? tr.appointmentEdit : tr.agendaDialogTitle} description={tr.appointmentFormDescription}>
  {#if open}<AppointmentForm appointment={editing} {date} onsaved={() => { open = false; refresh++; }} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
