<script lang="ts">
  import { PawPrint, Dna, Palette, ScanLine, TriangleAlert, StickyNote } from '@lucide/svelte';
  import { validateForm } from '$lib/validation';
  import { onMount, untrack, tick } from 'svelte';
  import { Button, Input, Textarea, Select, Checkbox, DateInput, MicrochipInput, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as patients from '$lib/api/patients';
  import type { Patient, Species, Sex, ApiError } from '$lib/api/types';
  let { patient, ownerId, onsaved, oncancel }: { patient?: Patient; ownerId: number; onsaved: (patient: Patient) => void; oncancel: () => void } = $props();
  // Each dialog mounts a fresh draft so refreshes cannot overwrite typing.
  const initial = untrack(() => patient);
  let name = $state(initial?.name ?? '');
  let species = $state(initial?.species ?? '');
  let speciesNote = $state(initial?.speciesNote ?? '');
  let breed = $state(initial?.breed ?? '');
  let sex = $state(initial?.sex ?? 'unknown');
  let neutered = $state(initial?.neutered == null ? 'unknown' : String(initial.neutered));
  let birthDate = $state(initial?.birthDate ?? '');
  let estimate = $state(initial?.birthDateIsEstimate ?? false);
  let color = $state(initial?.color ?? '');
  let microchip = $state(initial?.microchip ?? '');
  let notes = $state(initial?.notes ?? '');
  let alerts = $state(initial?.alerts ?? '');
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);
  let first = $state<HTMLInputElement | null>(null);
  const speciesOptions = (['cat', 'dog', 'bird', 'rabbit', 'rodent', 'ferret', 'reptile', 'other'] as Species[]).map(value => ({ value, label: tr[value] }));
  const sexOptions = ['male', 'female', 'unknown'].map(value => ({ value, label: tr[value as Sex] }));
  onMount(() => first?.focus());
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return; if (saving) return; errors = {};
    if (!name.trim()) errors.name = tr.requiredField;
    if (!species) errors.species = tr.requiredField;
    if (birthDate && (birthDate > new Date().toLocaleDateString('sv-SE') || !Number.isFinite(Date.parse(birthDate)) || new Date(birthDate).toISOString().slice(0, 10) !== birthDate)) errors.birthDate = tr.invalidDate;
    if (Object.keys(errors).length) { await tick(); first?.form?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    saving = true;
    const input = { ownerId, name: name.trim(), species: species as Species, speciesNote: speciesNote.trim() || null,
      breed: breed.trim() || null, sex: sex as Sex, neutered: neutered === 'unknown' ? null : neutered === 'true',
      birthDate: birthDate || null, birthDateIsEstimate: estimate, color: color.trim() || null, microchip: microchip.trim() || null, notes: notes.trim() || null, alerts: alerts.trim() || null };
    try { onsaved(patient ? await patients.update(patient.id, input) : await patients.create(input)); }
    catch (error) { const e = error as ApiError; errors = e?.field && ["name","species","speciesNote","breed","sex","neutered","birthDate","color","microchip","notes","alerts"].includes(e.field) ? { [e.field]: tr.invalidField } : { form: tr.saveError }; }
    finally { saving = false; }
  }
</script>
<form onsubmit={submit} novalidate class="grid gap-4">
  <RequiredNote />
  <Input icon={PawPrint} label={tr.patientName} bind:value={name} bind:ref={first} required error={errors.name} />
  <Select icon={PawPrint} label={tr.species} bind:value={species} options={speciesOptions} placeholder={tr.galleryChoose} required error={errors.species} />
  {#if species === 'other'}<Input icon={PawPrint} label={tr.speciesNote} bind:value={speciesNote} error={errors.speciesNote} />{/if}
  <Input icon={Dna} label={tr.breed} bind:value={breed} error={errors.breed} />
  <div class="grid gap-4 sm:grid-cols-2">
    <Select icon={PawPrint} label={tr.sex} bind:value={sex} options={sexOptions} error={errors.sex} />
    <Select icon={PawPrint} label={tr.neutered} bind:value={neutered} options={[{value:'true',label:tr.yes},{value:'false',label:tr.no},{value:'unknown',label:tr.unknown}]} error={errors.neutered} />
  </div>
  <DateInput label={tr.birthDate} bind:value={birthDate} max={new Date().toLocaleDateString('sv-SE')} hint={tr.birthDateHint} error={errors.birthDate} />
  <Checkbox label={tr.estimate} bind:checked={estimate} />
  <Input icon={Palette} label={tr.color} bind:value={color} error={errors.color} />
  <MicrochipInput icon={ScanLine} label={tr.microchip} bind:value={microchip} error={errors.microchip} />
  <Textarea icon={TriangleAlert} label={tr.clinicalAlerts} bind:value={alerts} placeholder={tr.clinicalAlertsPlaceholder} error={errors.alerts} />
  <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} error={errors.notes} />
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="mt-2 flex justify-end gap-2"><Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
