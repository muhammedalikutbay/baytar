import { isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { tr } from '$lib/i18n/tr';
import type { ExportTable } from './types';
import { exportHeaders } from '$lib/i18n/export';

export const exportTables: ExportTable[] = ['owners', 'patients', 'visits', 'vaccinations', 'prescriptions', 'charges', 'payments', 'inventory'];
export const overview = (from: string, to: string, today: string) => call('reports_overview', { from, to, today });

/** Asks for a file with the native save dialog and writes one table. Resolves to the written path, or null when cancelled. */
export async function exportTable(table: ExportTable, today: string): Promise<string | null> {
  const name = `baytar-${table}-${today}.csv`;
  const headers = exportHeaders()[table];
  if (!isTauri()) return call('reports_export_csv', { table, path: name, headers });
  const path = await save({ title: tr.reportsSaveCsv, defaultPath: name, filters: [{ name: 'CSV', extensions: ['csv'] }] });
  return path ? call('reports_export_csv', { table, path, headers }) : null;
}

/** Asks for a folder and writes every table into it. Resolves to the written paths, or null when cancelled. */
export async function exportAll(today: string): Promise<string[] | null> {
  const headers = exportHeaders();
  if (!isTauri()) return call('reports_export_all', { dir: '', today, headers });
  const dir = await open({ directory: true, multiple: false, title: tr.reportsChooseFolder });
  return typeof dir === 'string' ? call('reports_export_all', { dir, today, headers }) : null;
}
