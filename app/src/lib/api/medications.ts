import { call } from './client';
import type { NewMedication, MedicationPatch } from './types';
export const get = (id: number) => call('medication_get', { id });
export const create = (input: NewMedication) => call('medication_create', { input });
export const update = (id: number, patch: MedicationPatch) => call('medication_update', { id, patch });
export const softDelete = (id: number) => call('medication_delete', { id });
export const restore = (id: number) => call('medication_restore', { id });
export const list = () => call('medications_list', {});
export const search = (query: string) => call('medications_search', { query });
