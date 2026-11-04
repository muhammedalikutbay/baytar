<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { LucideIcon } from '@lucide/svelte';
  import IconChip from './IconChip.svelte';
  let { title, subtitle, icon, actions }:
    { title: string; subtitle?: string; icon?: LucideIcon; actions?: Snippet } = $props();
</script>

<header class="flex flex-wrap items-center justify-between gap-4">
  <div class="flex min-w-0 items-center gap-3.5">
    {#if icon}<IconChip {icon} />{/if}
    <div class="min-w-0"><h1 class="type-page text-balance">{title}</h1>{#if subtitle}<p class="mt-1.5 text-fg-muted">{subtitle}</p>{/if}</div>
  </div>
  {#if actions}<div class="flex flex-wrap items-center gap-2">{@render actions()}</div>{/if}
</header>
