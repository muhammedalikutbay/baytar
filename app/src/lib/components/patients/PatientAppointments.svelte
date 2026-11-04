<script lang="ts">
  import { page } from '$app/state';
  import { onMount } from 'svelte';
  import { CalendarDays } from '@lucide/svelte';
  import { Button, Card, Dialog, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as appointments from '$lib/api/appointments';
  import { get as getPatient } from '$lib/api/patients';
  import { get as getOwner } from '$lib/api/owners';
  import type { Appointment, AppointmentStatus, Patient, Owner } from '$lib/api/types';
  import AppointmentForm from '$lib/components/agenda/AppointmentForm.svelte';
  import AppointmentItem from '$lib/components/agenda/AppointmentItem.svelte';
  import { compareAppointments, isOverdue, type AgendaItem } from '$lib/components/agenda/appointments';
  let { revision = 0, onchange = () => {} }: { revision?: number; onchange?: () => void } = $props();
  let items = $state<AgendaItem[]>([]);
  let patient = $state<Patient>();
  let owner = $state<Owner>();
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let now = $state(new Date());
  let open = $state(false);
  let editing = $state<Appointment>();
  let busy = $state(false);
  let actionError = $state('');
  const upcoming = $derived(items.filter(item => item.appointment.status === 'planned' && !isOverdue(item.appointment, now)));
  const past = $derived(items.filter(item => !upcoming.some(next => next.appointment.id === item.appointment.id)).slice().reverse());
  onMount(() => { const timer = setInterval(() => now = new Date(), 30000); return () => clearInterval(timer); });
  $effect(() => {
    const id = Number(page.url.searchParams.get('id')); refresh; revision;
    let cancelled = false;
    loading = true; failed = false; items = []; patient = undefined; owner = undefined; open = false;
    (async () => {
      try {
        const pet = await getPatient(id), person = await getOwner(pet.ownerId);
        // The calendar API has no patient filter; retain only this patient's rows locally.
        const rows = await appointments.between('0001-01-01', '9999-12-31');
        if (!cancelled) { patient = pet; owner = person; items = rows.filter(row => row.appointment.patientId === id).map(row => ({ ...row, phone: person.phone ?? person.phoneRaw })).sort(compareAppointments); }
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  async function status(id: number, value: AppointmentStatus) {
    if (busy) return;
    busy = true; actionError = '';
    try { await appointments.setStatus(id, value); refresh++; onchange(); }
    catch { actionError = tr.actionError; }
    finally { busy = false; }
  }
  function edit(appointment: Appointment) { editing = appointment; open = true; }
</script>
<section aria-label={tr.appointments}><Card>
  <div class="flex flex-wrap items-center justify-between gap-3"><h2 class="type-section">{tr.appointments}</h2><Button variant="secondary" disabled={loading || !patient} onclick={() => { editing = undefined; open = true; }}>{tr.agendaAction}</Button></div>
  {#if actionError}<p role="alert" class="mt-4 text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status" class="mt-4">{tr.loading}</p>
  {:else if failed}<p role="alert" class="mt-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
  {:else if !items.length}<EmptyState icon={CalendarDays} title={tr.appointmentsEmpty} description={tr.appointmentsDescription} />
  {:else}<h3 class="type-caption mt-5 text-fg-muted">{tr.appointmentUpcoming}</h3><div class="divide-y divide-border">
    {#each upcoming as item (item.appointment.id)}<AppointmentItem {item} {busy} {now} showDate onedit={edit} onstatus={status} />{:else}<p class="py-4 text-fg-muted">{tr.appointmentNoUpcoming}</p>{/each}
  </div>{#if past.length}<h3 class="type-caption mt-5 text-fg-muted">{tr.appointmentPast}</h3><div class="divide-y divide-border">{#each past as item (item.appointment.id)}<AppointmentItem {item} {busy} {now} showDate onedit={edit} onstatus={status} />{/each}</div>{/if}{/if}
</Card></section>
<Dialog bind:open title={editing ? tr.appointmentEdit : tr.agendaDialogTitle} description={tr.appointmentFormDescription}>
  {#if open && patient && owner}<AppointmentForm appointment={editing} {patient} {owner} onsaved={() => { open = false; refresh++; onchange(); }} oncancel={() => open = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
