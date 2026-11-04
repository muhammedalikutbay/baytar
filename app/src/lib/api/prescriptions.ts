import { call } from './client';
import type { NewPrescription, PrescriptionPatch } from './types';
export const get = (id: number) => call('prescription_get', { id });
export const create = (input: NewPrescription) => call('prescription_create', { input });
export const update = (id: number, patch: PrescriptionPatch) => call('prescription_update', { id, patch });
export const softDelete = (id: number) => call('prescription_delete', { id });
export const restore = (id: number) => call('prescription_restore', { id });
export const listByVisit = (visitId: number) => call('prescriptions_by_visit', { visitId });
