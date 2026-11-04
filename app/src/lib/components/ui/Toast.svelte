<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Check, X } from '@lucide/svelte';
  import { cn } from '$lib/utils';
  import { tr } from '$lib/i18n/tr';
  let { open = $bindable(true), message, action, inline = false }:
    { open?: boolean; message: string; action?: Snippet; inline?: boolean } = $props();
</script>

<!-- Persistent by default: undo stays available until the user dismisses it. -->
<div role="status" aria-live="polite" aria-atomic="true">
  {#if open}
    <div class={cn('flex items-center gap-3 rounded-md border border-border bg-surface p-3 text-fg shadow-float',
      !inline && 'fixed right-6 bottom-6 z-30 max-w-[calc(100vw-3rem)]')}>
      <Check size={18} class="text-success" aria-hidden="true" />
      <span class="flex-1">{message}</span>
      {#if action}{@render action()}{/if}
      <button type="button" aria-label={tr.close} title={tr.close} onclick={() => open = false}
        class="flex size-11 shrink-0 items-center justify-center rounded-sm text-fg-muted hover:bg-surface-muted">
        <X size={16} aria-hidden="true" />
      </button>
    </div>
  {/if}
</div>
