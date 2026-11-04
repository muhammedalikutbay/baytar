<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { Files, FileText, Image as ImageIcon } from '@lucide/svelte';
  import { Card, Button, Badge, Checkbox, Select, Dialog, Input, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate } from '$lib/format';
  import * as api from '$lib/api/attachments';
  import type { Attachment, AttachmentKind } from '$lib/api/types';
  let { patientId, visitId = null, allowDrop = true, revision = 0, onchange }: { patientId: number; visitId?: number | null; allowDrop?: boolean; revision?: number; onchange?: () => void } = $props();
  let rows = $state<Attachment[]>([]), thumbnails = $state<Record<number, string>>({});
  let includeDeleted = $state(false), kind = $state<AttachmentKind>('document'), busy = $state(false), loading = $state(true), error = $state('');
  let refresh = $state(0), dragging = $state(false), renaming = $state<Attachment | null>(null), renameOpen = $state(false), title = $state('');
  let deleting = $state<Attachment | null>(null), deleteOpen = $state(false), preview = $state<Attachment | null>(null), previewOpen = $state(false);
  let previewUrl = $state(''), previewLoading = $state(false);
  const kindLabels: Record<AttachmentKind, string> = { photo: tr.clinicalKindPhoto, lab: tr.clinicalKindLab, imaging: tr.clinicalKindImaging, document: tr.clinicalKindDocument, other: tr.clinicalKindOther };
  const kindOptions = Object.entries(kindLabels).map(([value, label]) => ({ value, label }));
  $effect(() => {
    const id = patientId, visit = visitId, deleted = includeDeleted; refresh; revision;
    let cancelled = false; loading = true; error = '';
    (async () => {
      try {
        const list = await api.list(id, visit, deleted); if (cancelled) return; rows = list; thumbnails = {};
        await Promise.allSettled(list.filter(r => !r.deletedAt && r.mime.startsWith('image/')).map(async row => {
          const src = await api.thumbnail(row.id); if (src && !cancelled) thumbnails[row.id] = src;
        }));
      } catch { if (!cancelled) error = tr.loadError; } finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  onMount(() => {
    let disposed = false, unlisten: (() => void) | undefined;
    if (allowDrop && isTauri()) void getCurrentWebview().onDragDropEvent(event => {
      if (document.querySelector('[role="dialog"]')) return;
      dragging = event.payload.type === 'over' || event.payload.type === 'enter';
      if (event.payload.type === 'drop') { dragging = false; void addPaths(event.payload.paths); }
    }).then(stop => { if (disposed) stop(); else unlisten = stop; }).catch(() => { /* File selection remains available. */ });
    return () => { disposed = true; unlisten?.(); };
  });
  $effect(() => {
    const file = preview, open = previewOpen; let cancelled = false, source = '';
    previewUrl = ''; previewLoading = false;
    if (file && open && (file.mime.startsWith('image/') || file.mime === 'application/pdf')) {
      previewLoading = true;
      api.readBytes(file.id).then(data => {
        if (cancelled) return;
        source = URL.createObjectURL(new Blob([new Uint8Array(data)], { type: file.mime })); previewUrl = source;
      }).catch(() => { if (!cancelled) error = tr.clinicalFileError; }).finally(() => { if (!cancelled) previewLoading = false; });
    }
    return () => { cancelled = true; if (source) URL.revokeObjectURL(source); };
  });
  function changed() { refresh++; onchange?.(); }
  async function addPaths(paths: string[]) {
    if (busy || !paths.length) return; busy = true; error = ''; let failures = 0;
    for (const path of paths) { try { await api.addFromPath(patientId, visitId, kind, path); } catch { failures++; } }
    busy = false; changed();
    // Refresh resets the loading error, so preserve upload feedback separately.
    uploadError = failures ? tr.clinicalPartialFiles.replace('{count}', String(failures)) : '';
  }
  let uploadError = $state('');
  async function pick() { if (busy) return; try { await addPaths(await api.pickPaths()); } catch { uploadError = tr.clinicalFileError; } }
  async function rename() {
    if (!renaming || busy) return; busy = true; error = '';
    try { await api.rename(renaming.id, title.trim() || null); renameOpen = false; changed(); } catch { error = tr.clinicalFileError; } finally { busy = false; }
  }
  async function remove() { if (!deleting || busy) return; busy = true; error = ''; try { await api.softDelete(deleting.id); deleteOpen = false; changed(); } catch { error = tr.clinicalFileError; } finally { busy = false; } }
  async function restore(id: number) { busy = true; error = ''; try { await api.restore(id); changed(); } catch { error = tr.clinicalFileError; } finally { busy = false; } }
  async function external(file: Attachment) { error = ''; try { await api.openExternal(file.id); } catch { error = tr.clinicalFileError; } }
  async function save(file: Attachment) { error = ''; try { await api.exportFile(file); } catch { error = tr.clinicalFileError; } }
</script>
<div class={dragging ? 'rounded-lg outline-2 outline-accent' : ''}>
  <Card title={tr.clinicalFiles} icon={Files} count={rows.filter(r => !r.deletedAt).length}>
    {#snippet actions()}<Button size="sm" disabled={busy} onclick={pick}>{tr.clinicalAddFiles}</Button>{/snippet}
    <div class="mb-4 grid gap-3"><Select label={tr.clinicalKind} bind:value={kind} options={kindOptions} disabled={busy} /><p class="type-caption text-fg-muted">{tr.clinicalFileLimit} {kind === 'photo' ? tr.clinicalPhotoHint : ''}</p><Checkbox label={tr.clinicalShowDeleted} bind:checked={includeDeleted} /></div>
    {#if uploadError}<p role="alert" class="mb-3 text-danger">{uploadError}</p>{/if}
    {#if error && !renameOpen && !deleteOpen}<p role="alert" class="mb-3 text-danger">{error}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>{/if}
    {#if loading}<p role="status">{tr.loading}</p>
    {:else if !rows.length}<EmptyState icon={Files} title={tr.clinicalFilesEmpty} description={tr.clinicalFilesDescription}>{#snippet action()}<Button disabled={busy} onclick={pick}>{tr.clinicalAddFiles}</Button>{/snippet}</EmptyState>
    {:else}<ul class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each rows as file (file.id)}<li class="grid content-start gap-3 rounded-md border border-border p-3">
        <button type="button" disabled={!!file.deletedAt} class="flex h-32 items-center justify-center overflow-hidden rounded-sm bg-surface-muted" aria-label={`${tr.clinicalPreview}: ${file.title ?? file.fileName}`} onclick={() => { preview = file; previewOpen = true; error = ''; }}>
          {#if thumbnails[file.id]}<img src={thumbnails[file.id]} alt={file.title ?? file.fileName} class="size-full object-contain" />{:else if file.mime.startsWith('image/')}<ImageIcon size={32} class="text-fg-muted" />{:else}<FileText size={32} class="text-fg-muted" />{/if}
        </button>
        <p class="break-words font-medium">{file.title ?? file.fileName}</p>
        <div class="flex flex-wrap gap-2"><Badge variant={file.deletedAt ? 'neutral' : 'accent'}>{kindLabels[file.kind]}</Badge>{#if file.deletedAt}<Badge>{tr.clinicalDeleted}</Badge>{/if}</div>
        <p class="type-caption text-fg-muted tabular-nums">{formatDate(file.createdAt.slice(0,10))} · {(file.sizeBytes / 1024).toLocaleString('tr-TR', { maximumFractionDigits: 0 })} KB</p>
        {#if file.visitId}<a class="type-caption text-accent underline" href={`/patients/pet?id=${patientId}&visit=${file.visitId}#visit-${file.visitId}`}>{tr.clinicalVisitLink} #{file.visitId}</a>{:else}<span class="type-caption text-fg-muted">{tr.clinicalPatientFile}</span>{/if}
        <div class="flex flex-wrap gap-1">
          {#if file.deletedAt}<Button variant="secondary" size="sm" disabled={busy} onclick={() => restore(file.id)}>{tr.clinicalRestore}</Button>
          {:else}<Button variant="secondary" size="sm" onclick={() => external(file)}>{tr.clinicalOpenFile}</Button><Button variant="ghost" size="sm" onclick={() => save(file)}>{tr.clinicalSaveFile}</Button>
            <Button variant="ghost" size="sm" onclick={() => { renaming = file; title = file.title ?? ''; renameOpen = true; error = ''; }}>{tr.clinicalRename}</Button><Button variant="danger" size="sm" disabled={busy} onclick={() => { deleting = file; deleteOpen = true; error = ''; }}>{tr.delete}</Button>{/if}
        </div>
      </li>{/each}
    </ul>{/if}
  </Card>
</div>
<Dialog bind:open={renameOpen} title={tr.clinicalRename} description={renaming?.fileName ?? tr.clinicalFileTitle}>
  <form class="grid gap-4" onsubmit={event => { event.preventDefault(); void rename(); }}><Input label={tr.clinicalFileTitle} bind:value={title} disabled={busy} />{#if error}<p role="alert" class="text-danger">{error}</p>{/if}<div class="flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={() => renameOpen = false}>{tr.cancel}</Button><Button type="submit" disabled={busy}>{tr.save}</Button></div></form>
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={deleteOpen} title={tr.clinicalDeleteFile} description={tr.clinicalDeleteFileDescription}>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#snippet footer()}<Button variant="secondary" disabled={busy} onclick={() => deleteOpen = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
<Dialog bind:open={previewOpen} title={preview?.title ?? preview?.fileName ?? tr.clinicalPreview} description={tr.clinicalPreview}>
  {#if previewLoading}<p role="status">{tr.loading}</p>{:else if previewUrl && preview?.mime.startsWith('image/')}<img src={previewUrl} alt={preview?.title ?? preview?.fileName} class="max-h-[50dvh] w-full object-contain" />
  {:else if previewUrl && preview?.mime === 'application/pdf'}<iframe src={previewUrl} title={preview.title ?? preview.fileName} class="h-[50dvh] w-full rounded-sm border border-border" sandbox=""></iframe>
  {:else}<p class="text-fg-muted">{tr.clinicalPreviewUnavailable}</p>{/if}
  {#if error}<p role="alert" class="mt-3 text-danger">{error}</p>{/if}
  {#snippet footer()}{#if preview}<Button variant="secondary" onclick={() => external(preview!)}>{tr.clinicalOpenFile}</Button><Button onclick={() => save(preview!)}>{tr.clinicalSaveFile}</Button>{/if}{/snippet}
</Dialog>
