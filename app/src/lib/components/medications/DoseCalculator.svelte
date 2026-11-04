<script lang="ts">
  import { untrack } from 'svelte';
  import { Card, Combobox, Checkbox, Button, NumberInput } from '$lib/components/ui';
  import RecordPicker from '$lib/components/pickers/RecordPicker.svelte';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatNumber, formatUnits, parseDecimal } from '$lib/format';
  import * as medicationApi from '$lib/api/medications';
  import { latestWeight } from '$lib/api/visits';
  import { calculate } from '$lib/api/dose';
  import type { DoseInput, DoseRange, DoseResult, LatestWeight, Medication, RecordSelection } from '$lib/api/types';

  // Supply patientId from a visit; medications may be supplied by the host or loaded here.
  let { patientId, medications: supplied, initialMedicationId, initialWeightKg, onapply }: {
    patientId?: number; medications?: Medication[]; initialMedicationId?: number; initialWeightKg?: number | null;
    onapply?: (dose: { mg: number; ml: number | null; units: number | null; unit: string }) => void;
  } = $props();
  let loaded = $state<Medication[]>([]);
  let patient = $state<RecordSelection | null>(null);
  const effectivePatientId = $derived(patientId ?? patient?.patient?.id);
  let confirmed = $state(false);
  let loadError = $state(false);
  let reload = $state(0);
  let weightReload = $state(0);
  let weightError = $state(false);
  let weightLoading = $state(false);
  let recorded = $state<LatestWeight | null>(null);
  let selectedId = $state(untrack(() => initialMedicationId === undefined ? '' : String(initialMedicationId)));
  let weight = $state(untrack(() => initialWeightKg == null ? '' : String(initialWeightKg).replace('.', ',')));
  let minimum = $state('');
  let maximum = $state('');
  let concentration = $state('');
  let strength = $state('');
  let result = $state<DoseResult | null>(null);
  let calculationError = $state(false);
  let calculating = $state(false);
  let retry = $state(0);
  const catalog = $derived((supplied ?? loaded).filter(m => !m.deletedAt));
  const selected = $derived(catalog.find(m => String(m.id) === selectedId));
  const unit = $derived(selected?.form === 'tablet' ? tr.doseTabletUnit : selected?.form === 'capsule' ? tr.doseCapsuleUnit : tr.doseOtherUnit);
  $effect(() => {
    if (supplied !== undefined) return;
    reload; let cancelled = false; loadError = false;
    medicationApi.list().then(rows => { if (!cancelled) loaded = rows; }).catch(() => { if (!cancelled) loadError = true; });
    return () => { cancelled = true; };
  });
  $effect(() => {
    const medication = selected;
    minimum = String(medication?.doseMgPerKgMin ?? '').replace('.', ',');
    maximum = String(medication?.doseMgPerKgMax ?? '').replace('.', ',');
    concentration = String(medication?.concentrationMgPerMl ?? '').replace('.', ',');
    strength = String(medication?.strengthMgPerUnit ?? '').replace('.', ',');
  });
  $effect(() => {
    const id = effectivePatientId; weightReload;
    let cancelled = false;
    weight = initialWeightKg == null ? '' : String(initialWeightKg).replace('.', ','); recorded = null; weightError = false; weightLoading = id !== undefined && initialWeightKg == null;
    if (id !== undefined && initialWeightKg == null) {
      latestWeight(id).then(value => {
        if (!cancelled) { recorded = value; if (value) weight = String(value.weightKg).replace('.', ','); }
      }).catch(() => { if (!cancelled) weightError = true; })
        .finally(() => { if (!cancelled) weightLoading = false; });
    }
    return () => { cancelled = true; };
  });
  function applyDose() {
    if (!result || !onapply) return;
    onapply({ mg: Number(formatNumber(result.mg.min, 2).replace(',', '.')),
      ml: result.ml ? Number(formatNumber(result.ml.min, 2).replace(',', '.')) : null,
      units: result.units ? Number(formatUnits(result.units.min).replace(',', '.')) : null, unit });
  }
  const input = $derived<DoseInput>({ weightKg: parseDecimal(weight) ?? NaN, mgPerKgMin: parseDecimal(minimum) ?? NaN,
    mgPerKgMax: parseDecimal(maximum), concentrationMgPerMl: parseDecimal(concentration), strengthMgPerUnit: parseDecimal(strength) });
  const errors = $derived.by(() => {
    const e: Record<string, string> = {};
    if (weight.trim() && (!Number.isFinite(input.weightKg) || input.weightKg <= 0 || input.weightKg > 1500)) e.weight = tr.doseWeightError;
    for (const [key, text, value] of [
      ['minimum', minimum, input.mgPerKgMin], ['maximum', maximum, input.mgPerKgMax],
      ['concentration', concentration, input.concentrationMgPerMl], ['strength', strength, input.strengthMgPerUnit]
    ] as const) {
      if (text.trim() && value !== null && (!Number.isFinite(value) || value <= 0)) e[key] = tr.dosePositiveError;
    }
    if (input.mgPerKgMax !== null && input.mgPerKgMax < input.mgPerKgMin) e.maximum = tr.doseRangeError;
    return e;
  });
  $effect(() => {
    selectedId; minimum; maximum; weight; concentration; strength; effectivePatientId;
    confirmed = false;
  });
  $effect(() => {
    const value = input; const invalid = Object.keys(errors).length > 0;
    const ready = confirmed && weight.trim() && minimum.trim() && !invalid && !weightLoading;
    retry; let cancelled = false;
    result = null; calculationError = false; calculating = Boolean(ready);
    if (ready) {
      calculate(value).then(value => { if (!cancelled) result = value; })
        .catch(() => { if (!cancelled) calculationError = true; })
        .finally(() => { if (!cancelled) calculating = false; });
    }
    return () => { cancelled = true; };
  });
  function range(value: DoseRange, units = false): string {
    const format = units ? formatUnits : formatNumber;
    return format(value.min) + (value.max === null ? '' : '–' + format(value.max));
  }
  function exact(value: DoseRange): string {
    const format = (v: number) => new Intl.NumberFormat('tr-TR', { maximumSignificantDigits: 21, useGrouping: false }).format(v);
    return format(value.min) + (value.max === null ? '' : '–' + format(value.max));
  }
  const roundsToZero = $derived(result && [result.mg, result.ml, result.units].some((r, index) => r && (r.min < (index === 2 ? 0.125 : 0.005))));
</script>

<Card>
  <div class="grid gap-4">
    <h2 class="type-section">{tr.doseTitle}</h2>
    <p class="text-fg-muted">{tr.doseDisclaimer}</p>
    <Combobox label={tr.doseMedication} bind:value={selectedId} placeholder={tr.doseManual} items={catalog.map(m => ({ value: String(m.id), label: m.name, hint: m.activeIngredient ?? undefined }))} />
    {#if patientId === undefined}<fieldset class="grid gap-2"><legend class="mb-2 text-sm font-medium">{tr.dosePatient}</legend><RecordPicker bind:selected={patient} /></fieldset>{/if}
    {#if loadError}<div><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" size="sm" onclick={() => reload++}>{tr.retry}</Button></div>{/if}
    <NumberInput label={tr.doseWeight} bind:value={() => parseDecimal(weight), n => weight = n === null ? '' : String(n)} disabled={weightLoading} error={errors.weight} hint={recorded ? `${tr.doseWeightDate}: ${formatDate(recorded.visitOn)} · ${formatNumber(recorded.weightKg)} kg` : undefined} min={Number.MIN_VALUE} max={1500} unit={'kg'} />
    {#if weightLoading}<p role="status" class="text-fg-muted">{tr.loading}</p>
    {:else if weightError}<div><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" size="sm" onclick={() => weightReload++}>{tr.retry}</Button></div>
    {:else if effectivePatientId !== undefined && !recorded}<p class="type-caption text-fg-muted">{tr.doseNoWeight}</p>{/if}
    <div class="grid gap-4 sm:grid-cols-2">
      <NumberInput label={tr.doseMinimum} bind:value={() => parseDecimal(minimum), n => minimum = n === null ? '' : String(n)} error={errors.minimum} min={Number.MIN_VALUE} unit={'mg/kg'} />
      <NumberInput label={tr.doseMaximum} bind:value={() => parseDecimal(maximum), n => maximum = n === null ? '' : String(n)} error={errors.maximum} min={Number.MIN_VALUE} unit={'mg/kg'} />
      <NumberInput label={tr.medicationConcentration} bind:value={() => parseDecimal(concentration), n => concentration = n === null ? '' : String(n)} error={errors.concentration} min={Number.MIN_VALUE} unit={'mg/ml'} />
      <NumberInput label={tr.medicationStrength} bind:value={() => parseDecimal(strength), n => strength = n === null ? '' : String(n)} error={errors.strength} min={Number.MIN_VALUE} unit={'mg'} />
    </div>
    <Checkbox label={tr.doseConfirmRange} bind:checked={confirmed} />
    <div role="status" aria-live="polite" aria-atomic="true" class="rounded-lg border border-border-control bg-surface-muted p-6 tabular-nums">
      {#if calculating}<p class="text-fg-muted">{tr.loading}</p>
      {:else if calculationError}<p class="text-danger">{tr.doseError}</p><Button variant="secondary" size="sm" onclick={() => retry++}>{tr.retry}</Button>
      {:else if result}
        <p class="type-caption mb-2 text-fg-muted">{tr.dosePerDose}</p>
        <p class="type-page">{range(result.mg)} mg{#if result.ml}{' → '}{range(result.ml)} ml{/if}{#if result.units}{' → '}{range(result.units, true)} {unit}{/if}</p>
        <p class="mt-3 text-sm text-fg-muted">{tr.doseUsedRange}: {formatNumber(input.mgPerKgMin)}{input.mgPerKgMax === null ? '' : '–' + formatNumber(input.mgPerKgMax)} mg/kg</p>
        <p class="type-caption mt-2 text-fg-muted">{tr.doseRounded}</p>
        {#each result.warnings as warning}<p class="mt-2 text-highlight">{warning === 'verySmallVolume' ? tr.doseSmallVolume : tr.doseImpracticalUnits}</p>{/each}
        {#if roundsToZero}<p class="mt-2 text-highlight">{tr.doseTinyRounded}</p>{/if}
      {:else}<p class="text-fg-muted">{tr.dosePrompt}</p>{/if}
    </div>
    {#if onapply}<Button disabled={!result || calculating} onclick={applyDose}>{tr.doseApply}</Button>{/if}
    {#if result}<details class="type-caption text-fg-muted"><summary class="cursor-pointer">{tr.doseExact}</summary><p class="mt-2 break-all tabular-nums">{exact(result.mg)} mg{#if result.ml}{' → '}{exact(result.ml)} ml{/if}{#if result.units}{' → '}{exact(result.units)} {unit}{/if}</p></details>{/if}
  </div>
</Card>
