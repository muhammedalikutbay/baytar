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
            let mut patients = patients::list_by_owner(conn, id)?;
            // Keep siblings reachable; prioritize matching pets for pet-only searches.
            let owner_matches = owner.search_text.contains(&query)
                || (phone
                    && [owner.phone.as_deref(), owner.phone_alt.as_deref()]
                        .into_iter()
                        .flatten()
                        .any(|value| value.contains(&query)));
            if !owner_matches {
                patients.sort_by_key(|p| !p.search_text.contains(&query));
            }
            Ok(SearchHit { owner, patients })
        })
        .collect()
}

/// A bounded owner directory; total counts all active owners, independently of the filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Directory {
    pub total: u64,
    pub owners: Vec<SearchHit>,
}
/// Folded literal matching includes phone digits and pet names; matching owners retain every active pet.
pub fn directory(conn: &Connection, query: &str, limit: u32) -> Result<Directory> {
    let query = fold(query);
    let digits: String = query.chars().filter(char::is_ascii_digit).collect();
    let phone = !digits.is_empty()
        && query
            .chars()
            .all(|c| c.is_ascii_digit() || "+ ()-./".contains(c));
    let total = conn.query_row(
        "SELECT count(*) FROM owners WHERE deleted_at IS NULL",
        [],
        |r| r.get(0),
    )?;
    let mut stmt = conn.prepare("SELECT o.id FROM owners o WHERE o.deleted_at IS NULL AND (
        ?1 = '' OR instr(o.search_text, ?1) > 0 OR (?2 AND (
        instr(coalesce(o.phone,''), ?3) > 0 OR instr(coalesce(o.phone_alt,''), ?3) > 0)) OR
        EXISTS(SELECT 1 FROM patients p WHERE p.owner_id = o.id AND p.deleted_at IS NULL AND instr(p.search_text, ?1) > 0))
        ORDER BY o.search_text, o.id LIMIT ?4")?;
    let ids = stmt
        .query_map(params![query, phone, digits, limit], |r| r.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let owners = ids
        .into_iter()
        .map(|id| {
            let owner = owners::get(conn, id)?;
            let mut patients = patients::list_by_owner(conn, id)?;
            patients.sort_by(|a, b| fold(&a.name).cmp(&fold(&b.name)).then(a.id.cmp(&b.id)));
            Ok(SearchHit { owner, patients })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Directory { total, owners })
}

#[cfg(test)]
mod directory_tests {
    use super::*;
    use crate::{
        db::Database,
        repo::{owners::NewOwner, patients::NewPatient},
    };
    fn owner(conn: &Connection, name: &str) -> Owner {
        owners::create(
            conn,
            NewOwner {
                full_name: name.into(),
                phone: Some("+905321112233".into()),
                ..Default::default()
            },
        )
        .unwrap()
    }
    fn pet(conn: &Connection, owner: i64, name: &str) -> Patient {
        patients::create(
            conn,
            NewPatient {
                owner_id: owner,
                name: name.into(),
                species: "cat".into(),
                ..Default::default()
            },
        )
        .unwrap()
    }
    #[test]
    fn owner_search_keeps_all_active_pets() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let o = owner(c, "Synthetic Owner");
        let a = pet(c, o.id, "Alpha");
        let b = pet(c, o.id, "Beta");
        let deleted = pet(c, o.id, "Deleted");
        patients::soft_delete(c, deleted.id).unwrap();
        let hits = search(c, "synthetic", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(
            hits[0].patients.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![a.id, b.id]
        );
    }
    #[test]
    fn pet_search_keeps_siblings_after_matching_pets() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let o = owner(c, "Synthetic Owner");
        let sibling = pet(c, o.id, "Alpha");
        let matched = pet(c, o.id, "Zebra");
        let deleted = pet(c, o.id, "Zebra deleted");
        patients::soft_delete(c, deleted.id).unwrap();
        let hits = search(c, "zebra", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(
            hits[0].patients.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![matched.id, sibling.id]
        );
        assert!(search(c, "deleted", 10).unwrap().is_empty());
    }
    #[test]
    fn folded_matches_keep_all_pets_and_exclude_deleted_records() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        let o = owner(c, "İpek Işık Çığ Öşü");
        let p = pet(c, o.id, "Şimşek");
        let sibling = pet(c, o.id, "Pamuk");
        let deleted = pet(c, o.id, "Deleted pet");
        patients::soft_delete(c, deleted.id).unwrap();
        let gone = owner(c, "Deleted owner");
        pet(c, gone.id, "Hidden pet");
        owners::soft_delete(c, gone.id).unwrap();
        for query in [
            "IPEK",
            "isik",
            "cig osu",
            "5321112233",
            "532 111 22 33",
            "simsek",
        ] {
            let result = directory(c, query, 10).unwrap();
            assert_eq!(result.total, 1);
            assert_eq!(result.owners.len(), 1);
            assert_eq!(
                result.owners[0]
                    .patients
                    .iter()
                    .map(|p| p.id)
                    .collect::<Vec<_>>(),
                vec![sibling.id, p.id]
            );
        }
        for query in ["Deleted", "Hidden", "%", "_"] {
            assert!(directory(c, query, 10).unwrap().owners.is_empty());
        }
    }
    #[test]
    fn empty_query_is_ordered_and_limit_does_not_change_total() {
        let db = Database::open_in_memory().unwrap();
        let c = db.conn();
        owner(c, "Zeynep");
        owner(c, "Ayşe");
        owner(c, "Bora");
        let result = directory(c, "", 2).unwrap();
        assert_eq!(result.total, 3);
        assert_eq!(
            result
                .owners
                .iter()
                .map(|h| h.owner.full_name.as_str())
                .collect::<Vec<_>>(),
            vec!["Ayşe", "Bora"]
        );
        assert!(directory(c, "", 0).unwrap().owners.is_empty());
        assert_eq!(directory(c, "Bora", 1).unwrap().total, 3);
        assert_eq!(directory(c, "", 10).unwrap().owners.len(), 3);
    }
}
