import type { Audit, ApiError, Owner, Patient, Visit, NewVisit, VisitPatch, VaccineType, NewVaccineType, VaccineTypePatch, Vaccination, NewVaccination, VaccinationPatch, Medication, NewMedication, MedicationPatch, PrescriptionItem, NewPrescription, PrescriptionPatch, Appointment, NewAppointment, AppointmentPatch, LatestWeight, DueVaccination, CalendarAppointment, CommandMap } from './types';
interface Context { patientGet(id: number): Patient; ownerGet(id: number): Owner; fail(kind: ApiError["kind"], message: string, field?: string): never; fold(value: string): string; now(): string }
export function createClinicalMock(ctx: Context) {
let nextId = 100;
function audit(): Audit { const stamp = ctx.now(); return { id: nextId++, createdAt: stamp, updatedAt: stamp, deletedAt: null }; }
function deleted<T extends Audit>(row: T, value: boolean): T { if (!value && "isActive" in row) vaccine_typesPrepare(row as unknown as VaccineType, false); if (Boolean(row.deletedAt) === value) ctx.fail("conflict", "Record conflicts with its current state."); row.updatedAt = ctx.now(); row.deletedAt = value ? row.updatedAt : null; return row; }
function active<T extends Audit>(row: T): T { if (row.deletedAt) ctx.fail("not_found", "Record not found."); return row; }
const visits: Visit[] = [];
function visitsGet(id: number): Visit { return visits.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function visitsCreate(input: NewVisit): Visit {
const row: Visit = { patientId: input.patientId, visitOn: input.visitOn, visitTime: input.visitTime ?? null, reason: input.reason ?? null, findings: input.findings ?? null, diagnosis: input.diagnosis ?? null, treatment: input.treatment ?? null, weightKg: input.weightKg ?? null, temperatureC: input.temperatureC ?? null, notes: input.notes ?? null, ...audit() };
visitsPrepare(row, true); visits.push(row); return row; }
function visitsUpdate(id: number, patch: VisitPatch): Visit { const old = active(visitsGet(id)); const row = { ...old, ...patch }; visitsPrepare(row, false); Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
const vaccine_types: VaccineType[] = [];
function vaccine_typesGet(id: number): VaccineType { return vaccine_types.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function vaccine_typesCreate(input: NewVaccineType): VaccineType {
const row: VaccineType = { name: input.name, species: input.species ?? null, defaultIntervalDays: input.defaultIntervalDays ?? null, notes: input.notes ?? null, isActive: input.isActive ?? true, ...audit() };
vaccine_typesPrepare(row, true); vaccine_types.push(row); return row; }
function vaccine_typesUpdate(id: number, patch: VaccineTypePatch): VaccineType { const old = active(vaccine_typesGet(id)); const row = { ...old, ...patch }; vaccine_typesPrepare(row, false); Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
const vaccinations: Vaccination[] = [];
function vaccinationsGet(id: number): Vaccination { return vaccinations.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function vaccinationsCreate(input: NewVaccination): Vaccination {
const row: Vaccination = { patientId: input.patientId, vaccineTypeId: input.vaccineTypeId ?? null, vaccineName: input.vaccineName, givenOn: input.givenOn, nextDueOn: input.nextDueOn ?? null, batchNo: input.batchNo ?? null, visitId: input.visitId ?? null, notes: input.notes ?? null, ...audit() };
vaccinationsPrepare(row, true); vaccinations.push(row); return row; }
function vaccinationsUpdate(id: number, patch: VaccinationPatch): Vaccination { const old = active(vaccinationsGet(id)); const row = { ...old, ...patch }; vaccinationsPrepare(row, false); Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
const medications: Medication[] = [];
function medicationsGet(id: number): Medication { return medications.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function medicationsCreate(input: NewMedication): Medication {
const row: Medication = { name: input.name, activeIngredient: input.activeIngredient ?? null, form: input.form ?? null, concentrationMgPerMl: input.concentrationMgPerMl ?? null, strengthMgPerUnit: input.strengthMgPerUnit ?? null, doseMgPerKgMin: input.doseMgPerKgMin ?? null, doseMgPerKgMax: input.doseMgPerKgMax ?? null, route: input.route ?? null, notes: input.notes ?? null, ...audit() };
medicationsPrepare(row, true); medications.push(row); return row; }
function medicationsUpdate(id: number, patch: MedicationPatch): Medication { const old = active(medicationsGet(id)); const row = { ...old, ...patch }; medicationsPrepare(row, false); Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
const prescriptions: PrescriptionItem[] = [];
function prescriptionsGet(id: number): PrescriptionItem { return prescriptions.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function prescriptionsCreate(input: NewPrescription): PrescriptionItem {
const row: PrescriptionItem = { visitId: input.visitId, medicationId: input.medicationId ?? null, medicationName: input.medicationName ?? '', doseMg: input.doseMg ?? null, doseMl: input.doseMl ?? null, doseText: input.doseText ?? null, frequency: input.frequency ?? null, durationDays: input.durationDays ?? null, notes: input.notes ?? null, ...audit() };
prescriptionsPrepare(row, true); prescriptions.push(row); return row; }
function prescriptionsUpdate(id: number, patch: PrescriptionPatch): PrescriptionItem { const old = active(prescriptionsGet(id)); const row = { ...old, ...patch }; prescriptionsPrepare(row, row.medicationId !== old.medicationId); if (row.medicationId !== null && row.medicationId === old.medicationId) row.medicationName = old.medicationName; Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
const appointments: Appointment[] = [];
function appointmentsGet(id: number): Appointment { return appointments.find(r => r.id === id) ?? ctx.fail("not_found", "Record not found."); }
function appointmentsCreate(input: NewAppointment): Appointment {
const row: Appointment = { patientId: input.patientId ?? null, ownerId: input.ownerId ?? null, title: input.title, startsOn: input.startsOn, startsTime: input.startsTime ?? null, durationMin: input.durationMin ?? null, kind: input.kind ?? "other", status: input.status ?? "planned", seriesId: input.seriesId ?? null, notes: input.notes ?? null, ...audit() };
appointmentsPrepare(row, true); appointments.push(row); return row; }
function appointmentsUpdate(id: number, patch: AppointmentPatch): Appointment { const old = active(appointmentsGet(id)); const row = { ...old, ...patch }; appointmentsPrepare(row, false); Object.assign(old, row, { updatedAt: ctx.now() }); return old; }
function visitsList(patientId: number) { return visits.filter(v => !v.deletedAt && v.patientId === patientId).sort((a,b) => b.visitOn.localeCompare(a.visitOn) || (b.visitTime ?? "").localeCompare(a.visitTime ?? "") || b.id-a.id); }
function vaccinationsList(patientId: number) { return vaccinations.filter(v => !v.deletedAt && v.patientId === patientId).sort((a,b) => b.givenOn.localeCompare(a.givenOn) || b.id-a.id); }
function prescriptionsList(visitId: number) { return prescriptions.filter(v => !v.deletedAt && v.visitId === visitId); }
function vaccine_typesList() { return vaccine_types.filter(v => !v.deletedAt).sort((a,b) => Number(b.isActive)-Number(a.isActive) || a.name.localeCompare(b.name) || a.id-b.id); }
function medicationsList() { return medications.filter(v => !v.deletedAt).sort((a,b) => {const x=ctx.fold(a.name), y=ctx.fold(b.name); return (x<y?-1:x>y?1:0)||a.id-b.id;}); }
const handlers: Record<string, (args: unknown) => unknown> = {
'visit_get': (args) => { const { id } = args as CommandMap["visit_get"]["args"]; return visitsGet(id); },
'visit_create': (args) => { const { input } = args as CommandMap["visit_create"]["args"]; return visitsCreate(input); },
'visit_update': (args) => { const { id, patch } = args as CommandMap["visit_update"]["args"]; return visitsUpdate(id, patch); },
'visit_delete': (args) => { const { id } = args as CommandMap["visit_delete"]["args"]; return deleted(visitsGet(id), true); },
'visit_restore': (args) => { const { id } = args as CommandMap["visit_restore"]["args"]; return deleted(visitsGet(id), false); },
'visits_by_patient': (args) => { const { patientId } = args as CommandMap["visits_by_patient"]["args"]; return visitsList(patientId); },
'visit_latest_weight': (args) => { const { patientId } = args as CommandMap["visit_latest_weight"]["args"]; return latestWeight(patientId); },
'vaccine_type_get': (args) => { const { id } = args as CommandMap["vaccine_type_get"]["args"]; return vaccine_typesGet(id); },
'vaccine_type_create': (args) => { const { input } = args as CommandMap["vaccine_type_create"]["args"]; return vaccine_typesCreate(input); },
'vaccine_type_update': (args) => { const { id, patch } = args as CommandMap["vaccine_type_update"]["args"]; return vaccine_typesUpdate(id, patch); },
'vaccine_type_delete': (args) => { const { id } = args as CommandMap["vaccine_type_delete"]["args"]; return deleted(vaccine_typesGet(id), true); },
'vaccine_type_restore': (args) => { const { id } = args as CommandMap["vaccine_type_restore"]["args"]; return deleted(vaccine_typesGet(id), false); },
'vaccine_types_list': (args) => { void args; return vaccine_typesList(); },
'vaccination_get': (args) => { const { id } = args as CommandMap["vaccination_get"]["args"]; return vaccinationsGet(id); },
'vaccination_create': (args) => { const { input } = args as CommandMap["vaccination_create"]["args"]; return vaccinationsCreate(input); },
'vaccination_update': (args) => { const { id, patch } = args as CommandMap["vaccination_update"]["args"]; return vaccinationsUpdate(id, patch); },
'vaccination_delete': (args) => { const { id } = args as CommandMap["vaccination_delete"]["args"]; return deleted(vaccinationsGet(id), true); },
'vaccination_restore': (args) => { const { id } = args as CommandMap["vaccination_restore"]["args"]; return deleted(vaccinationsGet(id), false); },
'vaccinations_by_patient': (args) => { const { patientId } = args as CommandMap["vaccinations_by_patient"]["args"]; return vaccinationsList(patientId); },
'vaccination_suggest_next_due': (args) => { const { typeId, givenOn } = args as CommandMap["vaccination_suggest_next_due"]["args"]; return suggestNextDue(typeId, givenOn); },
'vaccinations_due_between': (args) => { const { from, to } = args as CommandMap["vaccinations_due_between"]["args"]; return due(from, to, false); },
'vaccinations_overdue': (args) => { const { today } = args as CommandMap["vaccinations_overdue"]["args"]; return due(null, today, true); },
'medication_get': (args) => { const { id } = args as CommandMap["medication_get"]["args"]; return medicationsGet(id); },
'medication_create': (args) => { const { input } = args as CommandMap["medication_create"]["args"]; return medicationsCreate(input); },
'medication_update': (args) => { const { id, patch } = args as CommandMap["medication_update"]["args"]; return medicationsUpdate(id, patch); },
'medication_delete': (args) => { const { id } = args as CommandMap["medication_delete"]["args"]; return deleted(medicationsGet(id), true); },
'medication_restore': (args) => { const { id } = args as CommandMap["medication_restore"]["args"]; return deleted(medicationsGet(id), false); },
'medications_list': (args) => { void args; return medicationsList(); },
'medications_search': (args) => { const { query } = args as CommandMap["medications_search"]["args"]; return medicationsList().filter(m => ctx.fold(m.name).includes(ctx.fold(query)) || ctx.fold(m.activeIngredient ?? "").includes(ctx.fold(query))); },
'prescription_get': (args) => { const { id } = args as CommandMap["prescription_get"]["args"]; return prescriptionsGet(id); },
'prescription_create': (args) => { const { input } = args as CommandMap["prescription_create"]["args"]; return prescriptionsCreate(input); },
'prescription_update': (args) => { const { id, patch } = args as CommandMap["prescription_update"]["args"]; return prescriptionsUpdate(id, patch); },
'prescription_delete': (args) => { const { id } = args as CommandMap["prescription_delete"]["args"]; return deleted(prescriptionsGet(id), true); },
'prescription_restore': (args) => { const { id } = args as CommandMap["prescription_restore"]["args"]; return deleted(prescriptionsGet(id), false); },
'prescriptions_by_visit': (args) => { const { visitId } = args as CommandMap["prescriptions_by_visit"]["args"]; return prescriptionsList(visitId); },
'appointment_get': (args) => { const { id } = args as CommandMap["appointment_get"]["args"]; return appointmentsGet(id); },
'appointment_create': (args) => { const { input } = args as CommandMap["appointment_create"]["args"]; return appointmentsCreate(input); },
'appointment_update': (args) => { const { id, patch } = args as CommandMap["appointment_update"]["args"]; return appointmentsUpdate(id, patch); },
'appointment_delete': (args) => { const { id } = args as CommandMap["appointment_delete"]["args"]; return deleted(appointmentsGet(id), true); },
'appointment_restore': (args) => { const { id } = args as CommandMap["appointment_restore"]["args"]; return deleted(appointmentsGet(id), false); },
'appointment_set_status': (args) => { const { id, status } = args as CommandMap["appointment_set_status"]["args"]; return appointmentsUpdate(id, { status }); },
'appointment_create_series': (args) => { const { base, count, everyDays } = args as CommandMap["appointment_create_series"]["args"]; return createSeries(base, count, everyDays); },
'appointments_between': (args) => { const { from, to } = args as CommandMap["appointments_between"]["args"]; return between(from, to); },
'appointments_today': (args) => { const { today } = args as CommandMap["appointments_today"]["args"]; return between(today, today); },
};

const asciiLower = (value: string) => value.replace(/[A-Z]/g, c => c.toLowerCase());
function required(field: string, value: string) { if (typeof value !== 'string' || !value.trim()) ctx.fail('invalid', 'Must not be empty.', field); }
function date(field: string, value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value) || !Number.isFinite(Date.parse(value)) || new Date(value).toISOString().slice(0,10) !== value) ctx.fail('invalid', 'Expected a valid YYYY-MM-DD date.', field);
}
function time(field: string, value: string | null) { if (value !== null && !/^([01]\d|2[0-3]):[0-5]\d$/.test(value)) ctx.fail('invalid', 'Expected a valid HH:MM time.', field); }
function positive(field: string, value: number | null) { if (value !== null && (!Number.isFinite(value) || value <= 0)) ctx.fail('invalid', 'Must be finite and greater than zero.', field); }
function integer(field: string, value: number | null) { positive(field, value); if (value !== null && !Number.isSafeInteger(value)) ctx.fail('invalid', 'Expected an integer.', field); }
function patient(id: number) { const p = active(ctx.patientGet(id)); active(ctx.ownerGet(p.ownerId)); return p; }
function visitsPrepare(r: Visit, _snapshot: boolean) {
  date('visitOn', r.visitOn); time('visitTime', r.visitTime); positive('weightKg', r.weightKg);
  if (r.temperatureC !== null && !Number.isFinite(r.temperatureC)) ctx.fail('invalid', 'Must be finite.', 'temperatureC');
  patient(r.patientId);
  if (vaccinations.some(v => v.visitId === r.id && v.patientId !== r.patientId)) ctx.fail('invalid', 'Must match linked vaccinations.', 'patientId');
}
function vaccine_typesPrepare(r: VaccineType, _snapshot: boolean) {
  required('name', r.name); integer('defaultIntervalDays', r.defaultIntervalDays);
  if (r.species !== null && !['cat','dog','bird','rabbit','other'].includes(r.species)) ctx.fail('invalid', 'Unknown species.', 'species');
  if (vaccine_types.some(v => v.id !== r.id && !v.deletedAt && asciiLower(v.name) === asciiLower(r.name))) ctx.fail('conflict', 'Record conflicts with its current state.');
}
function vaccinationsPrepare(r: Vaccination, _snapshot: boolean) {
  required('vaccineName', r.vaccineName); date('givenOn', r.givenOn); if (r.nextDueOn !== null) date('nextDueOn', r.nextDueOn);
  patient(r.patientId); if (r.vaccineTypeId !== null) vaccine_typesGet(r.vaccineTypeId);
  if (r.visitId !== null) { const visit = active(visitsGet(r.visitId)); if (visit.patientId !== r.patientId) ctx.fail('invalid', 'Visit must belong to patient.', 'visitId'); }
}
function medicationsPrepare(r: Medication, _snapshot: boolean) {
  required('name', r.name);
  if (r.form !== null && !['tablet','capsule','injection','suspension','drops','ointment','other'].includes(r.form)) ctx.fail('invalid', 'Unknown medication form.', 'form');
  for (const field of ['concentrationMgPerMl','strengthMgPerUnit','doseMgPerKgMin','doseMgPerKgMax'] as const) positive(field, r[field]);
  if (r.doseMgPerKgMin !== null && r.doseMgPerKgMax !== null && r.doseMgPerKgMax < r.doseMgPerKgMin) ctx.fail('invalid', 'Must be at least doseMgPerKgMin.', 'doseMgPerKgMax');
}
function prescriptionsPrepare(r: PrescriptionItem, snapshot: boolean) {
  const visit = active(visitsGet(r.visitId)); patient(visit.patientId);
  if (r.medicationId !== null && snapshot) r.medicationName = active(medicationsGet(r.medicationId)).name;
  required('medicationName', r.medicationName); positive('doseMg', r.doseMg); positive('doseMl', r.doseMl); integer('durationDays', r.durationDays);
}
function appointmentsPrepare(r: Appointment, _snapshot: boolean) {
  required('title', r.title); date('startsOn', r.startsOn); time('startsTime', r.startsTime); integer('durationMin', r.durationMin);
  if (!['visit','vaccine','reminder','other'].includes(r.kind)) ctx.fail('invalid', 'Unknown appointment kind.', 'kind');
  if (!['planned','done','cancelled'].includes(r.status)) ctx.fail('invalid', 'Unknown appointment status.', 'status');
  if (r.patientId !== null) { const p = patient(r.patientId); if (r.ownerId !== null && r.ownerId !== p.ownerId) ctx.fail('invalid', "Owner must match patient's owner.", 'ownerId'); }
  if (r.ownerId !== null) active(ctx.ownerGet(r.ownerId));
}
function addDays(value: string, days: number, field: string) {
  const next = new Date(value + 'T00:00:00.000Z'); next.setUTCDate(next.getUTCDate()+days);
  if (!Number.isFinite(next.getTime())) ctx.fail('invalid', 'Date overflow.', field);
  const result = next.toISOString().slice(0,10); date(field, result); return result;
}
function suggestNextDue(typeId: number, givenOn: string) { date('givenOn', givenOn); const interval = vaccine_typesGet(typeId).defaultIntervalDays; return interval === null ? null : addDays(givenOn, interval, 'defaultIntervalDays'); }
function latestWeight(patientId: number): LatestWeight | null { const visit = visitsList(patientId).find(v => v.weightKg !== null); return visit ? { weightKg: visit.weightKg!, visitOn: visit.visitOn } : null; }
function range(from: string, to: string) { date('from', from); date('to', to); if (from > to) ctx.fail('invalid', 'Must be on or after from.', 'to'); }
function due(from: string | null, to: string, overdue: boolean): DueVaccination[] {
  if (from !== null) range(from, to); else date('today', to);
  return vaccinations.filter(v => {
    if (v.deletedAt || v.nextDueOn === null || (from !== null && v.nextDueOn < from) || (overdue ? v.nextDueOn >= to : v.nextDueOn > to)) return false;
    const p = ctx.patientGet(v.patientId); if (p.deletedAt || p.deceasedOn || ctx.ownerGet(p.ownerId).deletedAt) return false;
    return !vaccinations.some(l => !l.deletedAt && l.patientId === v.patientId && ((v.vaccineTypeId !== null && v.vaccineTypeId === l.vaccineTypeId) || asciiLower(v.vaccineName) === asciiLower(l.vaccineName)) && (l.givenOn > v.givenOn || (l.givenOn === v.givenOn && l.id > v.id)));
  }).sort((a,b) => a.nextDueOn!.localeCompare(b.nextDueOn!) || a.id-b.id).map(v => { const p = ctx.patientGet(v.patientId); return { vaccination: v, patientName: p.name, ownerName: ctx.ownerGet(p.ownerId).fullName }; });
}
function between(from: string, to: string): CalendarAppointment[] {
  range(from,to);
  return appointments.filter(a => !a.deletedAt && a.startsOn >= from && a.startsOn <= to).sort((a,b) => a.startsOn.localeCompare(b.startsOn) || (a.startsTime ?? '').localeCompare(b.startsTime ?? '') || a.id-b.id).map(a => {
    const p = a.patientId === null ? null : ctx.patientGet(a.patientId), ownerId = a.ownerId ?? p?.ownerId;
    return { appointment: a, patientName: p?.name ?? null, ownerName: ownerId === undefined ? null : ctx.ownerGet(ownerId).fullName };
  });
}
function createSeries(base: NewAppointment, count: number, everyDays: number) {
  if (!Number.isInteger(count) || count < 1 || count > 52) ctx.fail('invalid', 'Must be between 1 and 52.', 'count');
  integer('everyDays', everyDays); date('startsOn', base.startsOn);
  const dates = Array.from({ length: count }, (_, index) => addDays(base.startsOn, index*everyDays, 'everyDays'));
  const seriesId = crypto.randomUUID().replaceAll('-', '');
  // Validate all inputs before changing the store.
  const size = appointments.length;
  try { return dates.map(startsOn => appointmentsCreate({ ...base, startsOn, seriesId })); }
  catch (error) { appointments.splice(size); throw error; }
}
for (const [name, interval, species] of [
  ['Karma aşı',365,'dog'], ['Kuduz aşısı',365,null], ['İç parazit',90,null], ['Dış parazit',30,null], ['Kedi karma aşısı',365,'cat'], ['Lösemi aşısı (FeLV)',365,'cat']
] as const) vaccine_typesCreate({ name, defaultIntervalDays: interval, species });
// Synthetic examples use the same patient ids as the owner/patient browser fixture.
const example = visitsCreate({ patientId: 4, visitOn: '2026-11-04', weightKg: 4.2 });
const medicine = medicationsCreate({ name: 'Example medication', activeIngredient: 'Example ingredient', form: 'tablet', strengthMgPerUnit: 50 });
prescriptionsCreate({ visitId: example.id, medicationId: medicine.id, medicationName: medicine.name, doseMg: 25 });
vaccinationsCreate({ patientId: 4, vaccineTypeId: vaccine_types[4].id, vaccineName: vaccine_types[4].name, givenOn: '2026-11-04', nextDueOn: '2027-11-04' });
appointmentsCreate({ patientId: 4, startsOn: '2026-11-05', startsTime: '09:00', title: 'Example appointment', kind: 'visit' });

return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args) };
}
