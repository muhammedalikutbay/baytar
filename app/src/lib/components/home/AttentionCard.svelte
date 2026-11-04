<script lang="ts">
  import { TriangleAlert } from '@lucide/svelte';
  import { Card, Badge } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatMoney } from '$lib/format';
  import type { DueVaccination, InventoryAlerts, Receivable } from '$lib/api/types';
  let { overdue, debts, alerts }: { overdue: DueVaccination[]; debts: Receivable[]; alerts: InventoryAlerts | null } = $props();
  const debtTotal = $derived(debts.reduce((sum, row) => sum + row.balanceMinor, 0));
  const stockCount = $derived(alerts ? alerts.lowStockCount + alerts.expiringCount + alerts.expiredCount : 0);
  const total = $derived(overdue.length + debts.length + stockCount);
</script>

<section aria-labelledby="cockpit-attention">
  <Card title={tr.cockpitAttention} icon={TriangleAlert} count={total} tone="highlight" headingId="cockpit-attention">
    {#if total === 0}<p class="text-fg-muted">{tr.cockpitAllClear}</p>{/if}
    <div class="grid gap-5">
      {#if overdue.length}
        <div>
          <div class="flex items-center justify-between gap-2"><h3 class="font-medium">{tr.cockpitOverdueVaccines}</h3><a class="type-caption text-accent underline" href="/vaccines">{overdue.length}</a></div>
          <ul class="mt-1 divide-y divide-border">
            {#each overdue.slice(0, 3) as row (row.vaccination.id)}
              <li class="flex flex-wrap items-center justify-between gap-2 py-2"><span>{row.patientName} · {row.vaccination.vaccineName}</span><span class="type-caption tabular-nums text-highlight">{formatDate(row.vaccination.nextDueOn)}</span></li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if debts.length}
        <div>
          <div class="flex items-center justify-between gap-2"><h3 class="font-medium">{tr.cockpitDebts}</h3><a class="type-caption text-accent underline" href="/accounts">{tr.cockpitDebtTotal}: <span class="tabular-nums">{formatMoney(debtTotal)}</span></a></div>
          <ul class="mt-1 divide-y divide-border">
            {#each debts.slice(0, 3) as row (row.ownerId)}
              <li class="flex flex-wrap items-center justify-between gap-2 py-2"><a class="hover:underline" href={`/accounts?ownerId=${row.ownerId}`}>{row.ownerName}</a><span class="tabular-nums text-highlight">{formatMoney(row.balanceMinor)}</span></li>
            {/each}
          </ul>
        </div>
      {/if}
      {#if alerts && stockCount > 0}
        <div>
          <div class="flex items-center justify-between gap-2"><h3 class="font-medium">{tr.cockpitStockAlerts}</h3><a class="type-caption text-accent underline" href="/inventory">{tr.homeAll}</a></div>
          <div class="mt-2 flex flex-wrap gap-2">
            {#if alerts.lowStockCount}<a href="/inventory"><Badge variant="highlight">{tr.cockpitLowStock}: {alerts.lowStockCount}</Badge></a>{/if}
            {#if alerts.expiringCount}<a href="/inventory"><Badge variant="highlight">{tr.cockpitExpiring}: {alerts.expiringCount}</Badge></a>{/if}
            {#if alerts.expiredCount}<a href="/inventory"><Badge variant="danger">{tr.cockpitExpired}: {alerts.expiredCount}</Badge></a>{/if}
          </div>
          {#if alerts.lowStock.length}<p class="type-caption mt-2 text-fg-muted">{alerts.lowStock.slice(0, 3).map(i => i.name).join(', ')}</p>{/if}
        </div>
      {/if}
    </div>
  </Card>
</section>
