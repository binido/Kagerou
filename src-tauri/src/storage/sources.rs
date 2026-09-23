use rusqlite::{params, OptionalExtension};

use super::models::{NewSource, ProviderInfo, Source};
use super::{Db, StorageError};

/// `last_refresh` holds unix milliseconds as text, or the empty string for a
/// source that has never been refreshed. It used to hold English prose, which
/// never aged and could not be translated; see migration 0011.
pub fn refreshed_now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_millis().to_string())
        .unwrap_or_default()
}

fn row_to_source(row: &rusqlite::Row) -> rusqlite::Result<Source> {
    Ok(Source {
        id: row.get("id")?,
        name: row.get("name")?,
        kind: row.get("type")?,
        value: row.get("value")?,
        status: row.get("status")?,
        last_refresh: row.get("last_refresh")?,
        origin_label: row.get("origin_label")?,
        provider: ProviderInfo {
            traffic_used: row.get("traffic_used")?,
            traffic_total: row.get("traffic_total")?,
            expires_at: row.get("expires_at")?,
            announce: row.get("announce")?,
            support_url: row.get("support_url")?,
        },
    })
}

pub fn list_all(db: &Db) -> Result<Vec<Source>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare("SELECT * FROM sources ORDER BY rowid")?;
    let rows = stmt.query_map([], row_to_source)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn get(db: &Db, id: &str) -> Result<Source, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT * FROM sources WHERE id = ?1",
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

/// Replaces what the provider said with what it says now. A field it
/// stopped sending is cleared, not kept from an older fetch.
pub fn set_provider(db: &Db, id: &str, info: &ProviderInfo) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE sources SET
            traffic_used = ?1,
            traffic_total = ?2,
            expires_at = ?3,
            announce = ?4,
            support_url = ?5
         WHERE id = ?6",
        params![
            info.traffic_used,
            info.traffic_total,
            info.expires_at,
            info.announce,
            info.support_url,
            id
        ],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

/// Removes a source. Profiles and groups that referenced it keep existing
/// with their `source_id` cleared by `ON DELETE SET NULL`. Deleting a
/// subscription together with its VPNs is `import::remove_subscription`,
/// which drops the group before the source.
pub fn delete(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM sources WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
