# Data model: schema v1

The canonical schema is `crates/baytar-core/src/db/migrations/0001_init.sql`.
Migration 1 replaces the unreleased placeholder in place; `PRAGMA user_version`
remains 1. The existing migration runner applies migrations in a transaction,
backs up an existing versioned database before upgrading it, and checks database
and foreign-key integrity before committing. A fresh database needs no backup.
Legacy databases must be copied and read separately, never migrated in place.

## Storage and application conventions

- Every table is SQLite `STRICT`. IDs use `id INTEGER PRIMARY KEY`, represented
  by Rust `i64`. `settings` is the exception: its natural key is `key TEXT PRIMARY KEY`.
- All foreign keys use `ON DELETE RESTRICT`, including optional relationships.
  Connections enable `PRAGMA foreign_keys = ON`. Soft deletion does not remove
  the parent or release its foreign-key restrictions.
- The eight user-facing tables (`owners`, `patients`, `visits`, `vaccine_types`,
  `vaccinations`, `medications`, `prescription_items`, `appointments`) each have
  the shared audit columns below. UI deletion sets `deleted_at`; it never
  hard-deletes. Normal lists exclude deleted rows, while historical references
  can still resolve them. Settings and import provenance have their own lifecycle.
- Timestamps are ISO 8601 UTC text, for example `2026-11-04T09:30:00.000Z`.
  Defaults are exactly `(strftime('%Y-%m-%dT%H:%M:%fZ','now'))`. There are no
  triggers: the app sets `updated_at` on edits, including soft deletion.
- Calendar dates are `YYYY-MM-DD` text. Optional clock times are `HH:MM` text
  in the clinic's local calendar; they are not UTC timestamps. A NULL appointment
  time means all day. The app validates calendar/time syntax before writing;
  `STRICT` enforces storage types, not valid dates or time zones.
- NULL means absent or unknown; it is distinct from an empty string. SQLite
  boolean flags use INTEGER 0/1 with CHECK constraints. Rust models use `bool`
  or `Option<bool>`. Other nullable columns use `Option<T>`. REAL maps to `f64`.
- Models in `baytar_core::model` derive serde Serialize/Deserialize with camelCase
  fields (for example `ownerId`, `birthDateIsEstimate`, `sourceSha256`). Dates,
  times, timestamps, and stored JSON are strings in these plain row models.
  Enum strings match the schema's lowercase spellings; `as_str` and `FromStr`
  convert canonical values without guessing aliases. Legacy aliases are handled
  separately by `legacy::normalize`, which re-exports the shared Species/Sex types.
- `owners.search_text` and `patients.search_text` are app-maintained search keys.
  Apply `baytar_core::text::fold` to indexed content and query text: lowercase
  with Turkish `İ` -> `i`, `I` -> `ı`, then fold `ç` -> `c`, `ğ` -> `g`,
  `ı` -> `i`, `ö` -> `o`, `ş` -> `s`, `ü` -> `u`. Trim and collapse Unicode
  whitespace to one ASCII space; keep digits and punctuation. Thus `İSTANBUL`
  becomes `istanbul`, `IŞIK` becomes `isik`, and `Çağrı Öztürk` becomes
  `cagri ozturk`. Display text retains its original spelling. Store relevant
  name/contact terms for owners and name/species/breed/identifier terms for patients.
- SQLite CHECK constraints validate enums, boolean flags, positive known weight,
  and ordered dose bounds. Application validation also checks dates, clinical
  inputs, and linked-record consistency: a vaccination's visit must belong to
  its patient, and an appointment's explicit owner should match its patient.
  Foreign keys alone do not enforce those multi-column relationships.

Shared columns on each of the eight user-facing tables:

| Column | Storage / default | Meaning |
| --- | --- | --- |
| `id` | INTEGER PRIMARY KEY | Stable local row identity; not a legacy ID. |
| `created_at` | TEXT NOT NULL, UTC now | Time the row was created in Baytar. |
| `updated_at` | TEXT NOT NULL, UTC now | Last edit time, explicitly set by the app after insertion. |
| `deleted_at` | TEXT nullable | Soft-deletion timestamp; NULL means not deleted. |

All table references below include those shared columns where stated. All other
columns are nullable unless marked required. An omitted nullable value defaults
to NULL. No automatic update or search-folding triggers exist.

## Tables and columns

### owners

An owner or clinic contact. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `full_name` | TEXT required | Display name, retained in its original spelling. |
| `phone` | TEXT | Normalized primary phone, including leading zero where recoverable. |
| `phone_raw` | TEXT | Original legacy/input phone representation before normalization. |
| `phone_alt` | TEXT | Additional contact number. |
| `email` | TEXT | Email contact. |
| `address` | TEXT | Postal or free-form address. |
| `notes` | TEXT | Owner notes and any preserved legacy values without a dedicated field. |
| `search_text` | TEXT required, default `''` | Folded search key, maintained by the app. |

### patients

An animal belonging to one owner. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `owner_id` | INTEGER required, FK owners.id | Current owner, including a generated owner for legacy orphans. |
| `name` | TEXT required | Patient's display name. |
| `species` | TEXT required, CHECK enum | `cat`, `dog`, `bird`, `rabbit`, or `other`. |
| `species_note` | TEXT | Original free-text species or explanation of `other`. |
| `breed` | TEXT | Breed description. |
| `sex` | TEXT required, default `unknown`, CHECK enum | `male`, `female`, or `unknown`. |
| `neutered` | INTEGER 0/1/NULL | No, yes, or unknown neuter status. |
| `birth_date` | TEXT date | Known or estimated date of birth. |
| `birth_date_is_estimate` | INTEGER 0/1 required, default 0 | Whether the recorded birth date is an estimate. |
| `color` | TEXT | Coat, plumage, or other color description. |
| `microchip` | TEXT | Identifier kept as text to retain leading zeroes. |
| `notes` | TEXT | Patient notes and preserved unmapped legacy values. |
| `deceased_on` | TEXT date | Known death date; independent of soft deletion. |
| `search_text` | TEXT required, default `''` | Folded search key, maintained by the app. |

### visits

A clinical encounter for one patient. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `patient_id` | INTEGER required, FK patients.id | Patient examined. |
| `visit_on` | TEXT date required | Encounter date. |
| `visit_time` | TEXT time | Encounter clock time, if known. |
| `reason` | TEXT | Presenting complaint or purpose. |
| `findings` | TEXT | Examination findings. |
| `diagnosis` | TEXT | Clinical assessment. |
| `treatment` | TEXT | Procedures or treatment description. |
| `weight_kg` | REAL, CHECK NULL or > 0 | Measured body weight in kilograms. |
| `temperature_c` | REAL | Measured temperature in degrees Celsius. |
| `notes` | TEXT | Additional encounter notes. |

### vaccine_types

Reusable vaccine catalog entries. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `name` | TEXT required | Catalog display name. |
| `species` | TEXT, CHECK species enum | Applicable species; NULL means any species. |
| `default_interval_days` | INTEGER | Suggested interval before the next dose, in days. |
| `notes` | TEXT | Catalog instructions or details. |
| `is_active` | INTEGER 0/1 required, default 1 | Available for new selections; independent of deletion. |

The unique index on `lower(name) WHERE deleted_at IS NULL` prevents duplicate
non-deleted names, including inactive entries. Deletion frees the name; restoring
a deleted entry can conflict with its replacement. SQLite's built-in `lower`
provides ASCII case handling here, not the Turkish search fold. This index does
not promise Turkish/accent-insensitive catalog-name uniqueness.

### vaccinations

A vaccine actually administered. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `patient_id` | INTEGER required, FK patients.id | Vaccinated patient. |
| `vaccine_type_id` | INTEGER, FK vaccine_types.id | Optional catalog reference. |
| `vaccine_name` | TEXT required | Name snapshot at administration, retained after catalog edits. |
| `given_on` | TEXT date required | Administration date. |
| `next_due_on` | TEXT date | Next scheduled dose date, if known. |
| `batch_no` | TEXT | Batch or lot identifier. |
| `visit_id` | INTEGER, FK visits.id | Related encounter, if one was recorded. |
| `notes` | TEXT | Administration notes. |

### medications

Reusable medication catalog entries and dose parameters. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `name` | TEXT required | Medication display name. |
| `active_ingredient` | TEXT | Active ingredient description. |
| `form` | TEXT, CHECK enum | `tablet`, `capsule`, `injection`, `suspension`, `drops`, `ointment`, or `other`; NULL if unknown. |
| `concentration_mg_per_ml` | REAL | Ingredient mass per milliliter for liquid calculations. |
| `strength_mg_per_unit` | REAL | Ingredient mass per tablet, capsule, or other unit. |
| `dose_mg_per_kg_min` | REAL | Lower dose bound in mg/kg. |
| `dose_mg_per_kg_max` | REAL | Upper dose bound in mg/kg; CHECK max >= min when both exist. |
| `route` | TEXT | Administration route. |
| `notes` | TEXT | Catalog instructions, caveats, or preserved details. |

Missing dose bounds are allowed independently. Clinical suitability and additional
numeric validation belong to the app and dose calculator.

### prescription_items

A medication/instruction line attached to a visit. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `visit_id` | INTEGER required, FK visits.id | Encounter that prescribed this item. |
| `medication_id` | INTEGER, FK medications.id | Optional catalog reference. |
| `medication_name` | TEXT required | Medication name snapshot at prescribing. |
| `dose_mg` | REAL | Prescribed dose mass in milligrams. |
| `dose_ml` | REAL | Prescribed dose volume in milliliters. |
| `dose_text` | TEXT | Human-readable dose/instructions, including nonnumeric doses. |
| `frequency` | TEXT | Administration frequency description. |
| `duration_days` | INTEGER | Treatment duration in days. |
| `notes` | TEXT | Additional prescription notes. |

### appointments

A dated appointment, reminder, or task. Includes all four shared columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `patient_id` | INTEGER, FK patients.id | Optional patient association. |
| `owner_id` | INTEGER, FK owners.id | Optional contact association; standalone tasks can omit both links. |
| `title` | TEXT required | Calendar/task display title. |
| `starts_on` | TEXT date required | Calendar date. |
| `starts_time` | TEXT time | Start clock time; NULL means all day. |
| `duration_min` | INTEGER | Scheduled duration in minutes, if known. |
| `kind` | TEXT required, default `other`, CHECK enum | `visit`, `vaccine`, `reminder`, or `other`. |
| `status` | TEXT required, default `planned`, CHECK enum | `planned`, `done`, or `cancelled`. |
| `series_id` | TEXT | Opaque application-generated recurring-series group identifier; no separate series table. |
| `notes` | TEXT | Task or scheduling notes. |

### settings

Local application preferences. No shared audit columns or numeric ID.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `key` | TEXT PRIMARY KEY | Unique preference key. |
| `value` | TEXT required | Encoded preference value, interpreted by the app. |
| `updated_at` | TEXT required, UTC now | Creation/last edit time; maintained by the app on edits. |

### import_runs

Import attempt provenance; not a user-editable clinical record. No shared audit columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `id` | INTEGER PRIMARY KEY | Import run identity. |
| `source_name` | TEXT | Source filename/display label, if available. |
| `source_sha256` | TEXT required | SHA-256 identity of the source snapshot used for duplicate detection. |
| `started_at` | TEXT required, UTC now | Start of the preview/import attempt. |
| `finished_at` | TEXT | Completion time, once the attempt finishes. |
| `status` | TEXT required, CHECK enum | `previewed`, `completed`, or `failed`; supplied explicitly, with no default. |
| `report_json` | TEXT | Serialized import report, including raw unparsed values and warnings. |

`report_json` is stored text, not a SQLite JSON-validated type. It can contain
personal data and remains local; it must never be copied into logs or fixtures.

### legacy_map

Idempotency and source-to-target identity mapping. No shared audit columns.

| Column | Type / requirement | Meaning |
| --- | --- | --- |
| `id` | INTEGER PRIMARY KEY | Mapping identity. |
| `import_run_id` | INTEGER required, FK import_runs.id | Run that created the mapping. |
| `source_sha256` | TEXT required | Same source identity used by the associated import run. |
| `source_table` | TEXT required | Exact detected legacy table name. |
| `source_id` | INTEGER required | Original row ID. |
| `target_table` | TEXT required | Baytar table receiving the row. |
| `target_id` | INTEGER required | Row ID in that table. |

`UNIQUE(source_sha256, source_table, source_id)` skips previously imported rows
from the same file even across separate runs. A different source hash has a
different identity namespace. `target_table`/`target_id` form a polymorphic
reference, so they cannot have one SQL foreign key; the importer validates the
target and hash consistency inside its write transaction.

## Relationships (text ER sketch)

```text
owners       1 <-- N patients
patients     1 <-- N visits
patients     1 <-- N vaccinations
vaccine_types 1 <-- N vaccinations       (catalog link optional)
visits       1 <-- N vaccinations       (visit link optional)
visits       1 <-- N prescription_items
medications  1 <-- N prescription_items (catalog link optional)
owners       1 <-- N appointments       (owner link optional)
patients     1 <-- N appointments       (patient link optional)
import_runs  1 <-- N legacy_map
legacy_map   * ... 1 target_table/target_id (app-validated polymorphic link)
settings: independent key/value preferences
```

All declared arrows restrict hard deletion. Name snapshots on vaccinations and
prescription items preserve the historical label even when catalogs change.

## Indexes

| Table | Indexed columns/expression | Purpose |
| --- | --- | --- |
| owners | `search_text` | Folded owner lookup. |
| patients | `search_text` | Folded patient lookup. |
| patients | `owner_id` | Owner's patient list. |
| visits | `patient_id, visit_on` | Dated patient history. |
| vaccine_types | UNIQUE `lower(name)` WHERE `deleted_at IS NULL` | Non-deleted catalog-name uniqueness. |
| vaccinations | `patient_id` | Patient vaccination history. |
| vaccinations | `next_due_on` WHERE `deleted_at IS NULL` | Upcoming dose lookup. |
| appointments | `starts_on` | Calendar date/range lookup. |
| prescription_items | `visit_id` | Encounter prescription lines. |
| legacy_map | UNIQUE `source_sha256, source_table, source_id` | Import idempotency. |

Primary keys also provide identity lookup. Search-text indexes are ordinary
B-trees, not full-text indexes; arbitrary substring scans are not guaranteed to
use them. Search terms must be folded by the app.

## Legacy PatientsCare mapping

Source details and malformed-value examples are in [legacy-schema.md](legacy-schema.md).
The importer detects available tables and columns, because installed copies can
differ. The mappings below describe the storage contract; this task does not
implement import writes. Unknown columns/tables and every unparsed value must be
preserved in the local report. Unknown or malformed values never crash import.

| Legacy field | Baytar destination | Conversion and preservation |
| --- | --- | --- |
| PatientOwner.Id | legacy_map.source_id -> owners.id | Allocate a new local ID; keep original identity under source table `PatientOwner`. |
| PatientOwner.OwnerName | owners.full_name | Preserve display spelling; app folds its searchable representation. Missing required names need a reported generated label. |
| PatientOwner.OwnerAdress | owners.address | Preserve free text; detect the original misspelled column. |
| PatientOwner.OwnerPhone | owners.phone_raw, owners.phone | Preserve original scalar text, including malformed input. Restore a lost leading zero for recognized Turkish numbers or replace `90` country prefix with `0`; report adjustments/unparsed values. Never invent digits. |
| Patient.Id | legacy_map.source_id -> patients.id | Allocate local ID; source table `Patient`. |
| Patient.OwnerId | patients.owner_id | Resolve through the owner mapping. Missing/deleted/unresolvable legacy owners use a generated `Sahibi bilinmeyen` owner and a report warning. |
| Patient.PatientName | patients.name | Preserve name; missing required names need a reported generated label. |
| Patient.PatientType | patients.species, patients.species_note | Shared legacy normalizer maps Turkish/ASCII aliases to cat/dog/bird/rabbit/other; unknown free text becomes other with the original in species_note and the report. Missing species needs other plus a warning because the target is required. |
| Patient.PatientGender | patients.sex, patients.notes/report | Erkek -> male; Dişi -> female; missing/unknown -> unknown. Preserve unrecognized raw text in notes and report. |
| Patient.BirthDate | patients.birth_date, patients.notes/report | Parse documented Turkish/English long dates and supported numeric/ISO dates through legacy::normalize. Keep an unparsed raw value in notes/report and leave birth_date NULL. Do not infer an estimate from malformed input. |
| Patient.RegistrationDate | patients.created_at (date at 00:00Z); notes when unparsable | The registration date is the patient's history, so the importer stores it in created_at at midnight UTC (the time part carries no meaning). An unparsable value is kept in notes. See docs/legacy-schema.md, Mapping. |
| Patient.PatientNote | patients.notes | Preserve the full original note; append clearly labelled preserved fields without replacing it. |
| ToDo.Id | legacy_map.source_id -> appointments.id | Allocate local ID; source table `ToDo`. |
| ToDo.OwnerId | appointments.owner_id | Resolve owner mapping; use/report the generated unknown owner when orphaned. Resolve conflicts with the linked patient's owner explicitly in the report. |
| ToDo.PatientId | appointments.patient_id | Resolve patient mapping; missing patient may remain NULL with the original ID preserved in notes/report. Do not create a clinical patient just to satisfy a calendar link. |
| ToDo.ToDoName | appointments.title | Preserve title. Missing required titles need a reported generated label. |
| ToDo.ToDoDate | appointments.starts_on | Parse YYYY-MM-DD. No legacy clock time means starts_time NULL. If unparsed, keep the complete row in the report for correction; never invent a required appointment date. |
| ToDo.ToDoNote | appointments.notes | Preserve free text and append any labelled raw values needing retention. |

Legacy to-dos become `kind = reminder`, `status = planned`, one appointment per
source row. Repeated dose-form rows are preserved independently; do not guess a
`series_id` from similar titles or notes. To-dos are plans, not evidence that a
vaccine was given: they do not create vaccinations or visits. Legacy has no
dedicated vaccine catalog, medication, or prescription data; those tables start
empty unless an independently detected source variant supplies documented fields.
Other absent patient/contact fields stay NULL or use their explicit schema defaults.

Import reads a consistent temporary copy (including source WAL/SHM where present)
read-only, previews counts/warnings first, then writes accepted target rows,
import provenance, and mappings together in one transaction. Reports retain rows
requiring correction and flag unknown columns, duplicates, orphans, and unparsed
dates/phones. Older builds lost writes containing apostrophes before saving;
that absent data cannot be recovered and must be described in the report.
