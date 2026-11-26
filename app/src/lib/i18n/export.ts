import { tr, importIssueKeys } from './tr';
import type { ExportTable } from '$lib/api/types';

/** Column order follows baytar-core::export; Rust validates the number of labels. */
export function exportHeaders(): Record<ExportTable, string[]> {
  return {
    owners: [tr.csvRecordId, tr.fullName, tr.phone, tr.phoneAlt, tr.email, tr.address, tr.city, tr.district, tr.notes, tr.csvCreatedOn],
    patients: [tr.csvRecordId, tr.owner, tr.patientName, tr.species, tr.breed, tr.sex, tr.neutered, tr.birthDate, tr.color, tr.microchip, tr.clinicalDeceasedDate, tr.notes],
    visits: [tr.csvRecordId, tr.date, tr.time, tr.appointmentPatient, tr.owner, tr.reason, tr.findings, tr.diagnosis, tr.treatment, tr.weight, tr.temperature, tr.notes],
    vaccinations: [tr.csvRecordId, tr.givenOn, tr.nextDueOn, tr.vaccineName, tr.batchNo, tr.appointmentPatient, tr.owner, tr.notes],
    prescriptions: [tr.csvRecordId, tr.csvVisitOn, tr.appointmentPatient, tr.owner, tr.medicationName, tr.doseMg, tr.doseMl, tr.doseText, tr.frequency, tr.durationDays, tr.notes],
    charges: [tr.csvRecordId, tr.date, tr.owner, tr.appointmentPatient, tr.billingDescription, tr.billingQuantity, tr.billingUnitPrice, tr.billingDiscount, tr.billingVat, tr.billingTotal, tr.notes],
    payments: [tr.csvRecordId, tr.date, tr.owner, tr.billingAmount, tr.billingMethod, tr.notes],
    inventory: [tr.csvRecordId, tr.inventoryName, tr.inventoryKind, tr.inventoryUnit, tr.inventoryStock, tr.inventoryPrice, tr.inventoryReorder, tr.inventoryActive, tr.notes]
  };
}

export function importReportLabels() {
  return {
    heading: tr.importReportHeading, source: tr.importReportSource, safe: tr.importOldSafe,
    counts: { found: tr.importReportFound, toImport: tr.importReportToImport, inserted: tr.importReportInserted, alreadyImported: tr.importReportAlready },
    countSummary: tr.importReportCounts, backup: tr.importReportBackup, issues: tr.importReportIssues,
    issueLabels: Object.fromEntries(Object.entries(importIssueKeys).map(([kind, key]) => [kind, tr[key]])),
    unknownIssue: tr.importReportUnknownIssue
  };
}
export type ImportReportLabels = ReturnType<typeof importReportLabels>;
