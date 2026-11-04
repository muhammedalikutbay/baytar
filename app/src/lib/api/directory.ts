import { call } from './client';
// Total is the active owner count; every returned owner includes all active patients.
export const list = (query = '', limit = 200) => call('directory_list', { query, limit });
