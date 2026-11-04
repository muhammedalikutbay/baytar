<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  let { variant = 'neutral', class: className, children, ...rest }:
    HTMLAttributes<HTMLSpanElement> & { variant?: 'neutral' | 'accent' | 'highlight' | 'success' | 'danger' } = $props();
  const variants = {
    neutral: 'bg-surface-muted text-fg-muted',
    accent: 'bg-accent-soft text-accent',
    highlight: 'bg-highlight-soft text-highlight',
    success: 'bg-success-soft text-success',
    danger: 'bg-danger-soft text-danger'
  };
</script>

<span {...rest} class={cn('type-caption inline-flex items-center gap-1.5 rounded-sm px-2 py-1', variants[variant], className)}>
  {@render children?.()}
</span>
