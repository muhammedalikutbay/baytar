import { tr } from '$lib/i18n/tr';
import type { MedicationForm } from '$lib/api/types';

export const formLabels: Record<MedicationForm, string> = {
  tablet: tr.medicationTablet, capsule: tr.medicationCapsule, injection: tr.medicationInjection,
  suspension: tr.medicationSuspension, drops: tr.medicationDrops, ointment: tr.medicationOintment, other: tr.medicationOther
};
export const formOptions = [{ value: '', label: tr.medicationFormUnknown },
  ...Object.entries(formLabels).map(([value, label]) => ({ value, label }))];
