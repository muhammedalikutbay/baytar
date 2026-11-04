<script lang="ts">
  import { Syringe, ScanLine, Hourglass, List, StickyNote } from '@lucide/svelte';
  import { validateForm } from '$lib/validation';
  import { untrack, tick } from 'svelte';
  import { Button, Input, Textarea, Select, DateInput, NumberInput, RequiredNote, TimeInput } from '$lib/components/ui';
  import { tr } from '$lib/i18n/tr';
  import { isValidDate, parseDecimal } from '$lib/format';
  import * as vaccinations from '$lib/api/vaccinations';
  import * as appointments from '$lib/api/appointments';
  import type { Vaccination, Appointment, AppointmentKind, AppointmentStatus } from '$lib/api/types';
  let { record, onsaved, oncancel }: { record: Vaccination | Appointment; onsaved: () => void; oncancel: () => void } = $props();
  const initial = untrack(() => record);
  const vaccine = 'vaccineName' in initial ? initial : null;
  const appointment = 'title' in initial ? initial : null;
  let name = $state(vaccine?.vaccineName ?? appointment?.title ?? '');
  let date = $state(vaccine?.givenOn ?? appointment?.startsOn ?? '');
  let time = $state(appointment?.startsTime ?? '');
  let duration = $state(String(appointment?.durationMin ?? ''));
  let kind = $state(appointment?.kind ?? 'other');
  let status = $state(appointment?.status ?? 'planned');
  let nextDue = $state(vaccine?.nextDueOn ?? '');
  let batch = $state(vaccine?.batchNo ?? '');
  let notes = $state(initial.notes ?? '');
  let errors = $state<Record<string, string>>({});
  let saving = $state(false);
  let form: HTMLFormElement;
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return; if (saving) return;
    errors = {};
    if (!name.trim()) errors.name = tr.requiredField;
    if (!isValidDate(date)) errors.date = tr.validDate;
    if (vaccine && nextDue && !isValidDate(nextDue)) errors.nextDue = tr.validDate;
    if (appointment && time && !/^([01]\d|2[0-3]):[0-5]\d$/.test(time)) errors.time = tr.validTime;
    const minutes = parseDecimal(duration);
    if (appointment && minutes !== null && (!Number.isSafeInteger(minutes) || minutes <= 0)) errors.duration = tr.positiveNumber;
    if (Object.keys(errors).length) { await tick(); form.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    saving = true;
    try {
      if (vaccine) await vaccinations.update(vaccine.id, { vaccineName: name.trim(), givenOn: date, nextDueOn: nextDue || null, batchNo: batch.trim() || null, notes: notes.trim() || null });
      else if (appointment) await appointments.update(appointment.id, { title: name.trim(), startsOn: date, startsTime: time || null, durationMin: minutes, kind, status, notes: notes.trim() || null });
      onsaved();
    } catch { errors.form = tr.saveError; } finally { saving = false; }
  }
</script>
<form bind:this={form} onsubmit={submit} novalidate class="grid gap-4">
  <RequiredNote />
  <fieldset disabled={saving} class="grid min-w-0 gap-4">
    <Input icon={Syringe} label={(vaccine ? tr.vaccineName : tr.appointmentTitle)} bind:value={name} required error={errors.name} />
    <DateInput label={tr.date} bind:value={date} required error={errors.date} />
    {#if vaccine}
      <DateInput label={tr.nextDue} bind:value={nextDue} error={errors.nextDue} />
      <Input icon={ScanLine} label={tr.batchNo} bind:value={batch} />
    {:else}
      <TimeInput label={tr.time} optional bind:value={time} error={errors.time} />
      <NumberInput icon={Hourglass} label={tr.durationMin} bind:value={() => parseDecimal(duration), n => duration = n === null ? '' : String(n)} error={errors.duration} min={1} max={2147483647} step={1} unit={tr.inputMinuteUnit} />
      <Select icon={List} label={tr.appointmentKind} bind:value={kind} options={(['visit', 'vaccine', 'reminder', 'other'] as AppointmentKind[]).map(value => ({value, label: value === 'vaccine' ? tr.vaccination : tr[value]}))} />
      <Select icon={List} label={tr.appointmentStatus} bind:value={status} options={(['planned', 'done', 'cancelled'] as AppointmentStatus[]).map(value => ({value, label: tr[value]}))} />
    {/if}
    <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
  </fieldset>
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving}>{saving ? tr.saving : tr.save}</Button></div>
</form>
