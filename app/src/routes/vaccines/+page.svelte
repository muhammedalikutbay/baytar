<script lang="ts">
  import { Syringe } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.vaccines} description={tr.vaccinesDescription} icon={Syringe}
  emptyTitle={tr.vaccinesEmptyTitle} emptyDescription={tr.vaccinesEmptyDescription}
  actionLabel={tr.vaccinesAction} dialogTitle={tr.vaccinesDialogTitle} dialogDescription={tr.vaccinesDialogDescription} />
