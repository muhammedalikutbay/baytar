<script lang="ts">
  import { Checkbox as Primitive } from 'bits-ui';
  import { Check, Minus } from '@lucide/svelte';
  const uid = $props.id();
  let { label, checked = $bindable(false), indeterminate = $bindable(false), disabled = false, id = uid, ...rest }:
    Omit<Primitive.RootProps, 'children' | 'child'> & { label: string } = $props();
</script>

<div class="flex min-h-11 items-center gap-2.5">
  <Primitive.Root {...rest} {id} {disabled} bind:checked bind:indeterminate
    class="inline-flex size-6 shrink-0 items-center justify-center rounded-[5px] border border-border-control bg-surface text-ink disabled:cursor-not-allowed disabled:bg-surface-muted disabled:text-fg-subtle">
    {#if indeterminate}<Minus size={16} strokeWidth={2} aria-hidden="true" />
    {:else if checked}<Check size={16} strokeWidth={2} aria-hidden="true" />{/if}
  </Primitive.Root>
  <label for={id} class={disabled ? 'flex min-h-11 items-center text-fg-subtle' : 'flex min-h-11 cursor-pointer items-center text-fg'}>{label}</label>
</div>
