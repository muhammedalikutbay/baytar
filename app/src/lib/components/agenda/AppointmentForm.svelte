<script lang="ts">
  import { ClipboardList, Hourglass, StickyNote, Repeat } from '@lucide/svelte';
  import { validateForm, parseDecimal } from '$lib/validation';
  import { untrack, tick } from 'svelte';
  import { Button, Checkbox, Input, Select, Textarea, DateInput, NumberInput, RequiredNote, TimeInput } from '$lib/components/ui';
  import RecordPicker from '$lib/components/pickers/RecordPicker.svelte';
  import * as appointments from '$lib/api/appointments';
  import { detail } from '$lib/api/owners';
  import { get as getPatient } from '$lib/api/patients';
  import type { Appointment, ApiError, Patient, Owner, NewAppointment, AppointmentKind, RecordSelection } from '$lib/api/types';
  import { formatDate, localDate, parseDate } from '$lib/format';
  import { seriesDates } from './calendar';
  import { tr } from '$lib/i18n/tr';
  import { kindLabels } from './appointments';
  let { appointment, patient, owner, date = localDate(), onsaved, oncancel }: {
    appointment?: Appointment; patient?: Patient; owner?: Owner; date?: string; onsaved: () => void; oncancel: () => void;
  } = $props();
  const initial = untrack(() => ({ appointment, patient, owner, date }));
  let title = $state(initial.appointment?.title ?? '');
  let titleEdited = $state(!!initial.appointment);
  let kind = $state<string>(initial.appointment?.kind === 'visit' && initial.appointment.title.startsWith(tr.appointmentControl) ? 'control' : initial.appointment?.kind ?? 'visit');
  let startsOn = $state(initial.appointment?.startsOn ?? initial.date);
  let startsTime = $state(initial.appointment?.startsTime ?? '');
  let duration = $state(String(initial.appointment?.durationMin ?? 30));
  let customDuration = $state(!!initial.appointment?.durationMin && ![15, 30, 45, 60].includes(initial.appointment.durationMin));
  let notes = $state(initial.appointment?.notes ?? '');
  let selected = $state<RecordSelection | null>(initial.patient && initial.owner ? { patient: initial.patient, owner: initial.owner } : null);
  let clinicTask = $state(!!initial.appointment && initial.appointment.patientId === null);
  let contextLoading = $state(false), contextFailed = $state(false), contextRetry = $state(0);
  let repeat = $state('none'), count = $state('2'), interval = $state('7');
  let saving = $state(false), errors = $state<Record<string, string>>({});
  let form = $state<HTMLFormElement>();
  const kinds = { ...kindLabels, control: tr.appointmentControl };
  const kindOptions = ['visit', 'vaccine', 'control', 'reminder', 'other'] as const;
  const positiveInteger = (value: string) => /^\d+$/.test(value) && Number.isSafeInteger(Number(value)) && Number(value) > 0;
  const everyDays = $derived(repeat === 'weekly' ? 7 : repeat === 'days' ? +interval : 1);
  const everyMonths = $derived(repeat === 'monthly' ? 1 : repeat === 'yearly' ? 12 : undefined);
  const autoTitle = $derived(kinds[kind as keyof typeof kinds] + (selected?.patient && !clinicTask ? ' — ' + selected.patient.name : ''));
  $effect(() => { if (!titleEdited) title = autoTitle; });
  const preview = $derived.by(() => {
    if (repeat === 'none' || !parseDate(startsOn) || !positiveInteger(count) || +count > 52 || !Number.isSafeInteger(everyDays) || everyDays < 1) return [];
    const dates = seriesDates(startsOn, +count, everyDays, everyMonths);
    return dates.every(Boolean) ? dates : [];
  });
  $effect(() => {
    const id = initial.appointment?.patientId ?? initial.patient?.id;
    contextRetry;
    if (id == null || (initial.patient && initial.owner && !initial.appointment)) return;
    let cancelled = false;
    contextLoading = true; contextFailed = false;
    (async () => {
      try {
        const pet = await getPatient(id);
        const person = (await detail(pet.ownerId)).owner;
        if (!cancelled) selected = { patient: pet, owner: person };
      } catch { if (!cancelled) contextFailed = true; }
      finally { if (!cancelled) contextLoading = false; }
    })();
    return () => { cancelled = true; };
  });
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (!validateForm(event.currentTarget as HTMLFormElement) || saving || contextLoading || contextFailed) return;
    errors = {};
    if (!clinicTask && !selected?.patient) errors.patientId = tr.requiredField;
    if (clinicTask && !title.trim()) errors.title = tr.requiredField;
    if (!parseDate(startsOn) || startsOn < '0001-01-01') errors.startsOn = tr.appointmentDateInvalid;
    if (duration && (!positiveInteger(duration) || +duration > 2147483647)) errors.durationMin = tr.appointmentNumberInvalid;
    if (repeat !== 'none' && (!positiveInteger(count) || +count < 2 || +count > 52)) errors.count = tr.appointmentRepeatHint;
    if (repeat !== 'none' && (!positiveInteger(String(everyDays)) || everyDays > 2147483647 || !preview.length)) errors.everyDays = tr.appointmentNumberInvalid;
    if (Object.keys(errors).length) { await tick(); form?.querySelector<HTMLElement>('[aria-invalid="true"], [role="alert"]')?.focus(); return; }
    saving = true;
    let savedTitle = title.trim() || autoTitle;
    if (kind === 'control' && !savedTitle.startsWith(tr.appointmentControl)) savedTitle = tr.appointmentControl + ' — ' + savedTitle;
    const input: NewAppointment = { title: savedTitle, kind: kind === 'control' ? 'visit' : kind as AppointmentKind, startsOn, startsTime: startsTime || null,
      durationMin: duration ? +duration : null, notes: notes.trim() || null,
      patientId: clinicTask ? null : selected?.patient?.id ?? null,
      ownerId: clinicTask ? initial.appointment?.ownerId ?? null : selected?.owner.id ?? null };
    try {
      if (initial.appointment) await appointments.update(initial.appointment.id, input);
      else if (repeat !== 'none') await appointments.createSeries(input, +count, everyDays, everyMonths);
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
  <fieldset disabled={saving || contextLoading} class="grid min-w-0 gap-4">
    <fieldset class="grid min-w-0 gap-3"><legend class="mb-2 text-sm font-medium">{tr.appointmentPatient}</legend>
      <Checkbox label={tr.appointmentClinicTask} bind:checked={clinicTask} />
      {#if contextLoading}<p role="status">{tr.loading}</p>{/if}
      {#if contextFailed}<p role="alert" class="text-danger">{tr.loadError}</p><Button variant="secondary" onclick={() => contextRetry++}>{tr.retry}</Button>{/if}
      {#if !clinicTask && !contextLoading && !contextFailed}<RecordPicker bind:selected />{/if}
      {#if errors.patientId}<p role="alert" tabindex="-1" class="text-sm text-danger">{errors.patientId}</p>{/if}
    </fieldset>
    <fieldset><legend class="mb-2 text-sm font-medium">{tr.appointmentKind}</legend><div class="flex flex-wrap gap-2">
      {#each kindOptions as value}<Button size="sm" variant="secondary" aria-pressed={kind === value} class={kind === value ? 'text-accent underline' : ''} onclick={() => kind = value}>{kinds[value]}</Button>{/each}
    </div></fieldset>
    <div class="grid gap-4 sm:grid-cols-2"><DateInput label={tr.appointmentDate} min="0001-01-01" max="9999-12-31" bind:value={startsOn} required error={errors.startsOn} /><TimeInput label={tr.appointmentTime} optional bind:value={startsTime} hint={tr.appointmentTimeHint} error={errors.startsTime} /></div>
    <fieldset><legend class="mb-2 text-sm font-medium">{tr.appointmentDuration}</legend><div class="flex flex-wrap gap-2">
      {#each [15, 30, 45, 60] as minutes}<Button size="sm" variant="secondary" aria-pressed={!customDuration && duration === String(minutes)} onclick={() => { duration = String(minutes); customDuration = false; }}>{minutes} {tr.inputMinuteUnit}</Button>{/each}
      <Button size="sm" variant="secondary" aria-pressed={customDuration} onclick={() => customDuration = true}>{tr.appointmentDurationCustom}</Button>
    </div>{#if customDuration}<div class="mt-3"><NumberInput icon={Hourglass} label={tr.appointmentDurationCustom} bind:value={() => parseDecimal(duration), n => duration = n === null ? '' : String(n)} error={errors.durationMin} min={1} max={2147483647} step={1} unit={tr.inputMinuteUnit} /></div>{/if}</fieldset>
    <Input icon={ClipboardList} label={tr.appointmentTitle} bind:value={title} oninput={() => titleEdited = true} required={clinicTask} error={errors.title} />
    {#if !initial.appointment}
      <Select label={tr.appointmentRepeatLabel} bind:value={repeat} options={[
        {value:'none', label:tr.appointmentRepeatNone}, {value:'weekly', label:tr.appointmentRepeatWeekly},
        {value:'monthly', label:tr.appointmentRepeatMonthly}, {value:'yearly', label:tr.appointmentRepeatYearly}, {value:'days', label:tr.appointmentRepeatDays}
      ]} />
      {#if repeat !== 'none'}<div class="grid gap-4 sm:grid-cols-2"><NumberInput icon={Repeat} label={tr.appointmentCount} bind:value={() => parseDecimal(count), n => count = n === null ? '' : String(n)} error={errors.count} hint={tr.appointmentRepeatHint} min={2} max={52} step={1} />
        {#if repeat === 'days'}<NumberInput icon={Repeat} label={tr.appointmentInterval} bind:value={() => parseDecimal(interval), n => interval = n === null ? '' : String(n)} error={errors.everyDays} min={1} max={2147483647} step={1} unit={tr.inputDayUnit} />{/if}</div>
        <div aria-live="polite"><h3 class="type-caption">{tr.appointmentPreview}</h3><ol class="mt-2 max-h-32 list-inside list-decimal overflow-y-auto text-fg-muted tabular-nums">{#each preview as day}<li>{formatDate(day)}</li>{/each}</ol></div>
      {/if}
    {:else}<p class="type-caption text-fg-muted">{tr.appointmentEditHint}</p>{/if}
    <Textarea icon={StickyNote} label={tr.notes} bind:value={notes} />
  </fieldset>
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || contextLoading || contextFailed}>{saving ? tr.saving : tr.save}</Button></div>
</form>
