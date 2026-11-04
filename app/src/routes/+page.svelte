<script lang="ts">
  import { onMount } from 'svelte';
  import { House, CalendarDays, Syringe, TriangleAlert, Plus, UserRoundPlus, Search } from '@lucide/svelte';
  import { Button, Card, Dialog, EmptyState, Badge, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { addDateDays, formatAgendaDate, formatDate, formatPhone, todayDate } from '$lib/format';
  import * as appointments from '$lib/api/appointments';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as owners from '$lib/api/owners';
  import * as patients from '$lib/api/patients';
  import { startupStatus } from '$lib/api/client';
  import type { CalendarAppointment, DueVaccination, StartupStatus } from '$lib/api/types';
  import AppointmentForm from '$lib/components/agenda/AppointmentForm.svelte';
  import OwnerForm from '$lib/components/patients/OwnerForm.svelte';

  type VaccineRow = DueVaccination & { phone: string | null };
  let today = $state(todayDate());
  let ownersCount = $state<number | null>(null);
  let todayRows = $state<CalendarAppointment[]>([]);
  let upcomingRows = $state<VaccineRow[]>([]);
  let overdueRows = $state<VaccineRow[]>([]);
  let overdueAppointments = $state<CalendarAppointment[]>([]);
  let status = $state<StartupStatus | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let busy = $state(false);
  let refresh = $state(0);
  let dialog = $state<'owner' | 'appointment' | null>(null);

  onMount(() => { const timer = setInterval(() => today = todayDate(), 60000); return () => clearInterval(timer); });
  $effect(() => {
    const day = today; refresh;
    let cancelled = false;
    loading = true; failed = false;
    (async () => {
      try {
        const [startup, ownerRows] = await Promise.all([startupStatus(), owners.list({ limit: 1 })]);
        if (cancelled) return;
        status = startup; ownersCount = ownerRows.length;
        if (!startup.ready || ownerRows.length === 0) return;
        const yesterday = addDateDays(day, -1);
        const [todayAppts, due, lateVaccines, pastAppts] = await Promise.all([
          appointments.today(day), vaccinations.dueBetween(day, addDateDays(day, 7)), vaccinations.overdue(day),
          appointments.between('0001-01-01', yesterday)
        ]);
        const overdueAppts = pastAppts.filter(row => row.appointment.status === 'planned');
        const allVaccines = [...due, ...lateVaccines];
        const phones = new Map<number, string | null>();
        await Promise.all([...new Set(allVaccines.map(row => row.vaccination.patientId))].map(async id => {
          const pet = await patients.get(id); const owner = await owners.get(pet.ownerId);
          phones.set(id, owner.phone ?? owner.phoneRaw);
        }));
        if (!cancelled) {
          todayRows = todayAppts;
          upcomingRows = due.map(row => ({ ...row, phone: phones.get(row.vaccination.patientId) ?? null }));
          overdueRows = lateVaccines.map(row => ({ ...row, phone: phones.get(row.vaccination.patientId) ?? null }));
          overdueAppointments = overdueAppts;
        }
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  const startupMessages: Record<string, string> = {
    newer_schema: tr.homeStartupNewer, not_baytar_database: tr.homeStartupDatabase,
    database_unavailable: tr.homeStartupUnavailable, backup_failed: tr.homeStartupBackup
  };
  const startupMessage = $derived(status && (status.ready === false || status.code === 'backup_failed')
    ? startupMessages[status.code ?? ''] ?? tr.homeStartupOther : '');
  const overdueCount = $derived(overdueRows.length + overdueAppointments.length);
  function focusSearch() { document.querySelector<HTMLInputElement>('.topbar input[type="search"]')?.focus(); }
  async function setDone(id: number) {
    if (busy) return;
    busy = true;
    try { await appointments.setStatus(id, 'done'); refresh++; }
    catch { failed = true; }
    finally { busy = false; }
  }
</script>

<div class="grid gap-8">
  <header class="flex flex-wrap items-end justify-between gap-4">
    <div><h1 class="type-page">{tr.homeDate}</h1><p class="mt-2 text-fg-muted tabular-nums">{formatAgendaDate(today)}</p></div>
    {#if !loading && ownersCount !== null && ownersCount > 0 && status?.ready}
      <div class="flex flex-wrap gap-2">
        <Button size="sm" variant="secondary" onclick={() => dialog = 'owner'}><UserRoundPlus size={16} aria-hidden="true" />{tr.homeNewOwner}</Button>
        <Button size="sm" variant="secondary" onclick={() => dialog = 'appointment'}><Plus size={16} aria-hidden="true" />{tr.homeAddAppointment}</Button>
        <Button size="sm" variant="secondary" onclick={focusSearch}><Search size={16} aria-hidden="true" />{tr.homeFindPatient}</Button>
      </div>
    {/if}
  </header>

  {#if loading}<p role="status" class="text-fg-muted">{tr.loading}</p>
  {:else if status?.ready === false}<Card><p role="alert" class="text-fg-muted">{startupMessage}</p></Card>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.homeLoadError}</p><Button class="mt-4" variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else if ownersCount === 0}
    <Card class="p-0 sm:p-0"><EmptyState icon={House} title={tr.homeWelcome} description={tr.homeWelcomeDescription}>
      {#snippet action()}<div class="flex flex-wrap justify-center gap-3"><a href="/import" class={buttonStyles()}>{tr.homeImport}</a><Button variant="secondary" onclick={() => dialog = 'owner'}>{tr.homeAddOwner}</Button></div>{/snippet}
    </EmptyState></Card>
  {:else}
    {#if status?.code === 'backup_failed'}<Card><p role="status" class="text-fg-muted">{startupMessage}</p></Card>{/if}
    <div class="grid gap-5 xl:grid-cols-2">
      <section aria-labelledby="today-appointments"><Card>
        <div class="mb-3 flex items-center justify-between gap-3"><h2 id="today-appointments" class="type-section flex items-center gap-2"><CalendarDays size={18} class="text-fg-muted" aria-hidden="true" />{tr.homeAppointments}</h2><a class="type-caption text-accent underline" href="/agenda">{tr.homeAll}</a></div>
        {#if !todayRows.length}<p class="text-fg-muted">{tr.homeNoAppointments}</p>
        {:else}<ul class="divide-y divide-border">{#each todayRows.slice(0, 5) as row (row.appointment.id)}<li class="flex flex-wrap items-center gap-3 py-3">
          <span class="w-12 tabular-nums text-fg-muted">{row.appointment.startsTime ?? tr.agendaAllDay}</span><div class="min-w-0 flex-1"><p class="font-medium break-words">{row.appointment.title}</p><p class="type-caption text-fg-muted">{row.patientName ?? ''}{row.patientName && row.ownerName ? ' · ' : ''}{row.ownerName ?? ''}</p></div>
          {#if row.appointment.status === 'planned'}<Button size="sm" variant="secondary" disabled={busy} onclick={() => setDone(row.appointment.id)}>{tr.appointmentDone}</Button>{:else}<Badge variant="success">{tr.appointmentDone}</Badge>{/if}
        </li>{/each}</ul>{/if}
      </Card></section>

      <section aria-labelledby="upcoming-vaccines"><Card>
        <div class="mb-3 flex items-center justify-between gap-3"><h2 id="upcoming-vaccines" class="type-section flex items-center gap-2"><Syringe size={18} class="text-fg-muted" aria-hidden="true" />{tr.homeUpcomingVaccines}</h2><a class="type-caption text-accent underline" href="/vaccines">{tr.homeAll}</a></div>
        {#if !upcomingRows.length}<p class="text-fg-muted">{tr.homeNoVaccines}</p>
        {:else}<ul class="divide-y divide-border">{#each upcomingRows.slice(0, 5) as row (row.vaccination.id)}<li class="flex flex-wrap items-center gap-x-4 gap-y-1 py-3"><div class="min-w-28 flex-1"><p class="font-medium">{row.patientName}</p><p class="type-caption text-fg-muted">{row.ownerName}{row.phone ? ` · ${formatPhone(row.phone)}` : ''}</p></div><span class="min-w-24 flex-1">{row.vaccination.vaccineName}</span><span class="type-caption tabular-nums">{formatDate(row.vaccination.nextDueOn)}</span></li>{/each}</ul>{/if}
      </Card></section>

      <section aria-labelledby="overdue-items" class="xl:col-span-2"><Card>
        <div class="mb-3 flex items-center justify-between gap-3"><h2 id="overdue-items" class="type-section flex items-center gap-2"><TriangleAlert size={18} class="text-highlight" aria-hidden="true" />{tr.homeOverdue}<Badge variant="highlight">{overdueCount}</Badge></h2><a class="type-caption text-accent underline" href="/agenda">{tr.homeAll}</a></div>
        {#if overdueCount === 0}<p class="text-fg-muted">{tr.homeNoOverdue}</p>
        {:else}<ul class="divide-y divide-border">
          {#each overdueRows.slice(0, 5) as row (row.vaccination.id)}<li class="flex flex-wrap items-center gap-3 py-3"><Badge variant="highlight">{tr.vaccination}</Badge><div class="min-w-0 flex-1"><p class="font-medium">{row.patientName} · {row.vaccination.vaccineName}</p><p class="type-caption text-fg-muted">{row.ownerName}{row.phone ? ` · ${formatPhone(row.phone)}` : ''}</p></div><span class="type-caption tabular-nums text-highlight">{formatDate(row.vaccination.nextDueOn)}</span></li>{/each}
          {#each overdueAppointments.slice(0, Math.max(0, 5 - Math.min(5, overdueRows.length))) as row (row.appointment.id)}<li class="flex flex-wrap items-center gap-3 py-3"><Badge variant="highlight">{tr.appointment}</Badge><div class="min-w-0 flex-1"><p class="font-medium">{row.appointment.title}</p><p class="type-caption text-fg-muted">{row.patientName ?? ''}{row.patientName && row.ownerName ? ' · ' : ''}{row.ownerName ?? ''}</p></div><span class="type-caption tabular-nums text-highlight">{formatDate(row.appointment.startsOn)}</span></li>{/each}
        </ul>{/if}
      </Card></section>
    </div>
  {/if}
</div>

<Dialog bind:open={() => dialog !== null, value => { if (!value) dialog = null; }} title={dialog === 'owner' ? tr.newOwner : tr.agendaDialogTitle} description={dialog === 'owner' ? tr.ownerFormDescription : tr.appointmentFormDescription}>
  {#if dialog === 'owner'}<OwnerForm onsaved={() => { dialog = null; refresh++; }} oncancel={() => dialog = null} />
  {:else if dialog === 'appointment'}<AppointmentForm date={today} onsaved={() => { dialog = null; refresh++; }} oncancel={() => dialog = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
