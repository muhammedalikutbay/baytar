import { call } from './client';
import type { ListQuery, NewOwner, OwnerPatch } from './types';
export const list = (query: ListQuery = {}) => call('owners_list', { query });
export const get = (id: number) => call('owner_get', { id });
export const detail = (id: number) => call('owner_detail', { id });
export const create = (input: NewOwner) => call('owner_create', { input });
export const update = (id: number, patch: OwnerPatch) => call('owner_update', { id, patch });
export const softDelete = (id: number) => call('owner_delete', { id });
export const restore = (id: number) => call('owner_restore', { id });
