import { isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { tr } from '$lib/i18n/tr';
export const getAll = () => call('settings_get_all', {});
export const set = (key: string, value: string) => call('settings_set', { key, value });
export const backups = () => call('backups_list', {});
export const createBackup = () => call('backup_create', {});
export const restoreBackup = (path: string) => call('backup_restore', { path });
export const info = () => call('app_info', {});
export const openDataFolder = () => call('open_data_folder', {});
export async function exportBackup(): Promise<string | null> {
  if (!isTauri()) return '/demo/USB';
  const path = await open({ directory: true, multiple: false, title: tr.settingsExportFolder });
  if (typeof path !== 'string') return null;
  return call('backup_export', { path });
}
