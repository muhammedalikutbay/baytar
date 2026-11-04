<script lang="ts">
  import { validateForm, parseDecimal } from '$lib/validation';
  import { tick, untrack } from 'svelte';
  import { Button, Input, Select, Textarea, NumberInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as types from '$lib/api/vaccine_types';
  import type { VaccineType, Species, ApiError } from '$lib/api/types';
  let { type, onsaved, oncancel }: { type?: VaccineType; onsaved: () => void; oncancel: () => void } = $props();
  const initial = untrack(() => type);
  let name = $state(initial?.name ?? '');
  let species = $state(initial?.species ?? '');
  let interval = $state(String(initial?.defaultIntervalDays ?? ''));
  let notes = $state(initial?.notes ?? '');
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);
  const options = [{ value: '', label: tr.anySpecies }, ...(['cat', 'dog', 'bird', 'rabbit', 'rodent', 'ferret', 'reptile', 'other'] as Species[]).map(value => ({ value, label: tr[value] }))];
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return; if (saving) return;
    errors = {};
    if (!name.trim()) errors.name = tr.requiredField;
    if (interval && (!Number.isSafeInteger(Number(interval)) || Number(interval) <= 0)) errors.defaultIntervalDays = tr.invalidField;
    if (Object.keys(errors).length) {
      await tick();
      (event.target as HTMLFormElement).querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
      return;
    }
    saving = true;
    try {
      const input = { name: name.trim(), species: (species || null) as Species | null, defaultIntervalDays: interval ? Number(interval) : null, notes: notes.trim() || null };
      if (initial) await types.update(initial.id, input); else await types.create(input);
      onsaved();
    } catch (error) {
      const e = error as ApiError;
      errors = e?.kind === 'conflict' ? { name: tr.vaccineDuplicate } : e?.field ? { [e.field]: tr.invalidField } : { form: tr.saveError };
    } finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4" novalidate>
  <Input label={tr.vaccineTypeName + ' *'} bind:value={name} required error={errors.name} />
  <Select label={tr.species} bind:value={species} {options} error={errors.species} />
  <NumberInput label={tr.vaccineInterval} bind:value={() => parseDecimal(interval), n => interval = n === null ? '' : String(n)} error={errors.defaultIntervalDays} min={1} max={2147483647} step={1} unit={tr.inputDayUnit} />
  <Textarea label={tr.notes} bind:value={notes} error={errors.notes} />
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
