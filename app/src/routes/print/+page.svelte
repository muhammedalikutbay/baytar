<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { page } from '$app/state';
  import { isTauri } from '@tauri-apps/api/core';
  import { Printer, X } from '@lucide/svelte';
  import { Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import Document from '$lib/components/print/Document.svelte';
  import { documentTitles, isDocumentKind, loadDocument, snapshotKey, type PrintDocument } from '$lib/components/print/documents';
  let data = $state<PrintDocument | null>(null);
  let loading = $state(true);
  let ready = $state(false);
  let error = $state('');
  let paper = $state<'A4' | 'A5'>('A4');
  let article = $state<HTMLElement>();
  // Page margin boxes keep the footer and real page counter out of the document flow.
  const pageStyle = $derived(`<style>@page { size: ${paper}; margin: 14mm 12mm 18mm;
    @bottom-left { content: ${JSON.stringify(tr.documentsPrepared)}; font-size: 8pt; color: black; }
    @bottom-right { content: ${JSON.stringify(tr.documentsPage + ' ')} counter(page); font-size: 8pt; color: black; }
  }</style>`);
  onMount(() => {
    let disposed = false;
    void (async () => {
      try {
        const kind = page.url.searchParams.get('doc');
        const id = Number(page.url.searchParams.get('id'));
        if (!isDocumentKind(kind) || !Number.isSafeInteger(id) || id <= 0) throw new Error('Invalid document');
        const snapshot = !isTauri() ? sessionStorage.getItem(snapshotKey(kind, id)) : null;
        const result: PrintDocument = snapshot ? JSON.parse(snapshot) : await loadDocument(kind, id);
        if (snapshot) sessionStorage.removeItem(snapshotKey(kind, id));
        if (disposed) return;
        data = result; loading = false;
        await tick();
        await document.fonts.ready;
        await Promise.all(Array.from(article?.querySelectorAll('img') ?? []).map(image => image.decode().catch(() => undefined)));
        await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
        if (!disposed) { ready = true; window.print(); }
      } catch { if (!disposed) { error = tr.documentsError; loading = false; } }
    })();
    return () => { disposed = true; };
  });
  async function close() {
    try {
      if (isTauri()) {
        const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
        await getCurrentWebviewWindow().close();
      } else window.close();
    } catch { error = tr.actionError; }
  }
</script>

<svelte:head><title>{data ? documentTitles[data.kind] : tr.documentsPrint} · {tr.appName}</title>{@html pageStyle}</svelte:head>
<div class="print-preview">
  <div class="toolbar">
    <Button variant="secondary" disabled={!ready} onclick={() => window.print()}><Printer size={16} aria-hidden="true" />{tr.documentsPrint}</Button>
    <label>{tr.documentsPaper}<select bind:value={paper}><option value="A4">A4</option><option value="A5">A5</option></select></label>
    <Button variant="secondary" onclick={close}><X size={16} aria-hidden="true" />{tr.windowClose}</Button>
    <p>{tr.documentsPrintHint}</p>
  </div>
  {#if loading}<p class="status" role="status">{tr.loading}</p>{/if}
  {#if error}<p class="status" role="alert">{error}</p>{/if}
  {#if data}<article bind:this={article} class:a5={paper === 'A5'} aria-label={documentTitles[data.kind]}>
    <Document {data} />
    <footer>{tr.documentsPrepared}</footer>
  </article>{/if}
</div>

<style>
  :global(body:has(.print-preview)) { background: white; color: black; color-scheme: light; }
  .print-preview { min-height: 100vh; background: white; color: black; }
  .toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; padding: 16px; border-bottom: 1px solid; background: var(--color-surface); color: var(--color-fg); }
  .toolbar label { display: flex; align-items: center; gap: 8px; }
  .toolbar select { min-height: 44px; border: 1px solid var(--color-border-control); padding: 8px; border-radius: var(--radius-sm); background: var(--color-surface); }
  .toolbar p { flex-basis: 100%; font-size: 12px; }
  .status { padding: 24px; }
  article { max-width: 210mm; min-height: 260mm; margin: 20px auto; padding: 14mm 12mm; font-size: 10pt; line-height: 1.5; overflow-wrap: anywhere; font-variant-numeric: tabular-nums; }
  article.a5 { max-width: 148mm; font-size: 9pt; }
  article :global(h1) { font-size: 19pt; font-weight: 700; }
  article :global(h2) { font-size: 12pt; font-weight: 600; }
  article :global(.clinic-header) { display: flex; gap: 6mm; align-items: flex-start; border-bottom: 1pt solid; padding-bottom: 5mm; break-inside: avoid; }
  article :global(.clinic-header img) { width: 24mm; height: 24mm; object-fit: contain; }
  article :global(.document-heading) { display: flex; justify-content: space-between; align-items: baseline; gap: 4mm; margin: 6mm 0; }
  article :global(.identity) { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 2mm 5mm; border-bottom: 0.5pt solid; padding-bottom: 5mm; margin-bottom: 6mm; }
  article :global(table) { width: 100%; border-collapse: collapse; table-layout: fixed; margin-top: 4mm; }
  article :global(th), article :global(td) { border: 0.5pt solid; padding: 2mm; text-align: left; vertical-align: top; white-space: pre-wrap; }
  article :global(th) { font-weight: 600; }
  article :global(thead) { display: table-header-group; }
  article :global(tr) { break-inside: avoid; }
  article :global(.signature-cell) { height: 12mm; }
  article :global(.signature) { margin: 15mm 0 0 auto; width: 55mm; min-height: 30mm; border: 0.5pt solid; padding: 3mm; text-align: center; break-inside: avoid; }
  article :global(.clinical) { margin: 5mm 0; }
  article :global(.clinical h2) { break-after: avoid; }
  article :global(.pre) { white-space: pre-wrap; }
  article :global(.balance) { text-align: right; margin-top: 5mm; }
  footer { margin-top: 12mm; border-top: 0.5pt solid; padding-top: 3mm; font-size: 8pt; }
  @media print {
    .toolbar, .status, footer { display: none; }
    article, article.a5 { width: auto; max-width: none; min-height: 0; padding: 0; margin: 0; }
    article :global(*) { color: black !important; background: transparent !important; box-shadow: none !important; }
    :global(html), :global(body) { margin: 0 !important; padding: 0 !important; background: white !important; }
  }
</style>
