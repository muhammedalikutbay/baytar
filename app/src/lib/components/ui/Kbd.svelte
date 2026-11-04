<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  let { class: className, children, ...rest }: HTMLAttributes<HTMLElement> = $props();
</script>

<kbd {...rest} class={cn('type-caption inline-flex min-h-6 items-center rounded-[5px] border border-border-strong bg-surface px-1.5 font-sans text-fg-muted', className)}>
  {@render children?.()}
</kbd>
