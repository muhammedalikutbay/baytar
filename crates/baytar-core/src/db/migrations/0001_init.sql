-- Schema v1. The app maintains updated_at and folded search_text; no triggers.
CREATE TABLE owners (
    id INTEGER PRIMARY KEY,
    full_name TEXT NOT NULL,
    phone TEXT,
    phone_raw TEXT,
    phone_alt TEXT,
    email TEXT,
    address TEXT,
    notes TEXT,
    search_text TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE patients (
    id INTEGER PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES owners(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    species TEXT NOT NULL CHECK (species IN ('cat','dog','bird','rabbit','other')),
    species_note TEXT,
    breed TEXT,
    sex TEXT NOT NULL DEFAULT 'unknown' CHECK (sex IN ('male','female','unknown')),
    neutered INTEGER CHECK (neutered IN (0,1)),
    birth_date TEXT,
    birth_date_is_estimate INTEGER NOT NULL DEFAULT 0 CHECK (birth_date_is_estimate IN (0,1)),
    color TEXT,
    microchip TEXT,
    notes TEXT,
    deceased_on TEXT,
    search_text TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE visits (
    id INTEGER PRIMARY KEY,
    patient_id INTEGER NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    visit_on TEXT NOT NULL,
    visit_time TEXT,
    reason TEXT,
    findings TEXT,
    diagnosis TEXT,
    treatment TEXT,
    weight_kg REAL CHECK (weight_kg IS NULL OR weight_kg > 0),
    temperature_c REAL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE vaccine_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    species TEXT CHECK (species IN ('cat','dog','bird','rabbit','other')),
    default_interval_days INTEGER,
    notes TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE vaccinations (
    id INTEGER PRIMARY KEY,
    patient_id INTEGER NOT NULL REFERENCES patients(id) ON DELETE RESTRICT,
    vaccine_type_id INTEGER REFERENCES vaccine_types(id) ON DELETE RESTRICT,
    vaccine_name TEXT NOT NULL,
    given_on TEXT NOT NULL,
    next_due_on TEXT,
    batch_no TEXT,
    visit_id INTEGER REFERENCES visits(id) ON DELETE RESTRICT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE medications (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    active_ingredient TEXT,
    form TEXT CHECK (form IN ('tablet','capsule','injection','suspension','drops','ointment','other')),
    concentration_mg_per_ml REAL,
    strength_mg_per_unit REAL,
    dose_mg_per_kg_min REAL,
    dose_mg_per_kg_max REAL,
    route TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT,
    CHECK (dose_mg_per_kg_min IS NULL OR dose_mg_per_kg_max IS NULL
           OR dose_mg_per_kg_max >= dose_mg_per_kg_min)
) STRICT;

CREATE TABLE prescription_items (
    id INTEGER PRIMARY KEY,
    visit_id INTEGER NOT NULL REFERENCES visits(id) ON DELETE RESTRICT,
    medication_id INTEGER REFERENCES medications(id) ON DELETE RESTRICT,
    medication_name TEXT NOT NULL,
    dose_mg REAL,
    dose_ml REAL,
    dose_text TEXT,
    frequency TEXT,
    duration_days INTEGER,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE appointments (
    id INTEGER PRIMARY KEY,
    patient_id INTEGER REFERENCES patients(id) ON DELETE RESTRICT,
    owner_id INTEGER REFERENCES owners(id) ON DELETE RESTRICT,
    title TEXT NOT NULL,
    starts_on TEXT NOT NULL,
    starts_time TEXT,
    duration_min INTEGER,
    kind TEXT NOT NULL DEFAULT 'other' CHECK (kind IN ('visit','vaccine','reminder','other')),
    status TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned','done','cancelled')),
    series_id TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    deleted_at TEXT
) STRICT;

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
) STRICT;

CREATE TABLE import_runs (
    id INTEGER PRIMARY KEY,
    source_name TEXT,
    source_sha256 TEXT NOT NULL,
    started_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    finished_at TEXT,
    status TEXT NOT NULL CHECK (status IN ('previewed','completed','failed')),
    report_json TEXT
) STRICT;

CREATE TABLE legacy_map (
    id INTEGER PRIMARY KEY,
    import_run_id INTEGER NOT NULL REFERENCES import_runs(id) ON DELETE RESTRICT,
    source_sha256 TEXT NOT NULL,
    source_table TEXT NOT NULL,
    source_id INTEGER NOT NULL,
    target_table TEXT NOT NULL,
    target_id INTEGER NOT NULL,
    UNIQUE (source_sha256, source_table, source_id)
) STRICT;

CREATE INDEX owners_search_text_idx ON owners(search_text);
CREATE INDEX patients_search_text_idx ON patients(search_text);
CREATE INDEX patients_owner_id_idx ON patients(owner_id);
CREATE INDEX visits_patient_id_visit_on_idx ON visits(patient_id, visit_on);
CREATE UNIQUE INDEX vaccine_types_name_idx ON vaccine_types(lower(name)) WHERE deleted_at IS NULL;
CREATE INDEX vaccinations_patient_id_idx ON vaccinations(patient_id);
CREATE INDEX vaccinations_next_due_on_idx ON vaccinations(next_due_on) WHERE deleted_at IS NULL;
CREATE INDEX appointments_starts_on_idx ON appointments(starts_on);
CREATE INDEX prescription_items_visit_id_idx ON prescription_items(visit_id);
