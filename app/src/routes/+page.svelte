<script lang="ts">
  import { House } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import PlaceholderPage from '$lib/components/PlaceholderPage.svelte';
</script>

<PlaceholderPage title={tr.home} description={tr.homeDescription} icon={House}
  emptyTitle={tr.homeEmptyTitle} emptyDescription={tr.homeEmptyDescription}
  actionLabel={tr.homeAction} href="/patients" />
