import { tr } from '$lib/i18n/tr';
import { formatDate, formatMoney, formatPhone } from '$lib/format';
import * as settings from '$lib/api/settings';
import type { ReminderItem, ReminderKind } from '$lib/api/types';
import { PLACEHOLDERS, renderTemplate, type Placeholder, type TemplateValues } from './template';

export const REMINDER_KINDS: readonly ReminderKind[] = ['vaccine', 'appointment', 'debt', 'followup'];

/** Settings keys; they must stay in sync with the allowlist in `app/src-tauri/src/settings.rs`. */
export const TEMPLATE_KEYS = {
  vaccine: 'reminder_template_vaccine',
  appointment: 'reminder_template_appointment',
  debt: 'reminder_template_debt',
  followup: 'reminder_template_followup'
} as const satisfies Record<ReminderKind, string>;

export const DEFAULT_TEMPLATES: Record<ReminderKind, string> = {
  vaccine: tr.remindersDefaultVaccine,
  appointment: tr.remindersDefaultAppointment,
  debt: tr.remindersDefaultDebt,
  followup: tr.remindersDefaultFollowup
};

export const TEMPLATE_LABELS: Record<ReminderKind, string> = {
  vaccine: tr.remindersTemplateVaccine,
  appointment: tr.remindersTemplateAppointment,
  debt: tr.remindersTemplateDebt,
  followup: tr.remindersTemplateFollowup
};

export const PLACEHOLDER_LABELS: Record<Placeholder, string> = {
  sahip: tr.remindersPhSahip, hasta: tr.remindersPhHasta, asi: tr.remindersPhAsi, tarih: tr.remindersPhTarih,
  saat: tr.remindersPhSaat, tutar: tr.remindersPhTutar, klinik: tr.remindersPhKlinik, telefon: tr.remindersPhTelefon
};
export { PLACEHOLDERS };

export type TemplateSet = Record<ReminderKind, string>;
export interface ClinicContact { name: string; phone: string }
export interface TemplateContext { templates: TemplateSet; clinic: ClinicContact }

/** A blank or missing stored template falls back to the shipped default. */
export function templatesFrom(values: Record<string, string>): TemplateSet {
  const result = { ...DEFAULT_TEMPLATES };
  for (const kind of REMINDER_KINDS) {
    const stored = values[TEMPLATE_KEYS[kind]];
    if (stored && stored.trim()) result[kind] = stored;
  }
  return result;
}

export async function loadTemplateContext(): Promise<TemplateContext> {
  const values = await settings.getAll();
  return { templates: templatesFrom(values), clinic: { name: values.clinic_name ?? '', phone: values.clinic_phone ?? '' } };
}

export function itemValues(item: ReminderItem, clinic: ClinicContact): TemplateValues {
  return {
    sahip: item.ownerName,
    hasta: item.patientName ?? '',
    asi: item.kind === 'vaccine' ? item.reason : '',
    tarih: formatDate(item.date),
    saat: item.time ?? tr.remindersAllDayTime,
    tutar: item.amountMinor == null ? '' : formatMoney(item.amountMinor),
    klinik: clinic.name,
    telefon: formatPhone(clinic.phone)
  };
}

export function renderItem(item: ReminderItem, context: TemplateContext): string {
  return renderTemplate(context.templates[item.kind], itemValues(item, context.clinic));
}

/** Example values for the editor preview. */
export function sampleValues(clinic: ClinicContact): TemplateValues {
  return {
    sahip: tr.remindersSampleOwner, hasta: tr.remindersSamplePet, asi: tr.remindersSampleVaccine,
    tarih: formatDate('2026-11-20'), saat: '10:30', tutar: formatMoney(125000), klinik: clinic.name, telefon: formatPhone(clinic.phone)
  };
}
