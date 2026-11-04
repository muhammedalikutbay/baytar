# Legacy PatientsCare database

What the Windows Forms app (tag `v0.1-winforms`, branch `legacy`) wrote to SQLite, and what the importer must
expect. Sources: `PatientCare/Services/DatabaseService.cs`, `PatientsAdd.cs`, `PatientEdit.cs`, `ToDoAdd.cs`,
`Operations.cs` on the `legacy` branch; `master` (commit `1baaa35`) is an older variant of the same app.

## Files
- Default names: `HastaTakipDB.db` (appsettings default) and `DBPatientCare.db` (dev/release settings).
- The connection string uses a relative `Data Source`, resolved against the process working directory.
  A user may therefore have several database files (one per folder the exe was started from).
- Users run copies other than the GitHub release. Column sets may differ: detect columns with
  `PRAGMA table_info` and map what exists. Never assume the exact schema below.

## Tables (as created by the app)
```sql
CREATE TABLE IF NOT EXISTS PatientOwner(
  "Id" INTEGER, "OwnerName" TEXT, "OwnerAdress" TEXT, "OwnerPhone" INTEGER,
  PRIMARY KEY("Id"));

CREATE TABLE IF NOT EXISTS Patient(
  "Id" INTEGER, "OwnerId" INTEGER, "PatientName" TEXT, "PatientType" TEXT, "PatientGender" TEXT,
  "BirthDate" TEXT, "RegistrationDate" TEXT, "PatientNote" TEXT,
  PRIMARY KEY("Id"), FOREIGN KEY("OwnerId") REFERENCES "PatientOwner"("Id"));

CREATE TABLE IF NOT EXISTS ToDo(
  "Id" INTEGER, "OwnerId" INTEGER, "PatientId" INTEGER, "ToDoName" TEXT, "ToDoDate" DATETIME, "ToDoNote" TEXT,
  PRIMARY KEY("Id"),
  FOREIGN KEY("OwnerId") REFERENCES "PatientOwner"("Id"),
  FOREIGN KEY("PatientId") REFERENCES "Patient"("Id"));
```
Note the misspelled column `OwnerAdress`. Foreign keys were never enforced (`PRAGMA foreign_keys` is off by default).

## Known data problems
| Field | Problem | Import rule |
|---|---|---|
| `OwnerPhone` | INTEGER affinity: `"05321234567"` was stored as `5321234567`; leading zero lost. Non-numeric input stays text. | Keep the raw value. Normalize: 10 digits starting with `5` → prepend `0`; `90` + 10 digits → `0` + 10 digits; otherwise keep as typed and flag. |
| `RegistrationDate` | `DateTime.Now.ToString("dd-MM-yyyy")` → `04-11-2026`. | Parse `dd-MM-yyyy`. |
| `BirthDate` | `DateTimePicker.Text` with the default Long format of the PC's culture. On Turkish Windows: `4 Kasım 2026 Çarşamba` (`d MMMM yyyy dddd`). On other cultures, e.g. `Wednesday, November 4, 2026`. | Parse Turkish and English long dates (month and day names, with or without weekday), plus `dd.MM.yyyy`, `dd-MM-yyyy`, `yyyy-MM-dd`, ISO date-time. Unknown → keep raw text in a note and flag. |
| `ToDoDate` | `yyyy-MM-dd` in a DATETIME column. | Parse ISO date. |
| `PatientType` | Combo values `Kedi`, `Köpek`, `Diğer`, but free text is possible. | Map to species `cat`, `dog`, `other`; anything else → `other` with the original text kept. |
| `PatientGender` | `Erkek`, `Dişi`, or free text. | Map to `male`, `female`, `unknown`. |
| Orphans | Deleting an owner did not delete their patients or to-dos. | Import under a generated owner "Sahibi bilinmeyen" and list them in the report. |
| Apostrophes | Older `master` builds concatenated strings into SQL: names or notes containing `'` failed to save. | Nothing to recover; mention in the report text. |
| To-do series | One "dose" form created N rows `range` days apart with the same name and note. | Import each row as its own reminder/appointment; keep the title. |

## Importer contract
1. Copy the selected file (and its `-wal`/`-shm` if present) to a temp folder; open the copy read-only.
2. Detect tables and columns; unknown tables and columns are listed, not fatal.
3. Build a preview: counts per table, normalized phones, unparsed dates, orphans, duplicates.
4. On confirm, write everything in one transaction; record `legacy_map(source_file_hash, source_table, source_id, target_table, target_id)` and an `import_runs` row with the report.
5. Importing the same file again skips rows already in `legacy_map`.

## Mapping
Implemented in `baytar_core::legacy::import` (`prepare` → `preview` → `execute`). Where this section differs
from the older sketches in `data-model.md`, this section is the contract.

Identity and safety:
- `source_sha256` is the SHA-256 of the copied main file followed by its `-wal` if present (equal to the
  plain file hash when there is no WAL). Source tables are recorded in `legacy_map` under the canonical
  names `PatientOwner`, `Patient`, `ToDo`; the generated owner uses `source_table = 'orphan_owner'`, `source_id = 0`.
- A missing `Id` column falls back to `rowid`; an `Id` that is not an integer or is repeated gets a
  negative substitute id and an `unusableId` issue.
- `execute` takes a `pre-import` backup first (waiting out a same-second name collision), then writes
  everything in one `IMMEDIATE` transaction and runs `integrity_check` + `foreign_key_check` before commit.
  `import_runs.status` is set to `completed` at the end together with `report_json`; any error rolls
  everything back and records nothing. Rows already in `legacy_map` for the same hash are skipped.

| Legacy | Baytar | Rule |
|---|---|---|
| `PatientOwner.OwnerName` | `owners.full_name` | Trimmed; empty → `İsimsiz sahip` (issue `missingName`). |
| `PatientOwner.OwnerPhone` | `owners.phone`, `phone_raw` | `phone_raw` is the original value (INTEGER without decimals). `phone` is the normalized number; if it cannot be normalized, the trimmed original is kept as typed and the row gets a `phoneUnparsed` issue. |
| `PatientOwner.OwnerAdress` | `owners.address` | Verbatim. |
| (derived) | `owners.search_text` | `fold(name + " " + phone digits)`. |
| `Patient.PatientName` | `patients.name` | Trimmed; empty → `İsimsiz`. |
| `Patient.PatientType` | `species`, `species_note` | `legacy::normalize`; no value → `other`; text that is not a known value → `other` with the text in `species_note` (issue `speciesUnrecognized`). |
| `Patient.PatientGender` | `sex`, `notes` | Known → `male`/`female`; no value → `unknown`; other text → `unknown` and `Eski kayıttaki cinsiyet: <raw>` appended to `notes`. |
| `Patient.BirthDate` | `birth_date`, `notes` | Parsed to `YYYY-MM-DD`; unparsed → `Eski kayıttaki doğum tarihi: <raw>` appended to `notes` (issue `dateUnparsed`). |
| `Patient.RegistrationDate` | `created_at` | Parsed date at `T00:00:00.000Z`; unparsed → `Eski kayıttaki kayıt tarihi: <raw>` in `notes`; absent → insertion time. |
| `Patient.PatientNote` | `notes` | Verbatim first; labelled lines above are appended after it, one per line. |
| `Patient.OwnerId` | `owner_id` | Through `legacy_map`. Missing or unknown → the generated owner `Sahibi bilinmeyen` (one per source file, created lazily, reused by later runs of the same file), `Eski kayıttaki sahip no: <id>` in `notes`, issue `orphanPatient`. |
| `ToDo` | `appointments` | `kind = 'reminder'`; `title = ToDoName` (empty → `Hatırlatma`); `notes = ToDoNote`. |
| `ToDo.ToDoDate` | `starts_on`, `status` | Date before the import day → `done` (the old app only showed today's list); today or later → `planned`. Unparsed or empty → import day, `planned`, `Eski kayıttaki tarih: <raw>` (or `Eski kayıtta tarih yoktu.`) in `notes`, issue `dateUnparsed`. |
| `ToDo.PatientId` / `OwnerId` | `patient_id` / `owner_id` | Through `legacy_map`. A missing owner falls back to the patient's owner; with neither, the generated owner is used (issue `orphanTodo`). A dangling id is kept in `notes` (`Eski kayıttaki hasta no:` / `sahip no:`) with an issue; an owner that disagrees with the patient's owner is kept and flagged `ownerMismatch`. |

Preview and report (`serde`, camelCase): counts found / already imported / to import per table, phones
adjusted and unparsed, unparsed dates per field, species and sex mapped to `other`/`unknown`, orphans, owners
that look like existing Baytar owners (same folded name and same normalized phone; reported, never skipped),
detected schema (unknown/missing tables and columns) and `issues: [{ table, sourceId, field, kind, raw }]`.
The stored `report_json` contains raw values and stays local. `baytar-cli import` prints aggregates only; raw
values need `--show-values`.
