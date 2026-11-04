<script lang="ts">
  import { page } from '$app/state';
  import { PawPrint } from '@lucide/svelte';
  import { Button, Card, Dialog, EmptyState, Toast } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatAge, formatDate } from '$lib/format';
  import * as patients from '$lib/api/patients';
  import * as owners from '$lib/api/owners';
  import type { Patient, Owner } from '$lib/api/types';
  import PatientForm from '$lib/components/patients/PatientForm.svelte';
  import PatientVisits from '$lib/components/patients/PatientVisits.svelte';
  import PatientVaccinations from '$lib/components/patients/PatientVaccinations.svelte';
  import PatientAppointments from '$lib/components/patients/PatientAppointments.svelte';
  let patient = $state<Patient | null>(null);
  let owner = $state<Owner | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let edit = $state(false);
  let confirm = $state(false);
  let busy = $state(false);
  let actionError = $state('');
  let toast = $state('');
  let toastOpen = $state(false);
  let deletedId = $state<number | null>(null);
  $effect(() => {
    const id = Number(page.url.searchParams.get('id')); refresh;
    let cancelled = false;
    patient = null; owner = null; loading = true; failed = false; edit = false; confirm = false;
    if (!Number.isSafeInteger(id) || id <= 0) { loading = false; return; }
    (async () => {
      try { const pet = await patients.get(id); const person = await owners.get(pet.ownerId);
        if (!cancelled && !pet.deletedAt && !person.deletedAt) { patient = pet; owner = person; }
      } catch (error: unknown) { if (!cancelled) failed = (error as { kind?: string })?.kind !== 'not_found'; } finally { if (!cancelled) loading = false; }
    })();
    return () => { cancelled = true; };
  });
  async function remove() {
    if (!patient || busy) return;
    busy = true; actionError = '';
    try { await patients.softDelete(patient.id); deletedId = patient.id; confirm = false; patient = null; toast = tr.patientDeleted; toastOpen = true; }
    catch { actionError = tr.actionError; } finally { busy = false; }
  }
  async function undo() {
    if (deletedId === null || busy) return;
    busy = true; actionError = '';
    try { await patients.restore(deletedId); deletedId = null; toast = tr.restored; toastOpen = true; refresh++; }
    catch { actionError = tr.actionError; } finally { busy = false; }
  }
</script>
<a href={owner ? `/patients/owner?id=${owner.id}` : '/patients'} class="text-accent underline">{owner ? tr.backOwner : tr.backPatients}</a>
<div class="mt-6 grid gap-8">
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if patient && owner}
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div><h1 class="type-page break-words">{patient.name}</h1><p class="mt-2 text-fg-muted">{tr[patient.species]}{patient.speciesNote ? ` · ${patient.speciesNote}` : ''}{patient.breed ? ` · ${patient.breed}` : ''}</p></div>
      <div class="flex gap-2"><Button variant="secondary" onclick={() => { actionError = ''; confirm = true; }}>{tr.deletePatient}</Button><Button onclick={() => edit = true}>{tr.edit}</Button></div>
    </div>
    <Card><dl class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
      <div><dt class="type-caption text-fg-muted">{tr.owner}</dt><dd class="mt-2"><a class="text-accent underline" href={`/patients/owner?id=${owner.id}`}>{owner.fullName}</a></dd></div>
      {#each [{label:tr.sex,value:tr[patient.sex]}, {label:tr.neutered,value:patient.neutered == null ? tr.unknown : patient.neutered ? tr.yes : tr.no}, {label:tr.age,value:formatAge(patient.birthDate,patient.birthDateIsEstimate)}, {label:tr.birthDate,value:patient.birthDate ? (patient.birthDateIsEstimate ? '~' : '') + formatDate(patient.birthDate) : tr.unknown}, {label:tr.microchip,value:patient.microchip ?? tr.unknown}, {label:tr.color,value:patient.color ?? tr.unknown}] as field}
        <div><dt class="type-caption text-fg-muted">{field.label}</dt><dd class="mt-2 break-words">{field.value}</dd></div>
      {/each}
    </dl>{#if patient.notes}<div class="mt-6 border-t border-border pt-5"><h2 class="type-caption text-fg-muted">{tr.notes}</h2><p class="mt-2 whitespace-pre-wrap break-words">{patient.notes}</p></div>{/if}</Card>
    <PatientVisits /><PatientVaccinations /><PatientAppointments />
  {:else}<Card><EmptyState icon={PawPrint} title={failed ? tr.loadError : tr.recordMissing} description={tr.recordMissingDescription}>
    {#snippet action()}{#if failed}<Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>{/if}{/snippet}
  </EmptyState></Card>{/if}
</div>
<Dialog bind:open={edit} title={tr.edit} description={tr.patientFormDescription}>
  {#if edit && patient}<PatientForm {patient} ownerId={patient.ownerId} onsaved={() => { edit = false; refresh++; }} oncancel={() => edit = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={confirm} title={tr.deletePatient} description={tr.deletePatientDescription}>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#snippet footer()}<Button variant="secondary" onclick={() => confirm = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
<Toast bind:open={toastOpen} message={toast}>
  {#snippet action()}{#if deletedId !== null}<Button variant="secondary" size="sm" disabled={busy} onclick={undo}>{tr.undo}</Button>{/if}{/snippet}
</Toast>
