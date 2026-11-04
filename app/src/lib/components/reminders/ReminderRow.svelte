<script lang="ts">
  import { Badge, Button } from '$lib/components/ui';
  import SpeciesIcon from '$lib/components/patients/SpeciesIcon.svelte';
  import { tr } from '$lib/i18n/tr';
  import { daysUntil, formatDate, formatMoney, formatPhone, whatsappPhone } from '$lib/format';
  import type { ReminderItem } from '$lib/api/types';
  let { item, today, contacted, busy = false, onwhatsapp, onmark }:
    { item: ReminderItem; today: string; contacted: boolean; busy?: boolean; onwhatsapp: (item: ReminderItem) => void; onmark: (item: ReminderItem) => void } = $props();
  const days = $derived(daysUntil(item.date, today) ?? 0);
  const relative = $derived(days === 0 ? tr.remindersToday : days === 1 ? tr.remindersTomorrow
    : days > 0 ? tr.remindersInDays.replace('{days}', String(days)) : tr.remindersDaysAgo.replace('{days}', String(-days)));
  // Overdue vaccines and old debts are the urgent ones; upcoming rows stay neutral.
  const urgent = $derived(item.kind === 'vaccine' ? days < 0 : item.kind === 'debt');
  const reason = $derived(item.kind === 'debt' ? tr.remindersDebtReason : item.reason);
  const detail = $derived(item.kind === 'debt' ? tr.remindersOldestDebt.replace('{date}', formatDate(item.date))
    : item.kind === 'followup' && item.lastVisitOn ? tr.remindersLastVisit.replace('{date}', formatDate(item.lastVisitOn))
    : formatDate(item.date) + (item.time ? ` · ${item.time}` : ''));
</script>

<li class={['flex flex-wrap items-center gap-3 py-4', contacted && 'opacity-70']}>
  <div class="flex min-w-48 flex-1 items-center gap-3">
    {#if item.species}<SpeciesIcon species={item.species} />{/if}
    <span class="min-w-0">
      <a href={`/patients/owner?id=${item.ownerId}`} class="block truncate font-semibold hover:underline">{item.ownerName}</a>
      <span class="type-caption block text-fg-muted">
        {#if item.patientId && item.patientName}<a href={`/patients/pet?id=${item.patientId}`} class="hover:underline">{item.patientName}</a>{:else}{tr.billingTitle}{/if}
        · <span class={item.phone ? '' : 'text-danger'}>{item.phone ? formatPhone(item.phone) : tr.remindersNoPhone}</span>
      </span>
    </span>
  </div>
  <div class="min-w-40 flex-1">
    <p>{reason}{#if item.kind === 'debt' && item.amountMinor != null} <strong class="tabular">{formatMoney(item.amountMinor)}</strong>{/if}</p>
    <p class="type-caption text-fg-muted">{detail}</p>
  </div>
  <Badge variant={urgent ? 'highlight' : 'neutral'}>{relative}</Badge>
  {#if contacted}<Badge variant="success">{tr.remindersContactedToday}</Badge>{/if}
  <div class="flex flex-wrap gap-2">
    <Button size="sm" variant="secondary" disabled={busy || !whatsappPhone(item.phone)} onclick={() => onwhatsapp(item)}>{tr.remindersWhatsApp}</Button>
    <Button size="sm" variant="ghost" disabled={busy} onclick={() => onmark(item)}>{tr.remindersMarkContacted}</Button>
  </div>
</li>
