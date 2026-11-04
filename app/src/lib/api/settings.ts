import { isTauri } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { tr } from '$lib/i18n/tr';
export const getAll = () => call('settings_get_all', {});
export const set = (key: string, value: string) => call('settings_set', { key, value });
export const backups = () => call('backups_list', {});
export const createBackup = () => call('backup_create', {});
export const backupBeforeUpdate = () => call('backup_before_update', {});
export const restoreBackup = (path: string) => call('backup_restore', { path });
export const info = () => call('app_info', {});
export const openDataFolder = () => call('open_data_folder', {});
export async function exportBackup(): Promise<string | null> {
  if (!isTauri()) return '/demo/USB';
  const path = await open({ directory: true, multiple: false, title: tr.settingsExportFolder });
  if (typeof path !== 'string') return null;
  return call('backup_export', { path });
}
export const openWhatsapp = (url: string) => call('open_whatsapp', { url });

export const emptyClinic = {
  clinic_name: '', vet_name: '', clinic_phone: '', clinic_address: '',
  clinic_title: '', clinic_registry_no: '', clinic_email: '',
  clinic_tax_office: '', clinic_tax_no: '', clinic_logo: ''
};
export type ClinicProfile = typeof emptyClinic;
export function clinicProfile(values: Record<string, string>): ClinicProfile {
  return Object.fromEntries(Object.keys(emptyClinic).map(key => [key, values[key] ?? ''])) as ClinicProfile;
}
export const LOGO_MAX_BYTES = 512 * 1024;
export function safeClinicLogo(value: string): string {
  return value.length <= LOGO_MAX_BYTES && /^data:image\/(png|jpeg|webp);base64,[A-Za-z0-9+/=]+$/.test(value) ? value : '';
}

/** Re-encode raster uploads so the persisted data URL fits the settings budget. */
export async function prepareClinicLogo(file: File): Promise<string> {
  if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type) || file.size > 20 * 1024 * 1024) throw new Error('Invalid logo');
  const url = URL.createObjectURL(file);
  try {
    const image = new Image(); image.src = url; await image.decode();
    const canvas = document.createElement('canvas');
    let scale = Math.min(1, 640 / Math.max(image.naturalWidth, image.naturalHeight));
    while (scale > 0.03) {
      canvas.width = Math.max(1, Math.round(image.naturalWidth * scale));
      canvas.height = Math.max(1, Math.round(image.naturalHeight * scale));
      const context = canvas.getContext('2d');
      if (!context) throw new Error('Canvas unavailable');
      context.drawImage(image, 0, 0, canvas.width, canvas.height);
      const result = canvas.toDataURL('image/png');
      if (safeClinicLogo(result)) return result;
      scale *= 0.75;
    }
    throw new Error('Logo too large');
  } finally { URL.revokeObjectURL(url); }
}
