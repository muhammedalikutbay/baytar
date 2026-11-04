<script lang="ts">
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { User, Building2, IdCard, MapPin, ExternalLink, Settings2, Moon, Stethoscope, ShieldCheck, Import, RefreshCw, Info, Upload, MessageCircle } from '@lucide/svelte';
  import { PageHeader, IconChip, Card, EmailInput, Input, Textarea, Select } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { applyBranding } from '$lib/branding.svelte';
  import { whatsappPhone } from '$lib/format';
  import { sendWhatsapp, parseWhatsappMode, whatsappModes, whatsappModeLabel, type WhatsappMode } from '$lib/whatsapp';
  import { theme } from '$lib/theme.svelte';
  import { updater } from '$lib/updater.svelte';
  import * as api from '$lib/api/settings';
  import type { AppInfo, BackupInfo } from '$lib/api/types';
  import { PhoneInput } from '$lib/components/ui';
  import LockCard from '$lib/components/lock/LockCard.svelte';
  import VisitTemplatesCard from '$lib/components/visits/VisitTemplatesCard.svelte';
  import ReminderTemplatesCard from '$lib/components/reminders/ReminderTemplatesCard.svelte';
  import { validateForm } from '$lib/validation';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import { buttonStyles } from '$lib/components/ui/button';

  const sections = [
    {value:'clinic',label:tr.settingsNavClinic}, {value:'appearance',label:tr.settingsNavAppearance},
    {value:'security',label:tr.settingsNavSecurity}, {value:'templates',label:tr.clinicalTemplates},
    {value:'reminders',label:tr.remindersTemplatesTitle}, {value:'whatsapp',label:tr.whatsappTitle},
    {value:'backups',label:tr.settingsBackups}, {value:'import',label:tr.settingsNavImport},
    {value:'updates',label:tr.settingsNavUpdate}, {value:'about',label:tr.settingsAbout}
  ];
  const activeSection = $derived(sections.find(s => s.value === page.url.searchParams.get('section'))?.value ?? 'clinic');
  let logoInput: HTMLInputElement;
  let whatsappMode = $state<WhatsappMode>('auto');
  let savedPhone = $state('');
  let whatsappBusy = $state(false);
  const modeHints = { auto: tr.whatsappAutoHint, app: tr.whatsappAppHint, web: tr.whatsappWebHint };
  async function setWhatsappMode(mode: WhatsappMode) {
    whatsappBusy = true;
    try { await api.set('whatsapp_mode', mode); whatsappMode = mode; error = ''; }
    catch { error = tr.settingsError; }
    finally { whatsappBusy = false; }
  }
  async function testWhatsapp() {
    const phone = whatsappPhone(savedPhone); if (!phone || whatsappBusy) return;
    whatsappBusy = true;
    try { await sendWhatsapp(phone, tr.whatsappTestText, whatsappMode); error = ''; }
    catch { error = tr.remindersWhatsAppError; }
    finally { whatsappBusy = false; }
  }
  let clinic = $state({ ...api.emptyClinic });
  let logoBusy = $state(false);
  let clinicBusy = $state(false);
  let clinicReady = $state(false);
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
      clinic = api.clinicProfile(values); clinicReady = true; savedPhone = clinic.clinic_phone; whatsappMode = parseWhatsappMode(values.whatsapp_mode);
      updateEnabled = (values.update_check_enabled ?? '1') !== '0'; backups = rows; info = appInfo;
    } catch { error = tr.settingsError; }
  }
  async function saveClinic() {
    if (clinicBusy || logoBusy || !clinicReady) return;
    clinicBusy = true; toast = '';
    try { await Promise.all(Object.entries(clinic).map(([key, value]) => api.set(key, value))); applyBranding(clinic); savedPhone = clinic.clinic_phone; toast = tr.settingsSaved; error = ''; }
    catch { error = tr.settingsError; }
    finally { clinicBusy = false; }
  }
  async function selectLogo(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0]; if (file) await prepareLogo(file); input.value = '';
  }
  async function prepareLogo(file: File) {
    if (logoBusy || clinicBusy || !clinicReady) return;
    logoBusy = true; toast = ''; error = '';
    try { clinic.clinic_logo = await api.prepareClinicLogo(file); }
    catch { error = tr.documentsLogoError; }
    finally { logoBusy = false; }
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
<div class="grid gap-6">
  <PageHeader title={tr.settings} subtitle={tr.settingsDescription} icon={Settings2}>
    {#snippet actions()}<button class={buttonStyles('primary')} disabled={busy} onclick={() => void backupNow()}>{tr.settingsBackupNow}</button>{/snippet}
  </PageHeader>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#if toast}<p role="status" class="text-success">{toast}</p>{/if}
  <div class="settings-grid">
    <nav class="settings-nav" aria-label={tr.settingsSection}>
      {#each sections as section}<a href={'?section=' + section.value} aria-current={activeSection === section.value ? 'page' : undefined} class="flex min-h-10 items-center rounded-sm px-3 text-sm hover:bg-surface-muted" class:text-accent={activeSection === section.value} class:bg-surface-muted={activeSection === section.value}>{section.label}</a>{/each}
    </nav>
    <div class="settings-select"><Select label={tr.settingsSection} value={activeSection} options={sections} onchange={event => void goto('?section=' + event.currentTarget.value, {noScroll:true, keepFocus:true})} /></div>
    <div class="min-w-0">
  <section id="appearance" hidden={activeSection !== 'appearance'} class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-border bg-surface p-5 sm:p-6">
    <div class="flex items-center gap-3"><IconChip icon={Moon} tone="neutral" size="sm" /><div><h2 class="type-section">{tr.theme}</h2><p class="text-sm text-fg-muted">{theme.preference === 'system' ? tr.themeFollowSystem : tr.themeManual}</p></div></div>
    <button class="type-caption min-h-7 text-accent underline underline-offset-4" onclick={() => theme.set('system')}>{tr.useSystemTheme}</button>
  </section>
  <section id="clinic" hidden={activeSection !== 'clinic'}><Card title={tr.documentsClinic} icon={Stethoscope}>
    <form class="grid gap-4 sm:grid-cols-2" onsubmit={(e) => { e.preventDefault(); if (validateForm(e.currentTarget)) void saveClinic(); }}>
      <fieldset disabled={!clinicReady || clinicBusy} class="contents">
      {#each [['clinic_name', tr.settingsClinicName], ['vet_name', tr.settingsVetName], ['clinic_title', tr.documentsVetTitle], ['clinic_registry_no', tr.documentsRegistry], ['clinic_tax_office', tr.documentsTaxOffice], ['clinic_tax_no', tr.documentsTaxNo]] as [key, label]}
        <Input {label} icon={key === 'vet_name' ? User : key === 'clinic_name' || key === 'clinic_tax_office' ? Building2 : IdCard} bind:value={clinic[key as keyof typeof clinic]} />
      {/each}
      <PhoneInput label={tr.settingsPhone} bind:value={clinic.clinic_phone} />
      <EmailInput label={tr.email} bind:value={clinic.clinic_email} />
      <div class="sm:col-span-2"><Textarea label={tr.settingsAddress} icon={MapPin} bind:value={clinic.clinic_address} /></div>
      <div class="grid gap-3 sm:col-span-2">
        <span class="text-sm font-medium">{tr.documentsLogo}</span>
        <input bind:this={logoInput} type="file" class="hidden" accept="image/png,image/jpeg,image/webp" disabled={logoBusy} onchange={selectLogo} />
        <div class="flex flex-wrap items-center gap-4">
          <button type="button" class="flex size-24 shrink-0 items-center justify-center overflow-hidden rounded-md border border-border-control bg-surface-muted" aria-label={tr.brandingUpload} disabled={logoBusy} onclick={() => logoInput.click()}>
            {#if api.safeClinicLogo(clinic.clinic_logo)}<img src={api.safeClinicLogo(clinic.clinic_logo)} alt={tr.documentsLogo} class="size-24 object-contain" />{:else}<Upload size={24} class="text-fg-muted" />{/if}
          </button>
          <div class="grid gap-2"><div class="flex flex-wrap gap-2">
            <button type="button" class={buttonStyles('secondary')} disabled={logoBusy} onclick={() => logoInput.click()}><Upload size={16} />{clinic.clinic_logo ? tr.brandingChange : tr.brandingUpload}</button>
            {#if clinic.clinic_logo}<button type="button" disabled={logoBusy} class={buttonStyles('ghost')} onclick={() => clinic.clinic_logo = ''}>{tr.brandingRemove}</button>{/if}
          </div></div>
        </div>
        <p class="text-sm text-fg-muted">{tr.documentsLogoHint}</p>
      </div>
      <div class="sm:col-span-2"><button class={buttonStyles('primary')} type="submit" disabled={logoBusy || clinicBusy || !clinicReady}>{clinicBusy || logoBusy ? tr.loading : tr.save}</button></div>
      </fieldset>
    </form>
  </Card></section>
  <section id="whatsapp" hidden={activeSection !== 'whatsapp'}>
    <Card title={tr.whatsappTitle} icon={MessageCircle}>
      <fieldset class="grid gap-3" disabled={!clinicReady || whatsappBusy}>
        <legend class="mb-2 text-sm font-medium">{tr.whatsappCurrentMode}</legend>
        {#each whatsappModes as mode}<label class="flex min-h-11 items-start gap-3"><input type="radio" name="whatsapp-mode" value={mode} checked={whatsappMode === mode} onchange={() => void setWhatsappMode(mode)} class="mt-1 accent-ink" /><span><span class="block font-medium">{whatsappModeLabel[mode]}</span><span class="text-sm text-fg-muted">{modeHints[mode]}</span></span></label>{/each}
      </fieldset>
      <h3 class="mt-4 font-medium">{tr.whatsappHow}</h3><p class="mt-1 text-sm text-fg-muted">{tr.whatsappExplanation}</p><p class="mt-2 text-sm text-fg-muted">{tr.whatsappInstallHint}</p>
      <button type="button" class={buttonStyles('secondary') + ' mt-4'} disabled={!clinicReady || !whatsappPhone(savedPhone) || whatsappBusy} onclick={() => void testWhatsapp()}>{tr.whatsappTest}</button>
      {#if !whatsappPhone(savedPhone)}<p class="mt-2 text-sm text-fg-muted">{tr.whatsappPhoneHint}</p>{/if}
    </Card>
  </section>
  <section id="security" hidden={activeSection !== 'security'}><LockCard /></section>
  <section id="templates" hidden={activeSection !== 'templates'}><VisitTemplatesCard /></section>
  <section id="reminders" hidden={activeSection !== 'reminders'}><ReminderTemplatesCard /></section>
  <section id="backups" hidden={activeSection !== 'backups'} class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={ShieldCheck} tone="accent" size="sm" />{tr.settingsBackups}</h2>
    <p class="mt-2 text-fg-muted">{tr.settingsBackupPolicy}</p>
    <div class="mt-4 flex flex-wrap gap-2"><button class={buttonStyles('primary')} disabled={busy} onclick={() => void backupNow()}>{tr.settingsBackupNow}</button><button class={buttonStyles('secondary')} onclick={() => void exportBackup()}>{tr.settingsExport}</button></div>
    <ul class="mt-4 divide-y divide-border">{#each backups as backup (backup.path)}<li class="flex flex-wrap items-center justify-between gap-3 py-3"><div><span class="tabular-nums">{date(backup.createdAt)}</span><span class="mx-2 text-fg-muted">· {kinds[backup.kind]}</span><span class="text-fg-muted">{size(backup.sizeBytes)}</span></div><button class={buttonStyles('secondary')} onclick={() => { restoreTarget = backup; restoreDialogOpen = true; }}>{tr.settingsRestore}</button></li>{:else}<li class="py-3 text-fg-muted">—</li>{/each}</ul>
  </section>
  <section id="import" hidden={activeSection !== 'import'} class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={Import} tone="neutral" size="sm" />{tr.settingsLegacy}</h2><a class="mt-3 inline-block text-accent underline" href="/import">{tr.importTitle}</a></section>
  <section id="updates" hidden={activeSection !== 'updates'} class="rounded-lg border border-border bg-surface p-5 sm:p-6">
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
  <section id="about" hidden={activeSection !== 'about'} class="rounded-lg border border-border bg-surface p-5 sm:p-6"><h2 class="type-section flex items-center gap-3"><IconChip icon={Info} tone="neutral" size="sm" />{tr.settingsAbout}</h2><p class="mt-3">{tr.version}: {info?.version ?? '—'}</p><p class="mt-1 text-fg-muted">{tr.settingsLicense}</p><a class="mt-2 inline-flex items-center gap-2 text-accent underline" href="https://github.com/muhammedalikutbay/baytar" target="_blank" rel="noreferrer"><ExternalLink size={16} />GitHub</a><p class="mt-4">{tr.localData}</p><p class="break-all text-sm text-fg-muted">{info?.dataFolder ?? '—'}</p><button class={`${buttonStyles('secondary')} mt-3`} onclick={() => void api.openDataFolder()}>{tr.settingsOpenFolder}</button></section>
    </div>
  </div>
</div>

<Dialog bind:open={restoreDialogOpen} title={tr.settingsRestoreTitle} description={tr.settingsRestoreDescription}>
  {#snippet footer()}<button class={buttonStyles('secondary')} onclick={() => { restoreTarget = null; restoreDialogOpen = false; }}>{tr.cancel}</button><button class={buttonStyles('primary')} disabled={busy} onclick={() => void restore()}>{tr.settingsRestore}</button>{/snippet}
</Dialog>

<style>
  .settings-grid { display: grid; grid-template-columns: 196px minmax(0, 1fr); gap: 24px; align-items: start; }
  .settings-nav { position: sticky; top: 64px; display: grid; gap: 4px; }
  .settings-select { display: none; }
  section[hidden] { display: none; }
  @media (max-width: 899px) {
    .settings-grid { grid-template-columns: minmax(0, 1fr); }
    .settings-nav { display: none; }
    .settings-select { display: block; }
  }
</style>
