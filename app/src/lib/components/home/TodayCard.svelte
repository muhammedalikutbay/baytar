<script lang="ts">
  import { CalendarDays } from '@lucide/svelte';
  import { Card, Badge, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { kindLabels } from '$lib/components/agenda/appointments';
  import type { CalendarAppointment } from '$lib/api/types';
  let { rows, busy, ondone, onadd }: {
    rows: CalendarAppointment[]; busy: boolean; ondone: (id: number) => void; onadd: () => void;
  } = $props();
  const sorted = $derived([...rows].sort((a, b) => (a.appointment.startsTime ?? '').localeCompare(b.appointment.startsTime ?? '')));
</script>

<section aria-labelledby="cockpit-today">
  <Card title={tr.cockpitToday} icon={CalendarDays} count={rows.length} tone="accent" headingId="cockpit-today">
    {#snippet actions()}<a class="type-caption text-accent underline" href="/agenda">{tr.homeAll}</a>{/snippet}
    {#if !rows.length}
      <div class="grid justify-items-start gap-3"><p class="text-fg-muted">{tr.cockpitNoAppointments}</p><Button size="sm" variant="secondary" onclick={onadd}>{tr.cockpitAddFirst}</Button></div>
    {:else}
      <ul class="divide-y divide-border">
        {#each sorted as row (row.appointment.id)}
          <li class="flex flex-wrap items-center gap-3 py-3">
            <span class="w-14 tabular-nums text-fg-muted">{row.appointment.startsTime ?? tr.cockpitAllDay}</span>
            <div class="min-w-0 flex-1">
              <p class="font-medium break-words">{row.patientName ?? row.appointment.title}</p>
              <p class="type-caption text-fg-muted">{row.ownerName ?? ''}{row.ownerName && row.patientName ? ' · ' : ''}{row.patientName ? row.appointment.title : ''}</p>
            </div>
            <Badge variant="neutral">{kindLabels[row.appointment.kind]}</Badge>
            {#if row.appointment.status === 'planned'}
              <Button size="sm" variant="secondary" disabled={busy} onclick={() => ondone(row.appointment.id)}>{tr.appointmentDone}</Button>
            {:else if row.appointment.status === 'done'}<Badge variant="success">{tr.appointmentDone}</Badge>
            {:else}<Badge variant="neutral">{tr.appointmentCancelled}</Badge>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  </Card>
</section>
