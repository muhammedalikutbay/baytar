<script lang="ts">
  import { Settings2 } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.settings} description={tr.settingsDescription} icon={Settings2}
  emptyTitle={tr.settingsEmptyTitle} emptyDescription={tr.settingsEmptyDescription}
  actionLabel={tr.settingsAction} dialogTitle={tr.settingsDialogTitle} dialogDescription={tr.settingsDialogDescription} settings />

<p class="mt-5"><a class="text-accent underline" href="/import">{tr.importTitle}</a></p>
