<script lang="ts">
  import '../app.css';
  import { onMount, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import { ShieldCheck } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import { theme } from '$lib/theme.svelte';
  import { updater } from '$lib/updater.svelte';
  import { buttonStyles } from '$lib/components/ui/button';
  import { navigation, settingsNavigation } from '$lib/navigation';
  import ThemeSwitch from '$lib/components/ThemeSwitch.svelte';
  import TitleBar from '$lib/components/layout/TitleBar.svelte';
  import { lock } from '$lib/lock.svelte';
  import LockScreen from '$lib/components/lock/LockScreen.svelte';

  let { children }: { children: Snippet } = $props();
  let version = $state<string | null>(null);
  const extraTitles: Record<string, string> = { '/import': tr.importTitle, '/design': tr.galleryTitle };
  const pathname = $derived(page.url.pathname.replace(/\/$/, '') || '/');
  const title = $derived(page.status >= 400 ? (page.status === 404 ? tr.notFoundTitle : tr.errorTitle)
    : extraTitles[pathname] ?? [...navigation, settingsNavigation].find(item => item.href !== '/'
        && (pathname === item.href || pathname.startsWith(`${item.href}/`)))?.title
      ?? (pathname === '/' ? tr.home : tr.appName));

  onMount(() => {
    if (pathname === '/print') return;
    const disposeTheme = theme.init();
    const disposeUpdater = updater.init();
    void lock.init();
    const disposeLock = lock.watch();
    if ('__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core')
        .then(({ invoke }) => invoke<string>('app_version'))
        .then(value => version = value)
        .catch((error: unknown) => console.error('Failed to fetch application version', error));
    }
    return () => { disposeTheme(); disposeUpdater(); disposeLock(); };
  });
</script>

<svelte:head>
  <title>{title} · {tr.appName}</title>
  <meta name="description" content={tr.tagline} />
  <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
  <link rel="alternate icon" href="/favicon.ico" />
</svelte:head>

{#if pathname === '/print'}
  {@render children()}
{:else}
<a href="#main-content" class="skip-link">{tr.skipToContent}</a>
<TitleBar {title} />
{#if lock.locked}
  <LockScreen />
{:else if lock.ready}
<div class="app-shell">
  <aside class="sidebar">
    <p class="sidebar-label type-caption px-5 pt-7 pb-3 text-fg-subtle">{tr.navWorkspace}</p>
    <nav aria-label={tr.navigation} class="grid gap-1 px-3">
      {#each navigation as item, index (item.href)}
        {#if index === 1}<p class="sidebar-label type-caption px-2 pt-5 pb-2 text-fg-subtle">{tr.navClinical}</p>{/if}
        <a href={item.href} class:active={(pathname === item.href || (item.href !== '/' && pathname.startsWith(item.href + '/')))} class="nav-item"
          aria-current={(pathname === item.href || (item.href !== '/' && pathname.startsWith(item.href + '/'))) ? 'page' : undefined} aria-label={item.title} title={item.title}>
          <item.icon size={21} strokeWidth={1.75} aria-hidden="true" />
          <span class="sidebar-label">{item.title}</span>
        </a>
      {/each}
    </nav>
    <div class="sidebar-footer">
      <a href={settingsNavigation.href} class:active={pathname === settingsNavigation.href} class="nav-item"
        aria-current={pathname === settingsNavigation.href ? 'page' : undefined} aria-label={tr.settings} title={tr.settings}>
        <settingsNavigation.icon size={21} strokeWidth={1.75} aria-hidden="true" />
        <span class="sidebar-label">{tr.settings}</span>
      </a>
      <div class="sidebar-label border-t border-border px-2 pt-4">
        <p class="type-caption flex items-center gap-2 text-fg-muted"><ShieldCheck size={14} aria-hidden="true" />{tr.localData}</p>
      </div>
      <div class="theme-area">
        <span class="sidebar-label type-caption tabular text-fg-subtle">{tr.version} {version ?? '0.2.0'}</span>
        <ThemeSwitch />
      </div>
    </div>
  </aside>

  <div class="content-canvas">
    {#if updater.banner}
      <section class="border-b border-border bg-surface px-4 py-3 sm:px-8" aria-label={tr.settingsUpdates}>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <p class="font-medium">{tr.updateReady.replace('{version}', updater.version)}</p>
          <div class="flex flex-wrap gap-2">
            <button class={buttonStyles('secondary', 'sm')} disabled={updater.busy || updater.checking} onclick={() => void updater.install()}>{updater.actionLabel}</button>
            <button class={buttonStyles('ghost', 'sm')} disabled={updater.busy} onclick={() => updater.dismiss()}>{tr.updateLater}</button>
          </div>
        </div>
        {#if updater.notes}<p class="mt-2 max-w-prose whitespace-pre-wrap break-words text-sm text-fg-muted">{updater.notes}</p>{/if}
        {#if updater.progress}<p role="status" class="mt-2 text-sm text-fg-muted">{updater.progress}</p>{/if}
        {#if updater.error}<p role="alert" class="mt-2 text-sm text-danger">{updater.error}</p>{/if}
      </section>
    {/if}
    <main id="main-content" tabindex="-1" class="content">
      {@render children()}
    </main>
  </div>
</div>
{/if}

{/if}

<style>
  .app-shell { display: grid; grid-template-columns: 232px minmax(0, 1fr); min-height: calc(100dvh - 40px); }
  .sidebar { position: sticky; top: 40px; display: flex; height: calc(100dvh - 40px); min-height: 560px; flex-direction: column; overflow-y: auto; border-right: 1px solid var(--color-border); background: var(--color-surface); }
  .nav-item { position: relative; display: flex; min-height: 44px; align-items: center; gap: 12px; border-radius: var(--radius-sm); padding: 0 12px; color: var(--color-fg-muted); transition: background var(--default-transition-duration); }
  .nav-item:hover { background: var(--color-surface-muted); color: var(--color-fg); }
  .nav-item.active { background: var(--color-accent-soft); color: var(--color-accent); font-weight: 600; }
  .nav-item.active::before { position: absolute; inset: 10px auto 10px 0; width: 3px; border-radius: var(--radius-pill); background: var(--color-accent); content: ''; }
  .sidebar-footer { display: grid; gap: 16px; margin-top: auto; padding: 32px 12px 20px; }
  .theme-area { display: flex; align-items: center; justify-content: space-between; gap: 8px; padding: 0 8px; }
  .content { width: 100%; max-width: 1264px; margin: 0 auto; padding: 36px 32px; }
  .skip-link { position: fixed; z-index: 60; top: 12px; left: 12px; transform: translateY(-200%); border-radius: var(--radius-sm); background: var(--color-ink); padding: 12px 16px; color: var(--color-ink-fg); }
  .skip-link:focus { transform: none; }
  @media (max-width: 1099px) {
    .app-shell { grid-template-columns: 76px minmax(0, 1fr); }
    .sidebar-label { display: none; }
    .nav-item { justify-content: center; padding: 0; }
    .sidebar-footer { justify-items: center; padding-top: 24px; }
    .sidebar-footer > .nav-item { width: 100%; }
    .theme-area { padding: 0; }
    .content { padding: 28px 24px; }
  }
  @media (max-width: 639px) {
    .content { padding: 24px 16px; }
  }
</style>
