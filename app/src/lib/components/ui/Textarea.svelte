<script lang="ts">
  // Labelled native control; bind value and optionally pass a leading lucide icon.
  import type { LucideIcon } from '@lucide/svelte';
  import type { HTMLTextareaAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  const uid = $props.id();
  let { icon, label, hint, error, id = uid, value = $bindable(''), class: className, ...rest }:
    HTMLTextareaAttributes & { icon?: LucideIcon; label: string; hint?: string; error?: string } = $props();
</script>

<div class="grid gap-1.5">
  <label for={id} class="text-sm font-medium text-fg">{label}{#if rest.required}<span class="text-danger" aria-hidden="true"> *</span>{/if}</label>
  <div class="relative">
    {#if icon}{@const Icon = icon}<span class="pointer-events-none absolute left-3 top-3 text-fg-muted" aria-hidden="true"><Icon size={16} /></span>{/if}
  <textarea {...rest} {id} bind:value rows={Math.max(3, rest.rows ?? 3)} aria-invalid={error ? true : rest['aria-invalid']}
    aria-describedby={[rest['aria-describedby'], hint && id + '-hint', error && id + '-error'].filter(Boolean).join(' ') || undefined}
    class={cn('min-h-10 text-sm w-full resize-y rounded-md border border-border-control bg-surface px-3 py-2 text-fg transition-colors hover:bg-surface-muted disabled:bg-surface-muted disabled:text-fg-subtle', icon && 'pl-10', error && 'border-danger', className)}></textarea>
  </div>
  {#if hint}<p id={id + '-hint'} class="type-caption text-fg-muted">{hint}</p>{/if}
  {#if error}<p id={id + '-error'} class="type-caption text-danger">{error}</p>{/if}
</div>
