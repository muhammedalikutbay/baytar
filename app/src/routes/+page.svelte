<script lang="ts">
  import { House } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import { Card, EmptyState, buttonStyles } from '$lib/components/ui';
</script>

<section>
  <header class="mb-8">
    <h1 class="type-page">{tr.home}</h1>
    <p class="mt-2 text-fg-muted">{tr.homeDescription}</p>
  </header>
  <Card class="p-0 sm:p-0">
    <EmptyState icon={House} title={tr.homeEmptyTitle} description={tr.homeEmptyDescription}>
      {#snippet action()}
        <div class="flex flex-wrap justify-center gap-3">
          <a href="/patients" class={buttonStyles()}>{tr.homeAction}</a>
          <a href="/import" class={buttonStyles('secondary')}>{tr.importTitle}</a>
        </div>
      {/snippet}
    </EmptyState>
  </Card>
  <p class="type-caption mt-5 text-fg-subtle">{tr.localData}</p>
</section>
