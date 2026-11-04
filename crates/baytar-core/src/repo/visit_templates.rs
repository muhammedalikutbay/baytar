//! User-authored SOAP templates. No templates are seeded.
use super::{clean, now, required, transaction, RepoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitTemplate {
    pub id: i64,
    pub name: String,
    pub reason: Option<String>,
    pub findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVisitTemplate {
    pub name: String,
    pub reason: Option<String>,
    pub findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitTemplatePatch {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "super::nullable")]
    pub reason: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::nullable")]
    pub findings: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::nullable")]
    pub diagnosis: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::nullable")]
    pub treatment: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::nullable")]
    pub notes: Option<Option<String>>,
}

const COLUMNS: &str =
    "id, name, reason, findings, diagnosis, treatment, notes, created_at, updated_at, deleted_at";
fn row(r: &Row<'_>) -> rusqlite::Result<VisitTemplate> {
    Ok(VisitTemplate {
        id: r.get(0)?,
        name: r.get(1)?,
        reason: r.get(2)?,
        findings: r.get(3)?,
        diagnosis: r.get(4)?,
        treatment: r.get(5)?,
        notes: r.get(6)?,
        created_at: r.get(7)?,
        updated_at: r.get(8)?,
        deleted_at: r.get(9)?,
    })
}
pub fn get(conn: &Connection, id: i64) -> Result<VisitTemplate> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM visit_templates WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(conn: &Connection, include_deleted: bool) -> Result<Vec<VisitTemplate>> {
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM visit_templates WHERE (?1 OR deleted_at IS NULL) ORDER BY lower(name), id"))?;
    let rows = stmt
        .query_map([include_deleted], row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn create(conn: &Connection, input: NewVisitTemplate) -> Result<VisitTemplate> {
    required("name", &input.name)?;
    let tx = transaction(conn)?;
    tx.execute("INSERT INTO visit_templates (name, reason, findings, diagnosis, treatment, notes, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)", params![input.name.trim(), clean(input.reason), clean(input.findings), clean(input.diagnosis), clean(input.treatment), clean(input.notes), now()])?;
    let result = get(&tx, tx.last_insert_rowid())?;
    tx.commit()?;
    Ok(result)
}
pub fn update(conn: &Connection, id: i64, patch: VisitTemplatePatch) -> Result<VisitTemplate> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    let name = patch.name.unwrap_or(old.name);
    required("name", &name)?;
    tx.execute("UPDATE visit_templates SET name = ?1, reason = ?2, findings = ?3, diagnosis = ?4, treatment = ?5, notes = ?6, updated_at = ?7 WHERE id = ?8", params![name.trim(), clean(patch.reason.unwrap_or(old.reason)), clean(patch.findings.unwrap_or(old.findings)), clean(patch.diagnosis.unwrap_or(old.diagnosis)), clean(patch.treatment.unwrap_or(old.treatment)), clean(patch.notes.unwrap_or(old.notes)), now(), id])?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<VisitTemplate> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<VisitTemplate> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<VisitTemplate> {
    let tx = transaction(conn)?;
    if get(&tx, id)?.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE visit_templates SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![deleted.then_some(&stamp), stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn template_lifecycle_and_nullable_patch() {
        let db = crate::db::Database::open_in_memory().unwrap();
        let conn = db.conn();
        assert!(list(conn, false).unwrap().is_empty());
        assert!(create(conn, NewVisitTemplate::default()).is_err());
        let first = create(
            conn,
            NewVisitTemplate {
                name: " Exam ".into(),
                reason: Some("Concern".into()),
                findings: Some("Observed".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let patch = serde_json::from_str(r#"{"reason":null,"name":"Follow-up"}"#).unwrap();
        let next = update(conn, first.id, patch).unwrap();
        assert_eq!(next.name, "Follow-up");
        assert_eq!(next.reason, None);
        assert_eq!(next.findings.as_deref(), Some("Observed"));
        assert_ne!(first.updated_at, next.updated_at);
        soft_delete(conn, first.id).unwrap();
        assert!(list(conn, false).unwrap().is_empty());
        assert_eq!(list(conn, true).unwrap().len(), 1);
        assert!(update(conn, first.id, VisitTemplatePatch::default()).is_err());
        restore(conn, first.id).unwrap();
        assert_eq!(list(conn, false).unwrap().len(), 1);
        assert!(matches!(restore(conn, first.id), Err(RepoError::Conflict)));
    }
}
