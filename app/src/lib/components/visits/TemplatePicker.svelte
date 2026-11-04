<script lang="ts">
  import { onMount } from 'svelte';
  import { Select, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as templates from '$lib/api/templates';
  import type { VisitTemplate } from '$lib/api/types';
  let { onselect }: { onselect: (template: VisitTemplate) => void } = $props();
  let rows = $state<VisitTemplate[]>([]), chosen = $state(''), loading = $state(true), failed = $state(false);
  async function load() { loading = true; failed = false; try { rows = await templates.list(); } catch { failed = true; } finally { loading = false; } }
  onMount(() => { void load(); });
</script>
<section class="grid gap-2 rounded-md border border-border p-4" aria-label={tr.clinicalFillTemplate}>
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if failed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>
  {:else if rows.length}
    <Select label={tr.clinicalFillTemplate} bind:value={chosen} placeholder={tr.clinicalChooseTemplate} options={rows.map(t => ({ value: String(t.id), label: t.name }))} />
    <Button variant="secondary" disabled={!chosen} onclick={() => { const template = rows.find(t => String(t.id) === chosen); if (template) onselect(template); }}>{tr.clinicalFillTemplate}</Button>
  {:else}<p class="type-section">{tr.clinicalTemplatesEmpty}</p><p class="type-caption text-fg-muted">{tr.clinicalTemplatesDescription}</p><a class="text-accent underline" href="/settings#visit-templates">{tr.clinicalNewTemplate}</a>{/if}
</section>
