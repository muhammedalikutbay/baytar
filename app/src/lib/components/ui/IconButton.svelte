<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  let { label, selected = false, class: className, children, type = 'button', ...rest }:
    HTMLButtonAttributes & { label: string; selected?: boolean } = $props();
</script>

<button {...rest} {type} aria-label={label} title={label}
  class={cn('inline-flex size-11 shrink-0 items-center justify-center rounded-sm border border-transparent transition-colors hover:bg-surface-muted disabled:cursor-not-allowed disabled:text-fg-subtle',
    selected ? 'border-border-control bg-surface-muted text-accent' : 'text-fg-muted', className)}>
  {@render children?.()}
</button>
