<script lang="ts">
  // Labelled native control; bind value and optionally pass a leading lucide icon.
  import type { LucideIcon } from '@lucide/svelte';
  import type { HTMLSelectAttributes } from 'svelte/elements';
  import { ChevronDown } from '@lucide/svelte';
  import { cn } from '$lib/utils';
  const uid = $props.id();
  let { icon, label, options, placeholder, error, id = uid, value = $bindable(''), class: className, ...rest }:
    Omit<HTMLSelectAttributes, 'value' | 'multiple' | 'children'> & {
      icon?: LucideIcon; label: string; options: { value: string; label: string }[]; placeholder?: string; error?: string; value?: string;
    } = $props();
</script>

<div class="grid gap-1.5">
  <label for={id} class="text-sm font-medium text-fg">{label}{#if rest.required}<span class="text-danger" aria-hidden="true"> *</span>{/if}</label>
  <div class="relative">
    {#if icon}{@const Icon = icon}<span class="pointer-events-none absolute left-3 top-3 text-fg-muted" aria-hidden="true"><Icon size={16} /></span>{/if}
    <select {...rest} {id} bind:value aria-invalid={error ? true : rest['aria-invalid']}
      aria-describedby={[rest['aria-describedby'], error && id + '-error'].filter(Boolean).join(' ') || undefined}
      class={cn('min-h-10 text-sm w-full appearance-none rounded-md border border-border-control bg-surface py-2 pr-10 pl-3 text-fg transition-colors hover:bg-surface-muted disabled:bg-surface-muted disabled:text-fg-subtle', icon && 'pl-10', error && 'border-danger', className)}>
      {#if placeholder}<option value="" disabled>{placeholder}</option>{/if}
      {#each options as option (option.value)}<option value={option.value}>{option.label}</option>{/each}
    </select>
    <ChevronDown size={16} class="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2 text-fg-muted" aria-hidden="true" />
  </div>
  {#if error}<p id={id + '-error'} class="type-caption text-danger">{error}</p>{/if}
</div>
