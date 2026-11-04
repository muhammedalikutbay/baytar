import { toE164 } from '$lib/validation';
import { tr, importIssueKeys } from '$lib/i18n/tr';
import { createBillingMock } from './billing-mock';
import { createClinicalMock } from './clinical-mock';
import { createVaccineMock } from './vaccinations-mock';
import { createInventoryMock } from './inventory-mock';
import { createAttachmentsMock } from './attachments-mock';
import { createTemplatesMock } from './templates-mock';
import { createLockMock } from './lock-mock';
import { createReportsMock } from './reports-mock';
import { createRemindersMock } from './reminders-mock';
import { calculateMock } from './dose-mock';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { ImportPreview, ImportReport, ApiError, CommandMap, ListQuery, NewOwner, NewPatient, Owner, OwnerPatch, Patient, PatientPatch, SearchHit } from './types';

export const startupStatus = () => call('startup_status', {});
export async function call<K extends keyof CommandMap>(command: K, args: CommandMap[K]['args']): Promise<CommandMap[K]['result']> {
  if (isTauri()) return invoke<CommandMap[K]['result']>(command, args);
  // Clone replies so callers cannot mutate the mock's stored rows through returned references.
  return structuredClone(await mock(command, args)) as CommandMap[K]['result'];
}
function fail(kind: ApiError['kind'], message: string, field?: string): never {
  throw { kind, message, ...(field ? { field } : {}) } satisfies ApiError;
}
function required(field: string, value: string) {
  if (typeof value !== 'string' || !value.trim()) fail('invalid', 'Must not be empty.', field);
}
export function fold(value: string): string {
  return value.replace(/İ/g, 'i').replace(/I/g, 'ı').toLowerCase()
    .replace(/[çğıöşü]/g, c => ({ ç: 'c', ğ: 'g', ı: 'i', ö: 'o', ş: 's', ü: 'u' })[c] ?? c)
    .trim().replace(/\s+/gu, ' ');
}
const normalizePhone = (value: string | null | undefined) => value ? toE164(value) || value : null;
let lastStamp = 0;
function now() { lastStamp = Math.max(Date.now(), lastStamp + 1); return new Date(lastStamp).toISOString(); }
let nextId = 1;
const owners: Owner[] = [];
const patients: Patient[] = [];
const byText = <T extends { searchText: string; id: number }>(a: T, b: T) =>
  (a.searchText < b.searchText ? -1 : a.searchText > b.searchText ? 1 : 0) || a.id - b.id;
function ownerGet(id: number): Owner { return owners.find(o => o.id === id) ?? fail('not_found', 'Record not found.'); }
function patientGet(id: number): Patient { return patients.find(p => p.id === id) ?? fail('not_found', 'Record not found.'); }
function activeOwner(id: number): Owner {
  const owner = ownerGet(id); if (owner.deletedAt) fail('not_found', 'Record not found.'); return owner;
}
function ownerKey(o: Owner) { o.searchText = fold([o.fullName, o.phone, o.phoneAlt].filter(Boolean).join(' ')); }
function patientKey(p: Patient) { p.searchText = fold([p.name, p.breed, p.speciesNote].filter(Boolean).join(' ')); }
function ownerCreate(input: NewOwner): Owner {
  required('fullName', input.fullName);
  const stamp = now();
  const owner: Owner = { id: nextId++, fullName: input.fullName, phone: normalizePhone(input.phone), phoneRaw: input.phone ?? null,
    phoneAlt: normalizePhone(input.phoneAlt), email: input.email ?? null, address: input.address ?? null, notes: input.notes ?? null,
    nationalId: input.nationalId ?? null, city: input.city ?? null, district: input.district ?? null,
    searchText: '', createdAt: stamp, updatedAt: stamp, deletedAt: null };
  ownerKey(owner); owners.push(owner); return owner;
}
function ownerUpdate(id: number, patch: OwnerPatch): Owner {
  const owner = activeOwner(id);
  if (patch.fullName !== undefined) required('fullName', patch.fullName);
  Object.assign(owner, patch);
  if (patch.phone !== undefined) { owner.phoneRaw = patch.phone; owner.phone = normalizePhone(patch.phone); }
  if (patch.phoneAlt !== undefined) owner.phoneAlt = normalizePhone(patch.phoneAlt);
  owner.updatedAt = now(); ownerKey(owner); return owner;
}
function validatePatient(input: NewPatient) {
  required('name', input.name);
  if (!['cat', 'dog', 'bird', 'rabbit', 'rodent', 'ferret', 'reptile', 'other'].includes(input.species)) fail('invalid', 'Unknown species.', 'species');
  if (!['male', 'female', 'unknown'].includes(input.sex ?? 'unknown')) fail('invalid', 'Unknown sex.', 'sex');
  for (const field of ['birthDate', 'deceasedOn'] as const) {
    const value = input[field];
    if (value != null && (!/^\d{4}-\d{2}-\d{2}$/.test(value) || !Number.isFinite(Date.parse(value)) || new Date(value).toISOString().slice(0, 10) !== value)) {
      fail('invalid', 'Expected a valid YYYY-MM-DD date.', field);
    }
  }
}
function patientCreate(input: NewPatient): Patient {
  validatePatient(input); activeOwner(input.ownerId); const stamp = now();
  const patient: Patient = { id: nextId++, ownerId: input.ownerId, name: input.name, species: input.species,
    speciesNote: input.speciesNote ?? null, breed: input.breed ?? null, sex: input.sex ?? 'unknown', neutered: input.neutered ?? null,
    birthDate: input.birthDate ?? null, birthDateIsEstimate: input.birthDateIsEstimate ?? false, color: input.color ?? null,
    microchip: input.microchip ?? null, notes: input.notes ?? null, deceasedOn: input.deceasedOn ?? null,
    alerts: input.alerts ?? null,
    searchText: '', createdAt: stamp, updatedAt: stamp, deletedAt: null };
  patientKey(patient); patients.push(patient); return patient;
}
function patientUpdate(id: number, patch: PatientPatch): Patient {
  const patient = patientGet(id); if (patient.deletedAt) fail('not_found', 'Record not found.');
  const merged = { ...patient, ...patch }; validatePatient(merged); activeOwner(merged.ownerId);
  Object.assign(patient, merged, { updatedAt: now() }); patientKey(patient); return patient;
}
function patientsByOwner(id: number): Patient[] {
  if (!owners.some(o => o.id === id && !o.deletedAt)) return [];
  return patients.filter(p => p.ownerId === id && !p.deletedAt).sort(byText);
}
function ownerDelete(id: number): number[] {
  const owner = ownerGet(id); if (owner.deletedAt) fail('conflict', 'Record already deleted.');
  const stamp = now(); const pets = patientsByOwner(id);
  for (const p of pets) { p.deletedAt = stamp; p.updatedAt = stamp; patientKey(p); }
  owner.deletedAt = stamp; owner.updatedAt = stamp; ownerKey(owner); return pets.map(p => p.id);
}
function ownerRestore(id: number): Owner {
  const owner = ownerGet(id); if (!owner.deletedAt) fail('conflict', 'Record is active.');
  const stamp = now();
  for (const p of patients) if (p.ownerId === id && p.deletedAt === owner.deletedAt) { p.deletedAt = null; p.updatedAt = stamp; patientKey(p); }
  owner.deletedAt = null; owner.updatedAt = stamp; ownerKey(owner); return owner;
}
function patientDeleted(id: number, deleted: boolean): Patient {
  const patient = patientGet(id); activeOwner(patient.ownerId);
  if (Boolean(patient.deletedAt) === deleted) fail('conflict', 'Record conflicts with its current state.');
  patient.updatedAt = now(); patient.deletedAt = deleted ? patient.updatedAt : null; patientKey(patient); return patient;
}
function search(query: string, limit: number): SearchHit[] {
  query = fold(query); if (!query || limit === 0) return [];
  const hits = owners.filter(o => !o.deletedAt).map(owner => ({ owner, patients: patientsByOwner(owner.id).filter(p => p.searchText.includes(query)) }))
    .filter(h => h.owner.searchText.includes(query) || h.patients.length > 0);
  const prefix = (h: SearchHit) => h.owner.searchText.startsWith(query) || h.patients.some(p => p.searchText.startsWith(query));
  return hits.sort((a, b) => Number(prefix(b)) - Number(prefix(a)) || byText(a.owner, b.owner)).slice(0, limit);
}
const first = ownerCreate({ fullName: 'Çağrı Şimşek', phone: '+905321112233' });
const second = ownerCreate({ fullName: 'İpek Öztürk', phone: '+905442223344' });
const third = ownerCreate({ fullName: 'Ayşe Işık' });
patientCreate({ ownerId: first.id, name: 'Pamuk', species: 'cat', breed: 'Ankara kedisi', sex: 'female', neutered: true, birthDate: '2022-04-12', color: 'Beyaz', microchip: '900108001234567' });
patientCreate({ ownerId: first.id, name: 'Şimşek', species: 'dog', breed: 'Golden retriever', sex: 'male', neutered: false, birthDate: '2024-09-01', birthDateIsEstimate: true, color: 'Sarı' });
patientCreate({ ownerId: second.id, name: 'Boncuk', species: 'bird' });
patientCreate({ ownerId: third.id, name: 'Fındık', species: 'rabbit' });

const clinical = createClinicalMock({ patientGet, ownerGet, fail, fold, now });
const vaccines = createVaccineMock({ clinical, patients, patientGet, ownerGet, fail });
const inventory = createInventoryMock({ now, fold, fail, catalog: clinical.call });
const files = createAttachmentsMock({ patientGet, visitGet: id => clinical.call('visit_get', { id }) as import('./types').Visit, fail, now });
const templates = createTemplatesMock({ fail, now });
const lockMock = createLockMock({ fail });
const reports = createReportsMock();

// Synthetic vaccination examples, relative to the local day for useful previews.
const demoDate = (offset: number) => {
  const date = new Date();
  date.setDate(date.getDate() + offset);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
};
const demoTypes = clinical.call('vaccine_types_list', {}) as import('./types').VaccineType[];
for (const [patientIndex, name, dueOffset] of [[0, 'Kuduz aşısı', -5], [0, 'İç parazit', 10], [1, 'Karma aşı', 25]] as const) {
  const type = demoTypes.find(row => row.name === name)!;
  clinical.call('vaccination_create', { input: { patientId: patients[patientIndex].id, vaccineTypeId: type.id,
    vaccineName: type.name, givenOn: demoDate(dueOffset - 365), nextDueOn: demoDate(dueOffset) } });
}
// Synthetic agenda examples relative to the browser's local day.
const mockDay = new Date();
const agendaDate = (offset: number) => {
  const date = new Date(mockDay.getFullYear(), mockDay.getMonth(), mockDay.getDate() + offset);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
};
for (const input of [
  { title: tr.appointmentSampleReminder, startsOn: agendaDate(0), kind: 'reminder' },
  { title: tr.appointmentSampleVisit, startsOn: agendaDate(0), startsTime: '10:00', durationMin: 30, patientId: 4, ownerId: first.id, kind: 'visit' },
  { title: tr.appointmentSampleVaccine, startsOn: agendaDate(0), startsTime: '14:30', patientId: 5, ownerId: first.id, kind: 'vaccine' },
  { title: tr.appointmentSampleVisit, startsOn: agendaDate(-1), patientId: 4, ownerId: first.id, kind: 'visit' },
  { title: tr.appointmentSampleVaccine, startsOn: agendaDate(2), patientId: 4, ownerId: first.id, kind: 'vaccine' },
  { title: tr.appointmentSampleVisit, startsOn: agendaDate(-2), patientId: 4, ownerId: first.id, kind: 'visit', status: 'done' },
  { title: tr.appointmentSampleVisit, startsOn: agendaDate(1), startsTime: '11:30', patientId: 5, ownerId: first.id, kind: 'visit' },
  { title: tr.appointmentSampleVisit, startsOn: agendaDate(5), startsTime: '15:00', patientId: 4, ownerId: first.id, kind: 'visit' }
]) clinical.call('appointment_create', { input });
// Synthetic measurements make the patient history and its weight trend reviewable in a browser.
for (const [visitOn, weightKg] of [['2026-09-12', 3.9], ['2026-10-03', 4.0], ['2026-10-24', 4.1]] as const) {
  clinical.call('visit_create', { input: { patientId: 4, visitOn, weightKg, reason: tr.visit } });
}

const billing = createBillingMock({ owners: () => owners, ownerGet, patientGet, visitGet: id => clinical.call('visit_get', { id }) as import('./types').Visit, now, fail });

const reminders = createRemindersMock({ vaccines, billing, clinical, patientGet, ownerGet, now, fail });

// Dispatch types are checked at the public call boundary and by each handler.

const importPrepared = new Map<string, ImportPreview>();
const importReports = new Map<string, ImportReport>();
const importedFiles = new Set<string>();
const settingValues: Record<string, string> = { update_check_enabled: '1' };
const mockBackups: import('./types').BackupInfo[] = [];
let importSequence = 0;
function mockPreview(path: string): ImportPreview {
  const found = { owners: 24, patients: 31, todos: 8 };
  const empty = { owners: 0, patients: 0, todos: 0 };
  const imported = importedFiles.has(path);
  return { sourceName: path, sourceSha256: path, schema: { tables: [], unknownTables: [], missingTables: [], unknownColumns: [], missingColumns: [] },
    found, alreadyImported: imported ? found : empty, toImport: imported ? empty : found,
    phonesAdjusted: 12, phonesUnparsed: 1, datesUnparsed: { 'Patient.BirthDate': 1 }, speciesMappedToOther: 1, speciesMissing: 0, sexMappedToUnknown: 2,
    orphanPatients: 1, orphanTodos: 0, duplicateOwners: 1, todosDone: 3, todosPlanned: 5, orphanOwnerNeeded: true,
    issues: [
      { table: 'Patient', sourceId: 7, field: 'BirthDate', kind: 'dateUnparsed', raw: '2019 ilkbahar' },
      { table: 'Patient', sourceId: 12, field: 'OwnerId', kind: 'orphanPatient', raw: '999' },
      { table: 'PatientOwner', sourceId: 4, field: 'OwnerName', kind: 'duplicateOwner', raw: 'Örnek Sahip' },
      { table: 'PatientOwner', sourceId: 9, field: 'OwnerPhone', kind: 'phoneUnparsed', raw: 'telefon yok' }
    ] };
}

function mock(command: keyof CommandMap, args: CommandMap[keyof CommandMap]['args']): unknown {
  if (billing.has(command)) return billing.call(command, args);
  if (vaccines.has(command)) return vaccines.call(command, args);
  if (inventory.has(command)) return inventory.call(command, args);
  if (files.has(command)) return files.call(command, args);
  if (templates.has(command)) return templates.call(command, args);
  if (reports.has(command)) return reports.call(command, args);
  if (reminders.has(command)) return reminders.call(command, args);
  if (command === 'patient_mark_deceased') {
    const { id, date } = args as CommandMap['patient_mark_deceased']['args'];
    const patient = patientGet(id);
    if (date > new Date().toLocaleDateString('sv-SE') || (patient.birthDate && date < patient.birthDate)) fail('invalid', 'Must be between birth and today.', 'deceasedOn');
    return patientUpdate(id, { deceasedOn: date });
  }
  if (clinical.has(command)) return clinical.call(command, args);
  if (lockMock.has(command)) return lockMock.call(command, args);
  switch (command) {
    case 'settings_get_all': return settingValues;
    case 'settings_set': { const { key, value } = args as { key: string; value: string }; settingValues[key] = value; return null; }
    case 'backups_list': return mockBackups;
    case 'backup_create': { const row = { path: '/demo/baytar-manual-20261104-120000Z.db', createdAt: new Date().toISOString(), kind: 'manual' as const, sizeBytes: 204800 }; mockBackups.unshift(row); return row; }
    case 'backup_before_update': { const row = { path: '/demo/baytar-pre-update-20261104-120000Z.db', createdAt: new Date().toISOString(), kind: 'pre-update' as const, sizeBytes: 204800 }; mockBackups.unshift(row); return row; }
    case 'backup_export': return (args as { path: string }).path + '/baytar-manual-20261104-120000Z.db';
    case 'backup_restore': return null;
    case 'app_info': return { version: 'dev', dataFolder: '/demo/Baytar', backupsFolder: '/demo/Baytar/backups', databaseSizeBytes: 204800 };
    case 'open_data_folder': return null;
    case 'open_whatsapp': return null;
    case 'dose_calculate': return calculateMock((args as CommandMap['dose_calculate']['args']).input);
    case 'import_candidates': return [{ path: '/demo/HastaTakipDB.db', fileName: 'HastaTakipDB.db', sizeBytes: 942080, modifiedAt: '2026-10-30T12:00:00Z', location: '/demo' }];
    case 'import_prepare': { const { path } = args as { path: string }; const token = String(++importSequence); const preview = mockPreview(path); importPrepared.set(token, preview); return { token, preview }; }
    case 'import_cancel': { importPrepared.delete((args as { token: string }).token); return null; }
    case 'import_execute': {
      const { token } = args as { token: string }; const preview = importPrepared.get(token) ?? fail('invalid', 'import_expired');
      const report: ImportReport = { ...preview, inserted: preview.toImport, skipped: preview.alreadyImported, runId: Number(token), orphanOwnerCreated: true, backupFile: 'baytar-pre-import-20261104-120000.db' };
      importedFiles.add(preview.sourceSha256); importPrepared.delete(token); importReports.set(token, report); return report;
    }
    case 'import_save_report': {
      const { tokenOrRunId } = args as { tokenOrRunId: string };
      const report = importReports.get(tokenOrRunId) ?? importPrepared.get(tokenOrRunId) ?? fail('invalid', 'import_expired');
      const content = [tr.importTitle, tr.importOldSafe, ...('backupFile' in report ? [tr.importBackup + ': ' + report.backupFile] : []), ...(['owners', 'patients', 'todos'] as const).map(k => tr[({ owners: 'importOwners', patients: 'importPatients', todos: 'importTodos' } as const)[k]] + ': ' + report.toImport[k]), ...report.issues.map(i => tr[importIssueKeys[i.kind]] + ': ' + (i.raw ?? tr.importNoValue))].join('\n');
      const url = URL.createObjectURL(new Blob([content], { type: 'text/plain;charset=utf-8' })); const link = document.createElement('a'); link.href = url; link.download = 'baytar-aktarma-raporu.txt'; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000); return null;
    }
    case 'startup_status': return { ready: true, error: null, code: null, foundVersion: null, supportedVersion: null };
    case 'owners_list': {
      const query = (args as { query: ListQuery }).query;
      return owners.filter(o => query.includeDeleted || !o.deletedAt).sort(byText).slice(query.offset ?? 0, (query.offset ?? 0) + (query.limit ?? 100));
    }
    case 'owner_create': return ownerCreate((args as { input: NewOwner }).input);
    case 'owner_update': { const { id, patch } = args as { id: number; patch: OwnerPatch }; return ownerUpdate(id, patch); }
    case 'patient_create': return patientCreate((args as { input: NewPatient }).input);
    case 'patient_update': { const { id, patch } = args as { id: number; patch: PatientPatch }; return patientUpdate(id, patch); }
    case 'patients_by_owner': return patientsByOwner((args as { ownerId: number }).ownerId);
    case 'directory_list': {
      const { query, limit } = args as CommandMap['directory_list']['args'];
      const text = fold(query), digits = text.replace(/\D/g, '');
      const phone = !!digits && /^[\d+ ()./\-]+$/.test(text);
      const active = owners.filter(o => !o.deletedAt).sort(byText);
      return { total: active.length, owners: active.filter(o => !text || o.searchText.includes(text)
        || (phone && [o.phone, o.phoneAlt].some(p => p?.replace(/\D/g, '').includes(digits)))
        || patientsByOwner(o.id).some(p => p.searchText.includes(text)))
        .slice(0, Math.max(0, Math.min(1000, limit))).map(owner => ({ owner, patients: patientsByOwner(owner.id) })) };
    }
    case 'search': { const { query, limit } = args as { query: string; limit: number }; return search(query, limit); }
    default: {
      const { id } = args as { id: number };
      switch (command) {
        case 'owner_get': return ownerGet(id);
        case 'owner_detail': return { owner: ownerGet(id), patients: patientsByOwner(id) };
        case 'owner_delete': return ownerDelete(id);
        case 'owner_restore': return ownerRestore(id);
        case 'patient_get': return patientGet(id);
        case 'patient_delete': return patientDeleted(id, true);
        case 'patient_restore': return patientDeleted(id, false);
      }
    }
  }
}
