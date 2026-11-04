import { call } from './client';
export const search = (query: string, limit = 50) => call('search', { query, limit });
