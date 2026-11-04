<script lang="ts">
  import type { Visit } from '$lib/api/types';
  import { tr } from '$lib/i18n/tr';
  import { formatNumber } from '$lib/format';
  let { visit }: { visit: Visit } = $props();
  const values = $derived([
    visit.weightKg === null ? '' : `${formatNumber(visit.weightKg)} kg`,
    visit.temperatureC === null ? '' : `${formatNumber(visit.temperatureC)} ${tr.inputTemperatureUnit}`,
    visit.heartRateBpm === null ? '' : `${tr.clinicalHeartRate} ${visit.heartRateBpm}${tr.clinicalPerMinute}`,
    visit.respRateBpm === null ? '' : `${tr.clinicalRespRate} ${visit.respRateBpm}${tr.clinicalPerMinute}`,
    visit.bodyCondition === null ? '' : `${tr.clinicalBcsShort} ${visit.bodyCondition}/9`
  ].filter(Boolean));
</script>
{#if values.length}<p class="type-caption mt-1 text-fg-muted tabular-nums" aria-label={tr.clinicalVitals}>{values.join(' · ')}</p>{/if}
