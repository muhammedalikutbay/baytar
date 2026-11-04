<script lang="ts">
  import { untrack } from 'svelte';
  import { Button, DateInput, RequiredNote } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { todayDate, parseDate } from '$lib/format';
  import { postponeVaccine } from '$lib/api/appointments';
  import type { VaccineDue } from '$lib/api/types';
  import DateChips from './DateChips.svelte';
  let { row, onsaved, oncancel }: { row: VaccineDue; onsaved: () => void; oncancel: () => void } = $props();
  let date = $state(untrack(() => row.dueOn > todayDate() ? row.dueOn : todayDate()));
  let saving = $state(false);
  let error = $state('');
  async function submit(event: SubmitEvent) {
    event.preventDefault(); if (saving) return;
    if (!parseDate(date)) { error = tr.invalidDate; return; }
    saving = true; error = '';
    try { await postponeVaccine(row.appointmentId, row.vaccinationId, date); onsaved(); }
    catch { error = tr.saveError; }
    finally { saving = false; }
  }
</script>
<form onsubmit={submit} class="grid gap-4">
  <RequiredNote />
  <p class="font-medium">{row.patientName} · {row.vaccineName}</p>
  <DateInput label={tr.date} bind:value={date} required />
  <DateChips bind:value={date} base={row.dueOn > todayDate() ? row.dueOn : todayDate()} />
  {#if error}<p role="alert" class="text-danger">{error}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
