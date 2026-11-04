//! Clinical commands; local file IO stays outside the database lock.
use crate::{AppDatabase, CommandError};
use base64::{engine::general_purpose::STANDARD, Engine};
use baytar_core::repo::{
    attachments::{self, Attachment, NewAttachment, MAX_SIZE_BYTES},
    visit_templates::{self, NewVisitTemplate, VisitTemplate, VisitTemplatePatch},
    RepoError,
};
use std::{
    fs,
    io::{Cursor, Read},
    path::Path,
};
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

fn invalid(field: &str, reason: &str) -> CommandError {
    RepoError::Invalid {
        field: field.into(),
        reason: reason.into(),
    }
    .into()
}

#[tauri::command]
pub fn visit_templates_list(
    state: State<'_, AppDatabase>,
    include_deleted: bool,
) -> Result<Vec<VisitTemplate>, CommandError> {
    state.with(|c| visit_templates::list(c, include_deleted))
}
#[tauri::command]
pub fn visit_template_get(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<VisitTemplate, CommandError> {
    state.with(|c| visit_templates::get(c, id))
}
#[tauri::command]
pub fn visit_template_create(
    state: State<'_, AppDatabase>,
    input: NewVisitTemplate,
) -> Result<VisitTemplate, CommandError> {
    state.with(|c| visit_templates::create(c, input))
}
#[tauri::command]
pub fn visit_template_update(
    state: State<'_, AppDatabase>,
    id: i64,
    patch: VisitTemplatePatch,
) -> Result<VisitTemplate, CommandError> {
    state.with(|c| visit_templates::update(c, id, patch))
}
#[tauri::command]
pub fn visit_template_delete(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<VisitTemplate, CommandError> {
    state.with(|c| visit_templates::soft_delete(c, id))
}
#[tauri::command]
pub fn visit_template_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<VisitTemplate, CommandError> {
    state.with(|c| visit_templates::restore(c, id))
}

#[tauri::command]
pub fn attachments_list(
    state: State<'_, AppDatabase>,
    patient_id: i64,
    visit_id: Option<i64>,
    include_deleted: bool,
) -> Result<Vec<Attachment>, CommandError> {
    state.with(|c| attachments::list(c, patient_id, visit_id, include_deleted))
}
#[tauri::command]
pub async fn attachment_add_from_path(
    state: State<'_, AppDatabase>,
    patient_id: i64,
    visit_id: Option<i64>,
    kind: String,
    path: String,
) -> Result<Attachment, CommandError> {
    let path = Path::new(&path);
    let mut file = fs::File::open(path).map_err(|_| CommandError::internal())?;
    if !file
        .metadata()
        .map_err(|_| CommandError::internal())?
        .is_file()
    {
        return Err(invalid("path", "expected a file"));
    }
    // A bounded read also handles files which grow between metadata and reading.
    let mut data = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_SIZE_BYTES as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|_| CommandError::internal())?;
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| invalid("path", "invalid file name"))?
        .to_owned();
    state.with(|c| {
        attachments::create(
            c,
            NewAttachment {
                patient_id,
                visit_id,
                kind,
                title: None,
                file_name,
                data,
            },
        )
    })
}
#[tauri::command]
pub fn attachment_rename(
    state: State<'_, AppDatabase>,
    id: i64,
    title: Option<String>,
) -> Result<Attachment, CommandError> {
    state.with(|c| attachments::rename(c, id, title))
}
#[tauri::command]
pub fn attachment_delete(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Attachment, CommandError> {
    state.with(|c| attachments::soft_delete(c, id))
}
#[tauri::command]
pub fn attachment_restore(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Attachment, CommandError> {
    state.with(|c| attachments::restore(c, id))
}
#[tauri::command]
pub fn attachment_read_bytes(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Vec<u8>, CommandError> {
    state.with(|c| attachments::read_bytes(c, id))
}

fn thumbnail(data: Vec<u8>) -> Result<String, CommandError> {
    let mut reader = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(|_| CommandError::internal())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(16384);
    limits.max_image_height = Some(16384);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|_| invalid("image", "image could not be decoded"))?
        .thumbnail(320, 320);
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|_| CommandError::internal())?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(output.into_inner())
    ))
}
#[tauri::command]
pub async fn attachment_thumbnail(
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<Option<String>, CommandError> {
    let mime = state.with(|c| attachments::get(c, id))?.mime;
    if !mime.starts_with("image/") {
        return Ok(None);
    }
    let bytes = state.with(|c| attachments::read_bytes(c, id))?;
    thumbnail(bytes).map(Some)
}
#[tauri::command]
pub async fn patient_photo(
    state: State<'_, AppDatabase>,
    patient_id: i64,
) -> Result<Option<String>, CommandError> {
    let photo = state.with(|c| {
        Ok(attachments::list(c, patient_id, None, false)?
            .into_iter()
            .find(|a| a.kind == "photo"))
    })?;
    match photo {
        Some(photo) => thumbnail(state.with(|c| attachments::read_bytes(c, photo.id))?).map(Some),
        None => Ok(None),
    }
}
#[tauri::command]
pub async fn attachment_open(
    app: tauri::AppHandle,
    state: State<'_, AppDatabase>,
    id: i64,
) -> Result<(), CommandError> {
    let attachment = state.with(|c| attachments::get(c, id))?;
    if attachments::mime_for_name(&attachment.file_name) == "application/octet-stream" {
        return Err(invalid(
            "mime",
            "unsupported file type for external opening",
        ));
    }
    let bytes = state.with(|c| attachments::read_bytes(c, id))?;
    let dir = app
        .path()
        .app_cache_dir()
        .map_err(|_| CommandError::internal())?
        .join("attachments");
    fs::create_dir_all(&dir).map_err(|_| CommandError::internal())?;
    // New directory per open prevents collisions and avoids overwriting another process's file.
    let dir = dir.join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&dir).map_err(|_| CommandError::internal())?;
    let path = dir.join(&attachment.file_name);
    fs::write(&path, bytes).map_err(|_| CommandError::internal())?;
    app.opener()
        .open_path(path.to_string_lossy(), None::<&str>)
        .map_err(|_| CommandError::internal())
}
#[tauri::command]
pub async fn attachment_export(
    state: State<'_, AppDatabase>,
    id: i64,
    path: String,
) -> Result<(), CommandError> {
    let bytes = state.with(|c| attachments::read_bytes(c, id))?;
    // Never overwrite a database or one of its sidecars, including the active clinic database.
    let extension = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ["db", "sqlite", "sqlite3", "db-wal", "db-shm"].contains(&extension.as_str()) {
        return Err(invalid("path", "cannot export over a database"));
    }
    fs::write(path, bytes).map_err(|_| CommandError::internal())
}

#[tauri::command]
pub fn patient_mark_deceased(
    state: State<'_, AppDatabase>,
    id: i64,
    date: String,
) -> Result<baytar_core::model::Patient, CommandError> {
    state.with(|c| baytar_core::repo::patients::mark_deceased(c, id, &date))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn image_thumbnail_is_bounded_and_malformed_images_fail() {
        let image = image::DynamicImage::new_rgba8(640, 480);
        let mut data = Cursor::new(Vec::new());
        image.write_to(&mut data, image::ImageFormat::Png).unwrap();
        let url = thumbnail(data.into_inner()).unwrap();
        let bytes = STANDARD
            .decode(url.strip_prefix("data:image/png;base64,").unwrap())
            .unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (320, 240));
        assert!(thumbnail(b"invalid".to_vec()).is_err());
    }
}
