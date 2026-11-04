<script lang="ts">
  import * as visits from '$lib/api/visits';
  import { mergedDue } from '$lib/api/vaccinations';
  import { formatDate, todayDate } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  let { patientId }: { patientId: number } = $props();
  let last = $state<string | null>(null), next = $state<string | null>(null), overdue = $state(false), failed = $state(false);
  $effect(() => {
    const id = patientId; let cancelled = false;
    Promise.all([visits.listByPatient(id), mergedDue({ patientId: id })]).then(([history, due]) => {
      if (cancelled) return;
      last = history.filter(v => v.visitOn <= todayDate()).sort((a,b) => b.visitOn.localeCompare(a.visitOn))[0]?.visitOn ?? null;
      const first = due.slice().sort((a,b) => a.dueOn.localeCompare(b.dueOn))[0];
      next = first ? first.vaccineName + ' · ' + formatDate(first.dueOn) : null;
      overdue = !!first && first.dueOn < todayDate();
    }).catch(() => { if (!cancelled) failed = true; });
    return () => { cancelled = true; };
  });
</script>
<span class="type-caption grid gap-1 text-fg-muted">
  {#if failed}<span>{tr.loadError}</span>{:else}
    <span>{tr.flowLastVisit}: {last ? formatDate(last) : '—'}</span>
    <span class:text-highlight={overdue}>{overdue ? tr.flowOverdueVaccine : tr.flowNextVaccine}: {next ?? '—'}</span>
  {/if}
</span>
