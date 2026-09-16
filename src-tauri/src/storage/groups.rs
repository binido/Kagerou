use rusqlite::{params, OptionalExtension};

use super::models::{NewProfileGroup, ProfileGroup};
use super::{Db, StorageError};

fn profile_ids_for_group(
    conn: &rusqlite::Connection,
    group_id: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM profiles WHERE group_id = ?1 ORDER BY position")?;
    let rows = stmt.query_map(params![group_id], |row| row.get(0))?;
    rows.collect()
}

pub fn list_all(db: &Db) -> Result<Vec<ProfileGroup>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, label, kind, source_id, is_open FROM profile_groups ORDER BY position",
    )?;
    let groups = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>("id")?,
                row.get::<_, String>("label")?,
                row.get::<_, String>("kind")?,
                row.get::<_, Option<String>>("source_id")?,
                row.get::<_, i64>("is_open")? != 0,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    groups
        .into_iter()
        .map(|(id, label, kind, source_id, open)| {
            let profile_ids = profile_ids_for_group(&conn, &id)?;
            Ok(ProfileGroup {
                id,
                label,
                kind,
                source_id,
                open,
                profile_ids,
            })
        })
        .collect::<Result<Vec<_>, rusqlite::Error>>()
        .map_err(StorageError::from)
}

pub fn get(db: &Db, id: &str) -> Result<ProfileGroup, StorageError> {
    let conn = db.lock();
    let row = conn
        .query_row(
            "SELECT id, label, kind, source_id, is_open FROM profile_groups WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>("id")?,
                    row.get::<_, String>("label")?,
                    row.get::<_, String>("kind")?,
                    row.get::<_, Option<String>>("source_id")?,
                    row.get::<_, i64>("is_open")? != 0,
                ))
            },
        )
        .optional()?
        .ok_or(StorageError::NotFound)?;
    let profile_ids = profile_ids_for_group(&conn, id)?;
    Ok(ProfileGroup {
        id: row.0,
        label: row.1,
        kind: row.2,
        source_id: row.3,
        open: row.4,
        profile_ids,
    })
}

pub fn insert(db: &Db, group: &NewProfileGroup) -> Result<(), StorageError> {
    let trimmed = group.label.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidInput(
            "group label cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM profile_groups",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO profile_groups (id, label, kind, source_id, is_open, position) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        params![group.id, trimmed, group.kind, group.source_id, next_position],
    )?;
    Ok(())
}

/// Renames a group. Refuses to rename the `default` kind group, matching
/// the frontend's rule that the default group's label is fixed.
pub fn rename(db: &Db, id: &str, label: &str) -> Result<(), StorageError> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidInput(
            "group label cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    let kind: Option<String> = conn
        .query_row(
            "SELECT kind FROM profile_groups WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    match kind.as_deref() {
        None => Err(StorageError::NotFound),
        Some("default") => Err(StorageError::InvalidInput(
            "the default group cannot be renamed".into(),
        )),
        Some(_) => {
            conn.execute(
                "UPDATE profile_groups SET label = ?1 WHERE id = ?2",
                params![trimmed, id],
            )?;
            Ok(())
        }
    }
}

pub fn set_open(db: &Db, id: &str, open: bool) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE profile_groups SET is_open = ?1 WHERE id = ?2",
        params![open as i64, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn delete(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM profile_groups WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
