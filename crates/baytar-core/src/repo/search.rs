use super::{owners, patients, Result};
use crate::{
    model::{Owner, Patient},
    text::fold,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub owner: Owner,
    pub patients: Vec<Patient>,
}
/// Literal folded substring matching; SQL wildcards have no special meaning.
pub fn search(conn: &Connection, query: &str, limit: u32) -> Result<Vec<SearchHit>> {
    let query = fold(query);
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let phone = query.len() >= 3 && query.chars().all(|c| c.is_ascii_digit());
    let mut stmt = conn.prepare("SELECT o.id FROM owners o WHERE o.deleted_at IS NULL AND (
        instr(o.search_text, ?1) > 0 OR (?2 AND (instr(coalesce(o.phone,''), ?1) > 0 OR instr(coalesce(o.phone_alt,''), ?1) > 0)) OR
        EXISTS(SELECT 1 FROM patients p WHERE p.owner_id = o.id AND p.deleted_at IS NULL AND instr(p.search_text, ?1) > 0))
        ORDER BY (substr(o.search_text, 1, length(?1)) = ?1 OR EXISTS(SELECT 1 FROM patients p WHERE p.owner_id = o.id AND p.deleted_at IS NULL AND substr(p.search_text, 1, length(?1)) = ?1)) DESC,
        o.search_text, o.id LIMIT ?3")?;
    let ids = stmt
        .query_map(params![query, phone, limit], |r| r.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    ids.into_iter()
        .map(|id| {
            let owner = owners::get(conn, id)?;
            let patients = patients::list_by_owner(conn, id)?
                .into_iter()
                .filter(|p| p.search_text.contains(&query))
                .collect();
            Ok(SearchHit { owner, patients })
        })
        .collect()
}
