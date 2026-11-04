<script lang="ts">
  import { Bell, CalendarDays, Stethoscope, Syringe } from '@lucide/svelte';
  import { Badge, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatPhone } from '$lib/format';
  import type { Appointment, AppointmentStatus } from '$lib/api/types';
  import { isOverdue, kindLabels, statusLabels, type AgendaItem } from './appointments';
  let { item, busy = false, showDate = false, now = new Date(), onedit, onstatus }: {
    item: AgendaItem; busy?: boolean; showDate?: boolean; now?: Date;
    onedit: (appointment: Appointment) => void; onstatus: (id: number, status: AppointmentStatus) => void;
  } = $props();
  const appointment = $derived(item.appointment);
  const Icon = $derived(({ visit: Stethoscope, vaccine: Syringe, reminder: Bell, other: CalendarDays })[appointment.kind]);
</script>
<article class="flex flex-wrap items-start gap-4 py-4">
  <div class="min-w-20 text-fg-muted tabular-nums">{#if showDate}<div class="type-caption">{formatDate(appointment.startsOn)}</div>{/if}<span>{appointment.startsTime ?? tr.agendaAllDay}</span></div>
  <div class="min-w-0 flex-1 basis-48">
    <div class="flex flex-wrap items-center gap-2"><Icon size={18} class="text-fg-muted" aria-hidden="true" /><h3 class="break-words font-medium">{appointment.title}</h3><span class="type-caption text-fg-muted">{kindLabels[appointment.kind]}</span></div>
    <p class="mt-1 break-words text-fg-muted">{#if item.patientName}<a class="text-accent underline" href={`/patients/pet?id=${appointment.patientId}`}>{item.patientName}</a>{/if}{item.patientName && item.ownerName ? ' · ' : ''}{item.ownerName}{item.phone ? ` · ${formatPhone(item.phone)}` : ''}</p>
    {#if appointment.notes}<p class="mt-1 whitespace-pre-wrap break-words text-fg-muted">{appointment.notes}</p>{/if}
    {#if appointment.status !== 'planned' || isOverdue(appointment, now)}<div class="mt-2 flex flex-wrap gap-2">{#if appointment.status !== 'planned'}<Badge variant={appointment.status === 'done' ? 'success' : 'neutral'}>{statusLabels[appointment.status]}</Badge>{/if}{#if isOverdue(appointment, now)}<Badge variant="highlight">{tr.appointmentOverdue}</Badge>{/if}</div>{/if}
  </div>
  <div class="flex flex-wrap gap-2" aria-label={appointment.title}>
    {#if appointment.status === 'planned'}<Button size="sm" variant="secondary" disabled={busy} onclick={() => onstatus(appointment.id, 'done')}>{tr.appointmentDone}</Button><Button size="sm" variant="ghost" disabled={busy} onclick={() => onstatus(appointment.id, 'cancelled')}>{tr.appointmentCancel}</Button>{/if}
    <Button size="sm" variant="ghost" disabled={busy} onclick={() => onedit(appointment)}>{tr.edit}</Button>
  </div>
</article>
