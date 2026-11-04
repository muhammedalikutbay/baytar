<script lang="ts">
  import { untrack } from 'svelte';
  import { Card, Input, Select, Button } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatDate, formatNumber, formatUnits, parseDecimal } from '$lib/format';
  import * as medicationApi from '$lib/api/medications';
  import { latestWeight } from '$lib/api/visits';
  import { calculate } from '$lib/api/dose';
  import type { DoseInput, DoseRange, DoseResult, LatestWeight, Medication } from '$lib/api/types';

  // Supply patientId from a visit; medications may be supplied by the host or loaded here.
  let { patientId, medications: supplied, initialMedicationId, initialWeightKg, onapply }: {
    patientId?: number; medications?: Medication[]; initialMedicationId?: number; initialWeightKg?: number | null;
    onapply?: (dose: { mg: number; ml: number | null; units: number | null; unit: string }) => void;
  } = $props();
  let loaded = $state<Medication[]>([]);
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
    const id = patientId; weightReload;
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
    const value = input; const invalid = Object.keys(errors).length > 0;
    const ready = weight.trim() && minimum.trim() && !invalid && !weightLoading;
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
    <Select label={tr.doseMedication} bind:value={selectedId} options={[{ value: '', label: tr.doseManual }, ...catalog.map(m => ({ value: String(m.id), label: m.name }))]} />
    {#if loadError}<div><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" size="sm" onclick={() => reload++}>{tr.retry}</Button></div>{/if}
    <Input label={tr.doseWeight} inputmode="decimal" bind:value={weight} disabled={weightLoading} error={errors.weight} hint={recorded ? `${tr.doseWeightDate}: ${formatDate(recorded.visitOn)} · ${formatNumber(recorded.weightKg)} kg` : undefined} />
    {#if weightLoading}<p role="status" class="text-fg-muted">{tr.loading}</p>
    {:else if weightError}<div><p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" size="sm" onclick={() => weightReload++}>{tr.retry}</Button></div>
    {:else if patientId !== undefined && !recorded}<p class="type-caption text-fg-muted">{tr.doseNoWeight}</p>{/if}
    <div class="grid gap-4 sm:grid-cols-2">
      <Input label={tr.doseMinimum} inputmode="decimal" bind:value={minimum} error={errors.minimum} />
      <Input label={tr.doseMaximum} inputmode="decimal" bind:value={maximum} error={errors.maximum} />
      <Input label={tr.medicationConcentration} inputmode="decimal" bind:value={concentration} error={errors.concentration} />
      <Input label={tr.medicationStrength} inputmode="decimal" bind:value={strength} error={errors.strength} />
    </div>
    <div role="status" aria-live="polite" aria-atomic="true" class="border-t border-border pt-4 tabular-nums">
      {#if calculating}<p class="text-fg-muted">{tr.loading}</p>
      {:else if calculationError}<p class="text-danger">{tr.doseError}</p><Button variant="secondary" size="sm" onclick={() => retry++}>{tr.retry}</Button>
      {:else if result}
        <p class="type-title">{range(result.mg)} mg{#if result.ml}{' → '}{range(result.ml)} ml{/if}{#if result.units}{' → '}{range(result.units, true)} {unit}{/if}</p>
        {#each result.warnings as warning}<p class="mt-2 text-highlight">{warning === 'verySmallVolume' ? tr.doseSmallVolume : tr.doseImpracticalUnits}</p>{/each}
        {#if roundsToZero}<p class="mt-2 text-highlight">{tr.doseTinyRounded}</p>{/if}
      {:else}<p class="text-fg-muted">{tr.dosePrompt}</p>{/if}
    </div>
    <p class="type-caption text-fg-muted">{tr.doseRounded}</p>
    {#if onapply}<Button disabled={!result || calculating} onclick={applyDose}>{tr.doseApply}</Button>{/if}
    {#if result}<details class="type-caption text-fg-muted"><summary class="cursor-pointer">{tr.doseExact}</summary><p class="mt-2 break-all tabular-nums">{exact(result.mg)} mg{#if result.ml}{' → '}{exact(result.ml)} ml{/if}{#if result.units}{' → '}{exact(result.units)} {unit}{/if}</p></details>{/if}
  </div>
</Card>
