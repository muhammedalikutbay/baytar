<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Button, Input, Textarea } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, daysUntil, dateAfterMonths } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as vaccineTypes from '$lib/api/vaccine_types';
  import type { Vaccination, VaccineType, ApiError } from '$lib/api/types';
  let { patientId, vaccineName = '', vaccineTypeId = null, onsaved, oncancel }: {
    patientId: number; vaccineName?: string; vaccineTypeId?: number | null;
    onsaved: (record: Vaccination) => void; oncancel: () => void;
  } = $props();
  const initial = untrack(() => ({ vaccineName, vaccineTypeId }));
  const uid = $props.id();
  let name = $state(initial.vaccineName);
  let givenOn = $state(todayDate());
  let nextDueOn = $state('');
  let batchNo = $state('');
  let notes = $state('');
  let types = $state<VaccineType[]>([]);
  let loaded = $state(false);
  let loadError = $state(false);
  let suggestionError = $state(false);
  let suggesting = $state(false);
  let saving = $state(false);
  let errors = $state<Record<string, string>>({});
  let first = $state<HTMLInputElement | null>(null);
  let revision = 0;
  const selected = $derived(types.find(type => type.name.trim().toLocaleLowerCase('tr-TR') === name.trim().toLocaleLowerCase('tr-TR'))
    ?? (name === initial.vaccineName ? types.find(type => type.id === initial.vaccineTypeId) : undefined));
  async function load() {
    loadError = false;
    try { types = (await vaccineTypes.list()).filter(type => type.isActive || type.id === initial.vaccineTypeId); loaded = true; }
    catch { loadError = true; }
  }
  onMount(() => { first?.focus(); void load(); });
  $effect(() => {
    const type = selected, date = givenOn;
    if (!loaded) return;
    const request = ++revision;
    suggestionError = false;
    suggesting = false;
    if (!type || daysUntil(date) === null) { nextDueOn = ''; return; }
    nextDueOn = '';
    suggesting = true;
    vaccinations.suggestNextDue(type.id, date).then(value => {
      if (request === revision) nextDueOn = value ?? '';
    }).catch(() => { if (request === revision) suggestionError = true; })
      .finally(() => { if (request === revision) suggesting = false; });
    return () => { revision++; };
  });
  function manualDate() { revision++; suggestionError = false; suggesting = false; }
  function quick(months: number) { manualDate(); nextDueOn = dateAfterMonths(givenOn, months); }
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving || suggesting || (!loaded && !loadError)) return;
    errors = {};
    if (!name.trim()) errors.vaccineName = tr.requiredField;
    if (daysUntil(givenOn) === null || givenOn > todayDate()) errors.givenOn = tr.invalidDate;
    if (nextDueOn && daysUntil(nextDueOn) === null) errors.nextDueOn = tr.invalidField;
    if (Object.keys(errors).length) return;
    saving = true;
    try {
      onsaved(await vaccinations.create({ patientId, vaccineTypeId: selected?.id ?? null,
        vaccineName: name.trim(), givenOn, nextDueOn: nextDueOn || null,
        batchNo: batchNo.trim() || null, notes: notes.trim() || null }));
    } catch (error) {
      const e = error as ApiError;
      errors = e?.field && ['vaccineName', 'givenOn', 'nextDueOn', 'batchNo', 'notes'].includes(e.field)
        ? { [e.field]: tr.invalidField } : { form: tr.saveError };
    } finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4" novalidate>
  <Input label={tr.vaccineName + ' *'} hint={tr.vaccineNameHint} list={uid} bind:value={name} bind:ref={first} required error={errors.vaccineName} />
  <datalist id={uid}>{#each types as type (type.id)}<option value={type.name}></option>{/each}</datalist>
  {#if !loaded && !loadError}<p role="status" class="text-fg-muted">{tr.loading}</p>{/if}
  {#if loadError}<p role="alert" class="text-danger">{tr.loadError}</p><Button size="sm" variant="secondary" onclick={load}>{tr.retry}</Button>{/if}
  <Input label={tr.givenOn + ' *'} type="date" bind:value={givenOn} max={todayDate()} required error={errors.givenOn} />
  <Input label={tr.nextDueOn} type="date" bind:value={nextDueOn} oninput={manualDate} error={errors.nextDueOn} />
  {#if suggesting}<p role="status" class="text-fg-muted">{tr.loading}</p>{/if}
  <div class="flex flex-wrap gap-2">
    <Button size="sm" variant="secondary" onclick={() => quick(1)}>{tr.plusMonth}</Button>
    <Button size="sm" variant="secondary" onclick={() => quick(3)}>{tr.plusThreeMonths}</Button>
    <Button size="sm" variant="secondary" onclick={() => quick(12)}>{tr.plusYear}</Button>
  </div>
  {#if suggestionError}<p role="alert" class="text-danger">{tr.suggestionError}</p>{/if}
  <Input label={tr.batchNo} bind:value={batchNo} error={errors.batchNo} />
  <Textarea label={tr.notes} bind:value={notes} error={errors.notes} />
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || suggesting || (!loaded && !loadError)}>{saving ? tr.saving : tr.save}</Button></div>
</form>
