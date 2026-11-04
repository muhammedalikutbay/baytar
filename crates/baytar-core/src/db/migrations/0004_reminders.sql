-- Schema v4: reminders contact log.
-- Append-only record of "I contacted this owner about this reminder". The reminders centre
-- derives its rows from vaccinations, appointments and charges; this table only remembers
-- who was contacted, when and how, so a reminder is not repeated the same day.
CREATE TABLE reminder_contacts (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('vaccine','appointment','debt','followup')),
    -- Stable key of the reminder, e.g. vaccine:<patientId>:<vaccineTypeId>:<dueOn>,
    -- appointment:<id>, debt:<ownerId>:<YYYY-MM-DD of oldest unpaid>, followup:<id>.
    ref_key TEXT NOT NULL CHECK (length(trim(ref_key)) > 0),
    owner_id INTEGER NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    channel TEXT NOT NULL CHECK (channel IN ('whatsapp','phone','other')),
    contacted_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    note TEXT
) STRICT;

CREATE INDEX reminder_contacts_ref_key_idx ON reminder_contacts(ref_key, contacted_at);
CREATE INDEX reminder_contacts_owner_id_idx ON reminder_contacts(owner_id);
