import { invoke, isTauri } from '@tauri-apps/api/core';
import type { ApiError, CommandMap, ListQuery, NewOwner, NewPatient, Owner, OwnerPatch, Patient, PatientPatch, SearchHit } from './types';

export const startupStatus = () => call('startup_status', {});
export async function call<K extends keyof CommandMap>(command: K, args: CommandMap[K]['args']): Promise<CommandMap[K]['result']> {
  if (isTauri()) return invoke<CommandMap[K]['result']>(command, args);
  // Clone replies so callers cannot mutate the mock's stored rows through returned references.
  return structuredClone(mock(command, args)) as CommandMap[K]['result'];
}
function fail(kind: ApiError['kind'], message: string, field?: string): never {
  throw { kind, message, ...(field ? { field } : {}) } satisfies ApiError;
}
function required(field: string, value: string) {
  if (typeof value !== 'string' || !value.trim()) fail('invalid', 'Must not be empty.', field);
}
function fold(value: string): string {
  return value.replace(/İ/g, 'i').replace(/I/g, 'ı').toLowerCase()
    .replace(/[çğıöşü]/g, c => ({ ç: 'c', ğ: 'g', ı: 'i', ö: 'o', ş: 's', ü: 'u' })[c] ?? c)
    .trim().replace(/\s+/gu, ' ');
}
const digits = (value: string | null | undefined) => value?.replace(/[^0-9]/g, '') || null;
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
  const owner: Owner = { id: nextId++, fullName: input.fullName, phone: digits(input.phone), phoneRaw: input.phone ?? null,
    phoneAlt: digits(input.phoneAlt), email: input.email ?? null, address: input.address ?? null, notes: input.notes ?? null,
    searchText: '', createdAt: stamp, updatedAt: stamp, deletedAt: null };
  ownerKey(owner); owners.push(owner); return owner;
}
function ownerUpdate(id: number, patch: OwnerPatch): Owner {
  const owner = activeOwner(id);
  if (patch.fullName !== undefined) required('fullName', patch.fullName);
  Object.assign(owner, patch);
  if (patch.phone !== undefined) { owner.phoneRaw = patch.phone; owner.phone = digits(patch.phone); }
  if (patch.phoneAlt !== undefined) owner.phoneAlt = digits(patch.phoneAlt);
  owner.updatedAt = now(); ownerKey(owner); return owner;
}
function validatePatient(input: NewPatient) {
  required('name', input.name);
  if (!['cat', 'dog', 'bird', 'rabbit', 'other'].includes(input.species)) fail('invalid', 'Unknown species.', 'species');
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
const first = ownerCreate({ fullName: 'Çağrı Şimşek', phone: '0532 111 22 33' });
const second = ownerCreate({ fullName: 'İpek Öztürk', phone: '0544 222 33 44' });
const third = ownerCreate({ fullName: 'Ayşe Işık' });
patientCreate({ ownerId: first.id, name: 'Pamuk', species: 'cat', breed: 'Ankara kedisi', sex: 'female' });
patientCreate({ ownerId: first.id, name: 'Şimşek', species: 'dog', sex: 'male' });
patientCreate({ ownerId: second.id, name: 'Boncuk', species: 'bird' });
patientCreate({ ownerId: third.id, name: 'Fındık', species: 'rabbit' });

// Dispatch types are checked at the public call boundary and by each handler.
function mock(command: keyof CommandMap, args: CommandMap[keyof CommandMap]['args']): unknown {
  switch (command) {
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
