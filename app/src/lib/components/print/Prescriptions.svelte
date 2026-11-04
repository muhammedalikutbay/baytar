<script lang="ts">
  import type { PrescriptionItem } from '$lib/api/types';
  import { formatNumber } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  let { items }: { items: PrescriptionItem[] } = $props();
  function dose(item: PrescriptionItem) {
    return [item.doseMg == null ? '' : `${formatNumber(item.doseMg)} mg`,
      item.doseMl == null ? '' : `${formatNumber(item.doseMl)} ml`, item.doseText].filter(Boolean).join(' / ') || '—';
  }
</script>
<table>
  <thead><tr><th>{tr.medicationName}</th><th>{tr.documentsDose}</th><th>{tr.frequency}</th><th>{tr.durationDays}</th><th>{tr.notes}</th></tr></thead>
  <tbody>{#each items as item (item.id)}<tr><td>{item.medicationName}</td><td>{dose(item)}</td><td>{item.frequency || '—'}</td><td>{item.durationDays == null ? '—' : formatNumber(item.durationDays)}</td><td>{item.notes || '—'}</td></tr>{:else}<tr><td colspan="5">{tr.documentsEmptyPrescription}</td></tr>{/each}</tbody>
</table>
