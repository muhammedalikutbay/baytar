<script lang="ts">
  import { ClipboardList, Stethoscope, ClipboardCheck, HeartPulse, StickyNote, Pill, Pipette, Repeat, Hourglass, List } from '@lucide/svelte';
  import { validateForm } from '$lib/validation';
  import { onMount, untrack, tick } from 'svelte';
  import { Button, Input, Textarea, Select, DateInput, NumberInput, Dialog, RequiredNote, TimeInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { isValidDate, localToday, parseDecimal } from '$lib/format';
  import * as visits from '$lib/api/visits';
  import * as prescriptions from '$lib/api/prescriptions';
  import * as medications from '$lib/api/medications';
  import type { Visit, Medication, PrescriptionItem, Patient, VisitTemplate, AttachmentKind } from '$lib/api/types';
  import * as patients from '$lib/api/patients';
  import * as attachments from '$lib/api/attachments';
  import VitalFields from './VitalFields.svelte';
  import TemplatePicker from './TemplatePicker.svelte';
  import PatientAttachments from '$lib/components/patients/PatientAttachments.svelte';
  import VisitCharges from '$lib/components/billing/VisitCharges.svelte';
  import DoseCalculator from '$lib/components/medications/DoseCalculator.svelte';
  import { formatUnits } from '$lib/format';
  import StockDeduction from '$lib/components/inventory/StockDeduction.svelte';

  let { patientId, visit, onsaved, oncancel }: {
    patientId: number; visit?: Visit; onsaved: (visit: Visit) => void; oncancel: () => void;
  } = $props();
  const initial = untrack(() => visit);
  const uid = $props.id();
  let savedVisit = initial;
  let visitOn = $state(initial?.visitOn ?? localToday());
  let visitTime = $state(initial?.visitTime ?? '');
  let reason = $state(initial?.reason ?? '');
  let findings = $state(initial?.findings ?? '');
  let diagnosis = $state(initial?.diagnosis ?? '');
  let treatment = $state(initial?.treatment ?? '');
  let weight = $state(String(initial?.weightKg ?? '').replace('.', ','));
  let temperature = $state(String(initial?.temperatureC ?? '').replace('.', ','));
  let notes = $state(initial?.notes ?? '');
  let heartRate = $state<number | null>(initial?.heartRateBpm ?? null);
  let respRate = $state<number | null>(initial?.respRateBpm ?? null);
  let bodyCondition = $state<number | null>(initial?.bodyCondition ?? null);
  let patient = $state<Patient | null>(null);
  let pendingTemplate = $state<VisitTemplate | null>(null);
  let overwriteOpen = $state(false);
  let fileKind = $state<AttachmentKind>('document');
  let pendingFiles = $state<{ path: string; kind: AttachmentKind }[]>([]);
  let fileError = $state('');
  function applyTemplate(template: VisitTemplate, overwrite: boolean) {
    if (template.reason?.trim() && (overwrite || !reason.trim())) reason = template.reason;
    if (template.findings?.trim() && (overwrite || !findings.trim())) findings = template.findings;
    if (template.diagnosis?.trim() && (overwrite || !diagnosis.trim())) diagnosis = template.diagnosis;
    if (template.treatment?.trim() && (overwrite || !treatment.trim())) treatment = template.treatment;
    if (template.notes?.trim() && (overwrite || !notes.trim())) notes = template.notes;
    overwriteOpen = false; pendingTemplate = null;
  }
  function chooseTemplate(template: VisitTemplate) {
    if ([[template.reason, reason], [template.findings, findings], [template.diagnosis, diagnosis], [template.treatment, treatment], [template.notes, notes]].some(([source, target]) => source?.trim() && target?.trim())) {
      pendingTemplate = template; overwriteOpen = true;
    } else applyTemplate(template, false);
  }
  async function pickFiles() {
    fileError = '';
    try { const paths = await attachments.pickPaths(); pendingFiles = [...pendingFiles, ...paths.map(path => ({ path, kind: fileKind }))]; }
    catch { fileError = tr.clinicalFileError; }
  }
  interface Line {
    key: number; id?: number; medicationId: string; name: string; doseMg: string; doseMl: string;
    doseText: string; frequency: string; duration: string; notes: string;
    /** True for lines created in this form: stock deduction applies until it succeeds. */
    fresh: boolean;
  }
  let nextKey = 0;
  let stockRefs = $state<Record<number, { consumeAfterSave: (table: 'vaccinations' | 'prescription_items', id: number) => Promise<void> } | undefined>>({});
  let lines = $state<Line[]>([]);
  let doseLineKey = $state<number | null>(null);
  let removed: number[] = [];
  let catalog = $state<Medication[]>([]);
  let loading = $state(true);
  let loadFailed = $state(false);
  let saving = $state(false);
  let errors = $state<Record<string, string>>({});
  let partial = $state(false);
  let form: HTMLFormElement;
  function draft(item?: PrescriptionItem): Line {
    return { key: nextKey++, id: item?.id, medicationId: String(item?.medicationId ?? ''), name: item?.medicationName ?? '',
      doseMg: String(item?.doseMg ?? '').replace('.', ','), doseMl: String(item?.doseMl ?? '').replace('.', ','),
      doseText: item?.doseText ?? '', frequency: item?.frequency ?? '', duration: String(item?.durationDays ?? ''), notes: item?.notes ?? '', fresh: !item };
  }
  async function load() {
    loading = true; loadFailed = false;
    try {
      const [list, items, pet] = await Promise.all([medications.list(), initial ? prescriptions.listByVisit(initial.id) : Promise.resolve([]), patients.get(patientId)]);
      catalog = list; lines = items.map(draft); patient = pet;
    } catch { loadFailed = true; } finally { loading = false; }
  }
  onMount(() => { void load(); });
  function selectMedication(line: Line, event: Event) {
    line.medicationId = (event.currentTarget as HTMLSelectElement).value;
    const medication = catalog.find(m => String(m.id) === line.medicationId);
    if (medication) line.name = medication.name;
  }
  function removeLine(line: Line) {
    if (doseLineKey === line.key) doseLineKey = null;
    if (line.id !== undefined) removed.push(line.id);
    lines = lines.filter(l => l.key !== line.key);
  }
  function checkNumber(key: string, value: string, positive = true, integer = false) {
    const parsed = parseDecimal(value);
    if (parsed !== null && (!Number.isFinite(parsed) || (positive && parsed <= 0) || (integer && !Number.isSafeInteger(parsed))))
      errors[key] = integer ? tr.integerDays : positive ? tr.positiveNumber : tr.invalidField;
  }
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return;
    if (saving || loading || loadFailed) return;
    errors = {};
    if (!isValidDate(visitOn)) errors.visitOn = tr.validDate;
    if (visitTime && !/^([01]\d|2[0-3]):[0-5]\d$/.test(visitTime)) errors.visitTime = tr.validTime;
    checkNumber('weight', weight); checkNumber('temperature', temperature, false);
    for (const line of lines) {
      if (!line.name.trim()) errors[`${line.key}-name`] = tr.requiredField;
      checkNumber(`${line.key}-mg`, line.doseMg); checkNumber(`${line.key}-ml`, line.doseMl);
      checkNumber(`${line.key}-duration`, line.duration, true, true);
    }
    if (Object.keys(errors).length) { await tick(); form.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    saving = true;
    try {
      const input = { patientId, visitOn, visitTime: visitTime || null, reason: reason.trim() || null,
        findings: findings.trim() || null, diagnosis: diagnosis.trim() || null, treatment: treatment.trim() || null,
        weightKg: parseDecimal(weight), temperatureC: parseDecimal(temperature), heartRateBpm: heartRate, respRateBpm: respRate, bodyCondition, notes: notes.trim() || null };
      savedVisit = savedVisit ? await visits.update(savedVisit.id, input) : await visits.create(input);
      // Keep returned IDs immediately: a retry after an interrupted multi-command save must not duplicate records.
      partial = true;
      for (const line of lines) {
        const input = { visitId: savedVisit.id, medicationId: line.medicationId ? Number(line.medicationId) : null,
          medicationName: line.name.trim(), doseMg: parseDecimal(line.doseMg), doseMl: parseDecimal(line.doseMl),
          doseText: line.doseText.trim() || null, frequency: line.frequency.trim() || null,
          durationDays: parseDecimal(line.duration), notes: line.notes.trim() || null };
        const saved = line.id === undefined ? await prescriptions.create(input) : await prescriptions.update(line.id, input);
        line.id = saved.id;
        if (line.fresh) {
          await stockRefs[line.key]?.consumeAfterSave('prescription_items', saved.id);
          line.fresh = false;
        }
      }
      while (removed.length) { await prescriptions.softDelete(removed[0]); removed.shift(); }
      while (pendingFiles.length) {
        const file = pendingFiles[0];
        await attachments.addFromPath(patientId, savedVisit.id, file.kind, file.path);
        pendingFiles = pendingFiles.slice(1);
      }
      partial = false; onsaved(savedVisit);
    } catch { errors.form = partial ? tr.partialVisitSave : tr.saveError; }
    finally { saving = false; }
  }
</script>

<form bind:this={form} onsubmit={submit} novalidate class="grid gap-4">
  <RequiredNote />
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    <div class="grid gap-4 sm:grid-cols-2">
      <DateInput label={tr.date} bind:value={visitOn} required error={errors.visitOn} />
      <TimeInput label={tr.time} optional bind:value={visitTime} error={errors.visitTime} />
    </div>
    <TemplatePicker onselect={chooseTemplate} />
    <section class="grid gap-3 border-t border-border pt-4"><h3 class="type-section text-accent">{tr.clinicalSoapSubjective}</h3><Textarea icon={ClipboardList} label={tr.reason} bind:value={reason} /></section>
    <section class="grid gap-3 border-t border-border pt-4"><h3 class="type-section text-accent">{tr.clinicalSoapObjective}</h3>
      <VitalFields species={patient?.species} bind:weightKg={() => parseDecimal(weight), n => weight = n === null ? '' : String(n)} bind:temperatureC={() => parseDecimal(temperature), n => temperature = n === null ? '' : String(n)} bind:heartRateBpm={heartRate} bind:respRateBpm={respRate} bind:bodyCondition />
      <Textarea icon={Stethoscope} label={tr.findings} bind:value={findings} /></section>
    <section class="grid gap-3 border-t border-border pt-4"><h3 class="type-section text-accent">{tr.clinicalSoapAssessment}</h3><Textarea icon={ClipboardCheck} label={tr.diagnosis} bind:value={diagnosis} /></section>
    <section class="grid gap-3 border-t border-border pt-4"><h3 class="type-section text-accent">{tr.clinicalSoapPlan}</h3><Textarea icon={HeartPulse} label={tr.treatment} bind:value={treatment} /></section>
    <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
    <section class="grid gap-4 border-t border-border pt-4" aria-label={tr.prescription}>
      <h3 class="type-section">{tr.prescription}</h3>
      {#if loading}<p role="status">{tr.loading}</p>
      {:else if loadFailed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>
      {:else}
        {#each lines as line (line.key)}
          <fieldset class="grid min-w-0 gap-4 rounded-md border border-border p-4">
            <legend class="type-caption px-1">{tr.medicationName} {lines.indexOf(line) + 1}</legend>
            <Select icon={Pill} label={tr.medicationCatalog} bind:value={line.medicationId} onchange={event => selectMedication(line, event)} options={[
              { value: '', label: tr.freeMedication },
              ...(!catalog.some(m => String(m.id) === line.medicationId) && line.medicationId ? [{ value: line.medicationId, label: line.name }] : []),
              ...catalog.map(m => ({ value: String(m.id), label: m.name }))
            ]} />
            <Input icon={Pill} id={`${uid}-${line.key}-name`} label={tr.medicationName} bind:value={line.name} readonly={!!line.medicationId} required error={errors[`${line.key}-name`]} />
            <div class="grid gap-4 sm:grid-cols-2">
              <NumberInput icon={Pill} label={tr.doseMg} bind:value={() => parseDecimal(line.doseMg), n => line.doseMg = n === null ? '' : String(n)} error={errors[`${line.key}-mg`]} min={Number.MIN_VALUE} unit={'mg'} />
              <NumberInput icon={Pipette} label={tr.doseMl} bind:value={() => parseDecimal(line.doseMl), n => line.doseMl = n === null ? '' : String(n)} error={errors[`${line.key}-ml`]} min={Number.MIN_VALUE} unit={'ml'} />
            </div>
            <Input icon={Pill} label={tr.doseText} bind:value={line.doseText} />
            <Button variant="ghost" size="sm" onclick={() => doseLineKey = doseLineKey === line.key ? null : line.key}>{tr.doseFromLine}</Button>
            {#if doseLineKey === line.key}
              <DoseCalculator patientId={patientId} medications={catalog}
                initialMedicationId={line.medicationId ? Number(line.medicationId) : undefined}
                initialWeightKg={parseDecimal(weight)} onapply={dose => {
                  line.doseMg = String(dose.mg).replace('.', ',');
                  line.doseMl = dose.ml === null ? '' : String(dose.ml).replace('.', ',');
                  if (dose.units !== null) line.doseText = `${formatUnits(dose.units)} ${dose.unit}`;
                  doseLineKey = null;
                }} />
            {/if}
            <Input icon={Repeat} label={tr.frequency} bind:value={line.frequency} hint={tr.frequencyHint} />
            <NumberInput icon={Hourglass} label={tr.durationDays} bind:value={() => parseDecimal(line.duration), n => line.duration = n === null ? '' : String(n)} error={errors[`${line.key}-duration`]} min={1} max={2147483647} step={1} unit={tr.inputDayUnit} />
            <Input icon={StickyNote} label={tr.notes} bind:value={line.notes} />
            {#if line.fresh && line.medicationId}<StockDeduction bind:this={stockRefs[line.key]} medicationId={Number(line.medicationId)} />{/if}
            <Button variant="secondary" size="sm" onclick={() => removeLine(line)}>{tr.removeLine}</Button>
          </fieldset>
        {/each}
        <Button variant="secondary" onclick={() => lines = [...lines, draft()]}>{tr.addPrescription}</Button>
      {/if}
    </section>
    <section class="grid gap-3 border-t border-border pt-4" aria-label={tr.clinicalFiles}>
      <h3 class="type-section text-accent">{tr.clinicalFiles}</h3>
      {#if initial}<PatientAttachments {patientId} visitId={initial.id} allowDrop={false} />{/if}
      <Select icon={List} label={tr.clinicalKind} bind:value={fileKind} options={[
        { value: 'document', label: tr.clinicalKindDocument }, { value: 'photo', label: tr.clinicalKindPhoto }, { value: 'lab', label: tr.clinicalKindLab }, { value: 'imaging', label: tr.clinicalKindImaging }, { value: 'other', label: tr.clinicalKindOther }
      ]} />
      <p class="type-caption text-fg-muted">{tr.clinicalVisitFilesHint} {tr.clinicalFileLimit}</p>
      <Button variant="secondary" onclick={pickFiles}>{tr.clinicalAddFiles}</Button>
      {#if pendingFiles.length}<ul aria-label={tr.clinicalPendingFiles} class="grid gap-2">{#each pendingFiles as file, index}<li class="flex items-center justify-between gap-2"><span class="truncate">{tr.clinicalPendingFiles} {index + 1}</span><Button variant="ghost" size="sm" onclick={() => pendingFiles = pendingFiles.filter((_, i) => i !== index)}>{tr.removeLine}</Button></li>{/each}</ul>{/if}
      {#if fileError}<p role="alert" class="text-danger">{fileError}</p>{/if}
    </section>
  </fieldset>
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2">
    <Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button>
    <Button type="submit" disabled={saving || loading || loadFailed}>{saving ? tr.saving : tr.save}</Button>
  </div>
</form>

{#if initial}<div class="mt-6"><VisitCharges visitId={initial.id} {patientId} date={initial.visitOn} /></div>{:else}<p class="type-caption mt-4 text-fg-muted">{tr.billingSaveVisitFirst}</p>{/if}
<Dialog bind:open={overwriteOpen} title={tr.clinicalTemplateOverwrite} description={tr.clinicalTemplateOverwriteDescription}>
  {#snippet footer()}<Button variant="secondary" onclick={() => overwriteOpen = false}>{tr.cancel}</Button><Button variant="secondary" onclick={() => { if (pendingTemplate) applyTemplate(pendingTemplate, false); }}>{tr.clinicalFillEmpty}</Button><Button onclick={() => { if (pendingTemplate) applyTemplate(pendingTemplate, true); }}>{tr.clinicalOverwrite}</Button>{/snippet}
</Dialog>
