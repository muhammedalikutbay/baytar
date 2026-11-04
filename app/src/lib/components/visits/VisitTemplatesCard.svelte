<script lang="ts">
  import { onMount } from 'svelte';
  import { Files } from '@lucide/svelte';
  import { Card, Button, Dialog, Input, Textarea, Checkbox, Badge, EmptyState } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { validateForm } from '$lib/validation';
  import * as templates from '$lib/api/templates';
  import type { VisitTemplate } from '$lib/api/types';
  let rows = $state<VisitTemplate[]>([]), loading = $state(true), error = $state(''), busy = $state(false), includeDeleted = $state(false);
  let open = $state(false), deleting = $state<VisitTemplate | null>(null), deleteOpen = $state(false), editing = $state<number | null>(null);
  let draft = $state({ name: '', reason: '', findings: '', diagnosis: '', treatment: '', notes: '' });
  async function load() { loading = true; error = ''; try { rows = await templates.list(includeDeleted); } catch { error = tr.loadError; } finally { loading = false; } }
  onMount(() => { void load(); });
  function edit(row?: VisitTemplate) { editing = row?.id ?? null; draft = { name: row?.name ?? '', reason: row?.reason ?? '', findings: row?.findings ?? '', diagnosis: row?.diagnosis ?? '', treatment: row?.treatment ?? '', notes: row?.notes ?? '' }; error = ''; open = true; }
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (busy || !validateForm(event.currentTarget as HTMLFormElement)) return;
    busy = true; error = '';
    try { const input = { name: draft.name.trim(), reason: draft.reason.trim() || null, findings: draft.findings.trim() || null, diagnosis: draft.diagnosis.trim() || null, treatment: draft.treatment.trim() || null, notes: draft.notes.trim() || null };
      if (editing === null) await templates.create(input); else await templates.update(editing, input); open = false; await load();
    } catch { error = tr.saveError; } finally { busy = false; }
  }
  async function remove() { if (!deleting || busy) return; busy = true; error = ''; try { await templates.softDelete(deleting.id); deleteOpen = false; await load(); } catch { error = tr.actionError; } finally { busy = false; } }
  async function restore(id: number) { busy = true; error = ''; try { await templates.restore(id); await load(); } catch { error = tr.actionError; } finally { busy = false; } }
</script>
<div id="visit-templates" class="scroll-mt-6">
  <Card title={tr.clinicalTemplates} icon={Files} count={rows.filter(r => !r.deletedAt).length}>
    {#snippet actions()}<Button size="sm" onclick={() => edit()}>{tr.clinicalNewTemplate}</Button>{/snippet}
    <p class="mb-4 text-fg-muted">{tr.clinicalTemplatesDescription}</p>
    <Checkbox label={tr.clinicalShowDeleted} bind:checked={includeDeleted} onchange={load} />
    {#if error && !open && !deleteOpen}<p role="alert" class="mt-3 text-danger">{error}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>{/if}
    {#if loading}<p role="status" class="mt-3">{tr.loading}</p>
    {:else if !rows.length}<EmptyState icon={Files} title={tr.clinicalTemplatesEmpty} description={tr.clinicalTemplatesDescription}>{#snippet action()}<Button onclick={() => edit()}>{tr.clinicalNewTemplate}</Button>{/snippet}</EmptyState>
    {:else}<ul class="mt-3 divide-y divide-border">{#each rows as row (row.id)}<li class="flex flex-wrap items-center justify-between gap-3 py-3"><span class="break-words font-medium">{row.name} {#if row.deletedAt}<Badge>{tr.clinicalDeleted}</Badge>{/if}</span><div class="flex gap-2">
      {#if row.deletedAt}<Button size="sm" variant="secondary" disabled={busy} onclick={() => restore(row.id)}>{tr.clinicalRestore}</Button>
      {:else}<Button size="sm" variant="secondary" onclick={() => edit(row)}>{tr.edit}</Button><Button size="sm" variant="danger" disabled={busy} onclick={() => { deleting = row; deleteOpen = true; error = ''; }}>{tr.delete}</Button>{/if}
    </div></li>{/each}</ul>{/if}
  </Card>
</div>
<Dialog bind:open title={editing === null ? tr.clinicalNewTemplate : tr.clinicalEditTemplate} description={tr.clinicalTemplatesDescription}>
  <form onsubmit={submit} class="grid gap-4">
    <fieldset disabled={busy} class="grid min-w-0 gap-4"><Input label={tr.clinicalTemplateName} bind:value={draft.name} required />
      <Textarea label={tr.clinicalSoapSubjective} bind:value={draft.reason} /><Textarea label={tr.clinicalSoapObjective} bind:value={draft.findings} />
      <Textarea label={tr.clinicalSoapAssessment} bind:value={draft.diagnosis} /><Textarea label={tr.clinicalSoapPlan} bind:value={draft.treatment} /><Textarea label={tr.notes} bind:value={draft.notes} />
    </fieldset>
    {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
    <div class="flex justify-end gap-2"><Button variant="secondary" disabled={busy} onclick={() => open = false}>{tr.cancel}</Button><Button type="submit" disabled={busy}>{busy ? tr.saving : tr.save}</Button></div>
  </form>
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={deleteOpen} title={tr.clinicalDeleteTemplate} description={tr.clinicalDeleteTemplateDescription}>
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  {#snippet footer()}<Button variant="secondary" disabled={busy} onclick={() => deleteOpen = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
