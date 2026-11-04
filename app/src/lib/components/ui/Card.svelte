<script lang="ts">
  import type { HTMLAttributes } from 'svelte/elements';
  import type { Snippet } from 'svelte';
  import type { LucideIcon } from '@lucide/svelte';
  import IconChip from './IconChip.svelte';
  import Badge from './Badge.svelte';
  import { cn } from '$lib/utils';
  let { class: className, children, title, icon, count, actions, tone = 'accent', headingId, ...rest }:
    Omit<HTMLAttributes<HTMLDivElement>, 'title'> & { title?: string; icon?: LucideIcon; count?: number; actions?: Snippet; tone?: 'accent' | 'highlight' | 'neutral' | 'danger' | 'success'; headingId?: string } = $props();
</script>

<div {...rest} class={cn('rounded-lg border border-border bg-surface p-5 sm:p-6', className)}>
  {#if title}
    <div class={cn('mb-4 flex flex-wrap items-center justify-between gap-3 rounded-md p-3', tone === 'highlight' ? 'bg-highlight-soft' : tone === 'accent' ? 'bg-accent-soft' : 'bg-surface-muted')}>
      <div class="flex min-w-0 items-center gap-2.5">
        {#if icon}<IconChip {icon} {tone} size="sm" class="ring-2 ring-surface" />{/if}
        <h2 id={headingId} class="type-section">{title}</h2>
        {#if count !== undefined}<Badge variant={tone === 'neutral' ? 'neutral' : tone} class="border border-border"><span class="tabular">{count}</span></Badge>{/if}
      </div>
      {#if actions}<div class="shrink-0">{@render actions()}</div>{/if}
    </div>
  {/if}
  {@render children?.()}
</div>
