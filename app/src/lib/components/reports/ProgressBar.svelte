<script lang="ts">
  import { formatNumber } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  const uid = $props.id();
  let { label, value, total }: { label: string; value: number; total: number } = $props();
  const percent = $derived(total > 0 ? Math.round(value / total * 100) : 0);
</script>

<div>
  <div class="mb-1.5 flex items-baseline justify-between gap-3">
    <span id={uid}>{label}</span>
    <span class="tabular text-fg-muted"><span class="font-semibold text-fg">%{formatNumber(percent, 0)}</span> · {formatNumber(value)} / {formatNumber(total)} {tr.reportsCoverageOf}</span>
  </div>
  <div role="progressbar" aria-labelledby={uid} aria-valuemin="0" aria-valuemax="100" aria-valuenow={percent} class="h-2.5 overflow-hidden rounded-pill bg-surface-muted">
    <div class="h-full rounded-pill bg-accent" style:width={percent + '%'}></div>
  </div>
</div>
