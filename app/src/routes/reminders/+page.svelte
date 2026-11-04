<script lang="ts">
  import { onMount } from 'svelte';
  import { getAll } from '$lib/api/settings';
  import { parseWhatsappMode, whatsappModeLabel, type WhatsappMode } from '$lib/whatsapp';
  let whatsappMode = $state<WhatsappMode>('auto');
  onMount(() => { void getAll().then(values => whatsappMode = parseWhatsappMode(values.whatsapp_mode)).catch(() => {}); });
  import { BellRing, Syringe, CalendarDays, Wallet, ClipboardCheck, CircleCheck, Hourglass } from '@lucide/svelte';
  import { PageHeader, Card, Button, Select, Checkbox, Dialog, StatTile, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { localDate, todayDate } from '$lib/format';
  import * as api from '$lib/api/reminders';
  import type { ContactChannel, ReminderItem, RemindersOverview } from '$lib/api/types';
  import ReminderRow from '$lib/components/reminders/ReminderRow.svelte';
  import ReminderMessage from '$lib/components/reminders/ReminderMessage.svelte';
  import TemplateEditor from '$lib/components/reminders/TemplateEditor.svelte';
  import { loadTemplateContext, renderItem, DEFAULT_TEMPLATES, type TemplateContext, type TemplateSet } from '$lib/components/reminders/templates';
  import type { Component } from 'svelte';
  import type { LucideProps } from '@lucide/svelte';

  const today = todayDate();
  let overview = $state<RemindersOverview | null>(null);
  let context = $state<TemplateContext>({ templates: { ...DEFAULT_TEMPLATES }, clinic: { name: '', phone: '' } });
  let vaccineDays = $state('7');
  let followupDays = $state('7');
  let debtDays = $state('30');
  let hideContacted = $state(false);
  let failed = $state(false);
  let refresh = $state(0);
  let busyKey = $state('');
  let actionError = $state('');
  let messaging = $state<{ item: ReminderItem; message: string } | null>(null);
  let templatesOpen = $state(false);

  $effect(() => {
    refresh;
    const query = { today, vaccineDays: Number(vaccineDays), debtDays: Number(debtDays), followupDays: Number(followupDays) };
    let cancelled = false;
    failed = false;
    api.overview(query).then(result => { if (!cancelled) overview = result; })
      .catch(() => { if (!cancelled) failed = true; });
    return () => { cancelled = true; };
  });
  $effect(() => { void loadTemplateContext().then(result => { context = result; }).catch(() => {}); });

  const contactedToday = (item: ReminderItem) => !!item.lastContact && localDate(new Date(item.lastContact.contactedAt)) === today;
  // Contacted rows sink to the bottom; the API order (by date) is kept inside each group.
  const arrange = (rows: ReminderItem[]) => {
    const pending = rows.filter(row => !contactedToday(row)), done = rows.filter(contactedToday);
    return hideContacted ? pending : [...pending, ...done];
  };
  const all = $derived(overview ? [...overview.vaccines, ...overview.appointments, ...overview.debts, ...overview.followups] : []);
  const pendingCount = $derived(all.filter(row => !contactedToday(row)).length);
  const doneCount = $derived(all.length - pendingCount);
  const sections = $derived(overview ? [
    { id: 'vaccines', title: tr.remindersVaccines, icon: Syringe, tone: 'accent' as const, rows: overview.vaccines, empty: tr.remindersVaccinesEmpty, hint: tr.remindersVaccinesEmptyHint, href: '/vaccines', action: tr.remindersGoVaccines },
    { id: 'appointments', title: tr.remindersAppointments, icon: CalendarDays, tone: 'accent' as const, rows: overview.appointments, empty: tr.remindersAppointmentsEmpty, hint: tr.remindersAppointmentsEmptyHint, href: '/agenda', action: tr.remindersGoAgenda },
    { id: 'debts', title: tr.remindersDebts, icon: Wallet, tone: 'highlight' as const, rows: overview.debts, empty: tr.remindersDebtsEmpty, hint: tr.remindersDebtsEmptyHint, href: '/accounts', action: tr.remindersGoAccounts },
    { id: 'followups', title: tr.remindersFollowups, icon: ClipboardCheck, tone: 'accent' as const, rows: overview.followups, empty: tr.remindersFollowupsEmpty, hint: tr.remindersFollowupsEmptyHint, href: '/agenda', action: tr.remindersPlanVisit }
  ] satisfies { id: string; title: string; icon: Component<LucideProps>; tone: 'accent' | 'highlight'; rows: ReminderItem[]; empty: string; hint: string; href: string; action: string }[] : []);

  const dayOptions = (values: number[], label: string) => values.map(days => ({ value: String(days), label: label.replace('{days}', String(days)) }));
  const windows = dayOptions([3, 7, 14, 30], tr.remindersWithinDays);
  const debtWindows = dayOptions([7, 14, 30, 60, 90], tr.remindersOlderThanDays);

  async function record(item: ReminderItem, channel: ContactChannel) {
    busyKey = item.refKey; actionError = '';
    try { await api.recordContact({ kind: item.kind, refKey: item.refKey, ownerId: item.ownerId, channel }); refresh++; }
    catch { actionError = tr.remindersContactError; }
    finally { busyKey = ''; }
  }
  function preview(item: ReminderItem) { actionError = ''; messaging = { item, message: renderItem(item, context) }; }
  async function sent(item: ReminderItem) { messaging = null; await record(item, 'whatsapp'); }
  function templatesSaved(templates: TemplateSet) { context = { ...context, templates }; }
</script>

<svelte:head><title>{tr.remindersTitle} · {tr.appName}</title></svelte:head>
<div class="grid gap-6">
  <PageHeader title={tr.remindersTitle} subtitle={tr.remindersDescription} icon={BellRing}>
    {#snippet actions()}<Button variant="secondary" onclick={() => templatesOpen = true}>{tr.remindersEditTemplates}</Button>{/snippet}
  </PageHeader>
  <aside class="rounded-md border border-border bg-surface-muted p-3 text-sm text-fg-muted"><p>{tr.whatsappExplanation}</p><p class="mt-1">{tr.whatsappCurrentMode}: {whatsappModeLabel[whatsappMode]} · <a class="text-accent underline" href="/settings#whatsapp">{tr.whatsappChangeSettings}</a></p></aside>

  {#if failed && !overview}
    <Card><p role="alert" class="text-danger">{tr.loadError}</p><div class="mt-3"><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></div></Card>
  {:else if !overview}
    <p role="status">{tr.loading}</p>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2">
      <StatTile icon={Hourglass} value={pendingCount} label={tr.remindersPending} hint={tr.remindersPendingHint} tone="highlight" />
      <StatTile icon={CircleCheck} value={doneCount} label={tr.remindersDoneToday} hint={tr.remindersDoneHint} tone="success" />
    </div>
    <Checkbox label={tr.remindersHideContacted} bind:checked={hideContacted} />
    {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
    {#if failed}<p role="alert" class="text-danger">{tr.loadError} <Button size="sm" variant="ghost" onclick={() => refresh++}>{tr.retry}</Button></p>{/if}

    {#each sections as section (section.id)}
      {@const rows = arrange(section.rows)}
      <Card title={section.title} icon={section.icon} tone={section.tone} count={section.rows.filter(row => !contactedToday(row)).length}>
        {#snippet actions()}
          {#if section.id === 'vaccines'}<Select label={tr.remindersPeriod} bind:value={vaccineDays} options={windows} />
          {:else if section.id === 'debts'}<Select label={tr.remindersPeriod} bind:value={debtDays} options={debtWindows} />
          {:else if section.id === 'followups'}<Select label={tr.remindersPeriod} bind:value={followupDays} options={windows} />{/if}
        {/snippet}
        {#if !section.rows.length}
          <div class="grid justify-items-start gap-3"><div><p class="font-medium">{section.empty}</p><p class="type-caption text-fg-muted">{section.hint}</p></div>
            <a href={section.href} class={buttonStyles('secondary', 'sm')}>{section.action}</a></div>
        {:else if !rows.length}
          <p class="text-fg-muted">{tr.remindersDoneToday}: {section.rows.length}</p>
        {:else}
          <ul class="divide-y divide-border">{#each rows as item (item.refKey)}
            <ReminderRow {item} {today} contacted={contactedToday(item)} busy={busyKey === item.refKey} onwhatsapp={preview} onmark={row => record(row, 'phone')} />
          {/each}</ul>
        {/if}
      </Card>
    {/each}
  {/if}
</div>

<Dialog bind:open={() => messaging !== null, value => { if (!value) messaging = null; }} title={tr.remindersMessageTitle} description={tr.remindersMessageDescription}>
  {#if messaging}<ReminderMessage item={messaging.item} initialMessage={messaging.message} onsent={sent} oncancel={() => messaging = null} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={templatesOpen} title={tr.remindersTemplatesDialogTitle} description={tr.remindersTemplatesDescription}>
  {#if templatesOpen}<TemplateEditor onsaved={templatesSaved} oncancel={() => templatesOpen = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
