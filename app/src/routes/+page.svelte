<script lang="ts">
  import { onMount } from 'svelte';
  import { House } from '@lucide/svelte';
  import { PageHeader, Button, Card, EmptyState, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { addDateDays, formatAgendaDate, todayDate } from '$lib/format';
  import * as appointments from '$lib/api/appointments';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as owners from '$lib/api/owners';
  import * as billing from '$lib/api/billing';
  import * as inventory from '$lib/api/inventory';
  import * as settings from '$lib/api/settings';
  import { startupStatus } from '$lib/api/client';
  import type { CalendarAppointment, DueVaccination, InventoryAlerts, Receivable, StartupStatus } from '$lib/api/types';
  import QuickActions from '$lib/components/home/QuickActions.svelte';
  import ActionDialogs, { type HomeDialog } from '$lib/components/home/ActionDialogs.svelte';
  import TodayCard from '$lib/components/home/TodayCard.svelte';
  import AttentionCard from '$lib/components/home/AttentionCard.svelte';
  import UpcomingVaccinesCard from '$lib/components/home/UpcomingVaccinesCard.svelte';
  import MonthStats from '$lib/components/home/MonthStats.svelte';

  let today = $state(todayDate());
  let ownersCount = $state<number | null>(null);
  let todayRows = $state<CalendarAppointment[]>([]);
  let upcomingRows = $state<DueVaccination[]>([]);
  let overdueRows = $state<DueVaccination[]>([]);
  let debts = $state<Receivable[]>([]);
  let alerts = $state<InventoryAlerts | null>(null);
  let vetName = $state('');
  let month = $state({ newOwners: 0, revenue: 0, collections: 0 });
  let status = $state<StartupStatus | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let busy = $state(false);
  let refresh = $state(0);
  let dialog = $state<HomeDialog | null>(null);

  onMount(() => { const timer = setInterval(() => today = todayDate(), 60000); return () => clearInterval(timer); });
  // Secondary widgets must never take the whole cockpit down.
  const soft = <T,>(promise: Promise<T>, fallback: T): Promise<T> => promise.catch(() => fallback);
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
        const monthStart = `${day.slice(0, 8)}01`;
        const [todayAppts, due, late, prefs, receivables, stock, summary, allOwners] = await Promise.all([
          appointments.today(day), vaccinations.dueBetween(day, addDateDays(day, 7)), vaccinations.overdue(day),
          soft(settings.getAll(), {} as Record<string, string>), soft(billing.receivables(), [] as Receivable[]),
          soft(inventory.alerts(day), null as InventoryAlerts | null),
          soft(billing.dailySummary(monthStart, day), null), soft(owners.list({ limit: 5000 }), [])
        ]);
        if (cancelled) return;
        todayRows = todayAppts; upcomingRows = due; overdueRows = late;
        vetName = prefs.vet_name?.trim() ?? '';
        debts = receivables.filter(row => row.balanceMinor > 0).sort((a, b) => b.balanceMinor - a.balanceMinor);
        alerts = stock;
        month = {
          newOwners: allOwners.filter(owner => owner.createdAt.slice(0, 10) >= monthStart).length,
          revenue: summary?.chargesMinor ?? 0, collections: summary?.collectionsMinor ?? 0
        };
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
  const greeting = $derived.by(() => {
    const hour = new Date().getHours();
    const base = hour < 12 ? tr.cockpitMorning : hour < 18 ? tr.cockpitDay : tr.cockpitEvening;
    return vetName ? `${base}, ${vetName}` : base;
  });
  const summaryLine = $derived(
    `${todayRows.length ? tr.cockpitSummaryAppointments.replace('{n}', String(todayRows.length)) : tr.cockpitSummaryNoAppointments}, ${
      overdueRows.length ? tr.cockpitSummaryOverdue.replace('{n}', String(overdueRows.length)) : tr.cockpitSummaryClear}`);
  async function setDone(id: number) {
    if (busy) return;
    busy = true;
    try { await appointments.setStatus(id, 'done'); refresh++; }
    catch { failed = true; }
    finally { busy = false; }
  }
  const ready = $derived(!loading && !failed && status?.ready === true && (ownersCount ?? 0) > 0);
</script>

<div class="grid gap-6">
  <PageHeader title={ready ? greeting : tr.homeDate} subtitle={ready ? `${formatAgendaDate(today)} · ${summaryLine}` : formatAgendaDate(today)} icon={House} />

  {#if loading}<p role="status" class="text-fg-muted">{tr.loading}</p>
  {:else if status?.ready === false}<Card><p role="alert" class="text-fg-muted">{startupMessage}</p></Card>
  {:else if failed}<Card><p role="alert" class="text-danger">{tr.homeLoadError}</p><Button class="mt-4" variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></Card>
  {:else if ownersCount === 0}
    <Card class="p-0 sm:p-0"><EmptyState icon={House} title={tr.homeWelcome} description={tr.homeWelcomeDescription}>
      {#snippet action()}<div class="flex flex-wrap justify-center gap-3"><a href="/import" class={buttonStyles()}>{tr.homeImport}</a><Button variant="secondary" onclick={() => dialog = 'owner'}>{tr.homeAddOwner}</Button></div>{/snippet}
    </EmptyState></Card>
  {:else}
    {#if status?.code === 'backup_failed'}<Card><p role="status" class="text-fg-muted">{startupMessage}</p></Card>{/if}
    <QuickActions onaction={action => dialog = action} />
    <div class="grid items-start gap-5 min-[1100px]:grid-cols-2">
      <div class="grid gap-5">
        <TodayCard rows={todayRows} {busy} ondone={setDone} onadd={() => dialog = 'appointment'} />
        <UpcomingVaccinesCard rows={upcomingRows} />
      </div>
      <div class="grid gap-5">
        <AttentionCard overdue={overdueRows} {debts} {alerts} />
        <MonthStats {today} />
      </div>
    </div>
  {/if}
</div>

<ActionDialogs bind:dialog date={today} onchanged={() => refresh++} />
