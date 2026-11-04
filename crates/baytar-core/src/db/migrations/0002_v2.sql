-- Schema v2: wider species list, E.164 owner phones, vitals, catalog metadata, billing,
-- inventory, attachments and visit templates. Money is stored as INTEGER minor units (kurus).
--
-- The migration runner executes this file inside one transaction with foreign_keys ON, and
-- PRAGMA foreign_keys cannot be switched inside a transaction. Rebuilding a table that other
-- tables reference therefore relies on PRAGMA defer_foreign_keys. SQLite keeps a counter of
-- violated deferred constraints: dropping a parent that still has children raises it, and
-- inserting the matching parent rows lowers it again. Copying rows into the new table BEFORE
-- dropping the old one would leave the counter positive and fail the commit, so each rebuild
-- is: copy rows aside, drop the old table, create the new one, copy the rows back (ids are
-- kept), drop the copy. The runner's foreign_key_check then confirms nothing dangles.
PRAGMA defer_foreign_keys = ON;

-- ---------------------------------------------------------------------------------------
-- patients: species CHECK extension (table rebuild) and the new alerts column.
-- ---------------------------------------------------------------------------------------
CREATE TABLE patients_copy AS SELECT * FROM patients;
DROP TABLE patients;

CREATE TABLE patients (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    species TEXT NOT NULL CHECK (species IN ('cat','dog','bird','rabbit','rodent','ferret','reptile','other')),
    species_note TEXT,
    breed TEXT,
    sex TEXT NOT NULL DEFAULT 'unknown' CHECK (sex IN ('male','female','unknown')),
    neutered INTEGER CHECK (neutered IN (0,1)),
    birth_date TEXT,
    birth_date_is_estimate INTEGER NOT NULL DEFAULT 0 CHECK (birth_date_is_estimate IN (0,1)),
    color TEXT,
    microchip TEXT,
    notes TEXT,
    alerts TEXT,
    deceased_on TEXT,
    search_text TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

INSERT INTO patients (id, owner_id, name, species, species_note, breed, sex, neutered, birth_date,
                         birth_date_is_estimate, color, microchip, notes, deceased_on, search_text,
                         created_at, updated_at, deleted_at)
SELECT id, owner_id, name, species, species_note, breed, sex, neutered, birth_date,
       birth_date_is_estimate, color, microchip, notes, deceased_on, search_text,
       created_at, updated_at, deleted_at
FROM patients_copy;
DROP TABLE patients_copy;
CREATE INDEX patients_search_text_idx ON patients(search_text);
CREATE INDEX patients_owner_id_idx ON patients(owner_id);

-- ---------------------------------------------------------------------------------------
-- vaccine_types: species CHECK extension (table rebuild) and catalog metadata.
-- ---------------------------------------------------------------------------------------
CREATE TABLE vaccine_types_copy AS SELECT * FROM vaccine_types;
DROP TABLE vaccine_types;

CREATE TABLE vaccine_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    species TEXT CHECK (species IN ('cat','dog','bird','rabbit','rodent','ferret','reptile','other')),
    default_interval_days INTEGER,
    notes TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
    is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1)),
    manufacturer TEXT,
    diseases TEXT,
    primary_doses INTEGER CHECK (primary_doses IS NULL OR primary_doses >= 1),
    primary_interval_days INTEGER CHECK (primary_interval_days IS NULL OR primary_interval_days > 0),
    min_age_weeks INTEGER CHECK (min_age_weeks IS NULL OR min_age_weeks >= 0),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

INSERT INTO vaccine_types (id, name, species, default_interval_days, notes, is_active,
                              created_at, updated_at, deleted_at)
SELECT id, name, species, default_interval_days, notes, is_active,
       created_at, updated_at, deleted_at
FROM vaccine_types_copy;
DROP TABLE vaccine_types_copy;
CREATE UNIQUE INDEX vaccine_types_name_idx ON vaccine_types(lower(name)) WHERE deleted_at IS NULL;

-- ---------------------------------------------------------------------------------------
-- owners: E.164 phones and new contact fields.
-- phone_raw is never touched. Only two patterns are converted (GLOB matches the whole value):
--   0 + 10 digits            -> +90 + the 10 digits after the zero
--   10 digits starting with 5 -> +90 + the 10 digits
-- Everything else (already international, spaced, foreign, free text) stays as stored.
-- ---------------------------------------------------------------------------------------
ALTER TABLE owners ADD COLUMN national_id TEXT;
ALTER TABLE owners ADD COLUMN city TEXT;
ALTER TABLE owners ADD COLUMN district TEXT;

UPDATE owners
SET phone = CASE
        WHEN phone GLOB '0[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]' THEN '+90' || substr(phone, 2)
        WHEN phone GLOB '5[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]' THEN '+90' || phone
        ELSE phone
    END,
    phone_alt = CASE
        WHEN phone_alt GLOB '0[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]' THEN '+90' || substr(phone_alt, 2)
        WHEN phone_alt GLOB '5[0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]' THEN '+90' || phone_alt
        ELSE phone_alt
    END;

-- ---------------------------------------------------------------------------------------
-- visits: vitals.
-- ---------------------------------------------------------------------------------------
ALTER TABLE visits ADD COLUMN heart_rate_bpm INTEGER CHECK (heart_rate_bpm IS NULL OR heart_rate_bpm > 0);
ALTER TABLE visits ADD COLUMN resp_rate_bpm INTEGER CHECK (resp_rate_bpm IS NULL OR resp_rate_bpm > 0);
ALTER TABLE visits ADD COLUMN body_condition INTEGER CHECK (body_condition IS NULL OR body_condition BETWEEN 1 AND 9);

-- ---------------------------------------------------------------------------------------
-- medications: catalog metadata.
-- ---------------------------------------------------------------------------------------
ALTER TABLE medications ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1));
ALTER TABLE medications ADD COLUMN category TEXT CHECK (category IN (
    'antibiotic','nsaid','analgesic','antiparasitic','anesthetic','sedative','corticosteroid',
    'antiemetic','gastro','cardio','dermatologic','ophthalmic','otic','vitamin','fluid','hormone',
    'antifungal','antiviral','other'));
ALTER TABLE medications ADD COLUMN manufacturer TEXT;
ALTER TABLE medications ADD COLUMN target_species TEXT;
ALTER TABLE medications ADD COLUMN prescription_required INTEGER CHECK (prescription_required IS NULL OR prescription_required IN (0,1));
ALTER TABLE medications ADD COLUMN pack_unit TEXT;

-- ---------------------------------------------------------------------------------------
-- appointments: planned vaccinations link to the vaccine type and, once given, the vaccination.
-- ---------------------------------------------------------------------------------------
ALTER TABLE appointments ADD COLUMN vaccine_type_id INTEGER REFERENCES vaccine_types(id) ON DELETE RESTRICT;
ALTER TABLE appointments ADD COLUMN vaccination_id INTEGER REFERENCES vaccinations(id) ON DELETE RESTRICT;
CREATE INDEX appointments_vaccine_type_id_idx ON appointments(vaccine_type_id);
CREATE INDEX appointments_vaccination_id_idx ON appointments(vaccination_id);

-- ---------------------------------------------------------------------------------------
-- New tables.
-- ---------------------------------------------------------------------------------------
CREATE TABLE services (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'other' CHECK (category IN (
        'exam','vaccination','surgery','lab','imaging','grooming','boarding','hospitalisation','other')),
    price_minor INTEGER CHECK (price_minor IS NULL OR price_minor >= 0),
    vat_rate INTEGER CHECK (vat_rate IS NULL OR vat_rate BETWEEN 0 AND 100),
    is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1)),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE inventory_items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'other' CHECK (kind IN ('medication','vaccine','supply','food','product','other')),
    medication_id INTEGER REFERENCES medications(id) ON DELETE RESTRICT,
    vaccine_type_id INTEGER REFERENCES vaccine_types(id) ON DELETE RESTRICT,
    unit TEXT NOT NULL DEFAULT 'adet',
    sale_price_minor INTEGER CHECK (sale_price_minor IS NULL OR sale_price_minor >= 0),
    reorder_level REAL CHECK (reorder_level IS NULL OR reorder_level >= 0),
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE inventory_lots (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES inventory_items(id) ON DELETE RESTRICT,
    lot_no TEXT,
    expires_on TEXT,
    quantity REAL NOT NULL DEFAULT 0 CHECK (quantity >= 0),
    cost_minor INTEGER CHECK (cost_minor IS NULL OR cost_minor >= 0),
    received_on TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE stock_movements (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES inventory_items(id) ON DELETE RESTRICT,
    lot_id INTEGER REFERENCES inventory_lots(id) ON DELETE RESTRICT,
    delta REAL NOT NULL CHECK (delta <> 0),
    reason TEXT NOT NULL CHECK (reason IN ('purchase','use','sale','adjust','expired','return')),
    ref_table TEXT,
    ref_id INTEGER,
    moved_on TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE charges (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    patient_id INTEGER REFERENCES patients(id) ON DELETE RESTRICT,
    visit_id INTEGER REFERENCES visits(id) ON DELETE RESTRICT,
    service_id INTEGER REFERENCES services(id) ON DELETE RESTRICT,
    inventory_item_id INTEGER REFERENCES inventory_items(id) ON DELETE RESTRICT,
    description TEXT NOT NULL,
    quantity REAL NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit_price_minor INTEGER NOT NULL CHECK (unit_price_minor >= 0),
    discount_minor INTEGER NOT NULL DEFAULT 0 CHECK (discount_minor >= 0),
    vat_rate INTEGER CHECK (vat_rate IS NULL OR vat_rate BETWEEN 0 AND 100),
    total_minor INTEGER NOT NULL CHECK (total_minor >= 0),
    charged_on TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE payments (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    paid_on TEXT NOT NULL,
    amount_minor INTEGER NOT NULL CHECK (amount_minor > 0),
    method TEXT NOT NULL DEFAULT 'cash' CHECK (method IN ('cash','card','transfer','other')),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE attachments (
    id INTEGER PRIMARY KEY,
    patient_id INTEGER NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    visit_id INTEGER REFERENCES visits(id) ON DELETE RESTRICT,
    kind TEXT NOT NULL DEFAULT 'other' CHECK (kind IN ('photo','lab','imaging','document','other')),
    title TEXT,
    file_name TEXT NOT NULL,
    mime TEXT NOT NULL,
    size_bytes INTEGER CHECK (size_bytes IS NULL OR size_bytes >= 0),
    sha256 TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

-- File bytes live in their own table so listing attachments never loads blobs.
CREATE TABLE attachment_data (
    attachment_id INTEGER PRIMARY KEY REFERENCES attachments(id) ON DELETE RESTRICT,
    data BLOB NOT NULL
) STRICT;

CREATE TABLE visit_templates (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    reason TEXT,
    findings TEXT,
    diagnosis TEXT,
    treatment TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE INDEX inventory_items_medication_id_idx ON inventory_items(medication_id);
CREATE INDEX inventory_items_vaccine_type_id_idx ON inventory_items(vaccine_type_id);
CREATE INDEX inventory_lots_item_id_idx ON inventory_lots(item_id);
CREATE INDEX inventory_lots_expires_on_idx ON inventory_lots(expires_on) WHERE deleted_at IS NULL;
CREATE INDEX stock_movements_item_id_idx ON stock_movements(item_id);
CREATE INDEX stock_movements_lot_id_idx ON stock_movements(lot_id);
CREATE INDEX charges_owner_id_idx ON charges(owner_id);
CREATE INDEX charges_patient_id_idx ON charges(patient_id);
CREATE INDEX charges_visit_id_idx ON charges(visit_id);
CREATE INDEX charges_service_id_idx ON charges(service_id);
CREATE INDEX charges_inventory_item_id_idx ON charges(inventory_item_id);
CREATE INDEX charges_charged_on_idx ON charges(charged_on);
CREATE INDEX payments_owner_id_idx ON payments(owner_id);
CREATE INDEX payments_paid_on_idx ON payments(paid_on);
CREATE INDEX attachments_patient_id_idx ON attachments(patient_id);
CREATE INDEX attachments_visit_id_idx ON attachments(visit_id);
