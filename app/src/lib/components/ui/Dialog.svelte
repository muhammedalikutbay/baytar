<script lang="ts">
  import { Dialog as Primitive } from 'bits-ui';
  import { X } from '@lucide/svelte';
  import type { Snippet } from 'svelte';
  import type { ResolvedTheme } from '$lib/theme.svelte';
  import { tr } from '$lib/i18n/tr';
  import { buttonStyles } from './button';
  let { open = $bindable(false), title, description, children, footer, theme, wide = false }:
    { open?: boolean; title: string; description: string; children?: Snippet; footer?: Snippet; theme?: ResolvedTheme; wide?: boolean } = $props();
</script>

<Primitive.Root bind:open>
  <Primitive.Portal>
    <Primitive.Overlay class="fixed inset-0 z-40 bg-overlay" />
    <Primitive.Content data-theme={theme}
      style={wide ? 'max-width: 960px' : 'max-width: 512px'}
      class="fixed top-1/2 left-1/2 z-50 max-h-[85dvh] w-[calc(100%_-_2rem)] -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-xl border border-border bg-surface p-6 text-fg shadow-float sm:p-8">
      <Primitive.Title class="type-title pr-10">{title}</Primitive.Title>
      <Primitive.Description class="mt-3 text-fg-muted">{description}</Primitive.Description>
      <Primitive.Close aria-label={tr.close} title={tr.close}
        class="absolute top-4 right-4 flex size-11 items-center justify-center rounded-sm text-fg-muted hover:bg-surface-muted">
        <X size={18} aria-hidden="true" />
      </Primitive.Close>
      {#if children}<div class="mt-6">{@render children()}</div>{/if}
      <div class="mt-8 flex flex-wrap justify-end gap-2">
        {#if footer}{@render footer()}
        {:else}<Primitive.Close class={buttonStyles('primary')}>{tr.understood}</Primitive.Close>{/if}
      </div>
    </Primitive.Content>
  </Primitive.Portal>
</Primitive.Root>
