<script lang="ts">
  import { ChartColumn, Stethoscope, PawPrint, Banknote, HandCoins, Hourglass, PieChart, ShieldCheck, Pill, Activity, Download } from '@lucide/svelte';
  import { PageHeader, Card, StatTile, Select, DateInput, Button, EmptyState, buttonStyles } from '$lib/components/ui';
  import BarChart from '$lib/components/reports/BarChart.svelte';
  import BarList from '$lib/components/reports/BarList.svelte';
  import ProgressBar from '$lib/components/reports/ProgressBar.svelte';
  import ExportCard from '$lib/components/reports/ExportCard.svelte';
  import { periodRange, previousRange, rangeProblem, type PeriodKind } from '$lib/components/reports/periods';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatMoney, formatNumber, localToday, dateAfterMonths } from '$lib/format';
  import * as reports from '$lib/api/reports';
  import type { ReportOverview, ReportMonth, Species } from '$lib/api/types';

  const today = localToday();
  let kind = $state<PeriodKind>('month');
  let customFrom = $state(dateAfterMonths(today, -1)), customTo = $state(today);
  let data = $state<ReportOverview | null>(null), before = $state<ReportOverview | null>(null);
  let loading = $state(true), error = $state(false), refresh = $state(0);

  const range = $derived(periodRange(kind, today, { from: customFrom, to: customTo }));
  const problem = $derived(kind === 'custom' ? rangeProblem(range) : null);

  $effect(() => {
    refresh;
    if (problem) return;
    const current = range, previous = previousRange(kind, range);
    let cancelled = false; loading = true; error = false;
    Promise.all([reports.overview(current.from, current.to, today), reports.overview(previous.from, previous.to, today)])
      .then(([now, then]) => { if (!cancelled) { data = now; before = then; } })
      .catch(() => { if (!cancelled) error = true; })
      .finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });

  const sum = (overview: ReportOverview | null, key: Exclude<keyof ReportMonth, 'month' | 'outstandingMinor'>) => overview?.months.reduce((total, m) => total + m[key], 0) ?? 0;
  const closing = (overview: ReportOverview | null) => overview?.months.at(-1)?.outstandingMinor ?? 0;
  const months = $derived(data?.months.map(m => m.month) ?? []);
  const hasActivity = $derived(!!data && (data.diagnoses.length > 0 || data.months.some(m => m.visits + m.newPatients + m.vaccinations + m.revenueMinor + m.collectionsMinor > 0)));

  function hint(current: number, previous: number | undefined, format: (value: number) => string): string | undefined {
    if (previous === undefined || !before) return undefined;
    const base = `${tr.reportsPreviousPeriod}: ${format(previous)}`;
    if (previous === 0) return base;
    const percent = Math.round((current - previous) / previous * 100);
    return `${tr.reportsVsPrevious} ${percent === 0 ? tr.reportsNoChange : `${percent > 0 ? '+' : ''}${formatNumber(percent, 0)}%`} · ${base}`;
  }
  const count = (value: number) => formatNumber(value, 0);
  const speciesLabel = (species: Species) => tr[species];
  const problemText = $derived(problem === 'order' ? tr.reportsRangeError : problem === 'long' ? tr.reportsRangeTooLong : problem === 'invalid' ? tr.invalidDate : '');
  const periods: { value: PeriodKind; label: string }[] = [
    { value: 'month', label: tr.reportsThisMonth }, { value: 'last3', label: tr.reportsLast3Months }, { value: 'year', label: tr.reportsThisYear },
    { value: 'lastYear', label: tr.reportsLastYear }, { value: 'custom', label: tr.reportsCustom }
  ];
</script>

<div class="grid gap-8">
  <PageHeader title={tr.reports} subtitle={tr.reportsDescription} icon={ChartColumn}>
    {#snippet actions()}<a href="#export" class={buttonStyles('primary')}><Download size={18} aria-hidden="true" />{tr.reportsExport}</a>{/snippet}
  </PageHeader>

  <Card>
    <div class="grid items-start gap-4 sm:grid-cols-2 xl:grid-cols-4">
      <Select label={tr.reportsPeriod} bind:value={kind} options={periods} />
      {#if kind === 'custom'}
        <DateInput label={tr.reportsFrom} bind:value={customFrom} max={customTo || undefined} />
        <DateInput label={tr.reportsTo} bind:value={customTo} min={customFrom || undefined} max={today} />
      {/if}
    </div>
    {#if problemText}<p role="alert" class="mt-3 text-danger">{problemText}</p>
    {:else}<p class="type-caption mt-4 text-fg-muted">{tr.reportsRangeLabel}: <span class="tabular">{formatDate(range.from)} – {formatDate(range.to)}</span></p>{/if}
  </Card>

  {#if error}
    <div><p role="alert" class="mb-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button></div>
  {:else if loading && !data}
    <p role="status" class="py-8 text-fg-muted">{tr.loading}</p>
  {:else if data && !problem}
    <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-5" aria-busy={loading}>
      <StatTile icon={Stethoscope} value={count(sum(data, 'visits'))} label={tr.reportsVisits} hint={hint(sum(data, 'visits'), sum(before, 'visits'), count)} />
      <StatTile icon={PawPrint} value={count(sum(data, 'newPatients'))} label={tr.reportsNewPatients} hint={hint(sum(data, 'newPatients'), sum(before, 'newPatients'), count)} />
      <StatTile icon={Banknote} tone="highlight" value={formatMoney(sum(data, 'revenueMinor'))} label={tr.reportsRevenue} hint={hint(sum(data, 'revenueMinor'), sum(before, 'revenueMinor'), formatMoney)} />
      <StatTile icon={HandCoins} tone="highlight" value={formatMoney(sum(data, 'collectionsMinor'))} label={tr.reportsCollections} hint={hint(sum(data, 'collectionsMinor'), sum(before, 'collectionsMinor'), formatMoney)} />
      <StatTile icon={Hourglass} tone="highlight" value={formatMoney(closing(data))} label={tr.reportsOutstanding} hint={hint(closing(data), closing(before), formatMoney) ?? tr.reportsOutstandingHint} />
    </div>

    {#if !hasActivity && data.species.length === 0 && data.coverage.patients === 0}
      <Card><EmptyState icon={ChartColumn} title={tr.reportsNoData} description={tr.reportsNoDataDescription} /></Card>
    {:else}
      <div class="grid gap-6 xl:grid-cols-2">
        <Card title={tr.reportsVisitsChart} icon={Stethoscope}>
          {#if hasActivity}<BarChart label={tr.reportsVisitsChart} {months} format={count} series={[{ name: tr.reportsVisits, tone: 'accent', values: data.months.map(m => m.visits) }]} />
          {:else}<p class="py-8 text-center text-fg-muted">{tr.reportsEmptyShort}</p>{/if}
        </Card>
        <Card title={tr.reportsMoneyChart} icon={Banknote} tone="highlight">
          {#if hasActivity}<BarChart label={tr.reportsMoneyChart} {months} format={formatMoney}
            series={[{ name: tr.reportsRevenue, tone: 'highlight', values: data.months.map(m => m.revenueMinor) }, { name: tr.reportsCollections, tone: 'accent', values: data.months.map(m => m.collectionsMinor) }]} />
          {:else}<p class="py-8 text-center text-fg-muted">{tr.reportsEmptyShort}</p>{/if}
        </Card>
        <Card title={tr.reportsSpeciesChart} icon={PieChart}>
          {#if data.species.length}
            <BarList rows={data.species.map(s => ({ label: speciesLabel(s.species), value: s.count }))} />
            <p class="type-caption mt-4 text-fg-muted">{tr.reportsSpeciesHint}</p>
          {:else}<p class="py-8 text-center text-fg-muted">{tr.reportsEmptyShort}</p>{/if}
        </Card>
        <Card title={tr.reportsCoverage} icon={ShieldCheck}>
          {#if data.coverage.patients > 0}
            <div class="grid gap-5">
              <ProgressBar label={tr.reportsRabies} value={data.coverage.rabiesCovered} total={data.coverage.patients} />
              <ProgressBar label={tr.reportsCoreAll} value={data.coverage.coreCovered} total={data.coverage.patients} />
            </div>
            <p class="type-caption mt-4 text-fg-muted">{tr.reportsCoverageHint}</p>
          {:else}<p class="py-8 text-center text-fg-muted">{tr.reportsCoverageEmpty}</p>{/if}
        </Card>
        <Card title={tr.reportsDiagnoses} icon={Activity} count={data.diagnoses.length}>
          {@render ranking(data.diagnoses, tr.reportsDiagnosis)}
        </Card>
        <Card title={tr.reportsMedicines} icon={Pill} count={data.medicines.length}>
          {@render ranking(data.medicines, tr.reportsMedicine)}
        </Card>
      </div>
    {/if}
  {/if}

  <ExportCard />
</div>

{#snippet ranking(rows: { name: string; count: number }[], heading: string)}
  {#if rows.length}
    <div class="overflow-x-auto"><table class="w-full text-left">
      <thead class="border-b border-border bg-surface-muted type-caption text-fg-muted"><tr>
        <th scope="col" class="px-3 py-3">{heading}</th><th scope="col" class="px-3 py-3 text-right">{tr.reportsCount}</th>
      </tr></thead>
      <tbody class="divide-y divide-border">{#each rows as row (row.name)}
        <tr><th scope="row" class="px-3 py-3 font-normal">{row.name}</th><td class="tabular px-3 py-3 text-right">{count(row.count)}</td></tr>
      {/each}</tbody>
    </table></div>
  {:else}<p class="py-8 text-center text-fg-muted">{tr.reportsEmptyShort}</p>{/if}
{/snippet}
