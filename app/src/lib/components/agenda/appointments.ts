import type { Appointment, CalendarAppointment } from '$lib/api/types';
import { get as getOwner } from '$lib/api/owners';
import { get as getPatient } from '$lib/api/patients';
import { localDate } from '$lib/format';
import { tr } from '$lib/i18n/tr';

export const kindLabels = { visit: tr.appointmentVisit, vaccine: tr.appointmentVaccine, reminder: tr.appointmentReminder, other: tr.other };
export const statusLabels = { planned: tr.appointmentPlanned, done: tr.appointmentDone, cancelled: tr.appointmentCancelled };
export type AgendaItem = CalendarAppointment & { phone: string | null };
export const compareAppointments = (a: AgendaItem, b: AgendaItem) => a.appointment.startsOn.localeCompare(b.appointment.startsOn)
  || (a.appointment.startsTime ?? '').localeCompare(b.appointment.startsTime ?? '') || a.appointment.id - b.appointment.id;
export function isOverdue(appointment: Appointment, now = new Date()): boolean {
  const today = localDate(now);
  return appointment.status === 'planned' && (appointment.startsOn < today || (appointment.startsOn === today
    && !!appointment.startsTime && appointment.startsTime < `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`));
}
export async function withPhones(rows: CalendarAppointment[]): Promise<AgendaItem[]> {
  const contacts = new Map<number, ReturnType<typeof getOwner>>();
  return Promise.all(rows.map(async row => {
    let phone: string | null = null;
    const ownerId = row.appointment.ownerId ?? (row.appointment.patientId === null ? null : (await getPatient(row.appointment.patientId)).ownerId);
    if (ownerId !== null) {
      if (!contacts.has(ownerId)) contacts.set(ownerId, getOwner(ownerId));
      const owner = await contacts.get(ownerId)!;
      phone = owner.phone ?? owner.phoneRaw;
    }
    return { ...row, phone };
  }));
}
