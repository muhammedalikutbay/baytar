<script lang="ts">
  // bits-ui provides roving focus, arrow keys, Escape, typeahead and focus restoration.
  import { DropdownMenu } from 'bits-ui';
  import { Ellipsis } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import { buttonStyles } from './button';
  let { items, label = tr.flowMoreActions, text = false }: {
    items: { label: string; onselect?: () => void; disabled?: boolean; danger?: boolean }[];
    label?: string; text?: boolean;
  } = $props();
</script>
<DropdownMenu.Root>
  <DropdownMenu.Trigger class={buttonStyles('secondary', 'sm')} aria-label={label} title={label}>
    {#if text}{label}{:else}<Ellipsis size={18} aria-hidden="true" />{/if}
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content sideOffset={6} align="end" class="z-[70] max-h-80 max-w-sm overflow-auto rounded-md border border-border-control bg-surface p-1 text-fg shadow-float">
      {#each items as item, i (i)}
        <DropdownMenu.Item disabled={item.disabled} onclick={() => item.onselect?.()} class={'min-h-10 cursor-default rounded-sm px-3 py-2 text-sm outline-none data-highlighted:bg-surface-muted data-disabled:text-fg-subtle ' + (item.danger ? 'text-danger' : '')}>{item.label}</DropdownMenu.Item>
      {/each}
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
