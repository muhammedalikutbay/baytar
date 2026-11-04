<script lang="ts">
  import { Stethoscope } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.patients} description={tr.patientsDescription} icon={Stethoscope}
  emptyTitle={tr.patientsEmptyTitle} emptyDescription={tr.patientsEmptyDescription}
  actionLabel={tr.patientsAction} dialogTitle={tr.patientsDialogTitle} dialogDescription={tr.patientsDialogDescription} />
