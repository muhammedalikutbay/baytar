<script lang="ts">
  import { validateForm } from '$lib/validation';
  import { onMount, tick, untrack } from 'svelte';
  import { Button, Input, Select, Textarea, NumberInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { parseDecimal } from '$lib/format';
  import * as medications from '$lib/api/medications';
  import type { ApiError, Medication, MedicationForm, NewMedication } from '$lib/api/types';
  import { formOptions } from './options';
  let { medication, onsaved, oncancel }: { medication?: Medication; onsaved: (value: Medication) => void; oncancel: () => void } = $props();
  const initial = untrack(() => medication);
  let name = $state(initial?.name ?? '');
  let activeIngredient = $state(initial?.activeIngredient ?? '');
  let form = $state(initial?.form ?? '');
  let route = $state(initial?.route ?? '');
  let notes = $state(initial?.notes ?? '');
  const numericFields = [
    { key: 'concentrationMgPerMl', label: tr.medicationConcentration },
    { key: 'strengthMgPerUnit', label: tr.medicationStrength },
    { key: 'doseMgPerKgMin', label: tr.doseMinimum },
    { key: 'doseMgPerKgMax', label: tr.doseMaximum }
  ] as const;
  type NumericKey = typeof numericFields[number]['key'];
  let numbers = $state<Record<NumericKey, string>>({
    concentrationMgPerMl: String(initial?.concentrationMgPerMl ?? '').replace('.', ','),
    strengthMgPerUnit: String(initial?.strengthMgPerUnit ?? '').replace('.', ','),
    doseMgPerKgMin: String(initial?.doseMgPerKgMin ?? '').replace('.', ','),
    doseMgPerKgMax: String(initial?.doseMgPerKgMax ?? '').replace('.', ',')
  });
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);
  let first = $state<HTMLInputElement | null>(null);
  onMount(() => first?.focus());
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return; if (saving) return;
    errors = {};
    if (!name.trim()) errors.name = tr.requiredField;
    const values = Object.fromEntries(numericFields.map(({ key }) => [key, parseDecimal(numbers[key])])) as Record<NumericKey, number | null>;
    for (const { key } of numericFields) {
      const value = values[key];
      if (value !== null && (!Number.isFinite(value) || value <= 0)) errors[key] = tr.dosePositiveError;
    }
    if (values.doseMgPerKgMin !== null && values.doseMgPerKgMax !== null && values.doseMgPerKgMax < values.doseMgPerKgMin) errors.doseMgPerKgMax = tr.doseRangeError;
    if (Object.keys(errors).length) { await tick(); first?.form?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    const input: NewMedication = { name: name.trim(), activeIngredient: activeIngredient.trim() || null,
      form: (form || null) as MedicationForm | null, route: route.trim() || null, notes: notes.trim() || null, ...values };
    saving = true;
    try { onsaved(initial ? await medications.update(initial.id, input) : await medications.create(input)); }
    catch (error) {
      const e = error as ApiError;
      errors = e?.field && ['name', 'activeIngredient', 'form', 'route', 'notes', ...numericFields.map(f => f.key)].includes(e.field)
        ? { [e.field]: tr.invalidField } : { formError: tr.saveError };
    } finally { saving = false; }
  }
</script>

<form onsubmit={submit} novalidate class="grid gap-4">
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    <Input label={tr.medicationName + ' *'} bind:value={name} bind:ref={first} required error={errors.name} />
    <Input label={tr.activeIngredient} bind:value={activeIngredient} error={errors.activeIngredient} />
    <Select label={tr.medicationForm} bind:value={form} options={formOptions} error={errors.form} />
    {#each numericFields as field (field.key)}
      <NumberInput label={field.label} bind:value={() => parseDecimal(numbers[field.key]), n => numbers[field.key] = n === null ? '' : String(n)} error={errors[field.key]} min={Number.MIN_VALUE} unit={field.key === 'concentrationMgPerMl' ? 'mg/ml' : field.key === 'strengthMgPerUnit' ? 'mg' : 'mg/kg'} />
    {/each}
    <Input label={tr.medicationRoute} bind:value={route} error={errors.route} />
    <Textarea label={tr.medicationNote} bind:value={notes} error={errors.notes} />
  </fieldset>
  {#if errors.formError}<p role="alert" class="text-danger">{errors.formError}</p>{/if}
  <div class="mt-2 flex justify-end gap-2">
    <Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button>
    <Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button>
  </div>
</form>
