<script lang="ts">
  import { onMount } from 'svelte';
  import { ExternalLink, Settings2, Moon, Stethoscope, ShieldCheck, Import, RefreshCw, Info } from '@lucide/svelte';
  import { PageHeader, IconChip } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { theme } from '$lib/theme.svelte';
  import { updater } from '$lib/updater.svelte';
  import * as api from '$lib/api/settings';
  import type { AppInfo, BackupInfo } from '$lib/api/types';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { buttonStyles } from '$lib/components/ui/button';

  let clinic = $state({ clinic_name: '', vet_name: '', clinic_phone: '', clinic_address: '' });
  let backups = $state<BackupInfo[]>([]);
  let info = $state<AppInfo | null>(null);
  let updateEnabled = $state(true);
  let busy = $state(false);
  let toast = $state('');
  let error = $state('');
  let restoreTarget = $state<BackupInfo | null>(null);
  let restoreDialogOpen = $state(false);
  onMount(() => { void load(); });
  async function load() {
    try {
      const [values, rows, appInfo] = await Promise.all([api.getAll(), api.backups(), api.info()]);
      clinic = { clinic_name: values.clinic_name ?? '', vet_name: values.vet_name ?? '', clinic_phone: values.clinic_phone ?? '', clinic_address: values.clinic_address ?? '' };
      updateEnabled = (values.update_check_enabled ?? '1') !== '0'; backups = rows; info = appInfo;
    } catch { error = tr.settingsError; }
  }
  async function saveClinic() {
    try { await Promise.all(Object.entries(clinic).map(([key, value]) => api.set(key, value))); toast = tr.settingsSaved; error = ''; }
    catch { error = tr.settingsError; }
  }
  async function backupNow() { busy = true; try { await api.createBackup(); backups = await api.backups(); toast = tr.settingsSaved; error = ''; } catch { error = tr.settingsBackupError; } finally { busy = false; } }
  async function exportBackup() { try { if (await api.exportBackup()) { toast = tr.settingsBackupExported; error = ''; } } catch { error = tr.settingsBackupError; } }
  async function restore() { if (!restoreTarget) return; busy = true; try { await api.restoreBackup(restoreTarget.path); restoreTarget = null; restoreDialogOpen = false; window.location.reload(); } catch { error = tr.settingsRestoreError; } finally { busy = false; } }
  async function setUpdates(event: Event) { updateEnabled = (event.currentTarget as HTMLInputElement).checked; try { await api.set('update_check_enabled', updateEnabled ? '1' : '0'); } catch { error = tr.settingsError; } }
  const kinds: Record<BackupInfo['kind'], string> = { daily: tr.settingsDaily, manual: tr.settingsManual, 'pre-update': tr.settingsPreUpdate, 'pre-import': tr.settingsPreImport, 'pre-restore': tr.settingsPreRestore, 'pre-migration': tr.settingsPreMigration };
  const date = (value: string) => new Intl.DateTimeFormat('tr-TR', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value));
  const size = (bytes: number) => bytes < 1048576 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / 1048576).toFixed(1)} MB`;
</script>

<svelte:head><title>{tr.settings} · {tr.appName}</title></svelte:head>
<main class="mx-auto max-w-5xl space-y-8 px-4 py-8 sm:px-8">
  <PageHeader title={tr.settings} subtitle={tr.settingsDescription} icon={Settings2}>
    {#snippet actions()}<button class={buttonStyles('primary')} disabled={busy} onclick={() => void backupNow()}>{tr.settingsBackupNow}</button>{/snippet}
  </PageHeader>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#if toast}<p role="status" class="text-success">{toast}</p>{/if}
  <section class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-surface p-5 sm:p-6">
    <div class="flex items-center gap-3"><IconChip icon={Moon} tone="neutral" size="sm" /><div><h2 class="type-section">{tr.theme}</h2><p class="text-sm text-fg-muted">{theme.preference === 'system' ? tr.themeFollowSystem : tr.themeManual}</p></div></div>
    <button class="type-caption min-h-7 text-accent underline underline-offset-4" onclick={() => theme.set('system')}>{tr.useSystemTheme}</button>
  </section>
  <section class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={Stethoscope} tone="accent" size="sm" />{tr.settingsClinic}</h2>
    <form class="mt-4 grid gap-4 sm:grid-cols-2" onsubmit={(e) => { e.preventDefault(); void saveClinic(); }}>
      {#each [['clinic_name', tr.settingsClinicName], ['vet_name', tr.settingsVetName], ['clinic_phone', tr.settingsPhone], ['clinic_address', tr.settingsAddress]] as [key, label]}
        <label class="grid gap-1 text-sm">{label}<input class="min-h-11 rounded-md border border-border-control bg-surface px-3 text-fg" bind:value={clinic[key as keyof typeof clinic]} /></label>
      {/each}
      <div class="sm:col-span-2"><button class={buttonStyles('primary')} type="submit">{tr.save}</button></div>
    </form>
  </section>
  <section class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={ShieldCheck} tone="accent" size="sm" />{tr.settingsBackups}</h2>
    <p class="mt-2 text-fg-muted">{tr.settingsBackupPolicy}</p>
    <div class="mt-4 flex flex-wrap gap-2"><button class={buttonStyles('primary')} disabled={busy} onclick={() => void backupNow()}>{tr.settingsBackupNow}</button><button class={buttonStyles('secondary')} onclick={() => void exportBackup()}>{tr.settingsExport}</button></div>
    <ul class="mt-4 divide-y divide-border">{#each backups as backup (backup.path)}<li class="flex flex-wrap items-center justify-between gap-3 py-3"><div><span class="tabular-nums">{date(backup.createdAt)}</span><span class="mx-2 text-fg-muted">· {kinds[backup.kind]}</span><span class="text-fg-muted">{size(backup.sizeBytes)}</span></div><button class={buttonStyles('secondary')} onclick={() => { restoreTarget = backup; restoreDialogOpen = true; }}>{tr.settingsRestore}</button></li>{:else}<li class="py-3 text-fg-muted">—</li>{/each}</ul>
  </section>
  <section class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={Import} tone="neutral" size="sm" />{tr.settingsLegacy}</h2><a class="mt-3 inline-block text-accent underline" href="/import">{tr.importTitle}</a></section>
  <section class="rounded-lg border border-border bg-surface p-5 sm:p-6">
    <h2 class="type-section flex items-center gap-3"><IconChip icon={RefreshCw} tone="neutral" size="sm" />{tr.settingsUpdates}</h2>
    <label class="mt-3 flex min-h-11 items-center gap-3"><input type="checkbox" class="size-5 accent-ink" checked={updateEnabled} onchange={setUpdates} />{tr.settingsUpdateCheck}</label>
    <div class="mt-3 flex flex-wrap gap-2">
      <button class={buttonStyles('secondary')} disabled={updater.checking || updater.busy} onclick={() => void updater.check({ silent: false })}>{tr.updateCheckNow}</button>
      {#if updater.version}<button class={buttonStyles('primary')} disabled={updater.checking || updater.busy} onclick={() => void updater.install()}>{updater.actionLabel}</button>{/if}
    </div>
    <div class="mt-3 text-sm" role="status">
      {#if updater.checking}<p class="text-fg-muted">{tr.updateChecking}</p>
      {:else if updater.result === 'current'}<p>{tr.updateCurrent}</p>
      {:else if updater.result === 'available'}<p>{tr.updateAvailable.replace('{version}', updater.version)}</p>
      {:else if updater.result === 'error'}<p class="text-danger">{tr.updateCheckError}</p>{/if}
      {#if updater.notes}<p class="mt-2 max-w-prose whitespace-pre-wrap break-words text-fg-muted">{updater.notes}</p>{/if}
      {#if updater.progress}<p class="mt-2 text-fg-muted">{updater.progress}</p>{/if}
    </div>
    {#if updater.error}<p role="alert" class="mt-2 text-sm text-danger">{updater.error}</p>{/if}
  </section>
  <section class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={Info} tone="neutral" size="sm" />{tr.settingsAbout}</h2><p class="mt-3">{tr.version}: {info?.version ?? '—'}</p><p class="mt-1 text-fg-muted">{tr.settingsLicense}</p><a class="mt-2 inline-flex items-center gap-2 text-accent underline" href="https://github.com/muhammedalikutbay/baytar" target="_blank" rel="noreferrer"><ExternalLink size={16} />GitHub</a><p class="mt-4">{tr.localData}</p><p class="break-all text-sm text-fg-muted">{info?.dataFolder ?? '—'}</p><button class={`${buttonStyles('secondary')} mt-3`} onclick={() => void api.openDataFolder()}>{tr.settingsOpenFolder}</button></section>
</main>

<Dialog bind:open={restoreDialogOpen} title={tr.settingsRestoreTitle} description={tr.settingsRestoreDescription}>
  {#snippet footer()}<button class={buttonStyles('secondary')} onclick={() => { restoreTarget = null; restoreDialogOpen = false; }}>{tr.cancel}</button><button class={buttonStyles('primary')} disabled={busy} onclick={() => void restore()}>{tr.settingsRestore}</button>{/snippet}
</Dialog>
