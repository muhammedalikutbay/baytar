<script lang="ts">
  import type { Component, Snippet } from 'svelte';
  import type { LucideProps } from '@lucide/svelte';
  let { icon: Icon, title, description, action }:
    { icon: Component<LucideProps>; title: string; description: string; action?: Snippet } = $props();
</script>

<div class="flex min-h-80 flex-col items-center justify-center px-5 py-12 text-center">
  <div class="mb-5 flex size-14 items-center justify-center rounded-lg border border-border bg-surface-muted text-fg-muted">
    <Icon size={26} strokeWidth={1.5} aria-hidden="true" />
  </div>
  <h2 class="type-title text-fg">{title}</h2>
  <p class="mt-2 max-w-96 text-fg-muted">{description}</p>
  {#if action}<div class="mt-6">{@render action()}</div>{/if}
</div>
