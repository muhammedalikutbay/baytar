import { tr } from '$lib/i18n/tr';
import type { MedicationCategory, MedicationForm } from '$lib/api/types';

export const categoryLabels: Record<MedicationCategory, string> = {
  antibiotic: tr.medicationAntibiotic, nsaid: tr.medicationNsaid, analgesic: tr.medicationAnalgesic,
  antiparasitic: tr.medicationAntiparasitic, anesthetic: tr.medicationAnesthetic, sedative: tr.medicationSedative,
  corticosteroid: tr.medicationCorticosteroid, antiemetic: tr.medicationAntiemetic, gastro: tr.medicationGastro,
  cardio: tr.medicationCardio, dermatologic: tr.medicationDermatologic, ophthalmic: tr.medicationOphthalmic,
  otic: tr.medicationOtic, vitamin: tr.medicationVitamin, fluid: tr.medicationFluid, hormone: tr.medicationHormone,
  antifungal: tr.medicationAntifungal, antiviral: tr.medicationAntiviral, other: tr.other
};

export const formLabels: Record<MedicationForm, string> = {
  tablet: tr.medicationTablet, capsule: tr.medicationCapsule, injection: tr.medicationInjection,
  suspension: tr.medicationSuspension, drops: tr.medicationDrops, ointment: tr.medicationOintment, other: tr.medicationOther
};
export const formOptions = [{ value: '', label: tr.medicationFormUnknown },
  ...Object.entries(formLabels).map(([value, label]) => ({ value, label }))];
