<script lang="ts">
  import { TriangleAlert } from '@lucide/svelte';
  import { Badge } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  let { alerts, deceasedOn }: { alerts: string | null; deceasedOn: string | null } = $props();
</script>
{#if alerts?.trim()}<span title={alerts} class="inline-flex text-highlight"><TriangleAlert size={16} aria-hidden="true" /><span class="sr-only">{tr.clinicalAlerts}: {alerts}</span></span>{/if}
{#if deceasedOn}<Badge>{tr.clinicalDeceased}</Badge>{/if}
