//! Read-only aggregate inspection of legacy PatientsCare databases.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use super::LegacyError;

#[derive(Debug, Clone, Serialize)]
pub struct InspectReport {
    pub user_version: i64,
    pub page_count: i64,
    pub tables: Vec<TableReport>,
    pub known_tables: BTreeMap<String, KnownTableReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TableReport {
    pub name: String,
    pub row_count: i64,
    pub columns: Vec<ColumnReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnReport {
    pub name: String,
    pub declared_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KnownTableReport {
    pub present: bool,
    pub missing_columns: Vec<String>,
    pub phone: Option<PhoneReport>,
    pub dates: BTreeMap<String, BTreeMap<String, i64>>,
    pub combo_values: BTreeMap<String, BTreeMap<String, i64>>,
    pub orphan_patients: Option<i64>,
    pub orphan_todos: Option<i64>,
    pub duplicate_looking_owners: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PhoneReport {
    pub storage_classes: BTreeMap<String, i64>,
    pub digit_length_buckets: BTreeMap<String, i64>,
    pub ten_digit_starting_with_five: i64,
    pub containing_non_digits: i64,
}

const KNOWN_TABLES: [&str; 3] = ["PatientOwner", "Patient", "ToDo"];
const DATE_CATEGORIES: [&str; 8] = [
    "dd-MM-yyyy",
    "dd.MM.yyyy",
    "yyyy-MM-dd",
    "iso-datetime",
    "long-tr",
    "long-en",
    "empty",
    "other",
];

/// Inspect a legacy database without permitting SQLite to create or modify sidecar files.
pub fn inspect(path: &Path) -> Result<InspectReport, LegacyError> {
    let uri = immutable_uri(path)?;
    let connection = Connection::open_with_flags(
        uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )?;
    let user_version = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let page_count = connection.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    let table_names = {
        let mut statement = connection.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let names = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        names
    };

    let mut tables = Vec::new();
    let mut columns_by_table = HashMap::new();
    for name in &table_names {
        let columns = table_columns(&connection, name)?;
        let sql = format!("SELECT COUNT(*) FROM {}", quote_identifier(name));
        let row_count = connection.query_row(&sql, [], |row| row.get(0))?;
        columns_by_table.insert(name.clone(), columns.clone());
        tables.push(TableReport {
            name: name.clone(),
            row_count,
            columns,
        });
    }

    let existing: HashSet<_> = table_names.iter().cloned().collect();
    let mut known_tables = BTreeMap::new();
    for table in KNOWN_TABLES {
        let present = existing.contains(table);
        let mut report = KnownTableReport {
            present,
            missing_columns: Vec::new(),
            phone: None,
            dates: BTreeMap::new(),
            combo_values: BTreeMap::new(),
            orphan_patients: None,
            orphan_todos: None,
            duplicate_looking_owners: None,
        };
        if present {
            let names: HashSet<_> = columns_by_table[table]
                .iter()
                .map(|column| column.name.as_str())
                .collect();
            let expected = match table {
                "PatientOwner" => &["OwnerPhone"][..],
                "Patient" => &[
                    "BirthDate",
                    "RegistrationDate",
                    "PatientType",
                    "PatientGender",
                    "OwnerId",
                ][..],
                "ToDo" => &["ToDoDate", "PatientId", "OwnerId"][..],
                _ => &[][..],
            };
            report.missing_columns = expected
                .iter()
                .filter(|name| !names.contains(**name))
                .map(|s| s.to_string())
                .collect();
        }
        known_tables.insert(table.to_string(), report);
    }

    if has_column(&known_tables, "PatientOwner", "OwnerPhone") {
        known_tables
            .get_mut("PatientOwner")
            .expect("known table inserted")
            .phone = Some(inspect_phones(&connection)?);
    }
    for (table, column) in [
        ("Patient", "BirthDate"),
        ("Patient", "RegistrationDate"),
        ("ToDo", "ToDoDate"),
    ] {
        if has_column(&known_tables, table, column) {
            let counts = inspect_dates(&connection, table, column)?;
            known_tables
                .get_mut(table)
                .expect("known table inserted")
                .dates
                .insert(column.to_string(), counts);
        }
    }
    for column in ["PatientType", "PatientGender"] {
        if has_column(&known_tables, "Patient", column) {
            let values = inspect_combos(&connection, column)?;
            known_tables
                .get_mut("Patient")
                .expect("known table inserted")
                .combo_values
                .insert(column.to_string(), values);
        }
    }
    if has_column(&known_tables, "Patient", "OwnerId")
        && has_column(&known_tables, "PatientOwner", "Id")
    {
        known_tables
            .get_mut("Patient")
            .expect("known table inserted")
            .orphan_patients = Some(orphan_count(
            &connection,
            "Patient",
            "OwnerId",
            "PatientOwner",
            "Id",
        )?);
    }
    if has_column(&known_tables, "ToDo", "PatientId")
        && has_column(&known_tables, "Patient", "Id")
        && has_column(&known_tables, "ToDo", "OwnerId")
        && has_column(&known_tables, "PatientOwner", "Id")
    {
        known_tables
            .get_mut("ToDo")
            .expect("known table inserted")
            .orphan_todos = Some(orphan_todo_count(&connection)?);
    }
    if has_column(&known_tables, "PatientOwner", "OwnerPhone") {
        known_tables
            .get_mut("PatientOwner")
            .expect("known table inserted")
            .duplicate_looking_owners = Some(duplicate_owner_count(&connection)?);
    }

    Ok(InspectReport {
        user_version,
        page_count,
        tables,
        known_tables,
    })
}

fn immutable_uri(path: &Path) -> Result<String, LegacyError> {
    let path = path
        .to_str()
        .ok_or(LegacyError::InvalidPath)?
        .replace('\\', "/");
    let encoded = path
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect::<String>();
    Ok(format!("file:{encoded}?mode=ro&immutable=1"))
}

fn table_columns(connection: &Connection, table: &str) -> Result<Vec<ColumnReport>, LegacyError> {
    let sql = format!("PRAGMA table_info({})", quote_identifier(table));
    let mut statement = connection.prepare(&sql)?;
    let columns = statement
        .query_map([], |row| {
            Ok(ColumnReport {
                name: row.get(1)?,
                declared_type: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(columns)
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn has_column(reports: &BTreeMap<String, KnownTableReport>, table: &str, column: &str) -> bool {
    reports.get(table).is_some_and(|report| {
        report.present
            && !report
                .missing_columns
                .iter()
                .any(|missing| missing == column)
    })
}

fn inspect_phones(connection: &Connection) -> Result<PhoneReport, LegacyError> {
    let sql = "SELECT typeof(\"OwnerPhone\"), CAST(\"OwnerPhone\" AS TEXT) FROM \"PatientOwner\"";
    let mut statement = connection.prepare(sql)?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
    })?;
    let mut storage_classes = BTreeMap::new();
    let mut digit_length_buckets: BTreeMap<String, i64> = ["0", "1-9", "10", "11", "12+"]
        .into_iter()
        .map(|s| (s.to_string(), 0))
        .collect();
    let (mut ten_start_five, mut non_digits) = (0, 0);
    for row in rows {
        let (storage, value) = row?;
        *storage_classes.entry(storage).or_insert(0) += 1;
        let value = value.unwrap_or_default();
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        if value.chars().any(|c| !c.is_ascii_digit()) {
            non_digits += 1;
        }
        let bucket = match digits.len() {
            0 => "0",
            1..=9 => "1-9",
            10 => "10",
            11 => "11",
            _ => "12+",
        };
        *digit_length_buckets
            .get_mut(bucket)
            .expect("bucket initialized") += 1;
        if digits.len() == 10 && digits.starts_with('5') {
            ten_start_five += 1;
        }
    }
    Ok(PhoneReport {
        storage_classes,
        digit_length_buckets,
        ten_digit_starting_with_five: ten_start_five,
        containing_non_digits: non_digits,
    })
}

fn inspect_dates(
    connection: &Connection,
    table: &str,
    column: &str,
) -> Result<BTreeMap<String, i64>, LegacyError> {
    let sql = format!(
        "SELECT CAST({} AS TEXT) FROM {}",
        quote_identifier(column),
        quote_identifier(table)
    );
    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map([], |row| row.get::<_, Option<String>>(0))?;
    let mut counts: BTreeMap<String, i64> = DATE_CATEGORIES
        .into_iter()
        .map(|s| (s.to_string(), 0))
        .collect();
    for value in rows {
        let category = date_category(value?.as_deref().unwrap_or(""));
        *counts.get_mut(category).expect("date category initialized") += 1;
    }
    Ok(counts)
}

fn date_category(value: &str) -> &'static str {
    let value = value.trim();
    if value.is_empty() {
        return "empty";
    }
    if value.len() == 10
        && value.as_bytes().get(2) == Some(&b'-')
        && value.as_bytes().get(5) == Some(&b'-')
    {
        return "dd-MM-yyyy";
    }
    if value.len() == 10
        && value.as_bytes().get(2) == Some(&b'.')
        && value.as_bytes().get(5) == Some(&b'.')
    {
        return "dd.MM.yyyy";
    }
    if value.len() == 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
    {
        return "yyyy-MM-dd";
    }
    if value.len() > 10
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(7) == Some(&b'-')
        && (value.contains('T') || value.contains(' '))
    {
        return "iso-datetime";
    }
    let lower = value.to_lowercase();
    if [
        "ocak", "şubat", "mart", "nisan", "mayıs", "haziran", "temmuz", "ağustos", "eylül", "ekim",
        "kasım", "aralık",
    ]
    .iter()
    .any(|month| lower.contains(month))
    {
        return "long-tr";
    }
    if [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ]
    .iter()
    .any(|month| lower.contains(month))
    {
        return "long-en";
    }
    "other"
}

fn inspect_combos(
    connection: &Connection,
    column: &str,
) -> Result<BTreeMap<String, i64>, LegacyError> {
    let known: &[&str] = if column == "PatientType" {
        &["Kedi", "Köpek", "Diğer"]
    } else {
        &["Erkek", "Dişi"]
    };
    let sql = format!(
        "SELECT CAST({} AS TEXT), COUNT(*) FROM \"Patient\" GROUP BY {}",
        quote_identifier(column),
        quote_identifier(column)
    );
    let mut statement = connection.prepare(&sql)?;
    let mut results = BTreeMap::new();
    let mut other = 0;
    for row in statement.query_map([], |row| {
        Ok((row.get::<_, Option<String>>(0)?, row.get::<_, i64>(1)?))
    })? {
        let (value, count) = row?;
        if value.as_deref().is_some_and(|value| known.contains(&value)) {
            *results
                .entry(value.expect("known value present"))
                .or_insert(0) += count;
        } else {
            other += count;
        }
    }
    results.insert("other".to_string(), other);
    Ok(results)
}

fn orphan_count(
    connection: &Connection,
    child: &str,
    child_column: &str,
    parent: &str,
    parent_column: &str,
) -> Result<i64, LegacyError> {
    let sql = format!("SELECT COUNT(*) FROM {child} c LEFT JOIN {parent} p ON c.{} = p.{} WHERE c.{} IS NULL OR p.{} IS NULL",
        quote_identifier(child_column), quote_identifier(parent_column), quote_identifier(child_column), quote_identifier(parent_column));
    Ok(connection.query_row(&sql, [], |row| row.get(0))?)
}

fn orphan_todo_count(connection: &Connection) -> Result<i64, LegacyError> {
    Ok(connection.query_row("SELECT COUNT(*) FROM ToDo t WHERE t.PatientId IS NULL OR NOT EXISTS (SELECT 1 FROM Patient p WHERE p.Id = t.PatientId) OR t.OwnerId IS NULL OR NOT EXISTS (SELECT 1 FROM PatientOwner o WHERE o.Id = t.OwnerId)", [], |row| row.get(0))?)
}

fn duplicate_owner_count(connection: &Connection) -> Result<i64, LegacyError> {
    let mut statement = connection.prepare(
        "SELECT CAST(OwnerPhone AS TEXT) FROM PatientOwner WHERE OwnerPhone IS NOT NULL",
    )?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    let mut counts: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let value = row?;
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            continue;
        }
        let normalized = if digits.len() == 10 && digits.starts_with('5') {
            format!("0{digits}")
        } else if digits.len() == 12 && digits.starts_with("90") {
            format!("0{}", &digits[2..])
        } else {
            digits
        };
        *counts.entry(normalized).or_insert(0) += 1;
    }
    Ok(counts.values().filter(|count| **count > 1).sum())
}
