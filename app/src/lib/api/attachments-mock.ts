import type { ApiError, Attachment, AttachmentKind, Patient, Visit } from './types';
import { tr } from '$lib/i18n/tr';
interface Context { now(): string; patientGet(id: number): Patient; visitGet(id: number): Visit; fail(kind: ApiError['kind'], message: string, field?: string): never }
const staged = new Map<string, File>();
export async function stageBrowserFile(file: File) { const key = crypto.randomUUID(); staged.set(key, file); return key; }
export function createAttachmentsMock(ctx: Context) {
  let next = 6000;
  const rows: Attachment[] = [], bytes = new Map<number, Uint8Array>();
  const get = (id: number) => rows.find(r => r.id === id) ?? ctx.fail('not_found', 'Record not found.');
  function active(row: Attachment) { if (row.deletedAt || ctx.patientGet(row.patientId).deletedAt) ctx.fail('not_found', 'Record not found.'); return row; }
  const read = (id: number) => { active(get(id)); return bytes.get(id) ?? new Uint8Array(); };
  function url(id: number) { return URL.createObjectURL(new Blob([new Uint8Array(read(id)).buffer], { type: get(id).mime })); }
  async function thumbnail(id: number): Promise<string | null> {
    if (!active(get(id)).mime.startsWith('image/')) return null;
    const source = url(id);
    try {
      const image = new Image(); image.src = source; await image.decode(); const scale = Math.min(1, 320 / image.width, 320 / image.height);
      const canvas = document.createElement('canvas'); canvas.width = Math.max(1, Math.round(image.width * scale)); canvas.height = Math.max(1, Math.round(image.height * scale));
      canvas.getContext('2d')?.drawImage(image, 0, 0, canvas.width, canvas.height); return canvas.toDataURL('image/png');
    } finally { URL.revokeObjectURL(source); }
  }
  const handlers: Record<string, (args: unknown) => unknown> = {
    attachments_list: args => { const { patientId, visitId, includeDeleted } = args as { patientId: number; visitId: number | null; includeDeleted: boolean }; ctx.patientGet(patientId); return rows.filter(r => r.patientId === patientId && (visitId === null || r.visitId === visitId) && (includeDeleted || !r.deletedAt)).sort((a,b) => b.createdAt.localeCompare(a.createdAt) || b.id - a.id); },
    attachment_add_from_path: async args => {
      const { patientId, visitId, kind, path } = args as { patientId: number; visitId: number | null; kind: AttachmentKind; path: string };
      ctx.patientGet(patientId); if (visitId !== null && ctx.visitGet(visitId).patientId !== patientId) ctx.fail('invalid', 'Visit must belong to patient.', 'visitId');
      const file = staged.get(path) ?? ctx.fail('not_found', 'File not found.');
      if (!file.size || file.size > 25 * 1024 * 1024) ctx.fail('invalid', 'File must contain between 1 byte and 25 MB.', 'sizeBytes');
      if (kind === 'photo' && !file.type.startsWith('image/')) ctx.fail('invalid', 'Photo must be an image.', 'kind');
      const data = new Uint8Array(await file.arrayBuffer()); const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', data)); const stamp = ctx.now();
      const row: Attachment = { id: next++, patientId, visitId, kind, title: null, fileName: file.name, mime: file.type || 'application/octet-stream', sizeBytes: file.size, sha256: Array.from(digest).map(b => b.toString(16).padStart(2, '0')).join(''), createdAt: stamp, updatedAt: stamp, deletedAt: null };
      rows.push(row); bytes.set(row.id, data); staged.delete(path); return row;
    },
    attachment_rename: args => { const { id, title } = args as { id: number; title: string | null }; const row = active(get(id)); row.title = title?.trim() || null; row.updatedAt = ctx.now(); return row; },
    attachment_read_bytes: args => Array.from(read((args as { id: number }).id)),
    attachment_thumbnail: args => thumbnail((args as { id: number }).id),
    patient_photo: args => { const row = rows.filter(r => r.patientId === (args as { patientId: number }).patientId && !r.deletedAt && r.kind === 'photo').sort((a,b) => b.createdAt.localeCompare(a.createdAt) || b.id-a.id)[0]; return row ? thumbnail(row.id) : null; },
    attachment_open: args => { const source = url((args as { id: number }).id); window.open(source, '_blank', 'noopener,noreferrer'); setTimeout(() => URL.revokeObjectURL(source), 60000); return null; },
    attachment_export: args => { const id = (args as { id: number }).id; const source = url(id); const link = document.createElement('a'); link.href = source; link.download = get(id).fileName; link.click(); setTimeout(() => URL.revokeObjectURL(source), 1000); return null; }
  };
  for (const [command, deleted] of [['attachment_delete', true], ['attachment_restore', false]] as const) handlers[command] = args => {
    const row = get((args as { id: number }).id); if (Boolean(row.deletedAt) === deleted) ctx.fail('conflict', 'Record conflicts with current state.'); row.updatedAt = ctx.now(); row.deletedAt = deleted ? row.updatedAt : null; return row;
  };
  // Synthetic files make the grid reviewable; these are not installed in a real clinic database.
  const stamp = ctx.now();
  for (const [fileName, title, kind] of [['blood-count.txt', tr.clinicalDemoLab, 'lab'], ['care-plan.txt', tr.clinicalDemoDocument, 'document']] as const) {
    const data = new TextEncoder().encode('Synthetic clinical attachment.'); const id = next++;
    rows.push({ id, patientId: 4, visitId: null, kind, title, fileName, mime: 'text/plain', sizeBytes: data.length, sha256: '', createdAt: stamp, updatedAt: stamp, deletedAt: null }); bytes.set(id, data);
  }
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args) };
}
