<script lang="ts">
  import { page } from '$app/state';
  import { PawPrint, TriangleAlert } from '@lucide/svelte';
  import { Button, PageHeader, MenuButton, Card, Dialog, EmptyState, Toast, Badge, DateInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { formatAge, formatDate } from '$lib/format';
  import * as patients from '$lib/api/patients';
  import * as owners from '$lib/api/owners';
  import type { Patient, Owner } from '$lib/api/types';
  import PatientForm from '$lib/components/patients/PatientForm.svelte';
  import PatientOverview from '$lib/components/patients/PatientOverview.svelte';
  import PatientVaccinations from '$lib/components/patients/PatientVaccinations.svelte';
  import PatientAppointments from '$lib/components/patients/PatientAppointments.svelte';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  import PatientVisits from '$lib/components/patients/PatientVisits.svelte';
  import PatientAvatar from '$lib/components/patients/PatientAvatar.svelte';
  import PatientAttachments from '$lib/components/patients/PatientAttachments.svelte';
  import VisitForm from '$lib/components/visits/VisitForm.svelte';
  let patient = $state<Patient | null>(null);
  let owner = $state<Owner | null>(null);
  let loading = $state(true);
  let failed = $state(false);
  let refresh = $state(0);
  let edit = $state(false);
  let addVisit = $state(false);
  let historyRevision = $state(0);
  let confirm = $state(false);
  let busy = $state(false);
  let actionError = $state('');
  let toast = $state('');
  let toastOpen = $state(false);
  let tab = $state<'history' | 'vaccines' | 'appointments' | 'files'>('history');
  let giveVaccine = $state(false);
  const tabs = [['history', tr.flowHistory], ['vaccines', tr.vaccines], ['appointments', tr.appointments], ['files', tr.clinicalFiles]] as const;
  function tabKeydown(event: KeyboardEvent) {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const index = tabs.findIndex(([key]) => key === tab);
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
    tab = tabs[next][0];
    (event.currentTarget as HTMLElement).querySelector<HTMLButtonElement>('[data-patient-tab="' + tab + '"]')?.focus();
  }
  let deceasedOpen = $state(false), deceasedDate = $state(''), deceasedError = $state('');
  const today = () => new Date().toLocaleDateString('sv-SE');
  async function markDeceased() {
    if (!patient || busy) return;
    if (!deceasedDate || deceasedDate > today() || (patient.birthDate && deceasedDate < patient.birthDate)) { deceasedError = tr.clinicalInvalidDeceased; return; }
    busy = true; deceasedError = '';
    try { patient = await patients.markDeceased(patient.id, deceasedDate); deceasedOpen = false; }
    catch { deceasedError = tr.actionError; } finally { busy = false; }
  }
  async function clearDeceased() {
    if (!patient || busy) return;
    busy = true; actionError = '';
    try { patient = await patients.update(patient.id, { deceasedOn: null }); } catch { actionError = tr.actionError; } finally { busy = false; }
  }
  let deletedId = $state<number | null>(null);
  $effect(() => {
    const id = Number(page.url.searchParams.get('id')); refresh;
    let cancelled = false;
    patient = null; owner = null; loading = true; failed = false; edit = false; confirm = false; addVisit = false;
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
  function setVisitOpen(open: boolean) { addVisit = open; if (!open) historyRevision++; }
</script>
<a href={owner ? `/patients/owner?id=${owner.id}` : '/patients'} class="text-accent underline">{owner ? tr.backOwner : tr.backPatients}</a>
<div class="mt-6 grid gap-8">
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#if loading}<p role="status">{tr.loading}</p>
  {:else if patient && owner}
    <PageHeader title={patient.name} subtitle={[tr[patient.species], patient.breed, formatAge(patient.birthDate, patient.birthDateIsEstimate)].filter(Boolean).join(' · ')} icon={PawPrint}>
      {#snippet actions()}
        <Button disabled={!!patient?.deceasedOn} onclick={() => addVisit = true}>{tr.addVisit}</Button>
        <Button variant="secondary" disabled={!!patient?.deceasedOn} onclick={() => giveVaccine = true}>{tr.vaccineGive}</Button>
        <Button variant="secondary" onclick={() => edit = true}>{tr.edit}</Button>
        <MenuButton items={[
          { label: patient?.deceasedOn ? tr.clinicalClearDeceased : tr.clinicalMarkDeceased, disabled: busy, onselect: () => { if (patient?.deceasedOn) void clearDeceased(); else { deceasedDate = today(); deceasedError = ''; deceasedOpen = true; } } },
          { label: tr.deletePatient, danger: true, onselect: () => { actionError = ''; confirm = true; } }
        ]} />
      {/snippet}
    </PageHeader>
    <div class="flex items-center gap-3"><PatientAvatar patientId={patient.id} species={patient.species} name={patient.name} revision={historyRevision} large fallback={false} />{#if patient.deceasedOn}<Badge>{tr.clinicalDeceased} · {formatDate(patient.deceasedOn)}</Badge>{/if}</div>
    <PatientOverview patientId={patient.id} revision={historyRevision} onappointments={() => tab = 'appointments'} onvaccines={() => tab = 'vaccines'} />
    {#if patient.alerts?.trim()}<div role="alert" class="flex items-start gap-3 rounded-md border border-highlight p-4 text-highlight"><TriangleAlert size={20} aria-hidden="true" class="mt-0.5 shrink-0" /><div><p class="type-caption">{tr.clinicalAlerts}</p><p class="whitespace-pre-wrap break-words">{patient.alerts}</p></div></div>{/if}
    <Card><dl class="grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
      <div><dt class="type-caption text-fg-muted">{tr.owner}</dt><dd class="mt-2"><a class="text-accent underline" href={`/patients/owner?id=${owner.id}`}>{owner.fullName}</a></dd></div>
      {#each [{label:tr.sex,value:tr[patient.sex]}, {label:tr.neutered,value:patient.neutered == null ? tr.unknown : patient.neutered ? tr.yes : tr.no}, {label:tr.age,value:formatAge(patient.birthDate,patient.birthDateIsEstimate)}, {label:tr.birthDate,value:patient.birthDate ? (patient.birthDateIsEstimate ? '~' : '') + formatDate(patient.birthDate) : tr.unknown}, {label:tr.microchip,value:patient.microchip ?? tr.unknown}, {label:tr.color,value:patient.color ?? tr.unknown}] as field}
        <div><dt class="type-caption text-fg-muted">{field.label}</dt><dd class="mt-2 break-words">{field.value}</dd></div>
      {/each}
    </dl>{#if patient.notes}<div class="mt-6 border-t border-border pt-5"><h2 class="type-caption text-fg-muted">{tr.notes}</h2><p class="mt-2 whitespace-pre-wrap break-words">{patient.notes}</p></div>{/if}</Card>
    <div role="tablist" tabindex="-1" aria-label={patient.name} onkeydown={tabKeydown} class="flex flex-wrap gap-2 border-b border-border">
      {#each tabs as [key, label]}<button type="button" role="tab" data-patient-tab={key} id={'patient-tab-' + key} aria-controls={'patient-panel-' + key} tabindex={tab === key ? 0 : -1} aria-selected={tab === key} class={'min-h-11 px-4 ' + (tab === key ? 'border-b-2 border-accent text-accent' : 'text-fg-muted')} onclick={() => tab = key}>{label}</button>{/each}
    </div>
    <div role="tabpanel" id={'patient-panel-' + tab} aria-labelledby={'patient-tab-' + tab}>
    {#key patient.id}{#if tab === 'history'}<PatientVisits patientId={patient.id} patientName={patient.name} revision={historyRevision} onchange={() => historyRevision++} />{:else if tab === 'vaccines'}<PatientVaccinations revision={historyRevision} onchange={() => historyRevision++} />{:else if tab === 'appointments'}<PatientAppointments revision={historyRevision} onchange={() => historyRevision++} />{:else}<PatientAttachments patientId={patient.id} onchange={() => historyRevision++} />{/if}{/key}
    </div>
  {:else}<Card><EmptyState icon={PawPrint} title={failed ? tr.loadError : tr.recordMissing} description={tr.recordMissingDescription}>
    {#snippet action()}{#if failed}<Button variant="secondary" onclick={() => refresh++}>{tr.retry}</Button>{/if}{/snippet}
  </EmptyState></Card>{/if}
</div>
<Dialog wide bind:open={() => addVisit, setVisitOpen} title={tr.visit + ' — ' + (patient?.name ?? '')} description={tr.visitFormDescription}>
  {#if addVisit && patient}<VisitForm patientId={patient.id} onsaved={() => setVisitOpen(false)} oncancel={() => setVisitOpen(false)} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={edit} title={tr.edit} description={tr.patientFormDescription}>
  {#if edit && patient}<PatientForm {patient} ownerId={patient.ownerId} onsaved={() => { edit = false; refresh++; }} oncancel={() => edit = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
<Dialog bind:open={deceasedOpen} title={tr.clinicalMarkDeceased} description={tr.clinicalDeceasedDescription}>
  <DateInput label={tr.clinicalDeceasedDate} bind:value={deceasedDate} max={today()} error={deceasedError} />
  {#snippet footer()}<Button variant="secondary" onclick={() => deceasedOpen = false}>{tr.cancel}</Button><Button disabled={busy} onclick={markDeceased}>{tr.save}</Button>{/snippet}
</Dialog>
<Dialog bind:open={confirm} title={tr.deletePatient} description={tr.deletePatientDescription}>
  {#if actionError}<p role="alert" class="text-danger">{actionError}</p>{/if}
  {#snippet footer()}<Button variant="secondary" onclick={() => confirm = false}>{tr.cancel}</Button><Button variant="danger" disabled={busy} onclick={remove}>{tr.delete}</Button>{/snippet}
</Dialog>
<Toast bind:open={toastOpen} message={toast}>
  {#snippet action()}{#if deletedId !== null}<Button variant="secondary" size="sm" disabled={busy} onclick={undo}>{tr.undo}</Button>{/if}{/snippet}
</Toast>

<Dialog bind:open={giveVaccine} title={tr.vaccineGive} description={tr.vaccineWorkflowDescription}>
  {#if giveVaccine && patient}<VaccinationForm patientId={patient.id} onsaved={() => { giveVaccine = false; historyRevision++; }} oncancel={() => giveVaccine = false} />{/if}
  {#snippet footer()}{/snippet}
</Dialog>
