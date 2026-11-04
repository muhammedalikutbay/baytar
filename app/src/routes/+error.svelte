<script lang="ts">
  import { page } from '$app/state';
  import { FileQuestion } from '@lucide/svelte';
  import { Card, EmptyState, buttonStyles } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
</script>

<h1 class="type-page mb-8">{page.status === 404 ? tr.notFoundTitle : tr.errorTitle}</h1>
<Card class="p-0 sm:p-0">
  <EmptyState icon={FileQuestion} title={String(page.status)}
    description={page.status === 404 ? tr.notFoundDescription : tr.errorDescription}>
    {#snippet action()}<a href="/" class={buttonStyles()}>{tr.backHome}</a>{/snippet}
  </EmptyState>
</Card>
