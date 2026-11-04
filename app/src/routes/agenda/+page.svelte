<script lang="ts">
  import { onMount } from 'svelte';
  import { CalendarDays, Plus } from '@lucide/svelte';
  import { PageHeader, Button, Card, Dialog, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { addDateDays, formatAgendaDate, formatDate, localDate, parseDate } from '$lib/format';
  import * as appointments from '$lib/api/appointments';
  import type { Appointment, AppointmentStatus } from '$lib/api/types';
  import AppointmentForm from '$lib/components/agenda/AppointmentForm.svelte';
  import AppointmentItem from '$lib/components/agenda/AppointmentItem.svelte';
  import { compareAppointments, withPhones, type AgendaItem } from '$lib/components/agenda/appointments';
  let date = $state(localDate());
  let view = $state<'day' | 'week'>('day');
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
  const from = $derived(view === 'day' ? date : monday);
  const to = $derived(view === 'day' ? date : addDateDays(monday, 6));
  const days = $derived(Array.from({ length: view === 'day' ? 1 : 7 }, (_, index) => addDateDays(from, index)));
  onMount(() => { const timer = setInterval(() => now = new Date(), 30000); return () => clearInterval(timer); });
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
  function move(direction: number) { const next = addDateDays(date, direction * (view === 'day' ? 1 : 7)); if (next && next >= '0001-01-08' && next <= '9999-12-24') date = next; }
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
    <div role="group" aria-label={tr.agenda} class="flex gap-2"><Button variant="secondary" aria-pressed={view === 'day'} class={view === 'day' ? 'bg-surface-muted underline' : ''} onclick={() => view = 'day'}>{tr.agendaDay}</Button><Button variant="secondary" aria-pressed={view === 'week'} class={view === 'week' ? 'bg-surface-muted underline' : ''} onclick={() => view = 'week'}>{tr.agendaWeek}</Button></div>
  </div>
  <h2 class="type-title tabular-nums" aria-live="polite">{view === 'day' ? formatAgendaDate(date) : `${formatDate(from)} – ${formatDate(to)}`}</h2>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" class="mt-4" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else if view === 'day' && !items.length}<Card title={formatAgendaDate(date)} icon={CalendarDays} count={0}><EmptyState icon={CalendarDays} title={tr.agendaDayEmpty} description={tr.agendaEmptyHint} /></Card>
  {:else}<div class="grid gap-4">{#each days as day (day)}
    {@const rows = items.filter(item => item.appointment.startsOn === day)}
    <Card title={formatAgendaDate(day)} icon={CalendarDays} count={rows.length}>
      {#snippet actions()}{#if view === 'week'}<Button size="sm" variant="ghost" onclick={() => { date = day; view = 'day'; }}>{tr.agendaDay}</Button>{/if}{/snippet}
      <div class="divide-y divide-border">{#each rows as item (item.appointment.id)}<AppointmentItem {item} {busy} {now} onedit={edit} onstatus={status} />{:else}<p class="py-4 text-fg-muted">{tr.agendaDayEmpty}</p>{/each}</div>
    </Card>
  {/each}</div>{/if}
</div>
<Dialog bind:open title={editing ? tr.appointmentEdit : tr.agendaDialogTitle} description={tr.appointmentFormDescription}>
  {#if open}<AppointmentForm appointment={editing} {date} onsaved={() => { open = false; refresh++; }} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
