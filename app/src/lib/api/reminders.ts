import { call } from './client';
import type { NewReminderContact, RemindersQuery } from './types';
export const overview = (query: RemindersQuery) => call('reminders_overview', { query });
export const recordContact = (input: NewReminderContact) => call('reminders_record_contact', { input });
