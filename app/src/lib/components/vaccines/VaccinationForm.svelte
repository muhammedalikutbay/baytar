<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Button, Input, Textarea, Select, DateInput, Checkbox } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, parseDate } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import { planVaccine } from '$lib/api/appointments';
  import * as vaccineTypes from '$lib/api/vaccine_types';
  import * as patients from '$lib/api/patients';
  import * as owners from '$lib/api/owners';
  import type { VaccineType, Patient, Owner, ApiError } from '$lib/api/types';
  import VaccinePatientPicker from './VaccinePatientPicker.svelte';
  import DateChips from './DateChips.svelte';
  import StockDeduction from '$lib/components/inventory/StockDeduction.svelte';
  let { patientId, vaccineTypeId = null, vaccineName = '', appointmentId = null, mode = 'give', initialDate = todayDate(), onsaved, oncancel }: {
    patientId?: number; vaccineTypeId?: number | null; vaccineName?: string; appointmentId?: number | null;
    mode?: 'give' | 'plan'; initialDate?: string; onsaved: () => void; oncancel: () => void;
  } = $props();
  const initial = untrack(() => ({ patientId, vaccineTypeId, initialDate }));
  let selected = $state<{ patient: Patient; owner: Owner } | null>(null);
  let typeId = $state(initial.vaccineTypeId ? String(initial.vaccineTypeId) : '');
  let date = $state(initial.initialDate);
  let nextDueOn = $state('');
  let batchNo = $state('');
  let notes = $state('');
  let time = $state('');
  let planNext = $state(true);
  let types = $state<VaccineType[]>([]);
  let loading = $state(true);
  let loadError = $state(false);
  let suggesting = $state(false);
  let suggestionError = $state(false);
  let saving = $state(false);
  let error = $state('');
  let revision = 0;
  let savedId: number | null = null;
  let stock = $state<{ consumeAfterSave: (table: 'vaccinations' | 'prescription_items', id: number) => Promise<void> } | undefined>();
  const available = $derived(types.filter(t => t.isActive && !t.deletedAt && selected && (!t.species || t.species === selected.patient.species)));
  const vaccine = $derived(available.find(t => t.id === Number(typeId)));
  async function load() {
    loading = true; loadError = false;
    try {
      types = await vaccineTypes.list();
      if (initial.patientId) {
        const patient = await patients.get(initial.patientId);
        selected = { patient, owner: await owners.get(patient.ownerId) };
      }
    } catch { loadError = true; }
    finally { loading = false; }
  }
  onMount(() => { void load(); });
  $effect(() => {
    const patient = selected?.patient.id, type = vaccine?.id, on = date;
    const request = ++revision;
    nextDueOn = ''; suggestionError = false; suggesting = false;
    if (mode !== 'give' || !patient || !type || !parseDate(on)) return;
    suggesting = true;
    vaccinations.suggestForPatient(patient, type, on).then(value => {
      if (revision === request) nextDueOn = value ?? '';
    }).catch(() => { if (revision === request) suggestionError = true; })
      .finally(() => { if (revision === request) suggesting = false; });
    return () => { revision++; };
  });
  function manualDue() { revision++; suggesting = false; suggestionError = false; }
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving || suggesting || loading) return;
    error = '';
    if (!selected) { error = tr.vaccineChoosePatient; return; }
    if (!vaccine) { error = tr.vaccineChoose; return; }
    if (!parseDate(date) || (mode === 'give' && date > todayDate())) { error = tr.invalidDate; return; }
    if (mode === 'give' && nextDueOn && (!parseDate(nextDueOn) || nextDueOn <= date)) { error = tr.vaccineNextAfterGiven; return; }
    saving = true;
    try {
      if (mode === 'plan') await planVaccine({ patientId: selected.patient.id, vaccineTypeId: vaccine.id, startsOn: date, startsTime: time || null, notes: notes.trim() || null });
      else {
        // Keep the saved id so a retry after a failed stock deduction never gives the vaccine twice.
        if (savedId === null) savedId = (await vaccinations.give({ vaccination: { patientId: selected.patient.id, vaccineTypeId: vaccine.id, vaccineName: vaccine.name, givenOn: date, nextDueOn: nextDueOn || null, batchNo: batchNo.trim() || null, notes: notes.trim() || null }, appointmentId, planNext })).id;
        await stock?.consumeAfterSave('vaccinations', savedId);
      }
      onsaved();
    } catch (e) { error = savedId !== null ? tr.inventoryConsumeSavedError : (e as ApiError)?.kind === 'invalid' ? tr.invalidField : tr.saveError; }
    finally { saving = false; }
  }
</script>
{#if loading}<p role="status">{tr.loading}</p>
{:else if loadError}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={load}>{tr.retry}</Button>
{:else}
<form onsubmit={submit} class="grid gap-4">
  <VaccinePatientPicker bind:selected locked={!!initial.patientId || !!appointmentId} />
  {#if selected}
    <Select label={tr.vaccineName} bind:value={typeId} required disabled={!!appointmentId} placeholder={tr.vaccineChoose} options={available.map(t => ({ value: String(t.id), label: t.name }))} />
    <p class="type-caption text-fg-muted">{available.length ? tr.vaccineCatalogHint : tr.vaccineNoCatalog}</p>
    {#if vaccineName && !vaccine}<p class="type-caption text-fg-muted">{vaccineName} · {tr.vaccineUntypedPlan}</p>{/if}
  {/if}
  <DateInput label={mode === 'give' ? tr.givenOn : tr.date} bind:value={date} required max={mode === 'give' ? todayDate() : '9999-12-31'} />
  {#if mode === 'plan'}
    <DateChips bind:value={date} />
    <Input label={tr.appointmentTime} type="time" bind:value={time} hint={tr.appointmentTimeHint} />
  {:else}
    <Input label={tr.batchNo} bind:value={batchNo} />
    <DateInput label={tr.nextDueOn} bind:value={nextDueOn} oninput={manualDue} />
    <p class="type-caption text-fg-muted">{tr.vaccineSuggestionHint}</p>
    {#if suggesting}<p role="status">{tr.loading}</p>{/if}
    {#if suggestionError}<p role="alert" class="text-danger">{tr.suggestionError}</p>{/if}
    <Checkbox label={tr.vaccinePlanNext} bind:checked={planNext} disabled={!nextDueOn} />
    {#if vaccine}<StockDeduction bind:this={stock} vaccineTypeId={vaccine.id} />{/if}
  {/if}
  <Textarea label={tr.notes} bind:value={notes} />
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={() => savedId !== null ? onsaved() : oncancel()}>{tr.cancel}</Button><Button type="submit" disabled={saving || suggesting || !selected || !vaccine}>{saving ? tr.saving : tr.save}</Button></div>
</form>
{/if}
