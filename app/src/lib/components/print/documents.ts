import { isTauri } from '@tauri-apps/api/core';
import * as settings from '$lib/api/settings';
import * as patients from '$lib/api/patients';
import * as owners from '$lib/api/owners';
import * as visits from '$lib/api/visits';
import * as vaccinations from '$lib/api/vaccinations';
import * as prescriptions from '$lib/api/prescriptions';
import * as appointments from '$lib/api/appointments';
import type { Owner, Patient, Visit, Vaccination, PrescriptionItem, Appointment } from '$lib/api/types';
import { localToday } from '$lib/format';
import { tr } from '$lib/i18n/tr';

export const documentTitles = {
  vaccination: tr.documentsVaccination, prescription: tr.prescription,
  summary: tr.documentsSummary, statement: tr.documentsStatement
};
export type DocumentKind = keyof typeof documentTitles;
export function isDocumentKind(value: string | null): value is DocumentKind {
  return value !== null && Object.hasOwn(documentTitles, value);
}
export interface StatementLine {
  id: string; date: string; description: string; debitMinor: number; creditMinor: number;
}
export interface StatementData { available: boolean; demo?: boolean; openingBalanceMinor: number; lines: StatementLine[] }
export interface PrintDocument {
  kind: DocumentKind; clinic: settings.ClinicProfile; owner: Owner; patient: Patient | null;
  visit: Visit | null; vaccinations: Vaccination[]; prescriptions: PrescriptionItem[];
  nextAppointment: Appointment | null; statement: StatementData; preparedOn: string;
}

/** TODO(billing): Wire the owner's complete charge/payment ledger here after billing merges.
 * Return all non-deleted rows in chronological order, with a stable same-day order.
 * Do not substitute an empty ledger on a command failure: an unknown balance is not zero.
 */
export async function loadStatement(ownerId: number): Promise<StatementData> {
  if (isTauri()) return { available: false, openingBalanceMinor: 0, lines: [] };
  return { available: true, demo: true, openingBalanceMinor: 0, lines: [
    { id: `charge-${ownerId}`, date: localToday(), description: tr.documentsDemoCharge, debitMinor: 125000, creditMinor: 0 },
    { id: `payment-${ownerId}`, date: localToday(), description: tr.documentsDemoPayment, debitMinor: 0, creditMinor: 75000 }
  ] };
}

function active<T extends { deletedAt: string | null }>(row: T): T {
  if (row.deletedAt) throw new Error('Record unavailable');
  return row;
}
export async function loadDocument(kind: DocumentKind, id: number): Promise<PrintDocument> {
  if (!Number.isSafeInteger(id) || id <= 0) throw new Error('Invalid document id');
  const clinic = settings.clinicProfile(await settings.getAll());
  const visit = kind === 'summary' || kind === 'prescription' ? active(await visits.get(id)) : null;
  const patient = kind === 'statement' ? null : active(await patients.get(visit?.patientId ?? id));
  const owner = active(await owners.get(patient?.ownerId ?? id));
  const result: PrintDocument = { kind, clinic, owner, patient, visit, vaccinations: [], prescriptions: [],
    nextAppointment: null, statement: { available: false, openingBalanceMinor: 0, lines: [] }, preparedOn: localToday() };
  if (kind === 'vaccination') result.vaccinations = (await vaccinations.listByPatient(id))
    .filter(row => !row.deletedAt).sort((a, b) => a.givenOn.localeCompare(b.givenOn) || a.id - b.id);
  if (visit) result.prescriptions = (await prescriptions.listByVisit(visit.id)).filter(row => !row.deletedAt);
  if (kind === 'summary' && patient) {
    const rows = await appointments.between(localToday(), '9999-12-31');
    result.nextAppointment = rows.map(row => row.appointment)
      .filter(row => !row.deletedAt && row.status === 'planned' && row.patientId === patient.id)
      .sort((a, b) => a.startsOn.localeCompare(b.startsOn) || (a.startsTime ?? '').localeCompare(b.startsTime ?? '') || a.id - b.id)[0] ?? null;
  }
  if (kind === 'statement') result.statement = await loadStatement(id);
  return result;
}
export const snapshotKey = (kind: DocumentKind, id: number) => `baytar-print-${kind}-${id}`;

export async function openDocument(kind: DocumentKind, id: number): Promise<void> {
  const url = `/print?doc=${kind}&id=${id}`;
  if (!isTauri()) {
    // Reserve the tab during the click; transfer the in-memory mock's current data.
    const tab = window.open('about:blank', '_blank');
    if (!tab) throw new Error(tr.documentsPopupError);
    try {
      tab.opener = null;
      const document = await loadDocument(kind, id);
      tab.sessionStorage.setItem(snapshotKey(kind, id), JSON.stringify(document));
      tab.location.replace(url);
    } catch (error) { tab.close(); throw error; }
    return;
  }
  const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
  await new Promise<void>((resolve, reject) => {
    const view = new WebviewWindow(`print-${Date.now()}-${Math.floor(Math.random() * 1000000)}`, {
      url, title: tr.documentsPrint, width: 900, height: 850, decorations: true
    });
    void view.once('tauri://created', () => resolve()).catch(reject);
    void view.once('tauri://error', () => reject(new Error(tr.documentsPopupError))).catch(reject);
  });
}
