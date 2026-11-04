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
