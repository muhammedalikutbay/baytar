<script lang="ts">
  import type { HTMLTextareaAttributes } from 'svelte/elements';
  import { cn } from '$lib/utils';
  const uid = $props.id();
  let { label, hint, error, id = uid, value = $bindable(''), class: className, ...rest }:
    HTMLTextareaAttributes & { label: string; hint?: string; error?: string } = $props();
</script>

<div class="grid gap-1.5">
  <label for={id} class="type-caption text-fg">{label}</label>
  <textarea {...rest} {id} bind:value rows={rest.rows ?? 3} aria-invalid={error ? true : rest['aria-invalid']}
    aria-describedby={[rest['aria-describedby'], hint && id + '-hint', error && id + '-error'].filter(Boolean).join(' ') || undefined}
    class={cn('min-h-24 w-full resize-y rounded-md border border-border-control bg-surface px-3 py-2.5 text-fg transition-colors hover:bg-surface-muted disabled:bg-surface-muted disabled:text-fg-subtle', error && 'border-danger', className)}></textarea>
  {#if hint}<p id={id + '-hint'} class="type-caption text-fg-muted">{hint}</p>{/if}
  {#if error}<p id={id + '-error'} class="type-caption text-danger">{error}</p>{/if}
</div>
