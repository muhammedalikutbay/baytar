<script lang="ts">
  import { formatNumber } from '$lib/format';
  let { rows, tone = 'accent' }: { rows: { label: string; value: number }[]; tone?: 'accent' | 'highlight' } = $props();
  const max = $derived(Math.max(1, ...rows.map(r => r.value)));
  const total = $derived(rows.reduce((sum, r) => sum + r.value, 0));
</script>

<ul class="grid gap-3">
  {#each rows as row (row.label)}
    <li>
      <div class="mb-1 flex items-baseline justify-between gap-3">
        <span>{row.label}</span>
        <span class="tabular text-fg-muted">{formatNumber(row.value)} <span class="type-caption">· %{formatNumber(total ? Math.round(row.value / total * 100) : 0, 0)}</span></span>
      </div>
      <svg viewBox="0 0 100 8" preserveAspectRatio="none" class="h-2 w-full" aria-hidden="true">
        <rect x="0" y="0" width="100" height="8" rx="4" class="fill-surface-muted" />
        <rect x="0" y="0" width={Math.max(row.value > 0 ? 2 : 0, row.value / max * 100)} height="8" rx="4" class={tone === 'accent' ? 'fill-accent' : 'fill-highlight'} />
      </svg>
    </li>
  {/each}
</ul>
