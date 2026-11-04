<script lang="ts">
  import '../app.css';
  import { onMount, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import { PawPrint, ShieldCheck } from '@lucide/svelte';
  import { tr } from '$lib/i18n/tr';
  import { theme } from '$lib/theme.svelte';
  import { navigation, settingsNavigation } from '$lib/navigation';
  import ThemeSwitch from '$lib/components/ThemeSwitch.svelte';
  import GlobalSearch from '$lib/components/patients/GlobalSearch.svelte';

  let { children }: { children: Snippet } = $props();
  let version = $state<string | null>(null);
  const extraTitles: Record<string, string> = { '/import': tr.importTitle, '/design': tr.galleryTitle };
  const pathname = $derived(page.url.pathname.replace(/\/$/, '') || '/');
  const title = $derived(page.status >= 400 ? (page.status === 404 ? tr.notFoundTitle : tr.errorTitle)
    : extraTitles[pathname] ?? [...navigation, settingsNavigation].find(item => item.href !== '/'
        && (pathname === item.href || pathname.startsWith(`${item.href}/`)))?.title
      ?? (pathname === '/' ? tr.home : tr.appName));

  onMount(() => {
    const disposeTheme = theme.init();
    if ('__TAURI_INTERNALS__' in window) {
      import('@tauri-apps/api/core')
        .then(({ invoke }) => invoke<string>('app_version'))
        .then(value => version = value)
        .catch((error: unknown) => console.error('Failed to fetch application version', error));
    }
    return disposeTheme;
  });
</script>

<svelte:head>
  <title>{title} · {tr.appName}</title>
  <meta name="description" content={tr.tagline} />
</svelte:head>

<a href="#main-content" class="skip-link">{tr.skipToContent}</a>
<div class="app-shell">
  <aside class="sidebar">
    <a href="/" class="wordmark" aria-label={tr.appName} title={tr.appName}>
      <PawPrint size={22} strokeWidth={1.75} class="text-accent" aria-hidden="true" />
      <span class="sidebar-label text-xl font-semibold tracking-tight">{tr.appName}</span>
    </a>
    <p class="sidebar-label type-caption px-5 pb-5 text-fg-subtle">{tr.workspace}</p>
    <nav aria-label={tr.navigation} class="grid gap-1 px-3">
      {#each navigation as item (item.href)}
        <a href={item.href} class:active={pathname === item.href} class="nav-item"
          aria-current={pathname === item.href ? 'page' : undefined} aria-label={item.title} title={item.title}>
          <item.icon size={19} strokeWidth={1.75} aria-hidden="true" />
          <span class="sidebar-label">{item.title}</span>
        </a>
      {/each}
    </nav>
    <div class="sidebar-footer">
      <a href={settingsNavigation.href} class:active={pathname === settingsNavigation.href} class="nav-item"
        aria-current={pathname === settingsNavigation.href ? 'page' : undefined} aria-label={tr.settings} title={tr.settings}>
        <settingsNavigation.icon size={19} strokeWidth={1.75} aria-hidden="true" />
        <span class="sidebar-label">{tr.settings}</span>
      </a>
      <div class="theme-area">
        <p class="sidebar-label type-caption mb-2 text-fg-subtle">{tr.theme}</p>
        <ThemeSwitch compact />
      </div>
      <div class="sidebar-label border-t border-border px-2 pt-4">
        <p class="type-caption flex items-center gap-2 text-fg-muted"><ShieldCheck size={14} aria-hidden="true" />{tr.localData}</p>
        {#if version}<p class="type-caption mt-2 text-fg-subtle">{tr.version} {version}</p>{/if}
      </div>
    </div>
  </aside>

  <div class="min-w-0">
    <header class="topbar">
      <div class="breadcrumb"><span class="text-fg-subtle">{tr.appName}</span><span class="text-border-control" aria-hidden="true">/</span><span class="font-medium">{title}</span></div>
      <GlobalSearch />
    </header>
    <main id="main-content" tabindex="-1" class="content">
      {@render children()}
    </main>
  </div>
</div>


<style>
  .app-shell { display: grid; grid-template-columns: 232px minmax(0, 1fr); min-height: 100dvh; }
  .sidebar { position: sticky; top: 0; display: flex; height: 100dvh; min-height: 560px; flex-direction: column; overflow-y: auto; border-right: 1px solid var(--color-border); background: var(--color-surface); }
  .wordmark { display: flex; min-height: 80px; align-items: center; gap: 10px; padding: 0 24px; }
  .nav-item { position: relative; display: flex; min-height: 44px; align-items: center; gap: 12px; border-radius: var(--radius-sm); padding: 0 12px; color: var(--color-fg-muted); transition: background var(--default-transition-duration); }
  .nav-item:hover { background: var(--color-surface-muted); color: var(--color-fg); }
  .nav-item.active { background: var(--color-surface-muted); color: var(--color-fg); font-weight: 500; }
  .nav-item.active::before { position: absolute; inset: 13px auto 13px 0; width: 2px; border-radius: var(--radius-pill); background: var(--color-accent); content: ''; }
  .sidebar-footer { display: grid; gap: 16px; margin-top: auto; padding: 32px 12px 20px; }
  .theme-area { padding: 0 8px; }
  .topbar { display: flex; min-height: 88px; align-items: center; justify-content: space-between; gap: 24px; border-bottom: 1px solid var(--color-border); background: var(--color-surface); padding: 16px 32px; }
  .breadcrumb { display: flex; align-items: center; gap: 12px; white-space: nowrap; font-size: 13px; }
  .content { width: 100%; max-width: 1264px; margin: 0 auto; padding: 36px 32px; }
  .skip-link { position: fixed; z-index: 60; top: 12px; left: 12px; transform: translateY(-200%); border-radius: var(--radius-sm); background: var(--color-ink); padding: 12px 16px; color: var(--color-ink-fg); }
  .skip-link:focus { transform: none; }
  @media (max-width: 1099px) {
    .app-shell { grid-template-columns: 76px minmax(0, 1fr); }
    .sidebar-label { display: none; }
    .wordmark { justify-content: center; padding: 0; }
    .nav-item { justify-content: center; padding: 0; }
    .sidebar-footer { justify-items: center; padding-top: 24px; }
    .sidebar-footer > .nav-item { width: 100%; }
    .theme-area { padding: 0; }
    .content { padding: 28px 24px; }
    .topbar { gap: 16px; padding: 16px 24px; }
  }
  @media (max-width: 639px) {
    .topbar { align-items: stretch; flex-direction: column; gap: 12px; padding: 16px; }
    .content { padding: 24px 16px; }
  }
</style>
