<script lang="ts">
  import { onMount } from 'svelte';
  import { Minus, Square, Copy, X } from '@lucide/svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { branding } from '$lib/branding.svelte';
  import { tr } from '$lib/i18n/tr';
  import Logo from '$lib/components/brand/Logo.svelte';
  import GlobalSearch from '$lib/components/patients/GlobalSearch.svelte';

  let { title }: { title: string } = $props();
  let maximized = $state(false);
  let mac = $state(false);
  let bar: HTMLElement;
  const nativeWindow = () => '__TAURI_INTERNALS__' in window ? getCurrentWindow() : null;
  async function act(action: 'minimize' | 'toggleMaximize' | 'close') {
    try { await nativeWindow()?.[action](); }
    catch (error) { console.error('Window action failed', action, error); }
  }
  onMount(() => {
    mac = navigator.userAgent.includes('Mac');
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const current = nativeWindow();
    async function syncMaximized() {
      try { const value = await current?.isMaximized(); if (!disposed) maximized = value ?? false; }
      catch (error) { console.error('Failed to read window state', error); }
    }
    if (current) {
      void syncMaximized();
      void current.onResized(() => void syncMaximized()).then(dispose => {
        if (disposed) dispose(); else unlisten = dispose;
      }).catch(error => console.error('Failed to watch window state', error));
    }
    // Intercept the second press so Tauri's drag-region handler cannot also toggle maximization.
    const onMouseDown = (event: MouseEvent) => {
      if (event.button === 0 && event.detail === 2 && (event.target as HTMLElement).closest('[data-tauri-drag-region]')
        && !(event.target as HTMLElement).closest('button, input, a')) event.stopPropagation();
    };
    const onDoubleClick = (event: MouseEvent) => {
      if (!(event.target as HTMLElement).closest('button, input, a, form')) void act('toggleMaximize');
    };
    bar.addEventListener('mousedown', onMouseDown);
    bar.addEventListener('mouseup', onMouseDown);
    bar.addEventListener('dblclick', onDoubleClick);
    return () => {
      disposed = true; unlisten?.();
      bar.removeEventListener('mousedown', onMouseDown);
      bar.removeEventListener('mouseup', onMouseDown);
      bar.removeEventListener('dblclick', onDoubleClick);
    };
  });
</script>

<header bind:this={bar} class:mac class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <span class="brand-logo" data-tauri-drag-region><Logo size={24} /></span>
    <span class="brand-name truncate" title={branding.name || tr.appName} data-tauri-drag-region>{branding.name || tr.appName}</span>
  </div>
  <div class="breadcrumb" data-tauri-drag-region>
    <span class="text-fg-subtle" data-tauri-drag-region>{tr.workspace}</span>
    <span class="text-fg-subtle" data-tauri-drag-region aria-hidden="true">/</span>
    <span class="truncate font-medium" data-tauri-drag-region>{title}</span>
  </div>
  <div class="window-search"><GlobalSearch /></div>
  {#if !mac}
    <div class="window-controls" role="group" aria-label={tr.windowControls}>
      <button type="button" aria-label={tr.windowMinimize} title={tr.windowMinimize} onclick={() => void act('minimize')}><Minus size={16} strokeWidth={1.5} aria-hidden="true" /></button>
      <button type="button" aria-label={maximized ? tr.windowRestore : tr.windowMaximize} title={maximized ? tr.windowRestore : tr.windowMaximize} onclick={() => void act('toggleMaximize')}>
        {#if maximized}<Copy size={14} strokeWidth={1.5} aria-hidden="true" />{:else}<Square size={14} strokeWidth={1.5} aria-hidden="true" />{/if}
      </button>
      <button type="button" class="close" aria-label={tr.windowClose} title={tr.windowClose} onclick={() => void act('close')}><X size={18} strokeWidth={1.5} aria-hidden="true" /></button>
    </div>
  {/if}
</header>

<style>
  .titlebar { position: sticky; top: 0; z-index: 40; display: flex; height: 40px; align-items: center; border-bottom: 1px solid var(--color-border); background: var(--color-surface); user-select: none; }
  .brand { display: flex; width: 232px; flex-shrink: 0; align-items: center; gap: 9px; padding-left: 22px; }
  .brand-name { font-size: 14px; font-weight: 700; letter-spacing: -0.03em; }
  .brand-logo { display: flex; }
  .brand-logo :global(svg) { pointer-events: none; }
  .breadcrumb { display: flex; width: calc(50% - 432px); min-width: 120px; align-items: center; gap: 10px; padding-left: 20px; font-size: 12px; }
  .breadcrumb > :first-child, .breadcrumb > :nth-child(2) { display: none; }
  .window-search { position: absolute; left: 50%; width: 32%; max-width: 360px; transform: translateX(-50%); }
  .window-search :global(input) { height: 28px; min-height: 28px; border-color: var(--color-border); border-radius: var(--radius-sm); background: var(--color-surface-muted); font-size: 12px; }
  .window-search :global(form > div.grid) { gap: 0; }
  .window-search :global(kbd) { padding-top: 0; padding-bottom: 0; font-size: 10px; }
  .window-controls { display: flex; align-self: stretch; margin-left: auto; }
  .window-controls button { display: flex; width: 46px; align-items: center; justify-content: center; color: var(--color-fg-muted); }
  .window-controls button:hover { background: var(--color-surface-muted); color: var(--color-fg); }
  .window-controls .close:hover { background: var(--color-window-close); color: var(--color-window-close-fg); }
  .window-controls button:focus-visible { outline-offset: -4px; }
  .mac { padding-left: 78px; }
  .mac .brand { width: 154px; padding-left: 12px; }
  @media (max-width: 1099px) {
    .brand { width: 76px; justify-content: center; padding-left: 0; }
    .brand-name { display: none; }
    .breadcrumb { width: calc(50% - 276px); }
    .mac .brand { width: 76px; padding-left: 0; }
    .mac .breadcrumb { width: calc(50% - 354px); }
  }
  @media (max-width: 639px) { .breadcrumb { display: none; } .window-search { left: 40%; width: 34%; } }
</style>
