import { call } from './client';
import type { NewVaccineType, VaccineTypePatch } from './types';
export const get = (id: number) => call('vaccine_type_get', { id });
export const create = (input: NewVaccineType) => call('vaccine_type_create', { input });
export const update = (id: number, patch: VaccineTypePatch) => call('vaccine_type_update', { id, patch });
export const softDelete = (id: number) => call('vaccine_type_delete', { id });
export const restore = (id: number) => call('vaccine_type_restore', { id });
export const list = () => call('vaccine_types_list', {});
