<script lang="ts">
  import { formatMonth } from '$lib/format';
  import { tr } from '$lib/i18n/tr';

  type Series = { name: string; values: number[]; tone: 'accent' | 'highlight' };
  let { months, series, format, label }:
    { months: string[]; series: Series[]; format: (value: number) => string; label: string } = $props();

  const W = 640, H = 220, left = 8, right = 8, top = 16, bottom = 28, plotH = H - top - bottom;
  const max = $derived(Math.max(1, ...series.flatMap(s => s.values)));
  const slot = $derived((W - left - right) / Math.max(1, months.length));
  const gap = $derived(Math.min(6, slot * 0.2));
  // Bars stay slim (max 40 px) and the group is centred in its month slot, so one month never fills the chart.
  const barW = $derived(Math.min(40, Math.max(1, (slot - gap * 2) / series.length - 1)));
  const groupW = $derived(series.length * (barW + 1) - 1);
  const step = $derived(Math.ceil(months.length / 12));
  const short = (month: string) => new Intl.DateTimeFormat('tr-TR', { month: 'short', timeZone: 'UTC' }).format(new Date(month + '-01T00:00:00Z'));
</script>

<div>
  <svg viewBox="0 0 {W} {H}" class="h-auto w-full" role="img" aria-label={label}>
    <line x1={left} x2={W - right} y1={H - bottom} y2={H - bottom} class="stroke-border-strong" stroke-width="1" />
    {#each [0.5, 1] as fraction (fraction)}
      <line x1={left} x2={W - right} y1={top + plotH * (1 - fraction)} y2={top + plotH * (1 - fraction)} class="stroke-border" stroke-width="1" stroke-dasharray="3 4" />
    {/each}
    <text x={W - right} y={top - 4} text-anchor="end" class="fill-fg-subtle tabular" font-size="10">{format(max)}</text>
    {#each months as month, i (month)}
      {#each series as s, j (s.name)}
        {@const height = (s.values[i] / max) * plotH}
        <rect x={left + i * slot + (slot - groupW) / 2 + j * (barW + 1)} y={top + plotH - height} width={barW} height={Math.max(height, s.values[i] > 0 ? 1 : 0)} rx="2"
          class={s.tone === 'accent' ? 'fill-accent' : 'fill-highlight'}><title>{formatMonth(month + '-01')} · {s.name}: {format(s.values[i])}</title></rect>
      {/each}
      {#if i % step === 0}
        <text x={left + i * slot + slot / 2} y={H - 10} text-anchor="middle" class="fill-fg-muted" font-size="11">{short(month)}</text>
      {/if}
    {/each}
  </svg>
  {#if series.length > 1}
    <ul class="type-caption mt-3 flex flex-wrap gap-x-5 gap-y-1 text-fg-muted">
      {#each series as s (s.name)}
        <li class="flex items-center gap-2"><span aria-hidden="true" class={'size-2.5 rounded-sm ' + (s.tone === 'accent' ? 'bg-accent' : 'bg-highlight')}></span>{s.name}</li>
      {/each}
    </ul>
  {/if}
  <table class="sr-only">
    <caption>{label}</caption>
    <thead><tr><th scope="col">{tr.reportsMonth}</th>{#each series as s (s.name)}<th scope="col">{s.name}</th>{/each}</tr></thead>
    <tbody>{#each months as month, i (month)}<tr><th scope="row">{formatMonth(month + '-01')}</th>{#each series as s (s.name)}<td>{format(s.values[i])}</td>{/each}</tr>{/each}</tbody>
  </table>
</div>
