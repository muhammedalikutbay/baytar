<script lang="ts">
  // Labelled native control; bind value/ref, pass a leading lucide icon or a trailing adornment snippet.
  import type { Snippet } from 'svelte';
  import type { LucideIcon } from '@lucide/svelte';
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  const uid = $props.id();
  let { icon, trailing, label, hint, error, labelClass, id = uid, value = $bindable(''), ref = $bindable(null), class: className, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { icon?: LucideIcon; trailing?: Snippet; label: string; hint?: string; error?: string; labelClass?: string; value?: string; ref?: HTMLInputElement | null } = $props();
</script>

<div class="grid gap-1.5">
  <label for={id} class={cn('text-sm font-medium text-fg', labelClass)}>{label}{#if rest.required}<span class="text-danger" aria-hidden="true"> *</span>{/if}</label>
  <div class="relative">
    {#if icon}{@const Icon = icon}<span class="pointer-events-none absolute left-3 top-3 text-fg-muted" aria-hidden="true"><Icon size={16} /></span>{/if}
    <input {...rest} {id} bind:this={ref} bind:value aria-invalid={error ? true : rest['aria-invalid']}
      aria-describedby={[rest['aria-describedby'], hint && id + '-hint', error && id + '-error'].filter(Boolean).join(' ') || undefined}
      class={cn('min-h-10 text-sm w-full min-w-0 rounded-md border border-border-control bg-surface px-3 text-fg transition-colors hover:bg-surface-muted disabled:bg-surface-muted disabled:text-fg-subtle', icon && 'pl-10', trailing && 'pr-10', error && 'border-danger', className)} />
    {#if trailing}<div class="pointer-events-none absolute inset-y-0 right-2 flex items-center">{@render trailing()}</div>{/if}
  </div>
  {#if hint}<p id={id + '-hint'} class="type-caption text-fg-muted">{hint}</p>{/if}
  {#if error}<p id={id + '-error'} class="type-caption text-danger">{error}</p>{/if}
</div>
