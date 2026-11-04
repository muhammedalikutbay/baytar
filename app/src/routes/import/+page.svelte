<script lang="ts">
  import { onMount } from 'svelte';
  import { beforeNavigate } from '$app/navigation';
  import { isTauri } from '@tauri-apps/api/core';
  import * as importer from '$lib/api/import';
  import type { ApiError, ImportCandidate, PreparedImport, ImportReport } from '$lib/api/types';
  import { tr } from '$lib/i18n/tr';
  import { Button, Card, buttonStyles } from '$lib/components/ui';
  import Counts from '$lib/components/import/Counts.svelte';
  import Findings from '$lib/components/import/Findings.svelte';
  let step = $state(0);
  let files = $state<ImportCandidate[]>([]);
  let selected = $state('');
  let prepared = $state<PreparedImport | null>(null);
  let report = $state<ImportReport | null>(null);
  let busy = $state(false);
  let searching = $state(true);
  let error = $state('');
  let saved = $state(false);
  let preview = $derived(prepared?.preview ?? report);
  let noNew = $derived(preview && Object.values(preview.toImport).every(n => n === 0));
  let repeated = $derived(preview && noNew && Object.values(preview.alreadyImported).some(n => n > 0));
  const steps = [tr.importPick, tr.importPreview, tr.importTransfer, tr.importResult];
  function showError(e: unknown) {
    const code = (e as ApiError)?.message;
    error = code === 'not_legacy_database' ? tr.importInvalid : code === 'import_backup_failed' ? tr.importBackupError : code === 'import_expired' ? tr.importExpired : code === 'report_save_failed' ? tr.importReportError : tr.importError;
  }
  async function discard() {
    if (prepared) { await importer.cancel(prepared.token); prepared = null; }
  }
  beforeNavigate(({ cancel }) => {
    if (busy) { cancel(); return; }
    if (prepared) { const token = prepared.token; void importer.cancel(token).catch(() => {}); }
  });
  onMount(() => {
    let live = true;
    void importer.candidates().then(result => { if (live) files = result; }).catch(showError).finally(() => { searching = false; });
    const unload = (event: BeforeUnloadEvent) => { if (busy) { event.preventDefault(); } };
    window.addEventListener('beforeunload', unload);
    return () => { live = false; window.removeEventListener('beforeunload', unload); };
  });
  async function pick() {
    error = '';
    try { const path = await importer.pickFile(); if (path) selected = path; } catch(e) { showError(e); }
  }
  async function prepare() {
    busy = true; error = '';
    try { await discard(); prepared = await importer.prepare(selected); step = 1; } catch(e) { showError(e); } finally { busy = false; }
  }
  async function backToFile() {
    error = ''; busy = true;
    try { await discard(); step = 0; } catch(e) { showError(e); } finally { busy = false; }
  }
  async function execute() {
    if (!prepared) return;
    busy = true; error = '';
    try { report = await importer.execute(prepared.token); prepared = null; step = 3; } catch(e) { showError(e); } finally { busy = false; }
  }
  async function saveReport() {
    error = ''; saved = false;
    try { const path = await importer.pickReportPath(); if (path && report) { await importer.saveReport(String(report.runId), path); saved = true; } } catch(e) { showError(e); }
  }
</script>
<svelte:head><title>{tr.importTitle} · {tr.appName}</title></svelte:head>
<section class="max-w-3xl" aria-busy={busy}>
  <header class="mb-8"><h1 class="type-page">{tr.importTitle}</h1><p class="mt-2 text-fg-muted">{tr.importDescription}</p></header>
  {#if !isTauri()}<p class="type-caption mb-4 text-fg-muted">{tr.importMock}</p>{/if}
  <ol class="mb-6 flex flex-wrap gap-4" aria-label={tr.importTitle}>
    {#each steps as label, index}<li aria-current={index === step ? 'step' : undefined} class={index === step ? 'font-semibold text-fg' : 'text-fg-muted'}><span class="tabular-nums">{index + 1}.</span> {label}</li>{/each}
  </ol>
  <Card>
    {#if step === 0}
      <h2 class="type-title">{tr.importPick}</h2>
      <p class="mt-3">{tr.importOldSafe}</p><p class="text-fg-muted">{tr.importBackupSafe}</p>
      {#if searching}<p class="mt-6" role="status">{tr.importSearch}</p>{:else if files.length === 0}<p class="mt-6 text-fg-muted">{tr.importNoCandidates}</p>{/if}
      <div class="my-6 space-y-3">
        {#each files as file}
          <label class="flex cursor-pointer items-start gap-3 rounded-md border border-border-control p-4 has-[:checked]:bg-surface-muted">
            <input class="mt-1 accent-ink" type="radio" name="legacy-file" value={file.path} bind:group={selected} disabled={busy} />
            <span class="min-w-0"><span class="font-medium">{file.fileName}</span><span class="block break-all text-fg-muted">{file.location}</span><span class="type-caption text-fg-muted">{file.modifiedAt ? new Date(file.modifiedAt).toLocaleDateString('tr-TR') : tr.importNoValue} · {(file.sizeBytes / 1024).toLocaleString('tr-TR', { maximumFractionDigits: 1 })} KB</span></span>
          </label>
        {/each}
      </div>
      <Button variant="secondary" onclick={pick} disabled={busy}>{tr.importOther}</Button>
      {#if selected}<p class="mt-3 break-all text-fg-muted">{tr.importSelected}: {selected}</p>{/if}
      <div class="mt-6"><Button onclick={prepare} disabled={!selected || busy}>{busy ? tr.importPreparing : tr.importNext}</Button></div>
    {:else if step === 1 && preview}
      <h2 class="type-title">{tr.importPreview}</h2>
      {#if noNew}<p class="my-6">{repeated ? tr.importNoNew : tr.importEmpty}</p>{:else}
        <Counts counts={preview.toImport} />
        <h3 class="type-section">{tr.importAutomatic}</h3>
        <p class="mt-2 text-fg-muted">{tr.importPhones} {preview.phonesAdjusted.toLocaleString('tr-TR')}</p><p class="mb-6 text-fg-muted">{tr.importDates}</p>
        <Findings {preview} />
      {/if}
      {#if !noNew && Object.values(preview.alreadyImported).some(n => n > 0)}<h3 class="type-section mt-6">{tr.importAlready}</h3><Counts counts={preview.alreadyImported} />{/if}
      <div class="mt-6 flex gap-3"><Button variant="secondary" onclick={backToFile} disabled={busy}>{tr.importBack}</Button>{#if !noNew}<Button onclick={() => step = 2}>{tr.importNext}</Button>{/if}</div>
    {:else if step === 2}
      <h2 class="type-title">{tr.importReady}</h2>
      <p class="mt-6">{tr.importOldSafe}</p><p class="mt-2">{tr.importBackupSafe}</p>
      <div class="mt-6 flex gap-3"><Button variant="secondary" disabled={busy} onclick={() => step = 1}>{tr.importBack}</Button><Button disabled={busy} onclick={execute}>{tr.importExecute}</Button></div>
      {#if busy}<p role="status" class="mt-6">{tr.importRunning}</p>{/if}
    {:else if report}
      <h2 class="type-title">{tr.importComplete}</h2>
      {#if repeated}<p class="my-6">{tr.importNoNew}</p>{:else}<Counts counts={report.inserted} />{/if}
      <p class="break-all text-fg-muted">{tr.importBackup}: {report.backupFile}</p>
      <div class="mt-6 flex flex-wrap gap-3"><Button variant="secondary" onclick={saveReport}>{tr.importSaveReport}</Button><a href="/patients" class={buttonStyles()}>{tr.homeAction}</a></div>
      {#if saved}<p role="status" class="mt-4 text-success">{tr.importReportSaved}</p>{/if}
    {/if}
    {#if error}<p role="alert" class="mt-6 text-danger">{error}</p>{/if}
  </Card>
  {#if step < 3}<div class="mt-4"><Button variant="ghost" disabled={busy} onclick={async () => { error = ''; try { await discard(); step = 0; selected = ''; } catch(e) { showError(e); } }}>{tr.cancel}</Button></div>{/if}
</section>
