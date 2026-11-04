<script lang="ts">
  import { Stethoscope, PawPrint, TrendingUp, Wallet } from '@lucide/svelte';
  import { StatTile } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatMoney } from '$lib/format';
  import * as reports from '$lib/api/reports';
  import type { ReportMonth } from '$lib/api/types';

  // This month so far, from the reports aggregate (one query for all four numbers).
  let { today }: { today: string } = $props();
  let month = $state<ReportMonth | null>(null);

  $effect(() => {
    const from = today.slice(0, 8) + '01';
    reports.overview(from, today, today)
      .then(result => month = result.months.find(m => m.month === today.slice(0, 7)) ?? null)
      .catch((error: unknown) => console.error('Failed to load month stats', error));
  });
</script>

<section aria-labelledby="cockpit-month" class="grid gap-3">
  <h2 id="cockpit-month" class="type-section">{tr.cockpitMonth}</h2>
  <div class="grid grid-cols-2 gap-3">
    <StatTile icon={Stethoscope} value={month?.visits ?? '—'} label={tr.reportsVisits} />
    <StatTile icon={PawPrint} value={month?.newPatients ?? '—'} label={tr.reportsNewPatients} />
    <StatTile icon={TrendingUp} tone="highlight" value={month ? formatMoney(month.revenueMinor) : '—'} label={tr.cockpitRevenue} />
    <StatTile icon={Wallet} tone="highlight" value={month ? formatMoney(month.collectionsMinor) : '—'} label={tr.cockpitCollections} />
  </div>
</section>
