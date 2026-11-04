<script lang="ts">
  import { onMount, untrack, tick } from 'svelte';
  import { Button, Input, Textarea, Select } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { isValidDate, localToday, parseDecimal } from '$lib/format';
  import * as visits from '$lib/api/visits';
  import * as prescriptions from '$lib/api/prescriptions';
  import * as medications from '$lib/api/medications';
  import type { Visit, Medication, PrescriptionItem } from '$lib/api/types';
  import DoseCalculator from '$lib/components/medications/DoseCalculator.svelte';
  import { formatUnits } from '$lib/format';

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
  interface Line {
    key: number; id?: number; medicationId: string; name: string; doseMg: string; doseMl: string;
    doseText: string; frequency: string; duration: string; notes: string;
  }
  let nextKey = 0;
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
      doseText: item?.doseText ?? '', frequency: item?.frequency ?? '', duration: String(item?.durationDays ?? ''), notes: item?.notes ?? '' };
  }
  async function load() {
    loading = true; loadFailed = false;
    try {
      const [list, items] = await Promise.all([medications.list(), initial ? prescriptions.listByVisit(initial.id) : Promise.resolve([])]);
      catalog = list; lines = items.map(draft);
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
        weightKg: parseDecimal(weight), temperatureC: parseDecimal(temperature), notes: notes.trim() || null };
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
      }
      while (removed.length) { await prescriptions.softDelete(removed[0]); removed.shift(); }
      partial = false; onsaved(savedVisit);
    } catch { errors.form = partial ? tr.partialVisitSave : tr.saveError; }
    finally { saving = false; }
  }
</script>

<form bind:this={form} onsubmit={submit} novalidate class="grid gap-4">
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    <div class="grid gap-4 sm:grid-cols-2">
      <Input label={tr.date + ' *'} type="date" bind:value={visitOn} required error={errors.visitOn} />
      <Input label={tr.time} type="time" bind:value={visitTime} error={errors.visitTime} />
    </div>
    <Textarea label={tr.reason} bind:value={reason} />
    <Textarea label={tr.findings} bind:value={findings} />
    <Textarea label={tr.diagnosis} bind:value={diagnosis} />
    <Textarea label={tr.treatment} bind:value={treatment} />
    <div class="grid gap-4 sm:grid-cols-2">
      <Input label={tr.weight} inputmode="decimal" bind:value={weight} error={errors.weight} />
      <Input label={tr.temperature} inputmode="decimal" bind:value={temperature} error={errors.temperature} />
    </div>
    <Textarea label={tr.notes} bind:value={notes} />
    <section class="grid gap-4 border-t border-border pt-4" aria-label={tr.prescription}>
      <h3 class="type-section">{tr.prescription}</h3>
      {#if loading}<p role="status">{tr.loading}</p>
      {:else if loadFailed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>
      {:else}
        {#each lines as line (line.key)}
          <fieldset class="grid min-w-0 gap-4 rounded-md border border-border p-4">
            <legend class="type-caption px-1">{tr.medicationName} {lines.indexOf(line) + 1}</legend>
            <Select label={tr.medicationCatalog} bind:value={line.medicationId} onchange={event => selectMedication(line, event)} options={[
              { value: '', label: tr.freeMedication },
              ...(!catalog.some(m => String(m.id) === line.medicationId) && line.medicationId ? [{ value: line.medicationId, label: line.name }] : []),
              ...catalog.map(m => ({ value: String(m.id), label: m.name }))
            ]} />
            <Input id={`${uid}-${line.key}-name`} label={tr.medicationName + ' *'} bind:value={line.name} readonly={!!line.medicationId} required error={errors[`${line.key}-name`]} />
            <div class="grid gap-4 sm:grid-cols-2">
              <Input label={tr.doseMg} inputmode="decimal" bind:value={line.doseMg} error={errors[`${line.key}-mg`]} />
              <Input label={tr.doseMl} inputmode="decimal" bind:value={line.doseMl} error={errors[`${line.key}-ml`]} />
            </div>
            <Input label={tr.doseText} bind:value={line.doseText} />
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
            <Input label={tr.frequency} bind:value={line.frequency} hint={tr.frequencyHint} />
            <Input label={tr.durationDays} inputmode="numeric" bind:value={line.duration} error={errors[`${line.key}-duration`]} />
            <Input label={tr.notes} bind:value={line.notes} />
            <Button variant="secondary" size="sm" onclick={() => removeLine(line)}>{tr.removeLine}</Button>
          </fieldset>
        {/each}
        <Button variant="secondary" onclick={() => lines = [...lines, draft()]}>{tr.addPrescription}</Button>
      {/if}
    </section>
  </fieldset>
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2">
    <Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button>
    <Button type="submit" disabled={saving || loading || loadFailed}>{saving ? tr.saving : tr.save}</Button>
  </div>
</form>
