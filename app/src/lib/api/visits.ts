import { call } from './client';
import type { NewVisit, VisitPatch } from './types';
export const get = (id: number) => call('visit_get', { id });
export const create = (input: NewVisit) => call('visit_create', { input });
export const update = (id: number, patch: VisitPatch) => call('visit_update', { id, patch });
export const softDelete = (id: number) => call('visit_delete', { id });
export const restore = (id: number) => call('visit_restore', { id });
export const listByPatient = (patientId: number) => call('visits_by_patient', { patientId });
export const latestWeight = (patientId: number) => call('visit_latest_weight', { patientId });
