//! Serializable preview and report types, plus aggregate-only text formatting.
//!
//! The default formatting never prints names, phones, notes or raw legacy values: the
//! orchestrator runs the CLI on real data. Raw values appear only when the caller asks
//! for them (`show_values`), and `without_values` strips them from the JSON form.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Serialize;

/// Row counts per known legacy table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub owners: u64,
    pub patients: u64,
    pub todos: u64,
}

impl Counts {
    pub fn total(&self) -> u64 {
        self.owners + self.patients + self.todos
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportIssueKind {
    PhoneUnparsed,
    DateUnparsed,
    SpeciesUnrecognized,
    SexUnrecognized,
    MissingName,
    UnusableId,
    OrphanPatient,
    OrphanTodo,
    DanglingPatientLink,
    DanglingOwnerLink,
    OwnerMismatch,
    DuplicateOwner,
}

impl ImportIssueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PhoneUnparsed => "phoneUnparsed",
            Self::DateUnparsed => "dateUnparsed",
            Self::SpeciesUnrecognized => "speciesUnrecognized",
            Self::SexUnrecognized => "sexUnrecognized",
            Self::MissingName => "missingName",
            Self::UnusableId => "unusableId",
            Self::OrphanPatient => "orphanPatient",
            Self::OrphanTodo => "orphanTodo",
            Self::DanglingPatientLink => "danglingPatientLink",
            Self::DanglingOwnerLink => "danglingOwnerLink",
            Self::OwnerMismatch => "ownerMismatch",
            Self::DuplicateOwner => "duplicateOwner",
        }
    }
}

/// One row-level finding for the UI. `raw` holds the original legacy text when there is one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportIssue {
    pub table: String,
    pub source_id: i64,
    pub field: String,
    pub kind: ImportIssueKind,
    pub raw: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedTable {
    pub name: String,
    pub known: bool,
    pub row_count: u64,
    pub columns: Vec<String>,
}

/// What `PRAGMA table_info` found in the copy of the legacy file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedSchema {
    pub tables: Vec<DetectedTable>,
    pub unknown_tables: Vec<String>,
    pub missing_tables: Vec<String>,
    /// `Table.Column` entries the importer does not know.
    pub unknown_columns: Vec<String>,
    /// `Table.Column` entries of known tables that are absent (values become empty).
    pub missing_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source_name: String,
    pub source_sha256: String,
    pub schema: DetectedSchema,
    /// Rows in the legacy file.
    pub found: Counts,
    /// Rows already present in `legacy_map` for this file; they are skipped.
    pub already_imported: Counts,
    /// Rows an import would insert now.
    pub to_import: Counts,
    pub phones_adjusted: u64,
    pub phones_unparsed: u64,
    /// Unparsed (or, for to-dos, missing) dates per `Table.Field`.
    pub dates_unparsed: BTreeMap<String, u64>,
    pub species_mapped_to_other: u64,
    pub species_missing: u64,
    pub sex_mapped_to_unknown: u64,
    pub orphan_patients: u64,
    pub orphan_todos: u64,
    /// Owners that match an existing Baytar owner by folded name and normalized phone.
    pub duplicate_owners: u64,
    /// To-dos that would be marked done (date before the import day) or planned.
    pub todos_done: u64,
    pub todos_planned: u64,
    /// Whether the generated `Sahibi bilinmeyen` owner has to be created.
    pub orphan_owner_needed: bool,
    pub issues: Vec<ImportIssue>,
}

impl ImportPreview {
    /// A copy without raw legacy values, safe for aggregate-only output.
    pub fn without_values(&self) -> ImportPreview {
        let mut copy = self.clone();
        copy.source_name = String::new();
        for issue in &mut copy.issues {
            issue.raw = None;
        }
        copy
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    #[serde(flatten)]
    pub preview: ImportPreview,
    pub inserted: Counts,
    pub skipped: Counts,
    pub run_id: i64,
    pub orphan_owner_created: bool,
    /// File name of the `pre-import` backup taken before any write.
    pub backup_file: String,
}

impl ImportReport {
    pub fn without_values(&self) -> ImportReport {
        let mut copy = self.clone();
        copy.preview = self.preview.without_values();
        copy
    }
}

/// Human-readable preview. Aggregates only unless `show_values` is set.
pub fn format_preview(preview: &ImportPreview, show_values: bool) -> String {
    let mut out = String::new();
    write_preview(&mut out, preview, show_values);
    out
}

/// Human-readable report. Aggregates only unless `show_values` is set.
pub fn format_report(report: &ImportReport, show_values: bool) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Import completed (run {})", report.run_id);
    let _ = writeln!(out, "Pre-import backup: {}", report.backup_file);
    let _ = writeln!(out, "{}", counts_line("Inserted", &report.inserted));
    let _ = writeln!(out, "{}", counts_line("Skipped", &report.skipped));
    let _ = writeln!(
        out,
        "Generated unknown owner created: {}",
        yes_no(report.orphan_owner_created)
    );
    out.push('\n');
    write_preview(&mut out, &report.preview, show_values);
    out
}

fn write_preview(out: &mut String, pv: &ImportPreview, show_values: bool) {
    let _ = writeln!(out, "Source SHA-256: {}", pv.source_sha256);
    if show_values {
        let _ = writeln!(out, "Source file: {}", pv.source_name);
    }
    let _ = writeln!(out, "{}", counts_line("Found", &pv.found));
    let _ = writeln!(
        out,
        "{}",
        counts_line("Already imported", &pv.already_imported)
    );
    let _ = writeln!(out, "{}", counts_line("To import", &pv.to_import));
    let _ = writeln!(
        out,
        "Phones: {} adjusted, {} unparsed",
        pv.phones_adjusted, pv.phones_unparsed
    );
    if pv.dates_unparsed.is_empty() {
        let _ = writeln!(out, "Dates unparsed: none");
    } else {
        for (field, count) in &pv.dates_unparsed {
            let _ = writeln!(out, "Dates unparsed in {field}: {count}");
        }
    }
    let _ = writeln!(
        out,
        "Species: {} free text mapped to other, {} missing",
        pv.species_mapped_to_other, pv.species_missing
    );
    let _ = writeln!(out, "Sex: {} mapped to unknown", pv.sex_mapped_to_unknown);
    let _ = writeln!(
        out,
        "Orphans: {} patients, {} to-dos (generated owner needed: {})",
        pv.orphan_patients,
        pv.orphan_todos,
        yes_no(pv.orphan_owner_needed)
    );
    let _ = writeln!(
        out,
        "Owners that look like existing Baytar owners: {}",
        pv.duplicate_owners
    );
    let _ = writeln!(
        out,
        "To-dos: {} would be done (past), {} planned",
        pv.todos_done, pv.todos_planned
    );
    let schema = &pv.schema;
    let _ = writeln!(
        out,
        "Missing tables: {}",
        list_or_none(&schema.missing_tables)
    );
    let _ = writeln!(
        out,
        "Missing columns: {}",
        list_or_none(&schema.missing_columns)
    );
    let _ = writeln!(
        out,
        "Unknown tables: {}",
        list_or_none(&schema.unknown_tables)
    );
    let _ = writeln!(
        out,
        "Unknown columns: {}",
        list_or_none(&schema.unknown_columns)
    );

    if show_values {
        let _ = writeln!(out, "Issues ({}):", pv.issues.len());
        for issue in &pv.issues {
            let raw = issue
                .raw
                .as_deref()
                .map(|raw| format!(" {raw:?}"))
                .unwrap_or_default();
            let _ = writeln!(
                out,
                "  {}#{} {} {}{}",
                issue.table,
                issue.source_id,
                issue.field,
                issue.kind.as_str(),
                raw
            );
        }
    } else {
        let mut by_kind: BTreeMap<&'static str, u64> = BTreeMap::new();
        for issue in &pv.issues {
            *by_kind.entry(issue.kind.as_str()).or_default() += 1;
        }
        let _ = writeln!(
            out,
            "Issues: {} (values hidden; use --show-values)",
            pv.issues.len()
        );
        for (kind, count) in by_kind {
            let _ = writeln!(out, "  {kind}: {count}");
        }
    }
}

fn counts_line(label: &str, counts: &Counts) -> String {
    format!(
        "{label}: {} owners, {} patients, {} to-dos",
        counts.owners, counts.patients, counts.todos
    )
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn list_or_none(items: &[String]) -> String {
    if items.is_empty() {
        "none".to_owned()
    } else {
        items.join(", ")
    }
}
