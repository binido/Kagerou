use rusqlite::{params, OptionalExtension};

use super::groups;
use super::models::{NewProfile, Profile, Protocol, TestOutcome};
use super::{Db, StorageError};

fn row_to_profile(row: &rusqlite::Row) -> rusqlite::Result<Profile> {
    let protocol: String = row.get("protocol")?;
    let url_kind: String = row.get("url_kind")?;
    Ok(Profile {
        id: row.get("id")?,
        name: row.get("name")?,
        region: row.get("region")?,
        protocol: protocol.parse().unwrap_or(Protocol::VLESS),
        origin: row.get("origin")?,
        group_id: row.get("group_id")?,
        source_id: row.get("source_id")?,
        selected: row.get::<_, i64>("selected")? != 0,
        url: TestOutcome::from_stored(&url_kind, row.get("url_millis")?).into(),
        key: row.get("key")?,
    })
}

pub fn list_all(db: &Db) -> Result<Vec<Profile>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, region, protocol, origin, group_id, source_id, selected, url_kind, url_millis, key
         FROM profiles ORDER BY group_id, position",
    )?;
    let rows = stmt.query_map([], row_to_profile)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn get(db: &Db, id: &str) -> Result<Profile, StorageError> {
    let conn = db.lock();
    conn.query_row(
        "SELECT id, name, region, protocol, origin, group_id, source_id, selected, url_kind, url_millis, key
         FROM profiles WHERE id = ?1",
        params![id],
        row_to_profile,
    )
    .optional()?
    .ok_or(StorageError::NotFound)
}

/// Inserts a new profile, appended to the end of its group's ordering.
pub fn insert(db: &Db, profile: &NewProfile) -> Result<(), StorageError> {
    let conn = db.lock();
    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM profiles WHERE group_id = ?1",
        params![profile.group_id],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO profiles (id, name, region, protocol, origin, group_id, source_id, selected, url_kind, url_millis, key, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 'notTested', NULL, ?8, ?9)",
        params![
            profile.id,
            profile.name,
            profile.region,
            profile.protocol.as_str(),
            profile.origin,
            profile.group_id,
            profile.source_id,
            profile.key,
            next_position,
        ],
    )?;
    Ok(())
}

/// Atomically makes `id` the only selected profile. Errors if `id` does not
/// exist, leaving the previous selection untouched. Safe to call
/// concurrently from multiple threads: the whole read-modify-write runs
/// inside a single transaction serialized by `Db`'s connection mutex, so
/// competing selections never race each other into an inconsistent state.
pub fn select_profile(db: &Db, id: &str) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let exists: bool = tx
        .query_row("SELECT 1 FROM profiles WHERE id = ?1", params![id], |_| {
            Ok(())
        })
        .optional()?
        .is_some();
    if !exists {
        return Err(StorageError::NotFound);
    }

    tx.execute("UPDATE profiles SET selected = 0 WHERE selected = 1", [])?;
    tx.execute(
        "UPDATE profiles SET selected = 1, last_selected_at = unixepoch() WHERE id = ?1",
        params![id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn rename(db: &Db, id: &str, name: &str) -> Result<(), StorageError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(StorageError::InvalidInput(
            "profile name cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE profiles SET name = ?1 WHERE id = ?2",
        params![trimmed, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn delete(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM profiles WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn set_test_outcome(db: &Db, id: &str, outcome: TestOutcome) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE profiles SET url_kind = ?1, url_millis = ?2 WHERE id = ?3",
        params![outcome.kind_str(), outcome.millis(), id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

/// The profiles most recently switched to, newest first, capped at `limit`.
/// Profiles never selected are left out: they are not "recent", and padding
/// the list with arbitrary ones would make the tray menu lie about what it
/// is.
pub fn recently_selected(db: &Db, limit: usize) -> Result<Vec<Profile>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, name, region, protocol, origin, group_id, source_id, selected, url_kind, url_millis, key
         FROM profiles
         WHERE last_selected_at IS NOT NULL
         ORDER BY last_selected_at DESC, name ASC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], row_to_profile)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

/// Resets the stored test result of every profile in `group_id` back to its
/// untested state. Idempotent on profiles that were never tested;
/// an unknown or empty group is not an error.
pub fn clear_test_results(db: &Db, group_id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    conn.execute(
        "UPDATE profiles
         SET url_kind = 'notTested', url_millis = NULL
         WHERE group_id = ?1",
        params![group_id],
    )?;
    Ok(())
}

/// Deletes every profile in `group_id` that did not answer its last test.
///
/// Not answering is what "unavailable" means here: a timeout, no response,
/// or a selector the core refused. A server that answered slowly is
/// available, and used to be deleted alongside them because the filter was
/// the display colour, which a latency over 400ms also carries. `skip_id`
/// (the active profile) is never deleted, and untested profiles survive by
/// construction. Returns how many rows were deleted.
pub fn delete_unavailable(
    db: &Db,
    group_id: &str,
    skip_id: Option<&str>,
) -> Result<usize, StorageError> {
    let conn = db.lock();
    let affected = match skip_id {
        Some(skip) => conn.execute(
            "DELETE FROM profiles
             WHERE group_id = ?1 AND url_kind IN ('timeout', 'noResponse', 'unavailable')
               AND id != ?2",
            params![group_id, skip],
        )?,
        None => conn.execute(
            "DELETE FROM profiles
             WHERE group_id = ?1 AND url_kind IN ('timeout', 'noResponse', 'unavailable')",
            params![group_id],
        )?,
    };
    Ok(affected)
}

/// Moves a profile into `target_group_id`, appending it at the end of that
/// group's ordering. Subscription groups are closed both ways: a refresh
/// replaces the whole group, so a profile moved in would be deleted by the
/// next one and a profile moved out would come back beside itself.
pub fn move_to_group(db: &Db, id: &str, target_group_id: &str) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let target_kind: Option<String> = tx
        .query_row(
            "SELECT kind FROM profile_groups WHERE id = ?1",
            params![target_group_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(target_kind) = target_kind else {
        return Err(StorageError::InvalidInput(format!(
            "group {target_group_id} does not exist"
        )));
    };
    let current_kind: Option<String> = tx
        .query_row(
            "SELECT g.kind FROM profiles p JOIN profile_groups g ON g.id = p.group_id WHERE p.id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(current_kind) = current_kind else {
        return Err(StorageError::NotFound);
    };
    if target_kind == "subscription" || current_kind == "subscription" {
        return Err(StorageError::InvalidInput(
            "subscription VPNs cannot be moved in or out of their group".into(),
        ));
    }

    let next_position: i64 = tx.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM profiles WHERE group_id = ?1",
        params![target_group_id],
        |row| row.get(0),
    )?;
    let affected = tx.execute(
        "UPDATE profiles SET group_id = ?1, position = ?2 WHERE id = ?3",
        params![target_group_id, next_position, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    tx.commit()?;
    Ok(())
}

/// Rewrites the `position` of every profile in `group_id` to match the
/// order of `ordered_ids`. Used to implement both "move up/down" (compute
/// the swapped order, then reorder) and drag-to-reorder in one primitive
/// rather than two position-swap-specific queries. Rejects the whole
/// operation — no partial reorder — if `ordered_ids` doesn't contain
/// exactly the profiles currently in that group.
pub fn reorder(db: &Db, group_id: &str, ordered_ids: &[String]) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;

    let mut current: Vec<String> = {
        let mut stmt =
            tx.prepare("SELECT id FROM profiles WHERE group_id = ?1 ORDER BY position")?;
        let rows = stmt.query_map(params![group_id], |row| row.get::<_, String>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    current.sort();
    let mut requested = ordered_ids.to_vec();
    requested.sort();
    if current != requested {
        return Err(StorageError::InvalidInput(
            "ordered_ids must be exactly the profiles currently in the group".into(),
        ));
    }

    for (position, id) in ordered_ids.iter().enumerate() {
        tx.execute(
            "UPDATE profiles SET position = ?1 WHERE id = ?2",
            params![position as i64, id],
        )?;
    }
    tx.commit()?;
    Ok(())
}

/// Which way a profile is being nudged within its group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl std::str::FromStr for Direction {
    type Err = StorageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            other => Err(StorageError::InvalidInput(format!(
                "unknown direction: {other}"
            ))),
        }
    }
}

/// Moves a profile one place within its own group. At the end it is already
/// at, this does nothing: a list's first row has nowhere to go up to, and
/// saying so is not worth an error.
pub fn move_within_group(db: &Db, id: &str, direction: Direction) -> Result<(), StorageError> {
    let profile = get(db, id)?;
    let group = groups::get(db, &profile.group_id)?;
    let index = group
        .profile_ids
        .iter()
        .position(|candidate| candidate == id)
        .ok_or(StorageError::NotFound)?;
    let target = match direction {
        Direction::Up => index.checked_sub(1),
        Direction::Down => index
            .checked_add(1)
            .filter(|i| *i < group.profile_ids.len()),
    };
    let Some(target) = target else {
        return Ok(());
    };
    let mut ordered = group.profile_ids;
    ordered.swap(index, target);
    reorder(db, &group.id, &ordered)
}

/// Lifts a profile out of its position and drops it where another one sits,
/// shifting the rest along. Both have to be in the same group.
pub fn move_before(db: &Db, from_id: &str, to_id: &str) -> Result<(), StorageError> {
    let from = get(db, from_id)?;
    let group = groups::get(db, &from.group_id)?;
    let mut ordered = group.profile_ids;
    let from_index = ordered
        .iter()
        .position(|c| c == from_id)
        .ok_or(StorageError::NotFound)?;
    let to_index = ordered
        .iter()
        .position(|c| c == to_id)
        .ok_or(StorageError::NotFound)?;
    let id = ordered.remove(from_index);
    ordered.insert(to_index, id);
    reorder(db, &group.id, &ordered)
}

#[cfg(test)]
mod tests;
