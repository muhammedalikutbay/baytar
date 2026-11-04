<script lang="ts">
  import { Download, FolderDown, FileSpreadsheet } from '@lucide/svelte';
  import { Button, Card, Toast } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { localToday } from '$lib/format';
  import * as reports from '$lib/api/reports';
  import type { ExportTable } from '$lib/api/types';

  const labels: Record<ExportTable, string> = {
    owners: tr.reportsExportOwners, patients: tr.reportsExportPatients, visits: tr.reportsExportVisits, vaccinations: tr.reportsExportVaccinations,
    prescriptions: tr.reportsExportPrescriptions, charges: tr.reportsExportCharges, payments: tr.reportsExportPayments, inventory: tr.reportsExportInventory
  };
  let busy = $state(false), failed = $state(false), message = $state(''), toastOpen = $state(false);

  function folderOf(path: string | undefined) { return path ? path.replace(/[\\/][^\\/]*$/, '') : ''; }
  async function run(task: () => Promise<string | string[] | null>) {
    busy = true; failed = false; toastOpen = false;
    try {
      const result = await task();
      if (result === null) return;
      message = Array.isArray(result) ? `${result.length} ${tr.reportsExportedMany}: ${folderOf(result[0])}` : `${tr.reportsExportedOne}: ${result}`;
      toastOpen = true;
    } catch { failed = true; } finally { busy = false; }
  }
</script>

<Card id="export" title={tr.reportsExport} icon={FileSpreadsheet} tone="neutral">
  <p class="mb-5 max-w-prose text-fg-muted">{tr.reportsExportDescription}</p>
  <ul class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
    {#each reports.exportTables as table (table)}
      <li class="flex items-center justify-between gap-3 rounded-md border border-border p-3">
        <span class="font-medium">{labels[table]}</span>
        <Button variant="secondary" size="sm" aria-label={`${labels[table]}: ${tr.reportsExportTable}`} disabled={busy}
          onclick={() => run(() => reports.exportTable(table, localToday()))}><Download size={16} aria-hidden="true" />CSV</Button>
      </li>
    {/each}
  </ul>
  <div class="mt-5 flex flex-wrap items-center gap-4">
    <Button disabled={busy} onclick={() => run(() => reports.exportAll(localToday()))}><FolderDown size={18} aria-hidden="true" />{tr.reportsExportAll}</Button>
    <p class="type-caption text-fg-muted">{tr.reportsExportFormat}</p>
  </div>
  {#if failed}<p role="alert" class="mt-4 text-danger">{tr.reportsExportError}</p>{/if}
</Card>
<Toast bind:open={toastOpen} {message} />
