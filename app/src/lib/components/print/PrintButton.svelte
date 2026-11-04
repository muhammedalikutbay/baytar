<script lang="ts">
  import { Printer } from '@lucide/svelte';
  import { Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { openDocument, documentTitles, type DocumentKind } from './documents';
  let { kind, id }: { kind: DocumentKind; id: number } = $props();
  let busy = $state(false);
  let error = $state('');
  async function open() {
    busy = true; error = '';
    try { await openDocument(kind, id); }
    catch { error = tr.documentsError + ' ' + tr.documentsPopupError; }
    finally { busy = false; }
  }
</script>
<div class="grid gap-2">
  <Button variant="secondary" disabled={busy} onclick={open}><Printer size={16} aria-hidden="true" />{tr.documentsPrint} · {documentTitles[kind]}</Button>
  {#if error}<p class="max-w-sm text-sm text-danger" role="alert">{error}</p>{/if}
</div>
