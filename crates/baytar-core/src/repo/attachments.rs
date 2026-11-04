//! Attachment metadata and bytes are committed together and backed up with SQLite.
use super::{active_patient, clean, invalid, now, required, transaction, RepoError, Result};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const MAX_SIZE_BYTES: usize = 25 * 1024 * 1024;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i64,
    pub patient_id: i64,
    pub visit_id: Option<i64>,
    pub kind: String,
    pub title: Option<String>,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
pub struct NewAttachment {
    pub patient_id: i64,
    pub visit_id: Option<i64>,
    pub kind: String,
    pub title: Option<String>,
    pub file_name: String,
    pub data: Vec<u8>,
}
const COLUMNS: &str = "id, patient_id, visit_id, kind, title, file_name, mime, size_bytes, sha256, created_at, updated_at, deleted_at";
fn row(r: &Row<'_>) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: r.get(0)?,
        patient_id: r.get(1)?,
        visit_id: r.get(2)?,
        kind: r.get(3)?,
        title: r.get(4)?,
        file_name: r.get(5)?,
        mime: r.get(6)?,
        size_bytes: r.get(7)?,
        sha256: r.get(8)?,
        created_at: r.get(9)?,
        updated_at: r.get(10)?,
        deleted_at: r.get(11)?,
    })
}
/// File type is inferred from the extension; bytes are never executed or parsed here.
pub fn mime_for_name(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "rtf" => "application/rtf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        _ => "application/octet-stream",
    }
}
pub fn get(conn: &Connection, id: i64) -> Result<Attachment> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM attachments WHERE id = ?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or(RepoError::NotFound)
}
pub fn list(
    conn: &Connection,
    patient_id: i64,
    visit_id: Option<i64>,
    include_deleted: bool,
) -> Result<Vec<Attachment>> {
    active_patient(conn, patient_id)?;
    let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM attachments WHERE patient_id = ?1 AND (?2 IS NULL OR visit_id = ?2) AND (?3 OR deleted_at IS NULL) ORDER BY created_at DESC, id DESC"))?;
    let rows = stmt
        .query_map(params![patient_id, visit_id, include_deleted], row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(rows)
}
pub fn create(conn: &Connection, input: NewAttachment) -> Result<Attachment> {
    required("fileName", &input.file_name)?;
    if input.file_name.contains(['/', '\\', '\0', ':'])
        || input.file_name == "."
        || input.file_name == ".."
    {
        return Err(invalid("fileName", "expected a file name without a path"));
    }
    if input.data.is_empty() || input.data.len() > MAX_SIZE_BYTES {
        return Err(invalid(
            "sizeBytes",
            "file must contain between 1 byte and 25 MB",
        ));
    }
    if !["photo", "lab", "imaging", "document", "other"].contains(&input.kind.as_str()) {
        return Err(invalid("kind", "unknown attachment kind"));
    }
    let mime = mime_for_name(&input.file_name);
    if input.kind == "photo" && !mime.starts_with("image/") {
        return Err(invalid("kind", "a photo must be an image"));
    }
    let tx = transaction(conn)?;
    active_patient(&tx, input.patient_id)?;
    if let Some(id) = input.visit_id {
        let visit = super::visits::get(&tx, id)?;
        if visit.patient_id != input.patient_id {
            return Err(invalid("visitId", "visit must belong to patient"));
        }
        if visit.deleted_at.is_some() {
            return Err(RepoError::NotFound);
        }
    }
    let hash = format!("{:x}", Sha256::digest(&input.data));
    tx.execute("INSERT INTO attachments (patient_id, visit_id, kind, title, file_name, mime, size_bytes, sha256, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)", params![input.patient_id, input.visit_id, input.kind, clean(input.title), input.file_name, mime, input.data.len() as i64, hash, now()])?;
    let id = tx.last_insert_rowid();
    tx.execute(
        "INSERT INTO attachment_data (attachment_id, data) VALUES (?1, ?2)",
        params![id, input.data],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn read_bytes(conn: &Connection, id: i64) -> Result<Vec<u8>> {
    let attachment = get(conn, id)?;
    active_patient(conn, attachment.patient_id)?;
    if attachment.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    Ok(conn.query_row(
        "SELECT data FROM attachment_data WHERE attachment_id = ?1",
        [id],
        |r| r.get(0),
    )?)
}
pub fn rename(conn: &Connection, id: i64, title: Option<String>) -> Result<Attachment> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    active_patient(&tx, old.patient_id)?;
    if old.deleted_at.is_some() {
        return Err(RepoError::NotFound);
    }
    tx.execute(
        "UPDATE attachments SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![clean(title), now(), id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}
pub fn soft_delete(conn: &Connection, id: i64) -> Result<Attachment> {
    set_deleted(conn, id, true)
}
pub fn restore(conn: &Connection, id: i64) -> Result<Attachment> {
    set_deleted(conn, id, false)
}
fn set_deleted(conn: &Connection, id: i64, deleted: bool) -> Result<Attachment> {
    let tx = transaction(conn)?;
    let old = get(&tx, id)?;
    active_patient(&tx, old.patient_id)?;
    if old.deleted_at.is_some() == deleted {
        return Err(RepoError::Conflict);
    }
    let stamp = now();
    tx.execute(
        "UPDATE attachments SET deleted_at = ?1, updated_at = ?2 WHERE id = ?3",
        params![deleted.then_some(&stamp), stamp, id],
    )?;
    let result = get(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn patient(conn: &Connection) -> i64 {
        conn.execute("INSERT INTO owners (full_name, search_text) VALUES ('Synthetic owner', 'synthetic owner')", []).unwrap();
        conn.execute("INSERT INTO patients (owner_id, name, species, search_text) VALUES (1, 'Synthetic patient', 'cat', 'synthetic patient')", []).unwrap();
        conn.last_insert_rowid()
    }
    fn input(patient_id: i64) -> NewAttachment {
        NewAttachment {
            patient_id,
            visit_id: None,
            kind: "document".into(),
            title: None,
            file_name: "test.PDF".into(),
            data: b"synthetic bytes".to_vec(),
        }
    }
    #[test]
    fn atomic_bytes_lifecycle_and_validation() {
        let db = crate::db::Database::open_in_memory().unwrap();
        let conn = db.conn();
        let id = patient(conn);
        let first = create(conn, input(id)).unwrap();
        assert_eq!(first.mime, "application/pdf");
        assert_eq!(first.size_bytes, 15);
        assert_eq!(
            first.sha256,
            format!("{:x}", Sha256::digest(b"synthetic bytes"))
        );
        assert_eq!(read_bytes(conn, first.id).unwrap(), b"synthetic bytes");
        let next = rename(conn, first.id, Some(" Lab results ".into())).unwrap();
        assert_eq!(next.title.as_deref(), Some("Lab results"));
        assert_ne!(first.updated_at, next.updated_at);
        soft_delete(conn, first.id).unwrap();
        assert!(read_bytes(conn, first.id).is_err());
        assert!(list(conn, id, None, false).unwrap().is_empty());
        assert_eq!(list(conn, id, None, true).unwrap().len(), 1);
        restore(conn, first.id).unwrap();
        assert_eq!(read_bytes(conn, first.id).unwrap(), b"synthetic bytes");
        let mut bad = input(id);
        bad.data = vec![0; MAX_SIZE_BYTES + 1];
        assert!(create(conn, bad).is_err());
        let mut bad = input(id);
        bad.data.clear();
        assert!(create(conn, bad).is_err());
        let mut bad = input(id);
        bad.file_name = "../test.pdf".into();
        assert!(create(conn, bad).is_err());
        let mut bad = input(id);
        bad.kind = "photo".into();
        assert!(create(conn, bad).is_err());
        let mut bad = input(id);
        bad.visit_id = Some(999);
        assert!(create(conn, bad).is_err());
        assert_eq!(list(conn, id, None, true).unwrap().len(), 1);
        assert_eq!(
            conn.query_row("SELECT count(*) FROM attachment_data", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }
    #[test]
    fn attachment_cannot_link_to_another_patients_visit() {
        let db = crate::db::Database::open_in_memory().unwrap();
        let conn = db.conn();
        let id = patient(conn);
        conn.execute("INSERT INTO patients (owner_id, name, species, search_text) VALUES (1, 'Other', 'dog', 'other')", []).unwrap();
        let visit = super::super::visits::create(
            conn,
            super::super::visits::NewVisit {
                patient_id: 2,
                visit_on: "2026-11-01".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let mut bad = input(id);
        bad.visit_id = Some(visit.id);
        assert!(matches!(create(conn, bad), Err(RepoError::Invalid { .. })));
        assert!(list(conn, id, None, true).unwrap().is_empty());
    }
}
