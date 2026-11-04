<script lang="ts">
  import { Syringe } from '@lucide/svelte';
  import { Card } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate } from '$lib/format';
  import type { DueVaccination } from '$lib/api/types';
  let { rows }: { rows: DueVaccination[] } = $props();
</script>

<section aria-labelledby="cockpit-upcoming">
  <Card title={tr.cockpitUpcoming} icon={Syringe} count={rows.length} tone="accent" headingId="cockpit-upcoming">
    {#snippet actions()}<a class="type-caption text-accent underline" href="/vaccines">{tr.homeAll}</a>{/snippet}
    {#if !rows.length}<p class="text-fg-muted">{tr.cockpitNoUpcoming}</p>
    {:else}
      <ul class="divide-y divide-border">
        {#each rows.slice(0, 6) as row (row.vaccination.id)}
          <li class="flex flex-wrap items-center gap-x-4 gap-y-1 py-3">
            <div class="min-w-28 flex-1"><p class="font-medium">{row.patientName}</p><p class="type-caption text-fg-muted">{row.ownerName}</p></div>
            <span class="min-w-24 flex-1">{row.vaccination.vaccineName}</span>
            <span class="type-caption tabular-nums">{formatDate(row.vaccination.nextDueOn)}</span>
          </li>
        {/each}
      </ul>
      {#if rows.length > 6}<p class="type-caption mt-2 text-fg-muted">{tr.cockpitMore.replace('{n}', String(rows.length - 6))}</p>{/if}
    {/if}
  </Card>
</section>
