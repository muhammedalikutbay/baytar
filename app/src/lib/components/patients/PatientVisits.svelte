<script lang="ts">
  import { Stethoscope, Syringe, ChevronDown } from '@lucide/svelte';
  import { Button, Card, Dialog, EmptyState, Toast } from '$lib/components/ui';
  import { isPastClinicalRecord } from './clinical';
  import PrintButton from '$lib/components/print/PrintButton.svelte';
  import VisitCharges from '$lib/components/billing/VisitCharges.svelte';
  import VisitVitals from '$lib/components/visits/VisitVitals.svelte';
  import VisitForm from '$lib/components/visits/VisitForm.svelte';
  import HistoryRecordForm from './HistoryRecordForm.svelte';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatMonth, formatNumber } from '$lib/format';
  import * as visits from '$lib/api/visits';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as prescriptions from '$lib/api/prescriptions';
  import type { Visit, Vaccination, PrescriptionItem } from '$lib/api/types';
  let { patientId, patientName = '', revision = 0, onchange = () => {} }: { patientId: number; patientName?: string; revision?: number; onchange?: () => void } = $props();
  type Entry = { kind: 'visit'; record: Visit } | { kind: 'vaccination'; record: Vaccination };
  let entries = $state<Entry[]>([]);
  let prescriptionItems = $state<Record<number, PrescriptionItem[]>>({});
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let editing = $state<Entry | null>(null);
  let deleting = $state<Entry | null>(null);
  let editOpen = $state(false);
  let deleteOpen = $state(false);
  let busy = $state(false);
  let actionError = $state('');
  interface Undo { key: number; entry: Entry; open: boolean }
  let undoKey = 0;
  let undos = $state<Undo[]>([]);
  $effect(() => {
    const id = patientId; revision; refresh;
    let cancelled = false;
    loading = true; failed = false;
    (async () => {
      try {
        const [encounters, vaccines] = await Promise.all([
          visits.listByPatient(id), vaccinations.listByPatient(id)
        ]);
        const items = await Promise.all(encounters.map(async visit => [visit.id, await prescriptions.listByVisit(visit.id)] as const));
        if (cancelled) return;
        prescriptionItems = Object.fromEntries(items);
        entries = [
          ...encounters.filter(v => isPastClinicalRecord(v.visitOn, v.visitTime)).map(record => ({ kind: 'visit' as const, record })),
          ...vaccines.filter(v => isPastClinicalRecord(v.givenOn)).map(record => ({ kind: 'vaccination' as const, record }))
        ].sort((a, b) => date(b).localeCompare(date(a)) || time(b).localeCompare(time(a)) || b.record.id - a.record.id || a.kind.localeCompare(b.kind));
      } catch { if (!cancelled) failed = true; }
      finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  function date(entry: Entry): string { return entry.kind === 'visit' ? entry.record.visitOn : entry.record.givenOn; }
  function time(entry: Entry): string { return entry.kind === 'visit' ? entry.record.visitTime ?? '' : ''; }
  function summary(entry: Entry): string { return entry.kind === 'visit' ? entry.record.reason || entry.record.diagnosis || tr.visit : entry.record.vaccineName; }
  function details(entry: Entry): { label: string; value: string | null }[] {
    if (entry.kind === 'visit') {
      const v = entry.record;
      return [{label:tr.reason,value:v.reason}, {label:tr.findings,value:v.findings}, {label:tr.diagnosis,value:v.diagnosis}, {label:tr.treatment,value:v.treatment},
        {label:tr.weight,value:v.weightKg === null ? null : formatNumber(v.weightKg)}, {label:tr.temperature,value:v.temperatureC === null ? null : formatNumber(v.temperatureC)}, {label:tr.notes,value:v.notes}];
    }
    return [{label:tr.nextDue,value:formatDate(entry.record.nextDueOn)}, {label:tr.batchNo,value:entry.record.batchNo}, {label:tr.notes,value:entry.record.notes}];
  }
  function prescriptionDetails(item: PrescriptionItem) {
    return [{label:tr.doseMg,value:item.doseMg === null ? null : formatNumber(item.doseMg)}, {label:tr.doseMl,value:item.doseMl === null ? null : formatNumber(item.doseMl)}, {label:tr.doseText,value:item.doseText}, {label:tr.frequency,value:item.frequency}, {label:tr.durationDays,value:item.durationDays === null ? null : formatNumber(item.durationDays)}, {label:tr.notes,value:item.notes}].filter(d => d.value);
  }
  const groups = $derived.by(() => {
    const result: { month: string; items: Entry[] }[] = [];
    for (const entry of entries) {
      const month = date(entry).slice(0, 7);
      if (result.at(-1)?.month !== month) result.push({ month, items: [] });
      result[result.length - 1].items.push(entry);
    }
    return result;
  });
  function api(entry: Entry) { return entry.kind === 'visit' ? visits : vaccinations; }
  async function remove() {
    if (!deleting || busy) return;
    const entry = deleting; busy = true; actionError = '';
    try {
      await api(entry).softDelete(entry.record.id);
      undos = [...undos, { key: undoKey++, entry, open: true }];
      deleteOpen = false; refresh++; onchange();
    } catch { actionError = tr.actionError; } finally { busy = false; }
  }
  async function undo(item: Undo) {
    if (busy) return;
    busy = true; actionError = '';
    try { await api(item.entry).restore(item.entry.record.id); item.open = false; refresh++; onchange(); }
    catch { actionError = tr.actionError; } finally { busy = false; }
  }
  function setEditOpen(open: boolean) { editOpen = open; if (!open) refresh++; }
  function closeEdit() { setEditOpen(false); onchange(); }
</script>

<section aria-label={tr.history}>
  <Card>
    <div class="flex flex-wrap items-center justify-between gap-4">
      <h2 class="type-section">{tr.history}</h2>
    </div>
    {#if actionError}<p role="alert" class="mt-4 text-danger">{actionError}</p>{/if}
    {#if loading}<p role="status" class="mt-4">{tr.loading}</p>
    {:else if failed}<p role="alert" class="my-4 text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>
    {:else if !entries.length}<EmptyState icon={Stethoscope} title={tr.historyEmpty} description={tr.historyDescription} />
    {:else}
      {#each groups as group (group.month)}
        <section class="mt-6" aria-label={formatMonth(group.month + '-01')}>
          <h3 class="type-caption mb-2 text-fg-muted">{formatMonth(group.month + '-01')}</h3>
          <div class="divide-y divide-border">
            {#each group.items as entry (`${entry.kind}-${entry.record.id}`)}
              <details class="py-3">
                <summary class="flex min-h-11 cursor-pointer items-center gap-3 rounded-sm hover:bg-surface-muted">
                  {#if entry.kind === 'visit'}<Stethoscope size={18} class="shrink-0 text-fg-muted" aria-hidden="true" />
                  {:else}<Syringe size={18} class="shrink-0 text-fg-muted" aria-hidden="true" />{/if}
                  <div class="min-w-0 flex-1"><p class="type-caption text-fg-muted"><time datetime={date(entry)}>{formatDate(date(entry))}</time>{time(entry) ? ` · ${time(entry)}` : ''} · {tr[entry.kind]}</p><p class="truncate">{summary(entry)}</p>{#if entry.kind === 'visit'}<VisitVitals visit={entry.record} />{/if}</div>
                  <ChevronDown size={16} class="shrink-0 text-fg-muted" aria-hidden="true" />
                </summary>
                <div class="mt-3 grid gap-4 pl-8">
                  <dl class="grid gap-3">
                    {#each details(entry).filter(d => d.value) as detail}<div><dt class="type-caption text-fg-muted">{detail.label}</dt><dd class="whitespace-pre-wrap break-words">{detail.value}</dd></div>{/each}
                  </dl>
                  {#if entry.kind === 'visit' && prescriptionItems[entry.record.id]?.length}
                    <section aria-label={tr.prescription}><h4 class="type-caption text-fg-muted">{tr.prescription}</h4>
                      <ul class="mt-2 grid gap-3">
                        {#each prescriptionItems[entry.record.id] as item (item.id)}
                          <li class="break-words rounded-sm border border-border p-3">
                            <p class="font-medium">{item.medicationName}</p>
                            <dl class="mt-1 grid gap-1">
                              {#each prescriptionDetails(item) as detail}<div><dt class="inline text-fg-muted">{detail.label}: </dt><dd class="inline whitespace-pre-wrap">{detail.value}</dd></div>{/each}
                            </dl>
                          </li>
                        {/each}
                      </ul>
                    </section>
                  {/if}
                  {#if entry.kind === 'visit'}<VisitCharges visitId={entry.record.id} {patientId} date={entry.record.visitOn} />{/if}
                  <div class="flex flex-wrap gap-2">{#if entry.kind === 'visit'}<PrintButton kind="prescription" id={entry.record.id} /><PrintButton kind="summary" id={entry.record.id} />{/if}<Button variant="secondary" size="sm" onclick={() => { editing = entry; editOpen = true; }}>{tr.edit}</Button><Button variant="danger" size="sm" disabled={busy} onclick={() => { actionError = ''; deleting = entry; deleteOpen = true; }}>{tr.delete}</Button></div>
                </div>
              </details>
            {/each}
          </div>
        </section>
      {/each}
    {/if}
  </Card>
</section>
<Dialog wide={editing?.kind === 'visit'} bind:open={() => editOpen, setEditOpen} title={editing?.kind === 'visit' ? tr.visit + ' — ' + patientName : tr.editHistory} description={editing?.kind === 'visit' ? tr.visitFormDescription : tr.historyFormDescription}>
  {#if editOpen && editing}
    {#if editing.kind === 'visit'}<VisitForm {patientId} visit={editing.record} onsaved={closeEdit} oncancel={closeEdit} />
    {:else}<HistoryRecordForm record={editing.record} onsaved={closeEdit} oncancel={() => editOpen = false} />{/if}
  {/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={deleteOpen} title={tr.deleteHistory} description={tr.deleteHistoryDescription}>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#snippet footer()}<Button variant="secondary" disabled={busy} onclick={() => deleteOpen = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
{#if undos.some(u => u.open)}
  <div class="fixed right-6 bottom-6 z-30 grid max-h-[40dvh] max-w-[calc(100vw-3rem)] gap-2 overflow-y-auto p-1">
    {#each undos as item (item.key)}<Toast bind:open={item.open} inline message={`${tr.recordDeleted} ${summary(item.entry)}`}>
      {#snippet action()}<Button variant="secondary" size="sm" disabled={busy} onclick={() => undo(item)}>{tr.undo}</Button>{/snippet}
    </Toast>{/each}
  </div>
{/if}
