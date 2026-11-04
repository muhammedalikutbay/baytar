<script lang="ts">
  import { Pill } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.medications} description={tr.medicationsDescription} icon={Pill}
  emptyTitle={tr.medicationsEmptyTitle} emptyDescription={tr.medicationsEmptyDescription}
  actionLabel={tr.medicationsAction} dialogTitle={tr.medicationsDialogTitle} dialogDescription={tr.medicationsDialogDescription} />
