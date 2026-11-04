<script lang="ts">
  import type { LucideIcon } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';
  let { icon: Icon, tone = 'accent', size = 'md', children, class: className }:
    { icon?: LucideIcon; tone?: 'accent' | 'highlight' | 'neutral' | 'danger' | 'success'; size?: 'sm' | 'md'; children?: Snippet; class?: string } = $props();
  const tones = {
    accent: 'bg-accent-soft text-accent', highlight: 'bg-highlight-soft text-highlight',
    neutral: 'bg-surface-muted text-fg-muted', danger: 'bg-danger-soft text-danger', success: 'bg-success-soft text-success'
  };
</script>

<span aria-hidden="true" class={cn('inline-flex shrink-0 items-center justify-center rounded-pill', size === 'sm' ? 'size-8' : 'size-10', tones[tone], className)}>
  {#if Icon}<Icon size={size === 'sm' ? 16 : 20} strokeWidth={1.75} />{:else}{@render children?.()}{/if}
</span>
