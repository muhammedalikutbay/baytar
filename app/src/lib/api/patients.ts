import { call } from './client';
import type { NewPatient, PatientPatch } from './types';
export const listByOwner = (ownerId: number) => call('patients_by_owner', { ownerId });
export const get = (id: number) => call('patient_get', { id });
export const create = (input: NewPatient) => call('patient_create', { input });
export const update = (id: number, patch: PatientPatch) => call('patient_update', { id, patch });
export const softDelete = (id: number) => call('patient_delete', { id });
export const restore = (id: number) => call('patient_restore', { id });
