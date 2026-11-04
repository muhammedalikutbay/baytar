import { isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { tr } from '$lib/i18n/tr';
import type { ExportTable } from './types';

export const exportTables: ExportTable[] = ['owners', 'patients', 'visits', 'vaccinations', 'prescriptions', 'charges', 'payments', 'inventory'];
export const overview = (from: string, to: string, today: string) => call('reports_overview', { from, to, today });

/** Asks for a file with the native save dialog and writes one table. Resolves to the written path, or null when cancelled. */
export async function exportTable(table: ExportTable, today: string): Promise<string | null> {
  const name = `baytar-${table}-${today}.csv`;
  if (!isTauri()) return call('reports_export_csv', { table, path: name });
  const path = await save({ title: tr.reportsSaveCsv, defaultPath: name, filters: [{ name: 'CSV', extensions: ['csv'] }] });
  return path ? call('reports_export_csv', { table, path }) : null;
}

/** Asks for a folder and writes every table into it. Resolves to the written paths, or null when cancelled. */
export async function exportAll(today: string): Promise<string[] | null> {
  if (!isTauri()) return call('reports_export_all', { dir: '', today });
  const dir = await open({ directory: true, multiple: false, title: tr.reportsChooseFolder });
  return typeof dir === 'string' ? call('reports_export_all', { dir, today }) : null;
}
