<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  const uid = $props.id();
  let { label, hint, error, labelClass, id = uid, value = $bindable(''), ref = $bindable(null), class: className, ...rest }:
    Omit<HTMLInputAttributes, 'value'> & { label: string; hint?: string; error?: string; labelClass?: string; value?: string; ref?: HTMLInputElement | null } = $props();
</script>

<div class="grid gap-1.5">
  <label for={id} class={cn('type-caption text-fg', labelClass)}>{label}</label>
  <input {...rest} {id} bind:this={ref} bind:value aria-invalid={error ? true : rest['aria-invalid']}
    aria-describedby={[rest['aria-describedby'], hint && id + '-hint', error && id + '-error'].filter(Boolean).join(' ') || undefined}
    class={cn('min-h-11 w-full min-w-0 rounded-md border border-border-control bg-surface px-3 text-fg transition-colors hover:bg-surface-muted disabled:bg-surface-muted disabled:text-fg-subtle', error && 'border-danger', className)} />
  {#if hint}<p id={id + '-hint'} class="type-caption text-fg-muted">{hint}</p>{/if}
  {#if error}<p id={id + '-error'} class="type-caption text-danger">{error}</p>{/if}
</div>
