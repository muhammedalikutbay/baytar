<script lang="ts">
  import type { Component } from 'svelte';
  import type { LucideProps } from '@lucide/svelte';
  import { Plus } from '@lucide/svelte';
  import { Button, Card, Dialog, EmptyState, buttonStyles } from '$lib/components/ui';
  import ThemeSwitch from './ThemeSwitch.svelte';
  import { tr } from '$lib/i18n/tr';
  let { title, description, emptyTitle, emptyDescription, icon, actionLabel, href, dialogTitle, dialogDescription, settings = false }:
    { title: string; description: string; emptyTitle: string; emptyDescription: string; icon: Component<LucideProps>;
      actionLabel: string; href?: string; dialogTitle?: string; dialogDescription?: string; settings?: boolean } = $props();
  let open = $state(false);
</script>

<section>
  <header class="mb-8">
    <h1 class="type-page">{title}</h1>
    <p class="mt-2 text-fg-muted">{description}</p>
  </header>
  <Card class="p-0 sm:p-0">
    <EmptyState {icon} title={emptyTitle} description={emptyDescription}>
      {#snippet action()}
        {#if href}<a {href} class={buttonStyles()}>{actionLabel}</a>
        {:else}<Button onclick={() => open = true}>{#if !settings}<Plus size={17} aria-hidden="true" />{/if}{actionLabel}</Button>{/if}
      {/snippet}
    </EmptyState>
  </Card>
  <p class="type-caption mt-5 text-fg-subtle">{tr.localData}</p>
</section>

{#if dialogTitle && dialogDescription}
  <Dialog bind:open title={dialogTitle} description={dialogDescription}>
    {#if settings}{#snippet children()}<ThemeSwitch />{/snippet}{/if}
  </Dialog>
{/if}
