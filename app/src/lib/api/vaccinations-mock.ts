import { dateAfterDays, todayDate, parseDate } from '$lib/format';
import type { ApiError, Appointment, CalendarAppointment, CommandMap, GiveVaccination, Owner, Patient, PlanVaccination, Vaccination, VaccineCalendarEvent, VaccineDue, VaccineDueQuery, VaccineType } from './types';
interface Context {
  clinical: { call(command: string, args: unknown): unknown };
  patients: Patient[];
  patientGet(id: number): Patient;
  ownerGet(id: number): Owner;
  fail(kind: ApiError['kind'], message: string, field?: string): never;
}
export function createVaccineMock(ctx: Context) {
  function request<K extends keyof CommandMap>(name: K, args: CommandMap[K]['args']): CommandMap[K]['result'] {
    return ctx.clinical.call(name, args) as CommandMap[K]['result'];
  }
  function patient(id: number) {
    const p = ctx.patientGet(id);
    if (p.deletedAt || p.deceasedOn || ctx.ownerGet(p.ownerId).deletedAt) ctx.fail('not_found', 'Patient unavailable.');
    return p;
  }
  function type(id: number, patientId: number) {
    const p = patient(patientId), t = request('vaccine_type_get', { id });
    if (t.deletedAt || !t.isActive) ctx.fail('not_found', 'Vaccine unavailable.');
    if (t.species && t.species !== p.species) ctx.fail('invalid', 'Vaccine species must match patient.', 'vaccineTypeId');
    return t;
  }
  const doses = (patientId: number) => request('vaccinations_by_patient', { patientId });
  const appointments = () => request('appointments_between', { from: '0001-01-01', to: '9999-12-31' });
  const plans = () => appointments().map(r => r.appointment).filter(a => a.kind === 'vaccine' && a.status === 'planned' && a.patientId !== null);
  function due(query: VaccineDueQuery): VaccineDue[] {
    const result: VaccineDue[] = [];
    const planned = plans();
    for (const p of ctx.patients.filter(p => !p.deletedAt && !p.deceasedOn && !ctx.ownerGet(p.ownerId).deletedAt)) {
      const owner = ctx.ownerGet(p.ownerId), seen = new Set<string>();
      const base = { patientId: p.id, patientName: p.name, species: p.species, ownerName: owner.fullName, phone: owner.phone || owner.phoneRaw };
      for (const a of planned.filter(a => a.patientId === p.id && a.vaccineTypeId).sort((a,b) => a.startsOn.localeCompare(b.startsOn) || a.id-b.id)) {
        const key = `type:${a.vaccineTypeId}`; if (seen.has(key)) continue; seen.add(key);
        result.push({ ...base, vaccineTypeId: a.vaccineTypeId, vaccineName: request('vaccine_type_get', { id: a.vaccineTypeId! }).name, dueOn: a.startsOn, appointmentId: a.id, vaccinationId: null });
      }
      const history = doses(p.id);
      for (const d of history) {
        const key = d.vaccineTypeId ? `type:${d.vaccineTypeId}` : `name:${d.vaccineName.toLowerCase()}`;
        if (seen.has(key)) continue; seen.add(key);
        if (d.nextDueOn) result.push({ ...base, vaccineTypeId: d.vaccineTypeId, vaccineName: d.vaccineName, dueOn: d.nextDueOn, appointmentId: null, vaccinationId: d.id });
      }
    }
    return result.filter(r => (!query.patientId || r.patientId === query.patientId) && (!query.species || r.species === query.species) && (!query.vaccineTypeId || r.vaccineTypeId === query.vaccineTypeId) && (!query.from || r.dueOn >= query.from) && (!query.to || r.dueOn <= query.to)).sort((a,b) => a.dueOn.localeCompare(b.dueOn));
  }
  function plan(input: PlanVaccination): Appointment {
    const p = patient(input.patientId), t = type(input.vaccineTypeId, p.id);
    return request('appointment_create', { input: { ...input, ownerId: p.ownerId, kind: 'vaccine', status: 'planned', title: `${t.name} — ${p.name}` } });
  }
  function give(input: GiveVaccination): Vaccination {
    const d = input.vaccination; patient(d.patientId);
    const t = d.vaccineTypeId ? type(d.vaccineTypeId, d.patientId) : null;
    if (d.nextDueOn && (!parseDate(d.nextDueOn) || d.nextDueOn <= d.givenOn)) ctx.fail('invalid', 'Must be after givenOn.', 'nextDueOn');
    if (input.planNext && d.nextDueOn && !t) ctx.fail('invalid', 'Catalog vaccine required.', 'vaccineTypeId');
    if (input.appointmentId) {
      const a = request('appointment_get', { id: input.appointmentId });
      if (a.deletedAt || a.status !== 'planned' || a.vaccinationId) ctx.fail('conflict', 'Appointment already completed.');
      if (a.kind !== 'vaccine' || a.patientId !== d.patientId || a.vaccineTypeId !== d.vaccineTypeId) ctx.fail('invalid', 'Appointment must match patient and vaccine.', 'appointmentId');
    }
    const result = request('vaccination_create', { input: { ...d, vaccineName: t?.name ?? d.vaccineName } });
    if (input.appointmentId) request('appointment_update', { id: input.appointmentId, patch: { status: 'done', vaccinationId: result.id } });
    if (input.planNext && d.nextDueOn && t && !plans().some(a => a.patientId === d.patientId && a.vaccineTypeId === t.id)) plan({ patientId: d.patientId, vaccineTypeId: t.id, startsOn: d.nextDueOn, startsTime: null, notes: null });
    return result;
  }
  const handlers: Record<string, (args: never) => unknown> = {
    vaccines_due_list: ({ query }: CommandMap['vaccines_due_list']['args']) => due(query),
    vaccines_plan: ({ input }: CommandMap['vaccines_plan']['args']) => plan(input),
    vaccines_give: ({ input }: CommandMap['vaccines_give']['args']) => give(input),
    vaccines_suggest: ({ patientId, typeId, givenOn }: CommandMap['vaccines_suggest']['args']) => {
      const t = type(typeId, patientId);
      const count = doses(patientId).filter(d => d.vaccineTypeId === typeId && d.givenOn <= givenOn).length + 1;
      const days = t.primaryDoses && count < t.primaryDoses ? t.primaryIntervalDays ?? t.defaultIntervalDays : t.defaultIntervalDays;
      return days ? dateAfterDays(givenOn, days) : null;
    },
    vaccines_calendar: ({ from, to }: CommandMap['vaccines_calendar']['args']) => {
      const events: VaccineCalendarEvent[] = [];
      for (const p of ctx.patients.filter(p => !p.deletedAt && !ctx.ownerGet(p.ownerId).deletedAt)) {
        for (const d of doses(p.id)) if (d.givenOn >= from && d.givenOn <= to) events.push({ patientId: p.id, patientName: p.name, vaccineName: d.vaccineName, on: d.givenOn, planned: false });
      }
      for (const a of plans()) {
        const p = ctx.patientGet(a.patientId!); if (p.deletedAt || p.deceasedOn || ctx.ownerGet(p.ownerId).deletedAt || a.startsOn < from || a.startsOn > to) continue;
        events.push({ patientId: p.id, patientName: p.name, vaccineName: a.vaccineTypeId ? request('vaccine_type_get', { id: a.vaccineTypeId }).name : a.title, on: a.startsOn, planned: true });
      }
      return events;
    },
    vaccines_postpone: ({ appointmentId, vaccinationId, on }: CommandMap['vaccines_postpone']['args']) => {
      if (appointmentId) {
        const a = request('appointment_get', { id: appointmentId });
        if (a.deletedAt || a.kind !== 'vaccine' || a.status !== 'planned') ctx.fail('conflict', 'Appointment unavailable.');
        return request('appointment_update', { id: appointmentId, patch: { startsOn: on } });
      }
      const d = request('vaccination_get', { id: vaccinationId! });
      if (!d.vaccineTypeId) ctx.fail('invalid', 'Catalog vaccine required.', 'vaccineTypeId');
      const existing = plans().find(a => a.patientId === d.patientId && a.vaccineTypeId === d.vaccineTypeId);
      return existing ? request('appointment_update', { id: existing.id, patch: { startsOn: on } }) : plan({ patientId: d.patientId, vaccineTypeId: d.vaccineTypeId!, startsOn: on, startsTime: null, notes: null });
    }
  };
  // Fixtures share the clinical mock's storage so the agenda and patient tabs stay consistent.
  const catalog = request('vaccine_types_list', {});
  const t = catalog.find(t => t.species === 'cat')!;
  request('vaccine_type_update', { id: t.id, patch: { primaryDoses: 2, primaryIntervalDays: 21 } });
  const p = ctx.patients.find(p => p.species === 'cat')!;
  plan({ patientId: p.id, vaccineTypeId: t.id, startsOn: dateAfterDays(todayDate(), 7), startsTime: '10:00', notes: null });
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args as never) };
}
