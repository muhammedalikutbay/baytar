<script lang="ts" module>
  export type HomeDialog = 'owner' | 'visit' | 'vaccine' | 'collect' | 'appointment';
</script>

<script lang="ts">
  import { Dialog } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import * as billing from '$lib/api/billing';
  import type { Owner, Patient } from '$lib/api/types';
  import OwnerForm from '$lib/components/patients/OwnerForm.svelte';
  import AppointmentForm from '$lib/components/agenda/AppointmentForm.svelte';
  import VaccinationForm from '$lib/components/vaccines/VaccinationForm.svelte';
  import VaccinePatientPicker from '$lib/components/vaccines/VaccinePatientPicker.svelte';
  import VisitForm from '$lib/components/visits/VisitForm.svelte';
  import OwnerPicker from '$lib/components/billing/OwnerPicker.svelte';
  import PaymentDialog from '$lib/components/billing/PaymentDialog.svelte';

  let { dialog = $bindable(null), date, onchanged }: { dialog?: HomeDialog | null; date: string; onchanged: () => void } = $props();
  let picked = $state<{ patient: Patient; owner: Owner } | null>(null);
  let payOwner = $state<{ id: number; balance: number } | null>(null);
  let payOpen = $state(false);
  let failed = $state(false);

  // Reset the transient picks whenever the dialog closes.
  $effect(() => { if (dialog === null) { picked = null; failed = false; } });

  function done() { dialog = null; onchanged(); }
  async function chooseOwner(owner: Owner) {
    failed = false;
    try {
      payOwner = { id: owner.id, balance: await billing.balance(owner.id) };
      dialog = null; payOpen = true;
    } catch { failed = true; }
  }
  const titles: Record<HomeDialog, string> = {
    owner: tr.newOwner, visit: tr.cockpitPickPatient, vaccine: tr.vaccineGive,
    collect: tr.cockpitPickOwner, appointment: tr.agendaDialogTitle
  };
  const descriptions: Record<HomeDialog, string> = {
    owner: tr.ownerFormDescription, visit: tr.cockpitPickPatientDescription, vaccine: tr.vaccineWorkflowDescription,
    collect: tr.cockpitPickOwnerDescription, appointment: tr.appointmentFormDescription
  };
</script>

<Dialog bind:open={() => dialog !== null, value => { if (!value) dialog = null; }}
  title={dialog === 'visit' && picked ? tr.addVisit : dialog ? titles[dialog] : ''}
  description={dialog === 'visit' && picked ? tr.visitFormDescription : dialog ? descriptions[dialog] : ''}>
  {#if dialog === 'owner'}<OwnerForm onsaved={done} oncancel={() => dialog = null} />
  {:else if dialog === 'appointment'}<AppointmentForm {date} onsaved={done} oncancel={() => dialog = null} />
  {:else if dialog === 'vaccine'}<VaccinationForm mode="give" onsaved={done} oncancel={() => dialog = null} />
  {:else if dialog === 'visit'}
    {#if picked}<VisitForm patientId={picked.patient.id} onsaved={done} oncancel={() => dialog = null} />
    {:else}<VaccinePatientPicker bind:selected={picked} />{/if}
  {:else if dialog === 'collect'}
    <OwnerPicker onselect={chooseOwner} />
    {#if failed}<p role="alert" class="mt-3 text-danger">{tr.loadError}</p>{/if}
  {/if}
  {#snippet footer()}{/snippet}
</Dialog>

{#if payOpen && payOwner}
  <PaymentDialog bind:open={payOpen} ownerId={payOwner.id} balance={payOwner.balance} onsaved={onchanged} />
{/if}
