//! Lossless import of a legacy PatientsCare database into Baytar.
//!
//! The flow is `prepare` (copy + hash + schema detection), `preview` (no writes) and
//! `execute` (backup, then one transaction). Preview and execute share one planning pass,
//! so the numbers a user confirms are the numbers that get written.
//!
//! Safety rules:
//! * the source path is only read; everything else happens on a private copy,
//! * missing or unknown tables and columns are tolerated and reported,
//! * unparsable values are kept (in notes or `*_raw` columns) and listed as issues,
//! * every legacy row gets a `legacy_map` entry, so importing the same file again skips it,
//! * any error rolls the whole transaction back; the pre-import backup stays.

mod report;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{Local, NaiveDate};
use rusqlite::types::Value;
use rusqlite::{params, Connection, OpenFlags, Transaction, TransactionBehavior};
use sha2::{Digest, Sha256};

use super::normalize::{self, Raw, Sex, Species, Warning};
use super::LegacyError;
use crate::backup::{self, BackupError, BackupKind};
use crate::db::{check_integrity, Database};
use crate::text::fold;

pub use report::{
    format_preview, format_report, Counts, DetectedSchema, DetectedTable, ImportIssue,
    ImportIssueKind, ImportPreview, ImportReport,
};

/// Name of the owner generated for patients and to-dos whose legacy owner is gone.
pub const UNKNOWN_OWNER_NAME: &str = "Sahibi bilinmeyen";

const UNKNOWN_OWNER_NOTE: &str =
    "Eski kayıtta sahibi bulunamayan hastalar ve hatırlatmalar için oluşturuldu.";
const FALLBACK_OWNER_NAME: &str = "İsimsiz sahip";
const FALLBACK_PATIENT_NAME: &str = "İsimsiz";
const FALLBACK_TODO_TITLE: &str = "Hatırlatma";

const SQLITE_HEADER: &[u8; 16] = b"SQLite format 3\0";
const OWNER_TABLE: &str = "PatientOwner";
const PATIENT_TABLE: &str = "Patient";
const TODO_TABLE: &str = "ToDo";
const ORPHAN_OWNER_TABLE: &str = "orphan_owner";

const KNOWN_TABLES: [(&str, &[&str]); 3] = [
    (
        OWNER_TABLE,
        &["Id", "OwnerName", "OwnerAdress", "OwnerPhone"],
    ),
    (
        PATIENT_TABLE,
        &[
            "Id",
            "OwnerId",
            "PatientName",
            "PatientType",
            "PatientGender",
            "BirthDate",
            "RegistrationDate",
            "PatientNote",
        ],
    ),
    (
        TODO_TABLE,
        &[
            "Id",
            "OwnerId",
            "PatientId",
            "ToDoName",
            "ToDoDate",
            "ToDoNote",
        ],
    ),
];

/// A private copy of a legacy file plus everything learned about it. The copy is deleted
/// when this value is dropped.
#[derive(Debug)]
pub struct PreparedImport {
    _dir: tempfile::TempDir,
    copy_path: PathBuf,
    source_sha256: String,
    source_name: String,
    schema: DetectedSchema,
    layout: Layout,
}

impl PreparedImport {
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn schema(&self) -> &DetectedSchema {
        &self.schema
    }
}

/// Actual (case-preserving) table and column names found for the known legacy tables.
#[derive(Debug, Default)]
struct Layout {
    tables: HashMap<&'static str, TableLayout>,
}

#[derive(Debug)]
struct TableLayout {
    name: String,
    /// Canonical column name -> column name as found in the file.
    columns: HashMap<&'static str, String>,
}

/// Copies the legacy file (and its `-wal`/`-shm` if present) into a fresh folder under
/// `work_dir`, hashes it and detects its schema. The source is only read.
pub fn prepare(source: &Path, work_dir: &Path) -> Result<PreparedImport, LegacyError> {
    check_sqlite_header(source)?;
    fs::create_dir_all(work_dir)?;
    let dir = tempfile::Builder::new()
        .prefix("baytar-import-")
        .tempdir_in(work_dir)?;
    let copy_path = dir.path().join("legacy.db");
    fs::copy(source, &copy_path)?;
    let mut hashed = vec![copy_path.clone()];
    for suffix in ["-wal", "-shm"] {
        let from = with_suffix(source, suffix);
        if from.is_file() {
            let to = with_suffix(&copy_path, suffix);
            fs::copy(&from, &to)?;
            if suffix == "-wal" {
                hashed.push(to);
            }
        }
    }
    // The identity covers the main file plus pending WAL frames: that is the content the
    // copy actually exposes. Without a WAL it equals the plain SHA-256 of the file.
    let source_sha256 = sha256_of(&hashed)?;
    let source_name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    let connection = open_copy_at(&copy_path)?;
    let (schema, layout) = detect(&connection)?;
    if layout.tables.is_empty() {
        return Err(LegacyError::NotLegacyDatabase);
    }
    Ok(PreparedImport {
        _dir: dir,
        copy_path,
        source_sha256,
        source_name,
        schema,
        layout,
    })
}

/// Counts and findings for the prepared file against `target`. Writes nothing.
pub fn preview(p: &PreparedImport, target: &Database) -> Result<ImportPreview, LegacyError> {
    Ok(build_plan(p, target.conn())?.preview)
}

/// Takes a `pre-import` backup, then writes everything in one transaction. On any error
/// nothing is recorded and the backup stays.
pub fn execute(
    p: &PreparedImport,
    target: &mut Database,
    backups_dir: &Path,
) -> Result<ImportReport, LegacyError> {
    let backup = backup_before_import(target, backups_dir)?;
    let backup_file = backup
        .path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    let tx = target
        .conn_mut()
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let plan = build_plan(p, &tx)?;
    tx.execute(
        "INSERT INTO import_runs (source_name, source_sha256, status) VALUES (?1, ?2, 'previewed')",
        params![p.source_name, p.source_sha256],
    )?;
    let run_id = tx.last_insert_rowid();

    let mut writer = Writer {
        tx: &tx,
        run_id,
        sha: &p.source_sha256,
        owners: plan.mapped(OWNER_TABLE),
        patients: plan.mapped(PATIENT_TABLE),
        orphan_owner: plan.mapping.get(&(ORPHAN_OWNER_TABLE, 0)).copied(),
        orphan_owner_created: false,
    };
    let mut inserted = Counts::default();
    for owner in plan.owners.iter().filter(|owner| !owner.skip) {
        writer.insert_owner(owner)?;
        inserted.owners += 1;
    }
    for patient in plan.patients.iter().filter(|patient| !patient.skip) {
        writer.insert_patient(patient)?;
        inserted.patients += 1;
    }
    for todo in plan.todos.iter().filter(|todo| !todo.skip) {
        writer.insert_todo(todo)?;
        inserted.todos += 1;
    }
    if inserted != plan.preview.to_import {
        return Err(LegacyError::Inconsistent(format!(
            "planned {:?} but wrote {:?}",
            plan.preview.to_import, inserted
        )));
    }
    let orphan_owner_created = writer.orphan_owner_created;

    let report = ImportReport {
        skipped: plan.preview.already_imported,
        preview: plan.preview,
        inserted,
        run_id,
        orphan_owner_created,
        backup_file,
    };
    let json = serde_json::to_string(&report)?;
    tx.execute(
        "UPDATE import_runs SET status = 'completed', report_json = ?1, \
         finished_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?2",
        params![json, run_id],
    )?;
    check_integrity(&tx)?;
    tx.commit()?;
    log::info!(
        "legacy import {run_id} completed: {} owners, {} patients, {} to-dos inserted",
        report.inserted.owners,
        report.inserted.patients,
        report.inserted.todos
    );
    Ok(report)
}

/// The backup name has one-second resolution; wait out a collision with an earlier backup
/// (for instance a repeated import of the same file) instead of failing.
fn backup_before_import(target: &Database, dir: &Path) -> Result<backup::BackupInfo, LegacyError> {
    let mut attempts = 0;
    loop {
        match backup::create(target, dir, BackupKind::PreImport) {
            Ok(info) => return Ok(info),
            Err(BackupError::NameCollision(_)) if attempts < 3 => {
                attempts += 1;
                std::thread::sleep(Duration::from_millis(1100));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

// ---------------------------------------------------------------------------------------
// Copy, hash, schema detection
// ---------------------------------------------------------------------------------------

fn check_sqlite_header(source: &Path) -> Result<(), LegacyError> {
    let mut header = [0u8; 16];
    match File::open(source)?.read_exact(&mut header) {
        Ok(()) if &header == SQLITE_HEADER => Ok(()),
        Ok(()) => Err(LegacyError::NotLegacyDatabase),
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            Err(LegacyError::NotLegacyDatabase)
        }
        Err(error) => Err(error.into()),
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

fn sha256_of(paths: &[PathBuf]) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    for path in paths {
        let mut file = File::open(path)?;
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn open_copy_at(path: &Path) -> Result<Connection, LegacyError> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(map_open_error)
}

fn open_copy(p: &PreparedImport) -> Result<Connection, LegacyError> {
    open_copy_at(&p.copy_path)
}

fn map_open_error(error: rusqlite::Error) -> LegacyError {
    match &error {
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::NotADatabase =>
        {
            LegacyError::NotLegacyDatabase
        }
        _ => error.into(),
    }
}

fn quote(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn detect(connection: &Connection) -> Result<(DetectedSchema, Layout), LegacyError> {
    let names: Vec<String> = {
        let mut statement = connection
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' \
                 AND name NOT LIKE 'sqlite_%' ORDER BY name",
            )
            .map_err(map_open_error)?;
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(map_open_error)?;
        rows.collect::<Result<_, _>>()?
    };

    let mut schema = DetectedSchema::default();
    let mut layout = Layout::default();
    for name in names {
        let mut statement = connection.prepare(&format!("PRAGMA table_info({})", quote(&name)))?;
        let columns: Vec<String> = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<_, _>>()?;
        let row_count: i64 = connection.query_row(
            &format!("SELECT COUNT(*) FROM {}", quote(&name)),
            [],
            |row| row.get(0),
        )?;
        let known = KNOWN_TABLES
            .iter()
            .find(|(canonical, _)| canonical.eq_ignore_ascii_case(&name));
        match known {
            Some((canonical, expected)) => {
                let mut found = HashMap::new();
                for column in &columns {
                    match expected
                        .iter()
                        .find(|candidate| candidate.eq_ignore_ascii_case(column))
                    {
                        Some(candidate) => {
                            found.entry(*candidate).or_insert_with(|| column.clone());
                        }
                        None => schema.unknown_columns.push(format!("{canonical}.{column}")),
                    }
                }
                for candidate in *expected {
                    if !found.contains_key(candidate) {
                        schema
                            .missing_columns
                            .push(format!("{canonical}.{candidate}"));
                    }
                }
                layout.tables.insert(
                    canonical,
                    TableLayout {
                        name: name.clone(),
                        columns: found,
                    },
                );
            }
            None => schema.unknown_tables.push(name.clone()),
        }
        schema.tables.push(DetectedTable {
            name,
            known: known.is_some(),
            row_count: u64::try_from(row_count).unwrap_or_default(),
            columns,
        });
    }
    for (canonical, _) in KNOWN_TABLES {
        if !layout.tables.contains_key(canonical) {
            schema.missing_tables.push(canonical.to_owned());
        }
    }
    Ok((schema, layout))
}

// ---------------------------------------------------------------------------------------
// Reading source rows
// ---------------------------------------------------------------------------------------

static NULL: Value = Value::Null;

struct SourceRow {
    rowid: i64,
    values: HashMap<&'static str, Value>,
}

impl SourceRow {
    fn get(&self, column: &str) -> &Value {
        self.values.get(column).unwrap_or(&NULL)
    }
}

fn read_table(
    connection: &Connection,
    layout: &Layout,
    table: &'static str,
) -> Result<Vec<SourceRow>, LegacyError> {
    let Some(table_layout) = layout.tables.get(table) else {
        return Ok(Vec::new());
    };
    let wanted = KNOWN_TABLES
        .iter()
        .find(|(name, _)| *name == table)
        .map_or(&[][..], |(_, columns)| *columns);
    let present: Vec<(&'static str, &String)> = wanted
        .iter()
        .filter_map(|column| {
            table_layout
                .columns
                .get(column)
                .map(|found| (*column, found))
        })
        .collect();
    let mut sql = String::from("SELECT rowid");
    for (_, found) in &present {
        sql.push_str(", ");
        sql.push_str(&quote(found));
    }
    sql.push_str(&format!(
        " FROM {} ORDER BY rowid",
        quote(&table_layout.name)
    ));
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([], |row| {
        let mut values = HashMap::new();
        for (index, (canonical, _)) in present.iter().enumerate() {
            let value = match row.get::<_, Value>(index + 1)? {
                Value::Blob(bytes) => Value::Text(String::from_utf8_lossy(&bytes).into_owned()),
                other => other,
            };
            values.insert(*canonical, value);
        }
        Ok(SourceRow {
            rowid: row.get(0)?,
            values,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn raw(value: &Value) -> Raw<'_> {
    match value {
        Value::Integer(number) => Raw::Integer(*number),
        Value::Real(number) => Raw::Real(*number),
        Value::Text(text) => Raw::Text(text),
        Value::Null | Value::Blob(_) => Raw::Null,
    }
}

fn text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        Value::Integer(number) => Some(number.to_string()),
        Value::Real(number) => Some(number.to_string()),
        Value::Null | Value::Blob(_) => None,
    }
}

/// Text that has something other than whitespace in it, otherwise `None`.
fn nonblank(value: &Value) -> Option<String> {
    text(value).filter(|text| !text.trim().is_empty())
}

/// The original text, untouched, unless it is only whitespace.
fn verbatim(value: &Value) -> Option<String> {
    text(value).filter(|text| !text.trim().is_empty())
}

fn id_value(value: &Value) -> Option<i64> {
    match value {
        Value::Integer(number) => Some(*number),
        Value::Real(number) if number.fract() == 0.0 && number.abs() < 9.0e15 => {
            Some(*number as i64)
        }
        Value::Text(text) => text.trim().parse().ok(),
        _ => None,
    }
}

/// The legacy `Id`, or a stable substitute when the column is missing, not a number, or
/// repeated (tables from other app variants are not guaranteed to have a primary key).
fn source_id(row: &SourceRow, table: &str, seen: &mut HashSet<i64>, pv: &mut ImportPreview) -> i64 {
    let has_id_column = row.values.contains_key("Id");
    if let Some(id) = id_value(row.get("Id")) {
        if seen.insert(id) {
            return id;
        }
    } else if !has_id_column {
        let id = row.rowid;
        seen.insert(id);
        return id;
    }
    let mut id = -row.rowid;
    while !seen.insert(id) {
        id -= 1 << 40;
    }
    add_issue(
        pv,
        table,
        id,
        "Id",
        ImportIssueKind::UnusableId,
        text(row.get("Id")),
    );
    id
}

fn add_issue(
    pv: &mut ImportPreview,
    table: &str,
    source_id: i64,
    field: &str,
    kind: ImportIssueKind,
    raw: Option<String>,
) {
    pv.issues.push(ImportIssue {
        table: table.to_owned(),
        source_id,
        field: field.to_owned(),
        kind,
        raw,
    });
}

enum DateField {
    Parsed(NaiveDate),
    Blank,
    Unparsed(String),
}

fn read_date(value: &Value) -> DateField {
    let parsed = normalize::date(raw(value));
    match (parsed.value, parsed.raw) {
        (Some(date), _) => DateField::Parsed(date),
        (None, Some(original)) if !original.trim().is_empty() => DateField::Unparsed(original),
        _ => DateField::Blank,
    }
}

fn compose_notes(base: Option<String>, extra: Vec<String>) -> Option<String> {
    let mut parts: Vec<String> = base.into_iter().collect();
    parts.extend(extra);
    (!parts.is_empty()).then(|| parts.join("\n"))
}

// ---------------------------------------------------------------------------------------
// Planning
// ---------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerRef {
    Source(i64),
    Orphan,
}

struct PlannedOwner {
    source_id: i64,
    full_name: String,
    phone: Option<String>,
    phone_raw: Option<String>,
    address: Option<String>,
    search_text: String,
    skip: bool,
}

struct PlannedPatient {
    source_id: i64,
    owner: OwnerRef,
    name: String,
    species: Species,
    species_note: Option<String>,
    sex: Sex,
    birth_date: Option<String>,
    notes: Option<String>,
    created_at: Option<String>,
    search_text: String,
    skip: bool,
}

struct PlannedTodo {
    source_id: i64,
    patient: Option<i64>,
    owner: OwnerRef,
    title: String,
    starts_on: String,
    status: &'static str,
    notes: Option<String>,
    skip: bool,
}

struct Plan {
    owners: Vec<PlannedOwner>,
    patients: Vec<PlannedPatient>,
    todos: Vec<PlannedTodo>,
    /// (source table, source id) -> target id, for rows already imported from this file.
    mapping: HashMap<(&'static str, i64), i64>,
    preview: ImportPreview,
}

impl Plan {
    fn mapped(&self, table: &'static str) -> HashMap<i64, i64> {
        self.mapping
            .iter()
            .filter(|((name, _), _)| *name == table)
            .map(|((_, source), target)| (*source, *target))
            .collect()
    }
}

fn static_table(name: &str) -> Option<&'static str> {
    [OWNER_TABLE, PATIENT_TABLE, TODO_TABLE, ORPHAN_OWNER_TABLE]
        .into_iter()
        .find(|known| *known == name)
}

fn load_mapping(
    target: &Connection,
    sha: &str,
) -> Result<HashMap<(&'static str, i64), i64>, LegacyError> {
    let mut statement = target.prepare(
        "SELECT source_table, source_id, target_id FROM legacy_map WHERE source_sha256 = ?1",
    )?;
    let rows = statement.query_map([sha], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut mapping = HashMap::new();
    for row in rows {
        let (table, source, target_id) = row?;
        if let Some(table) = static_table(&table) {
            mapping.insert((table, source), target_id);
        }
    }
    Ok(mapping)
}

/// (folded name, phone) of every live Baytar owner that has a phone.
fn existing_owner_keys(target: &Connection) -> Result<HashSet<(String, String)>, LegacyError> {
    let mut statement = target.prepare(
        "SELECT full_name, phone FROM owners WHERE deleted_at IS NULL AND phone IS NOT NULL",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut keys = HashSet::new();
    for row in rows {
        let (name, phone) = row?;
        keys.insert((fold(&name), phone.trim().to_owned()));
    }
    Ok(keys)
}

fn empty_preview(p: &PreparedImport) -> ImportPreview {
    ImportPreview {
        source_name: p.source_name.clone(),
        source_sha256: p.source_sha256.clone(),
        schema: p.schema.clone(),
        found: Counts::default(),
        already_imported: Counts::default(),
        to_import: Counts::default(),
        phones_adjusted: 0,
        phones_unparsed: 0,
        dates_unparsed: BTreeMap::new(),
        species_mapped_to_other: 0,
        species_missing: 0,
        sex_mapped_to_unknown: 0,
        orphan_patients: 0,
        orphan_todos: 0,
        duplicate_owners: 0,
        todos_done: 0,
        todos_planned: 0,
        orphan_owner_needed: false,
        issues: Vec::new(),
    }
}

fn count_date(pv: &mut ImportPreview, field: &str) {
    *pv.dates_unparsed.entry(field.to_owned()).or_default() += 1;
}

/// Reads the copy and decides, without writing anything, what each legacy row becomes.
fn build_plan(p: &PreparedImport, target: &Connection) -> Result<Plan, LegacyError> {
    let source = open_copy(p)?;
    let mapping = load_mapping(target, &p.source_sha256)?;
    let existing = existing_owner_keys(target)?;
    let today = Local::now().date_naive();
    let mut pv = empty_preview(p);

    // Owners.
    let mut owners = Vec::new();
    let mut owner_ids = HashSet::new();
    let mut seen = HashSet::new();
    for row in read_table(&source, &p.layout, OWNER_TABLE)? {
        let id = source_id(&row, OWNER_TABLE, &mut seen, &mut pv);
        owner_ids.insert(id);
        let name = text(row.get("OwnerName")).unwrap_or_default();
        let name = name.trim();
        let full_name = if name.is_empty() {
            add_issue(
                &mut pv,
                OWNER_TABLE,
                id,
                "OwnerName",
                ImportIssueKind::MissingName,
                None,
            );
            FALLBACK_OWNER_NAME.to_owned()
        } else {
            name.to_owned()
        };

        let normalized = normalize::phone(raw(row.get("OwnerPhone")));
        let phone_raw = normalized
            .raw
            .filter(|original| !original.trim().is_empty());
        let phone = match (&normalized.value, &phone_raw) {
            (Some(e164), _) => Some(e164.clone()),
            // Unrecognized input is kept as typed so the vet still sees it.
            (None, Some(original)) => Some(original.trim().to_owned()),
            (None, None) => None,
        };
        if phone_raw.is_some() {
            match normalized.warning {
                Some(Warning::Adjusted) => pv.phones_adjusted += 1,
                Some(Warning::Unparsed) => {
                    pv.phones_unparsed += 1;
                    add_issue(
                        &mut pv,
                        OWNER_TABLE,
                        id,
                        "OwnerPhone",
                        ImportIssueKind::PhoneUnparsed,
                        phone_raw.clone(),
                    );
                }
                _ => {}
            }
        }

        let skip = mapping.contains_key(&(OWNER_TABLE, id));
        if !skip {
            if let Some(e164) = &normalized.value {
                if existing.contains(&(fold(&full_name), e164.clone())) {
                    pv.duplicate_owners += 1;
                    add_issue(
                        &mut pv,
                        OWNER_TABLE,
                        id,
                        "OwnerName",
                        ImportIssueKind::DuplicateOwner,
                        None,
                    );
                }
            }
        }
        owners.push(PlannedOwner {
            source_id: id,
            search_text: crate::repo::owners::search_key(&full_name, phone.as_deref(), None, None),
            full_name,
            phone,
            phone_raw,
            address: verbatim(row.get("OwnerAdress")),
            skip,
        });
    }

    // Patients.
    let mut patients = Vec::new();
    let mut patient_owner: HashMap<i64, OwnerRef> = HashMap::new();
    let mut seen = HashSet::new();
    for row in read_table(&source, &p.layout, PATIENT_TABLE)? {
        let id = source_id(&row, PATIENT_TABLE, &mut seen, &mut pv);
        let mut extra = Vec::new();

        let name = text(row.get("PatientName")).unwrap_or_default();
        let name = name.trim();
        let name = if name.is_empty() {
            add_issue(
                &mut pv,
                PATIENT_TABLE,
                id,
                "PatientName",
                ImportIssueKind::MissingName,
                None,
            );
            FALLBACK_PATIENT_NAME.to_owned()
        } else {
            name.to_owned()
        };

        let owner_value = row.get("OwnerId");
        let owner = match id_value(owner_value).filter(|owner| owner_ids.contains(owner)) {
            Some(owner) => OwnerRef::Source(owner),
            None => {
                pv.orphan_patients += 1;
                let original = nonblank(owner_value);
                if let Some(original) = &original {
                    extra.push(format!("Eski kayıttaki sahip no: {}", original.trim()));
                }
                add_issue(
                    &mut pv,
                    PATIENT_TABLE,
                    id,
                    "OwnerId",
                    ImportIssueKind::OrphanPatient,
                    original,
                );
                OwnerRef::Orphan
            }
        };
        patient_owner.insert(id, owner);

        let species_value = normalize::species(raw(row.get("PatientType")));
        let mut species_note = None;
        if species_value.value.is_none() {
            pv.species_missing += 1;
        }
        if species_value.warning == Some(Warning::Adjusted) {
            pv.species_mapped_to_other += 1;
            let original = species_value.raw.clone().unwrap_or_default();
            species_note = Some(original.trim().to_owned());
            add_issue(
                &mut pv,
                PATIENT_TABLE,
                id,
                "PatientType",
                ImportIssueKind::SpeciesUnrecognized,
                Some(original),
            );
        }
        let sex_value = normalize::sex(raw(row.get("PatientGender")));
        if sex_value.warning == Some(Warning::Adjusted) {
            pv.sex_mapped_to_unknown += 1;
            let original = sex_value.raw.clone().unwrap_or_default();
            extra.push(format!("Eski kayıttaki cinsiyet: {}", original.trim()));
            add_issue(
                &mut pv,
                PATIENT_TABLE,
                id,
                "PatientGender",
                ImportIssueKind::SexUnrecognized,
                Some(original),
            );
        }

        let mut birth_date = None;
        match read_date(row.get("BirthDate")) {
            DateField::Parsed(date) => birth_date = Some(date.format("%Y-%m-%d").to_string()),
            DateField::Blank => {}
            DateField::Unparsed(original) => {
                count_date(&mut pv, "Patient.BirthDate");
                extra.push(format!("Eski kayıttaki doğum tarihi: {}", original.trim()));
                add_issue(
                    &mut pv,
                    PATIENT_TABLE,
                    id,
                    "BirthDate",
                    ImportIssueKind::DateUnparsed,
                    Some(original),
                );
            }
        }
        let mut created_at = None;
        match read_date(row.get("RegistrationDate")) {
            DateField::Parsed(date) => {
                created_at = Some(date.format("%Y-%m-%dT00:00:00.000Z").to_string());
            }
            DateField::Blank => {}
            DateField::Unparsed(original) => {
                count_date(&mut pv, "Patient.RegistrationDate");
                extra.push(format!("Eski kayıttaki kayıt tarihi: {}", original.trim()));
                add_issue(
                    &mut pv,
                    PATIENT_TABLE,
                    id,
                    "RegistrationDate",
                    ImportIssueKind::DateUnparsed,
                    Some(original),
                );
            }
        }

        let search_text = fold(&format!(
            "{name} {}",
            species_note.as_deref().unwrap_or_default()
        ));
        patients.push(PlannedPatient {
            source_id: id,
            owner,
            name,
            species: species_value.value.unwrap_or(Species::Other),
            species_note,
            sex: sex_value.value.unwrap_or(Sex::Unknown),
            birth_date,
            notes: compose_notes(verbatim(row.get("PatientNote")), extra),
            created_at,
            search_text,
            skip: mapping.contains_key(&(PATIENT_TABLE, id)),
        });
    }

    // To-dos.
    let mut todos = Vec::new();
    let mut seen = HashSet::new();
    for row in read_table(&source, &p.layout, TODO_TABLE)? {
        let id = source_id(&row, TODO_TABLE, &mut seen, &mut pv);
        let mut extra = Vec::new();

        let title = text(row.get("ToDoName")).unwrap_or_default();
        let title = title.trim();
        let title = if title.is_empty() {
            add_issue(
                &mut pv,
                TODO_TABLE,
                id,
                "ToDoName",
                ImportIssueKind::MissingName,
                None,
            );
            FALLBACK_TODO_TITLE.to_owned()
        } else {
            title.to_owned()
        };

        let patient_value = row.get("PatientId");
        let owner_value = row.get("OwnerId");
        let patient = id_value(patient_value).filter(|patient| patient_owner.contains_key(patient));
        let direct_owner = id_value(owner_value).filter(|owner| owner_ids.contains(owner));
        if patient.is_none() {
            if let Some(original) = nonblank(patient_value) {
                extra.push(format!("Eski kayıttaki hasta no: {}", original.trim()));
                add_issue(
                    &mut pv,
                    TODO_TABLE,
                    id,
                    "PatientId",
                    ImportIssueKind::DanglingPatientLink,
                    Some(original),
                );
            }
        }
        if direct_owner.is_none() {
            if let Some(original) = nonblank(owner_value) {
                extra.push(format!("Eski kayıttaki sahip no: {}", original.trim()));
                add_issue(
                    &mut pv,
                    TODO_TABLE,
                    id,
                    "OwnerId",
                    ImportIssueKind::DanglingOwnerLink,
                    Some(original),
                );
            }
        }
        if patient.is_none() && direct_owner.is_none() {
            pv.orphan_todos += 1;
            add_issue(
                &mut pv,
                TODO_TABLE,
                id,
                "OwnerId",
                ImportIssueKind::OrphanTodo,
                None,
            );
        }
        let patient_owner_ref = patient.and_then(|patient| patient_owner.get(&patient).copied());
        if let (Some(direct), Some(from_patient)) = (direct_owner, patient_owner_ref) {
            if from_patient != OwnerRef::Source(direct) {
                add_issue(
                    &mut pv,
                    TODO_TABLE,
                    id,
                    "OwnerId",
                    ImportIssueKind::OwnerMismatch,
                    None,
                );
            }
        }
        let owner = direct_owner
            .map(OwnerRef::Source)
            .or(patient_owner_ref)
            .unwrap_or(OwnerRef::Orphan);

        let (starts_on, status) = match read_date(row.get("ToDoDate")) {
            DateField::Parsed(date) => (
                date.format("%Y-%m-%d").to_string(),
                if date < today { "done" } else { "planned" },
            ),
            other => {
                count_date(&mut pv, "ToDo.ToDoDate");
                let original = match other {
                    DateField::Unparsed(original) => {
                        extra.push(format!("Eski kayıttaki tarih: {}", original.trim()));
                        Some(original)
                    }
                    _ => {
                        extra.push("Eski kayıtta tarih yoktu.".to_owned());
                        None
                    }
                };
                add_issue(
                    &mut pv,
                    TODO_TABLE,
                    id,
                    "ToDoDate",
                    ImportIssueKind::DateUnparsed,
                    original,
                );
                (today.format("%Y-%m-%d").to_string(), "planned")
            }
        };

        let skip = mapping.contains_key(&(TODO_TABLE, id));
        if !skip {
            if status == "done" {
                pv.todos_done += 1;
            } else {
                pv.todos_planned += 1;
            }
        }
        todos.push(PlannedTodo {
            source_id: id,
            patient,
            owner,
            title,
            starts_on,
            status,
            notes: compose_notes(verbatim(row.get("ToDoNote")), extra),
            skip,
        });
    }

    // Totals.
    pv.found = Counts {
        owners: owners.len() as u64,
        patients: patients.len() as u64,
        todos: todos.len() as u64,
    };
    pv.already_imported = Counts {
        owners: owners.iter().filter(|row| row.skip).count() as u64,
        patients: patients.iter().filter(|row| row.skip).count() as u64,
        todos: todos.iter().filter(|row| row.skip).count() as u64,
    };
    pv.to_import = Counts {
        owners: pv.found.owners - pv.already_imported.owners,
        patients: pv.found.patients - pv.already_imported.patients,
        todos: pv.found.todos - pv.already_imported.todos,
    };
    let needs_orphan = patients
        .iter()
        .any(|row| !row.skip && row.owner == OwnerRef::Orphan)
        || todos
            .iter()
            .any(|row| !row.skip && row.owner == OwnerRef::Orphan);
    pv.orphan_owner_needed = needs_orphan && !mapping.contains_key(&(ORPHAN_OWNER_TABLE, 0));

    Ok(Plan {
        owners,
        patients,
        todos,
        mapping,
        preview: pv,
    })
}

// ---------------------------------------------------------------------------------------
// Writing
// ---------------------------------------------------------------------------------------

struct Writer<'a> {
    tx: &'a Transaction<'a>,
    run_id: i64,
    sha: &'a str,
    owners: HashMap<i64, i64>,
    patients: HashMap<i64, i64>,
    orphan_owner: Option<i64>,
    orphan_owner_created: bool,
}

impl Writer<'_> {
    fn map_row(
        &self,
        source_table: &str,
        source_id: i64,
        target_table: &str,
        target_id: i64,
    ) -> Result<(), LegacyError> {
        self.tx.execute(
            "INSERT INTO legacy_map (import_run_id, source_sha256, source_table, source_id, \
             target_table, target_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.run_id,
                self.sha,
                source_table,
                source_id,
                target_table,
                target_id
            ],
        )?;
        Ok(())
    }

    fn insert_owner(&mut self, owner: &PlannedOwner) -> Result<(), LegacyError> {
        self.tx.execute(
            "INSERT INTO owners (full_name, phone, phone_raw, address, search_text) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                owner.full_name,
                owner.phone,
                owner.phone_raw,
                owner.address,
                owner.search_text
            ],
        )?;
        let id = self.tx.last_insert_rowid();
        self.map_row(OWNER_TABLE, owner.source_id, "owners", id)?;
        self.owners.insert(owner.source_id, id);
        Ok(())
    }

    /// The generated owner for this file: created on first use, reused afterwards.
    fn resolve_owner(&mut self, owner: OwnerRef) -> Result<i64, LegacyError> {
        match owner {
            OwnerRef::Source(source) => self.owners.get(&source).copied().ok_or_else(|| {
                LegacyError::Inconsistent(format!("owner {source} has no target row"))
            }),
            OwnerRef::Orphan => {
                if let Some(id) = self.orphan_owner {
                    return Ok(id);
                }
                self.tx.execute(
                    "INSERT INTO owners (full_name, notes, search_text) VALUES (?1, ?2, ?3)",
                    params![
                        UNKNOWN_OWNER_NAME,
                        UNKNOWN_OWNER_NOTE,
                        fold(UNKNOWN_OWNER_NAME)
                    ],
                )?;
                let id = self.tx.last_insert_rowid();
                self.map_row(ORPHAN_OWNER_TABLE, 0, "owners", id)?;
                self.orphan_owner = Some(id);
                self.orphan_owner_created = true;
                Ok(id)
            }
        }
    }

    fn insert_patient(&mut self, patient: &PlannedPatient) -> Result<(), LegacyError> {
        let owner_id = self.resolve_owner(patient.owner)?;
        self.tx.execute(
            "INSERT INTO patients (owner_id, name, species, species_note, sex, birth_date, notes, \
             search_text, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, \
             COALESCE(?9, strftime('%Y-%m-%dT%H:%M:%fZ','now')))",
            params![
                owner_id,
                patient.name,
                patient.species.as_str(),
                patient.species_note,
                patient.sex.as_str(),
                patient.birth_date,
                patient.notes,
                patient.search_text,
                patient.created_at
            ],
        )?;
        let id = self.tx.last_insert_rowid();
        self.map_row(PATIENT_TABLE, patient.source_id, "patients", id)?;
        self.patients.insert(patient.source_id, id);
        Ok(())
    }

    fn insert_todo(&mut self, todo: &PlannedTodo) -> Result<(), LegacyError> {
        let owner_id = self.resolve_owner(todo.owner)?;
        let patient_id = match todo.patient {
            Some(source) => Some(self.patients.get(&source).copied().ok_or_else(|| {
                LegacyError::Inconsistent(format!("patient {source} has no target row"))
            })?),
            None => None,
        };
        self.tx.execute(
            "INSERT INTO appointments (patient_id, owner_id, title, starts_on, kind, status, notes) \
             VALUES (?1, ?2, ?3, ?4, 'reminder', ?5, ?6)",
            params![patient_id, owner_id, todo.title, todo.starts_on, todo.status, todo.notes],
        )?;
        let id = self.tx.last_insert_rowid();
        self.map_row(TODO_TABLE, todo.source_id, "appointments", id)?;
        Ok(())
    }
}
