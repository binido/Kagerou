use rusqlite::params;

use super::models::{NewRoutingRule, RoutingPreset, RoutingRule};
use super::{Db, StorageError};

pub fn list_presets(db: &Db) -> Result<Vec<RoutingPreset>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn
        .prepare("SELECT id, label, description, enabled FROM routing_presets ORDER BY position")?;
    let rows = stmt.query_map([], |row| {
        Ok(RoutingPreset {
            id: row.get("id")?,
            label: row.get("label")?,
            description: row.get("description")?,
            enabled: row.get::<_, i64>("enabled")? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn set_preset(db: &Db, id: &str, enabled: bool) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE routing_presets SET enabled = ?1 WHERE id = ?2",
        params![enabled as i64, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn list_rules(db: &Db) -> Result<Vec<RoutingRule>, StorageError> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, match_value, outbound, selected FROM routing_rules ORDER BY position",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(RoutingRule {
            id: row.get("id")?,
            match_value: row.get("match_value")?,
            outbound: row.get("outbound")?,
            selected: row.get::<_, i64>("selected")? != 0,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(StorageError::from)
}

pub fn insert_rule(db: &Db, rule: &NewRoutingRule) -> Result<(), StorageError> {
    if rule.match_value.trim().is_empty() {
        return Err(StorageError::InvalidInput(
            "rule match pattern cannot be empty".into(),
        ));
    }
    let conn = db.lock();
    let next_position: i64 = conn.query_row(
        "SELECT COALESCE(MAX(position) + 1, 0) FROM routing_rules",
        [],
        |row| row.get(0),
    )?;
    conn.execute(
        "INSERT INTO routing_rules (id, match_value, outbound, selected, position) VALUES (?1, ?2, ?3, 0, ?4)",
        params![rule.id, rule.match_value.trim(), rule.outbound, next_position],
    )?;
    Ok(())
}

/// Selects exactly one rule at a time, mirroring the frontend's single-row
/// "active rule" selection semantics.
pub fn select_rule(db: &Db, id: &str) -> Result<(), StorageError> {
    let mut conn = db.lock();
    let tx = conn.transaction()?;
    let affected = tx.execute(
        "UPDATE routing_rules SET selected = 1 WHERE id = ?1",
        params![id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    tx.execute(
        "UPDATE routing_rules SET selected = 0 WHERE id != ?1",
        params![id],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn update_rule_outbound(db: &Db, id: &str, outbound: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE routing_rules SET outbound = ?1 WHERE id = ?2",
        params![outbound, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[derive(Default)]
pub struct RulePatch<'a> {
    pub match_value: Option<&'a str>,
    pub outbound: Option<&'a str>,
}

/// General partial update for a rule's match pattern and/or outbound.
/// Selection state is intentionally not patchable here - use
/// `select_rule`, which enforces the single-selected-rule invariant.
pub fn update_rule(db: &Db, id: &str, patch: &RulePatch) -> Result<(), StorageError> {
    if let Some(m) = patch.match_value {
        if m.trim().is_empty() {
            return Err(StorageError::InvalidInput(
                "rule match pattern cannot be empty".into(),
            ));
        }
    }
    let conn = db.lock();
    let affected = conn.execute(
        "UPDATE routing_rules SET match_value = COALESCE(?1, match_value), outbound = COALESCE(?2, outbound) WHERE id = ?3",
        params![patch.match_value.map(str::trim), patch.outbound, id],
    )?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub fn delete_rule(db: &Db, id: &str) -> Result<(), StorageError> {
    let conn = db.lock();
    let affected = conn.execute("DELETE FROM routing_rules WHERE id = ?1", params![id])?;
    if affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
