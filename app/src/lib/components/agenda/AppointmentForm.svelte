<script lang="ts">
  import { untrack, tick } from 'svelte';
  import { Button, Checkbox, Input, Select, Textarea } from '$lib/components/ui';
  import * as appointments from '$lib/api/appointments';
  import { search } from '$lib/api/search';
  import { detail } from '$lib/api/owners';
  import { get as getPatient } from '$lib/api/patients';
  import type { Appointment, ApiError, Patient, Owner, NewAppointment, AppointmentKind } from '$lib/api/types';
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
  let selectedPatient = $state(initial.patient ?? null);
  let selectedOwner = $state(initial.owner ?? null);
  let patientId = $state(initial.appointment?.patientId ?? initial.patient?.id ?? null);
  let ownerId = $state(initial.appointment?.ownerId ?? initial.owner?.id ?? initial.patient?.ownerId ?? null);
  let query = $state('');
  let options = $state<{ patient: Patient; owner: Owner }[]>([]);
  let searchLoading = $state(false);
  let searchFailed = $state(false);
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
        if (!cancelled) { selectedPatient = pet; selectedOwner = person; }
      } catch { if (!cancelled) contextFailed = true; }
      finally { if (!cancelled) contextLoading = false; }
    })();
    return () => { cancelled = true; };
  });
  $effect(() => {
    const text = query.trim();
    let cancelled = false;
    options = []; searchFailed = false;
    if (!text) { searchLoading = false; return; }
    searchLoading = true;
    const timer = setTimeout(async () => {
      try {
        const hits = await search(text, 10);
        const full = await Promise.all(hits.map(hit => detail(hit.owner.id)));
        if (!cancelled) options = full.flatMap((hit, index) => {
          const matchingPatients = hits[index].patients;
          return (matchingPatients.length ? matchingPatients : hit.patients).map(patient => ({ patient, owner: hit.owner }));
        });
      } catch { if (!cancelled) searchFailed = true; }
      finally { if (!cancelled) searchLoading = false; }
    }, 200);
    return () => { cancelled = true; clearTimeout(timer); };
  });
  async function submit(event: SubmitEvent) {
    event.preventDefault();
    if (saving || contextLoading) return;
    errors = {};
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
  <Input label={tr.appointmentTitle + ' *'} bind:value={title} required error={errors.title} />
  <Select label={tr.appointmentKind} bind:value={kind} options={Object.entries(kindLabels).map(([value, label]) => ({ value, label }))} />
  <Input label={tr.appointmentDate + ' *'} type="date" min="0001-01-01" max="9999-12-31" bind:value={startsOn} required error={errors.startsOn} />
  <div class="grid grid-cols-2 gap-4"><Input label={tr.appointmentTime} type="time" bind:value={startsTime} hint={tr.appointmentTimeHint} error={errors.startsTime} /><Input label={tr.appointmentDuration} inputmode="numeric" bind:value={duration} error={errors.durationMin} /></div>
  <fieldset class="grid min-w-0 gap-2"><legend class="type-caption mb-2">{tr.appointmentPatient}</legend>
    {#if contextLoading}<p role="status">{tr.loading}</p>{/if}
    {#if contextFailed}<p role="alert" class="text-danger">{tr.loadError}</p>{/if}
    {#if selectedPatient || selectedOwner}<p class="break-words">{selectedPatient?.name}{selectedPatient && selectedOwner ? ' · ' : ''}{selectedOwner?.fullName}</p>
      <Button variant="ghost" size="sm" disabled={saving || contextLoading} onclick={() => { selectedPatient = null; selectedOwner = null; patientId = null; ownerId = null; }}>{tr.appointmentClearPatient}</Button>
    {:else}<p class="type-caption text-fg-muted">{tr.appointmentNoPatient}</p>{/if}
    <Input label={tr.appointmentSearch} type="search" bind:value={query} autocomplete="off" disabled={saving || contextLoading} />
    {#if query.trim()}<div class="max-h-48 overflow-y-auto rounded-md border border-border p-2" aria-live="polite">
      {#if searchLoading}<p role="status">{tr.loading}</p>{:else if searchFailed}<p role="alert" class="text-danger">{tr.loadError}</p>{:else if !options.length}<p>{tr.noResults}</p>{/if}
      {#each options as option (option.patient.id)}<Button variant="ghost" class="w-full justify-start text-left" disabled={saving} onclick={() => { selectedPatient = option.patient; selectedOwner = option.owner; patientId = option.patient.id; ownerId = option.owner.id; query = ''; }}>{option.patient.name} · {option.owner.fullName}</Button>{/each}
    </div>{/if}
    {#if errors.patientId || errors.ownerId}<p role="alert" class="text-danger">{tr.invalidField}</p>{/if}
  </fieldset>
  <Textarea label={tr.notes} bind:value={notes} />
  {#if !initial.appointment}<Checkbox label={tr.appointmentRepeat} bind:checked={repeat} />
    {#if repeat}<div class="grid grid-cols-2 gap-4"><Input label={tr.appointmentCount} inputmode="numeric" bind:value={count} error={errors.count} hint={tr.appointmentRepeatHint} /><Input label={tr.appointmentInterval} inputmode="numeric" bind:value={interval} error={errors.everyDays} /></div>
      <div aria-live="polite"><h3 class="type-caption">{tr.appointmentPreview}</h3><ol class="mt-2 max-h-40 list-inside list-decimal overflow-y-auto text-fg-muted tabular-nums">{#each preview as day}<li>{formatDate(day)}</li>{/each}</ol></div>
    {/if}
  {:else}<p class="type-caption text-fg-muted">{tr.appointmentEditHint}</p>{/if}
  {#if errors.form}<p role="alert" class="text-danger">{errors.form}</p>{/if}
  <div class="flex justify-end gap-2"><Button variant="secondary" disabled={saving} onclick={oncancel}>{tr.cancel}</Button><Button type="submit" disabled={saving || contextLoading}>{saving ? tr.saving : tr.save}</Button></div>
</form>
