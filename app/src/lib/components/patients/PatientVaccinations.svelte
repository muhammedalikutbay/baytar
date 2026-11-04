<script lang="ts">
  import { Syringe } from '@lucide/svelte';
  import { Card, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
</script>
<section aria-label={tr.vaccines}><Card><h2 class="type-section">{tr.vaccines}</h2><EmptyState icon={Syringe} title={tr.patientVaccinesEmpty} description={tr.patientVaccinesDescription} /></Card></section>
