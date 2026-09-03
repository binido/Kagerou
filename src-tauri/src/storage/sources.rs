use rusqlite::{params, OptionalExtension};

use super::models::{NewSource, Source};
use super::{Db, StorageError};

fn row_to_source(row: &rusqlite::Row) -> rusqlite::Result<Source> {
    Ok(Source {
        id: row.get("id")?,
        name: row.get("name")?,
        kind: row.get("type")?,
        value: row.get("value")?,
        status: row.get("status")?,
        last_refresh: row.get("last_refresh")?,
        origin_label: row.get("origin_label")?,
    })
}

pub fn list_all(db: &Db) -> Result<Vec<Source>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare("SELECT id, name, type, value, status, last_refresh, origin_label FROM sources ORDER BY rowid")?;
    let rows = stmt.query_map([], row_to_source)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn get(db: &Db, id: &str) -> Result<Source, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT id, name, type, value, status, last_refresh, origin_label FROM sources WHERE id = ?1",
        params![id],
        row_to_source,
    )
    .optional()?
    .ok_or(StorageError::NotFound)
}

pub fn insert(db: &Db, source: &NewSource) -> Result<(), StorageError> {
    let name = source.name.trim();
    let value = source.value.trim();
    if name.is_empty() || value.is_empty() {
        return Err(StorageError::InvalidInput(
            "source name and value cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    conn.execute(
        "INSERT INTO sources (id, name, type, value, status, last_refresh, origin_label) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![source.id, name, source.kind, value, source.status, source.last_refresh, source.origin_label],
    )?;
    Ok(())
}

pub struct SourcePatch<'a> {
    pub name: Option<&'a str>,
    pub value: Option<&'a str>,
    pub status: Option<&'a str>,
    pub last_refresh: Option<&'a str>,
}

pub fn update(db: &Db, id: &str, patch: &SourcePatch) -> Result<(), StorageError> {
    if let Some(name) = patch.name {
        if name.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "source name cannot be empty".into(),
            ));
        }
    }

    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE sources SET
            name = COALESCE(?1, name),
            value = COALESCE(?2, value),
            status = COALESCE(?3, status),
            last_refresh = COALESCE(?4, last_refresh)
         WHERE id = ?5",
        params![
            patch.name.map(str::trim),
            patch.value,
            patch.status,
            patch.last_refresh,
            id
        ],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

/// Removes a source. Profiles that referenced it keep existing (their
/// `source_id` is cleared via `ON DELETE SET NULL`) rather than being
/// deleted, matching the frontend's `removeSource` behavior of demoting
/// imported profiles to local ones instead of losing them.
pub fn delete(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM sources WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: &str, value: &str) -> NewSource {
        NewSource {
            id: id.into(),
            name: format!("Source {id}"),
            kind: "url".into(),
            value: value.into(),
            status: "up-to-date".into(),
            last_refresh: "Updated just now".into(),
            origin_label: "Remote URL".into(),
        }
    }

    #[test]
    fn insert_then_get_round_trips() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &source("s1", "https://example.com/sub")).unwrap();
        let s = get(&db, "s1").unwrap();
        assert_eq!(s.value, "https://example.com/sub");
        assert_eq!(s.kind, "url");
    }

    #[test]
    fn insert_rejects_blank_value() {
        let db = Db::open_in_memory().unwrap();
        assert!(matches!(
            insert(&db, &source("s1", "   ")).unwrap_err(),
            StorageError::InvalidInput(_)
        ));
    }

    #[test]
    fn update_is_a_partial_patch() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &source("s1", "https://example.com/sub")).unwrap();
        update(
            &db,
            "s1",
            &SourcePatch {
                name: None,
                value: None,
                status: Some("refresh-due"),
                last_refresh: None,
            },
        )
        .unwrap();
        let s = get(&db, "s1").unwrap();
        assert_eq!(s.status, "refresh-due");
        assert_eq!(
            s.name, "Source s1",
            "untouched fields must survive a partial patch"
        );
    }

    #[test]
    fn update_rejects_blank_name() {
        let db = Db::open_in_memory().unwrap();
        insert(&db, &source("s1", "https://example.com/sub")).unwrap();
        let err = update(
            &db,
            "s1",
            &SourcePatch {
                name: Some("  "),
                value: None,
                status: None,
                last_refresh: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));
    }

    #[test]
    fn update_unknown_source_is_not_found() {
        let db = Db::open_in_memory().unwrap();
        let err = update(
            &db,
            "ghost",
            &SourcePatch {
                name: None,
                value: None,
                status: Some("ready"),
                last_refresh: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::NotFound));
    }

    #[test]
    fn deleting_a_source_clears_but_does_not_delete_its_profiles() {
        let db = Db::open_in_memory().unwrap();
        crate::storage::groups::insert(
            &db,
            &crate::storage::models::NewProfileGroup {
                id: "g".into(),
                label: "G".into(),
                kind: "custom".into(),
                source_id: None,
            },
        )
        .unwrap();
        insert(&db, &source("s1", "https://example.com/sub")).unwrap();
        crate::storage::profiles::insert(
            &db,
            &crate::storage::models::NewProfile {
                id: "p1".into(),
                name: "P1".into(),
                region: "r".into(),
                protocol: crate::storage::models::Protocol::VLESS,
                origin: "imported".into(),
                group_id: "g".into(),
                source_id: Some("s1".into()),
                key: "vless://p1".into(),
            },
        )
        .unwrap();

        delete(&db, "s1").unwrap();

        let profile = crate::storage::profiles::get(&db, "p1").unwrap();
        assert_eq!(profile.source_id, None);
    }
}
