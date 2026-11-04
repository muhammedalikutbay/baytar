import { isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { tr } from '$lib/i18n/tr';
export const candidates = () => call('import_candidates', {});
export const prepare = (path: string) => call('import_prepare', { path });
export const execute = (token: string) => call('import_execute', { token });
export const cancel = (token: string) => call('import_cancel', { token });
export const saveReport = (tokenOrRunId: string, path: string) => call('import_save_report', { tokenOrRunId, path });
export async function pickFile(): Promise<string | null> {
  if (!isTauri()) return '/demo/HastaTakipDB.db';
  return open({ multiple: false, directory: false, title: tr.importPick, filters: [{ name: tr.importFileType, extensions: ['db'] }] });
}
export async function pickReportPath(): Promise<string | null> {
  if (!isTauri()) return 'baytar-aktarma-raporu.txt';
  return save({ title: tr.importSaveReport, defaultPath: 'baytar-aktarma-raporu.txt', filters: [{ name: tr.importReportType, extensions: ['txt'] }] });
}
