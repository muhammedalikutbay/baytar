import { isTauri } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { call } from './client';
import { stageBrowserFile } from './attachments-mock';
import { tr } from '$lib/i18n/tr';
import type { Attachment, AttachmentKind } from './types';

export const extensions = ['jpg', 'jpeg', 'png', 'gif', 'webp', 'bmp', 'tif', 'tiff', 'pdf', 'txt', 'csv', 'rtf', 'doc', 'docx', 'xls', 'xlsx', 'odt', 'ods'];
export const list = (patientId: number, visitId: number | null = null, includeDeleted = false) => call('attachments_list', { patientId, visitId, includeDeleted });
export const addFromPath = (patientId: number, visitId: number | null, kind: AttachmentKind, path: string) => call('attachment_add_from_path', { patientId, visitId, kind, path });
export const rename = (id: number, title: string | null) => call('attachment_rename', { id, title });
export const softDelete = (id: number) => call('attachment_delete', { id });
export const restore = (id: number) => call('attachment_restore', { id });
export const thumbnail = (id: number) => call('attachment_thumbnail', { id });
export const photo = (patientId: number) => call('patient_photo', { patientId });
export const readBytes = (id: number) => call('attachment_read_bytes', { id });
export const openExternal = (id: number) => call('attachment_open', { id });
export async function exportFile(file: Attachment) {
  if (!isTauri()) return call('attachment_export', { id: file.id, path: file.fileName });
  const path = await save({ title: tr.clinicalSaveFile, defaultPath: file.fileName });
  if (path) await call('attachment_export', { id: file.id, path });
}
export async function pickPaths(): Promise<string[]> {
  if (isTauri()) {
    const paths = await open({ title: tr.clinicalAddFiles, multiple: true, filters: [{ name: tr.clinicalFiles, extensions }] });
    return paths === null ? [] : typeof paths === 'string' ? [paths] : paths;
  }
  return new Promise(resolve => {
    const input = document.createElement('input'); input.type = 'file'; input.multiple = true;
    input.accept = extensions.map(e => '.' + e).join(',');
    input.onchange = async () => resolve(await Promise.all(Array.from(input.files ?? []).map(stageBrowserFile)));
    input.oncancel = () => resolve([]); input.click();
  });
}
