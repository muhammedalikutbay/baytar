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
  settings_get_all: { args: Record<string, never>; result: Record<string, string> };
  settings_set: { args: { key: string; value: string }; result: null };
  backups_list: { args: Record<string, never>; result: BackupInfo[] };
  backup_create: { args: Record<string, never>; result: BackupInfo };
  backup_before_update: { args: Record<string, never>; result: BackupInfo };
  backup_export: { args: { path: string }; result: string };
  backup_restore: { args: { path: string }; result: null };
  app_info: { args: Record<string, never>; result: AppInfo };
  open_data_folder: { args: Record<string, never>; result: null };
  open_whatsapp: { args: { url: string }; result: null };
  dose_calculate: { args: { input: DoseInput }; result: DoseResult };
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

export interface BackupInfo { path: string; createdAt: string; kind: 'daily' | 'manual' | 'pre-migration' | 'pre-restore' | 'pre-import' | 'pre-update'; sizeBytes: number }
export interface AppInfo { version: string; dataFolder: string; backupsFolder: string; databaseSizeBytes: number }
export interface DoseInput {
  weightKg: number;
  mgPerKgMin: number;
  mgPerKgMax: number | null;
  concentrationMgPerMl: number | null;
  strengthMgPerUnit: number | null;
}
export interface DoseRange { min: number; max: number | null }
export type DoseWarning = 'verySmallVolume' | 'impracticalUnitCount';
export interface DoseResult { mg: DoseRange; ml: DoseRange | null; units: DoseRange | null; warnings: DoseWarning[] }

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
export type MedicationForm = 'tablet' | 'capsule' | 'injection' | 'suspension' | 'drops' | 'ointment' | 'other';
export type AppointmentKind = 'visit' | 'vaccine' | 'reminder' | 'other';
export type AppointmentStatus = 'planned' | 'done' | 'cancelled';
export interface Visit extends Audit {
  patientId: number;
  visitOn: string;
  visitTime: string | null;
  reason: string | null;
  findings: string | null;
  diagnosis: string | null;
  treatment: string | null;
  weightKg: number | null;
  temperatureC: number | null;
  notes: string | null;
}
export type NewVisit = Pick<Visit, 'patientId' | 'visitOn'> & Partial<Omit<Visit, keyof Audit | 'patientId' | 'visitOn'>>;
export type VisitPatch = Partial<NewVisit>;
export interface VaccineType extends Audit {
  name: string;
  species: Species | null;
  defaultIntervalDays: number | null;
  notes: string | null;
  isActive: boolean;
}
export type NewVaccineType = Pick<VaccineType, 'name'> & Partial<Omit<VaccineType, keyof Audit | 'name'>>;
export type VaccineTypePatch = Partial<NewVaccineType>;
export interface Vaccination extends Audit {
  patientId: number;
  vaccineTypeId: number | null;
  vaccineName: string;
  givenOn: string;
  nextDueOn: string | null;
  batchNo: string | null;
  visitId: number | null;
  notes: string | null;
}
export type NewVaccination = Pick<Vaccination, 'patientId' | 'vaccineName' | 'givenOn'> & Partial<Omit<Vaccination, keyof Audit | 'patientId' | 'vaccineName' | 'givenOn'>>;
export type VaccinationPatch = Partial<NewVaccination>;
export interface Medication extends Audit {
  name: string;
  activeIngredient: string | null;
  form: MedicationForm | null;
  concentrationMgPerMl: number | null;
  strengthMgPerUnit: number | null;
  doseMgPerKgMin: number | null;
  doseMgPerKgMax: number | null;
  route: string | null;
  notes: string | null;
}
export type NewMedication = Pick<Medication, 'name'> & Partial<Omit<Medication, keyof Audit | 'name'>>;
export type MedicationPatch = Partial<NewMedication>;
export interface PrescriptionItem extends Audit {
  visitId: number;
  medicationId: number | null;
  medicationName: string;
  doseMg: number | null;
  doseMl: number | null;
  doseText: string | null;
  frequency: string | null;
  durationDays: number | null;
  notes: string | null;
}
export type NewPrescription = Pick<PrescriptionItem, 'visitId'> & Partial<Omit<PrescriptionItem, keyof Audit | 'visitId'>>;
export type PrescriptionPatch = Partial<NewPrescription>;
export interface Appointment extends Audit {
  patientId: number | null;
  ownerId: number | null;
  title: string;
  startsOn: string;
  startsTime: string | null;
  durationMin: number | null;
  kind: AppointmentKind;
  status: AppointmentStatus;
  seriesId: string | null;
  notes: string | null;
}
export type NewAppointment = Pick<Appointment, 'title' | 'startsOn'> & Partial<Omit<Appointment, keyof Audit | 'title' | 'startsOn'>>;
export type AppointmentPatch = Partial<NewAppointment>;
export interface LatestWeight { weightKg: number; visitOn: string }
export interface DueVaccination { vaccination: Vaccination; patientName: string; ownerName: string }
export interface CalendarAppointment { appointment: Appointment; patientName: string | null; ownerName: string | null }
export interface CommandMap {
  visit_get: { args: { id: number }; result: Visit };
  visit_create: { args: { input: NewVisit }; result: Visit };
  visit_update: { args: { id: number; patch: VisitPatch }; result: Visit };
  visit_delete: { args: { id: number }; result: Visit };
  visit_restore: { args: { id: number }; result: Visit };
  visits_by_patient: { args: { patientId: number }; result: Visit[] };
  visit_latest_weight: { args: { patientId: number }; result: LatestWeight | null };
  vaccine_type_get: { args: { id: number }; result: VaccineType };
  vaccine_type_create: { args: { input: NewVaccineType }; result: VaccineType };
  vaccine_type_update: { args: { id: number; patch: VaccineTypePatch }; result: VaccineType };
  vaccine_type_delete: { args: { id: number }; result: VaccineType };
  vaccine_type_restore: { args: { id: number }; result: VaccineType };
  vaccine_types_list: { args: Record<string, never>; result: VaccineType[] };
  vaccination_get: { args: { id: number }; result: Vaccination };
  vaccination_create: { args: { input: NewVaccination }; result: Vaccination };
  vaccination_update: { args: { id: number; patch: VaccinationPatch }; result: Vaccination };
  vaccination_delete: { args: { id: number }; result: Vaccination };
  vaccination_restore: { args: { id: number }; result: Vaccination };
  vaccinations_by_patient: { args: { patientId: number }; result: Vaccination[] };
  vaccination_suggest_next_due: { args: { typeId: number; givenOn: string }; result: string | null };
  vaccinations_due_between: { args: { from: string; to: string }; result: DueVaccination[] };
  vaccinations_overdue: { args: { today: string }; result: DueVaccination[] };
  medication_get: { args: { id: number }; result: Medication };
  medication_create: { args: { input: NewMedication }; result: Medication };
  medication_update: { args: { id: number; patch: MedicationPatch }; result: Medication };
  medication_delete: { args: { id: number }; result: Medication };
  medication_restore: { args: { id: number }; result: Medication };
  medications_list: { args: Record<string, never>; result: Medication[] };
  medications_search: { args: { query: string }; result: Medication[] };
  prescription_get: { args: { id: number }; result: PrescriptionItem };
  prescription_create: { args: { input: NewPrescription }; result: PrescriptionItem };
  prescription_update: { args: { id: number; patch: PrescriptionPatch }; result: PrescriptionItem };
  prescription_delete: { args: { id: number }; result: PrescriptionItem };
  prescription_restore: { args: { id: number }; result: PrescriptionItem };
  prescriptions_by_visit: { args: { visitId: number }; result: PrescriptionItem[] };
  appointment_get: { args: { id: number }; result: Appointment };
  appointment_create: { args: { input: NewAppointment }; result: Appointment };
  appointment_update: { args: { id: number; patch: AppointmentPatch }; result: Appointment };
  appointment_delete: { args: { id: number }; result: Appointment };
  appointment_restore: { args: { id: number }; result: Appointment };
  appointment_set_status: { args: { id: number; status: AppointmentStatus }; result: Appointment };
  appointment_create_series: { args: { base: NewAppointment; count: number; everyDays: number }; result: Appointment[] };
  appointments_between: { args: { from: string; to: string }; result: CalendarAppointment[] };
  appointments_today: { args: { today: string }; result: CalendarAppointment[] };
}
