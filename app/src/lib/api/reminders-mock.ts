import { addDateDays, isValidDate } from '$lib/format';
import type { ApiError, CalendarAppointment, CommandMap, NewReminderContact, Owner, Patient, Receivable, ReminderContact, ReminderItem, RemindersOverview, RemindersQuery, VaccineDue, Visit } from './types';

interface Context {
  vaccines: { call(command: string, args: unknown): unknown };
  billing: { call(command: string, args: unknown): unknown };
  clinical: { call(command: string, args: unknown): unknown };
  patientGet(id: number): Patient;
  ownerGet(id: number): Owner;
  now(): string;
  fail(kind: ApiError['kind'], message: string, field?: string): never;
}

const KINDS = ['vaccine', 'appointment', 'debt', 'followup'];
const CHANNELS = ['whatsapp', 'phone', 'other'];

/** Browser-preview twin of the Rust reminders repo: derives the four lists from the other mocks. */
export function createRemindersMock(ctx: Context) {
  const contacts: ReminderContact[] = [];
  let nextId = 1;
  const ask = <K extends keyof CommandMap>(source: { call(command: string, args: unknown): unknown }, command: K, args: CommandMap[K]['args']) =>
    source.call(command, args) as CommandMap[K]['result'];
  const phoneOf = (owner: Owner) => owner.phone || owner.phoneRaw;
  const lastContact = (refKey: string) => contacts.filter(c => c.refKey === refKey).at(-1) ?? null;
  const base = (kind: ReminderItem['kind'], refKey: string, owner: Owner, date: string): ReminderItem => ({
    kind, refKey, ownerId: owner.id, ownerName: owner.fullName, phone: phoneOf(owner), patientId: null, patientName: null, species: null,
    reason: '', date, time: null, amountMinor: null, lastVisitOn: null, lastContact: lastContact(refKey)
  });
  const alive = (patient: Patient) => !patient.deletedAt && !patient.deceasedOn && !ctx.ownerGet(patient.ownerId).deletedAt;

  function overview(query: RemindersQuery): RemindersOverview {
    if (!isValidDate(query.today)) ctx.fail('invalid', 'Expected a valid YYYY-MM-DD date.', 'today');
    for (const [field, value] of [['vaccineDays', query.vaccineDays], ['debtDays', query.debtDays], ['followupDays', query.followupDays]] as const) {
      if (!Number.isInteger(value) || value < 1 || value > 365) ctx.fail('invalid', 'Must be between 1 and 365.', field);
    }
    const { today } = query, tomorrow = addDateDays(today, 1);
    const vaccines = ask(ctx.vaccines, 'vaccines_due_list', { query: { to: addDateDays(today, query.vaccineDays) } }).map((due: VaccineDue): ReminderItem => {
      const patient = ctx.patientGet(due.patientId), owner = ctx.ownerGet(patient.ownerId);
      return { ...base('vaccine', `vaccine:${due.patientId}:${due.vaccineTypeId ?? 'n-' + due.vaccineName.toLowerCase()}:${due.dueOn}`, owner, due.dueOn),
        patientId: patient.id, patientName: patient.name, species: patient.species, reason: due.vaccineName };
    });
    const withPatient = (row: CalendarAppointment, kind: 'appointment' | 'followup'): ReminderItem | null => {
      const a = row.appointment, patient = a.patientId ? ctx.patientGet(a.patientId) : null;
      if (patient && !alive(patient)) return null;
      const ownerId = a.ownerId ?? patient?.ownerId;
      if (!ownerId || ctx.ownerGet(ownerId).deletedAt) return null;
      return { ...base(kind, `${kind}:${a.id}`, ctx.ownerGet(ownerId), a.startsOn), patientId: patient?.id ?? null, patientName: patient?.name ?? null,
        species: patient?.species ?? null, reason: a.title, time: a.startsTime };
    };
    const planned = (from: string, to: string) => ask(ctx.clinical, 'appointments_between', { from, to }).filter(r => r.appointment.status === 'planned');
    const appointments = planned(today, tomorrow).filter(r => r.appointment.kind === 'visit' || r.appointment.kind === 'other')
      .map(r => withPatient(r, 'appointment')).filter((r): r is ReminderItem => r !== null);
    const followups = planned(addDateDays(today, 2), addDateDays(today, query.followupDays)).filter(r => r.appointment.kind === 'visit' && r.appointment.patientId)
      .flatMap(r => {
        const item = withPatient(r, 'followup');
        const last = ask(ctx.clinical, 'visits_by_patient', { patientId: r.appointment.patientId! }).filter((v: Visit) => !v.deletedAt && v.visitOn <= today).map(v => v.visitOn).sort().at(-1);
        return item && last ? [{ ...item, lastVisitOn: last }] : [];
      });
    const cutoff = addDateDays(today, -query.debtDays);
    const debts = ask(ctx.billing, 'accounts_receivables', {}).filter((r: Receivable) => r.oldestUnpaidOn <= cutoff).map((r: Receivable) => {
      const owner = ctx.ownerGet(r.ownerId);
      return { ...base('debt', `debt:${r.ownerId}:${r.oldestUnpaidOn}`, owner, r.oldestUnpaidOn), amountMinor: r.balanceMinor };
    }).sort((a: ReminderItem, b: ReminderItem) => a.date.localeCompare(b.date) || a.ownerId - b.ownerId);
    return { vaccines, appointments, debts, followups };
  }

  function record(input: NewReminderContact): ReminderContact {
    if (!KINDS.includes(input.kind)) ctx.fail('invalid', 'Unknown reminder kind.', 'kind');
    if (!CHANNELS.includes(input.channel)) ctx.fail('invalid', 'Unknown contact channel.', 'channel');
    const key = input.refKey.trim();
    if (!key.startsWith(input.kind + ':') || key.length <= input.kind.length + 1) ctx.fail('invalid', 'Must start with the reminder kind.', 'refKey');
    ctx.ownerGet(input.ownerId);
    const row: ReminderContact = { id: nextId++, kind: input.kind, refKey: key, ownerId: input.ownerId, channel: input.channel, contactedAt: ctx.now(), note: input.note?.trim() || null };
    contacts.push(row);
    return row;
  }

  const handlers: Record<string, (args: never) => unknown> = {
    reminders_overview: ({ query }: CommandMap['reminders_overview']['args']) => overview(query),
    reminders_record_contact: ({ input }: CommandMap['reminders_record_contact']['args']) => record(input)
  };
  return { has: (command: string) => command in handlers, call: (command: string, args: unknown) => handlers[command](args as never) };
}
