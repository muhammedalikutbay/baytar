<script lang="ts">
  import { ClipboardList, List, Hourglass, StickyNote, Repeat } from '@lucide/svelte';
  import { validateForm, parseDecimal } from '$lib/validation';
  import { untrack, tick } from 'svelte';
  import { Button, Checkbox, Input, Select, Textarea, DateInput, NumberInput, RequiredNote, TimeInput } from '$lib/components/ui';
  import * as appointments from '$lib/api/appointments';
  import RecordPicker from '$lib/components/pickers/RecordPicker.svelte';
  import { detail } from '$lib/api/owners';
  import { get as getPatient } from '$lib/api/patients';
  import type { Appointment, ApiError, Patient, Owner, NewAppointment, AppointmentKind, RecordSelection } from '$lib/api/types';
  import { addDateDays, formatDate, localDate, parseDate } from '$lib/format';
  import { tr } from '$lib/i18n/tr';
  import { kindLabels } from './appointments';
  let { appointment, patient, owner, date = localDate(), onsaved, oncancel }: {
    appointment?: Appointment; patient?: Patient; owner?: Owner; date?: string; onsaved: () => void; oncancel: () => void;
  } = $props();
  const initial = untrack(() => ({ appointment, patient, owner, date }));
  let title = $state(initial.appointment?.title ?? '');
  let kind = $state<string>(initial.appointment?.kind ?? 'visit');
  let startsOn = $state(initial.appointment?.startsOn ?? initial.date);
  let startsTime = $state(initial.appointment?.startsTime ?? '');
  let duration = $state(String(initial.appointment?.durationMin ?? ''));
  let notes = $state(initial.appointment?.notes ?? '');
  let patientId = $state(initial.appointment?.patientId ?? initial.patient?.id ?? null);
  let ownerId = $state(initial.appointment?.ownerId ?? initial.owner?.id ?? initial.patient?.ownerId ?? null);
  let selected = $state<RecordSelection | null>(initial.patient && initial.owner ? { patient: initial.patient, owner: initial.owner } : null);
  let noPatient = $state(initial.appointment ? !initial.appointment.patientId : false);
  let contextLoading = $state(false);
  let contextFailed = $state(false);
  let repeat = $state(false);
  let count = $state('2');
  let interval = $state('7');
  let saving = $state(false);
  let errors = $state<Record<string, string>>({});
  let form = $state<HTMLFormElement>();
  const positiveInteger = (value: string) => /^\d+$/.test(value) && Number.isSafeInteger(Number(value)) && Number(value) > 0;
  const preview = $derived.by(() => {
    if (!repeat || !parseDate(startsOn) || !positiveInteger(count) || +count > 52 || !positiveInteger(interval)) return [];
    const dates = Array.from({ length: +count }, (_, index) => addDateDays(startsOn, index * +interval));
    return dates.every(Boolean) ? dates : [];
  });
  $effect(() => {
    const existing = initial.appointment;
    if (!existing) return;
    let cancelled = false;
    contextLoading = true;
    (async () => {
      try {
        const pet = existing.patientId === null ? null : await getPatient(existing.patientId);
        const id = existing.ownerId ?? pet?.ownerId;
        const person = id == null ? null : (await detail(id)).owner;
        if (!cancelled) { selected = person ? { owner: person, ...(pet ? { patient: pet } : {}) } : null; }
      } catch { if (!cancelled) contextFailed = true; }
      finally { if (!cancelled) contextLoading = false; }
    })();
    return () => { cancelled = true; };
  });
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement)) return;
    if (saving || contextLoading || contextFailed) return;
    errors = {};
    patientId = noPatient ? null : selected?.patient?.id ?? null;
    ownerId = noPatient ? null : selected?.owner.id ?? null;
    if (!noPatient && !patientId) errors.patientId = tr.requiredField;
    if (!title.trim() && selected?.patient && !noPatient) title = kindLabels[kind as AppointmentKind] + ' — ' + selected.patient.name;
    if (!title.trim()) errors.title = tr.requiredField;
    if (!parseDate(startsOn) || startsOn < '0001-01-01') errors.startsOn = tr.appointmentDateInvalid;
    if (startsTime && !/^([01]\d|2[0-3]):[0-5]\d$/.test(startsTime)) errors.startsTime = tr.invalidField;
    if (duration && (!positiveInteger(duration) || +duration > 2147483647)) errors.durationMin = tr.appointmentNumberInvalid;
    if (repeat && (!positiveInteger(count) || +count > 52)) errors.count = tr.appointmentRepeatHint;
    if (repeat && (!positiveInteger(interval) || +interval > 2147483647 || !preview.length)) errors.everyDays = tr.appointmentNumberInvalid;
    if (Object.keys(errors).length) { await tick(); form?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus(); return; }
    saving = true;
    const input: NewAppointment = { title: title.trim(), kind: kind as AppointmentKind, startsOn, startsTime: startsTime || null,
      durationMin: duration ? +duration : null, notes: notes.trim() || null, patientId, ownerId };
    try {
      if (initial.appointment) await appointments.update(initial.appointment.id, input);
      else if (repeat) await appointments.createSeries(input, +count, +interval);
      else await appointments.create(input);
      onsaved();
    } catch (error) {
      const e = error as ApiError;
      errors = { form: tr.saveError, ...(e?.field ? { [e.field]: tr.invalidField } : {}) };
    } finally { saving = false; }
  }
</script>
<form bind:this={form} onsubmit={submit} novalidate class="grid gap-4">
  <RequiredNote />
  <fieldset class="grid min-w-0 gap-2"><legend class="mb-2 text-sm font-medium">{tr.appointmentPatient}</legend>
    {#if contextLoading}<p role="status">{tr.loading}</p>
    {:else if contextFailed}<p role="alert" class="text-danger">{tr.loadError}</p>
    {:else if initial.patient}<p class="font-medium">{initial.patient.name} · {initial.owner?.fullName}</p>
    {:else}<Checkbox label={tr.flowNoPatient} bind:checked={noPatient} />{#if !noPatient}<RecordPicker bind:selected />{/if}{/if}
    {#if errors.patientId}<p role="alert" class="text-danger">{errors.patientId}</p>{/if}
  </fieldset>

  <Input icon={ClipboardList} label={tr.appointmentTitle} bind:value={title} required={noPatient} error={errors.title} />
  <Select icon={List} label={tr.appointmentKind} bind:value={kind} options={Object.entries(kindLabels).map(([value, label]) => ({ value, label }))} />
  <DateInput label={tr.appointmentDate} min="0001-01-01" max="9999-12-31" bind:value={startsOn} required error={errors.startsOn} />
  <div class="grid grid-cols-2 gap-4"><TimeInput label={tr.appointmentTime} optional bind:value={startsTime} hint={tr.appointmentTimeHint} error={errors.startsTime} /><NumberInput icon={Hourglass} label={tr.appointmentDuration} bind:value={() => parseDecimal(duration), n => duration = n === null ? '' : String(n)} error={errors.durationMin} min={1} max={2147483647} step={1} unit={tr.inputMinuteUnit} /></div>
  <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
  {#if !initial.appointment}<Checkbox label={tr.appointmentRepeat} bind:checked={repeat} />
    {#if repeat}<div class="grid grid-cols-2 gap-4"><NumberInput icon={Repeat} label={tr.appointmentCount} bind:value={() => parseDecimal(count), n => count = n === null ? '' : String(n)} error={errors.count} hint={tr.appointmentRepeatHint} min={2} max={52} step={1} /><NumberInput icon={Repeat} label={tr.appointmentInterval} bind:value={() => parseDecimal(interval), n => interval = n === null ? '' : String(n)} error={errors.everyDays} min={1} max={2147483647} step={1} unit={tr.inputDayUnit} /></div>
      <div aria-live="polite"><h3 class="type-caption">{tr.appointmentPreview}</h3><ol class="mt-2 max-h-40 list-inside list-decimal overflow-y-auto text-fg-muted tabular-nums">{#each preview as day}<li>{formatDate(day)}</li>{/each}</ol></div>
    {/if}
  {:else}<p class="type-caption text-fg-muted">{tr.appointmentEditHint}</p>{/if}
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || contextLoading}>{saving ? tr.saving : tr.save}</Button></div>
</form>
