<script lang="ts">
  import { CalendarDays } from '@lucide/svelte';
  import { Card, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
</script>
<section aria-label={tr.appointments}><Card><h2 class="type-section">{tr.appointments}</h2><EmptyState icon={CalendarDays} title={tr.appointmentsEmpty} description={tr.appointmentsDescription} /></Card></section>
