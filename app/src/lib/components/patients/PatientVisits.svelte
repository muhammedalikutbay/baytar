<script lang="ts">
  import { Stethoscope } from '@lucide/svelte';
  import { Card, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
</script>
<section aria-label={tr.visits}><Card><h2 class="type-section">{tr.visits}</h2><EmptyState icon={Stethoscope} title={tr.visitsEmpty} description={tr.visitsDescription} /></Card></section>
