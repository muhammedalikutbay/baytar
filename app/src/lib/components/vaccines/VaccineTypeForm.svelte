<script lang="ts">
  import { untrack } from 'svelte';
  import { Button, Input, Select, Textarea } from '$lib/components/ui';
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
  const options = [{ value: '', label: tr.anySpecies }, ...(['cat', 'dog', 'bird', 'rabbit', 'other'] as Species[]).map(value => ({ value, label: tr[value] }))];
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving) return;
    errors = {};
    if (!name.trim()) errors.name = tr.requiredField;
    if (interval && (!Number.isSafeInteger(Number(interval)) || Number(interval) <= 0)) errors.defaultIntervalDays = tr.invalidField;
    if (Object.keys(errors).length) return;
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
  <Input label={tr.vaccineInterval} inputmode="numeric" pattern="[0-9]+" bind:value={interval} error={errors.defaultIntervalDays} />
  <Textarea label={tr.notes} bind:value={notes} error={errors.notes} />
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
