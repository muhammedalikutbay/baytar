<script lang="ts">
  import { UserRoundPlus, TrendingUp, Wallet } from '@lucide/svelte';
  import { StatTile } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatMoney } from '$lib/format';
  // TODO: visits and new patients need a reports command (no global visit/patient listing exists yet).
  let { newOwners, revenueMinor, collectionsMinor }: { newOwners: number; revenueMinor: number; collectionsMinor: number } = $props();
</script>

<section aria-labelledby="cockpit-month" class="grid gap-3">
  <h2 id="cockpit-month" class="type-section">{tr.cockpitMonth}</h2>
  <div class="grid gap-3 sm:grid-cols-3">
    <StatTile icon={UserRoundPlus} value={newOwners} label={tr.cockpitNewOwners} />
    <StatTile icon={TrendingUp} tone="highlight" value={formatMoney(revenueMinor)} label={tr.cockpitRevenue} />
    <StatTile icon={Wallet} tone="highlight" value={formatMoney(collectionsMinor)} label={tr.cockpitCollections} />
  </div>
</section>
