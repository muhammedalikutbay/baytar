export type Species = 'cat' | 'dog' | 'bird' | 'rabbit' | 'other';
export type Sex = 'male' | 'female' | 'unknown';
export interface Audit { id: number; createdAt: string; updatedAt: string; deletedAt: string | null }
export interface Owner extends Audit {
  fullName: string; phone: string | null; phoneRaw: string | null; phoneAlt: string | null;
  email: string | null; address: string | null; notes: string | null; searchText: string;
}
export interface Patient extends Audit {
  ownerId: number; name: string; species: Species; speciesNote: string | null; breed: string | null;
  sex: Sex; neutered: boolean | null; birthDate: string | null; birthDateIsEstimate: boolean;
  color: string | null; microchip: string | null; notes: string | null; deceasedOn: string | null; searchText: string;
}
// Phone is the value as typed: the backend stores its digits and preserves phoneRaw.
export type NewOwner = Pick<Owner, 'fullName'> & Partial<Pick<Owner, 'phone' | 'phoneAlt' | 'email' | 'address' | 'notes'>>;
export type OwnerPatch = Partial<NewOwner>;
export type NewPatient = Pick<Patient, 'ownerId' | 'name' | 'species'> & Partial<Omit<Patient, keyof Audit | 'ownerId' | 'name' | 'species' | 'searchText'>>;
export type PatientPatch = Partial<NewPatient>;
export interface ListQuery { includeDeleted?: boolean; limit?: number; offset?: number }
export interface OwnerDetail { owner: Owner; patients: Patient[] }
export type SearchHit = OwnerDetail;
export interface ApiError { kind: 'invalid' | 'not_found' | 'conflict' | 'internal'; field?: string; message: string }
export interface StartupStatus {
  ready: boolean; error: ApiError | null;
  code: 'newer_schema' | 'not_baytar_database' | 'database_unavailable' | 'backup_failed' | null;
  foundVersion: number | null; supportedVersion: number | null;
}
export interface CommandMap {
  import_candidates: { args: Record<string, never>; result: ImportCandidate[] };
  import_prepare: { args: { path: string }; result: PreparedImport };
  import_execute: { args: { token: string }; result: ImportReport };
  import_cancel: { args: { token: string }; result: null };
  import_save_report: { args: { tokenOrRunId: string; path: string }; result: null };
  startup_status: { args: Record<string, never>; result: StartupStatus };
  owners_list: { args: { query: ListQuery }; result: Owner[] };
  owner_get: { args: { id: number }; result: Owner };
  owner_detail: { args: { id: number }; result: OwnerDetail };
  owner_create: { args: { input: NewOwner }; result: Owner };
  owner_update: { args: { id: number; patch: OwnerPatch }; result: Owner };
  owner_delete: { args: { id: number }; result: number[] };
  owner_restore: { args: { id: number }; result: Owner };
  patients_by_owner: { args: { ownerId: number }; result: Patient[] };
  patient_get: { args: { id: number }; result: Patient };
  patient_create: { args: { input: NewPatient }; result: Patient };
  patient_update: { args: { id: number; patch: PatientPatch }; result: Patient };
  patient_delete: { args: { id: number }; result: Patient };
  patient_restore: { args: { id: number }; result: Patient };
  search: { args: { query: string; limit: number }; result: SearchHit[] };
}

export interface ImportCandidate { path: string; fileName: string; sizeBytes: number; modifiedAt: string | null; location: string }
export interface ImportCounts { owners: number; patients: number; todos: number }
export type ImportIssueKind = 'phoneUnparsed' | 'dateUnparsed' | 'speciesUnrecognized' | 'sexUnrecognized' | 'missingName' | 'unusableId' | 'orphanPatient' | 'orphanTodo' | 'danglingPatientLink' | 'danglingOwnerLink' | 'ownerMismatch' | 'duplicateOwner';
export interface ImportIssue { table: string; sourceId: number; field: string; kind: ImportIssueKind; raw: string | null }
export interface ImportPreview {
  sourceName: string; sourceSha256: string;
  schema: { tables: { name: string; known: boolean; rowCount: number; columns: string[] }[]; unknownTables: string[]; missingTables: string[]; unknownColumns: string[]; missingColumns: string[] };
  found: ImportCounts; alreadyImported: ImportCounts; toImport: ImportCounts;
  phonesAdjusted: number; phonesUnparsed: number; datesUnparsed: Record<string, number>;
  speciesMappedToOther: number; speciesMissing: number; sexMappedToUnknown: number;
  orphanPatients: number; orphanTodos: number; duplicateOwners: number; todosDone: number; todosPlanned: number;
  orphanOwnerNeeded: boolean; issues: ImportIssue[];
}
// The core report flattens its preview into the report.
export interface ImportReport extends ImportPreview { inserted: ImportCounts; skipped: ImportCounts; runId: number; orphanOwnerCreated: boolean; backupFile: string }
export interface PreparedImport { token: string; preview: ImportPreview }
