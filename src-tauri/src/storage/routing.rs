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
/// Selection state is intentionally not patchable here — use
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_migration_seeds_the_default_presets() {
        let db = Db::open_in_memory().unwrap();
        let presets = list_presets(&db).unwrap();
        let ids: Vec<_> = presets.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["bypass-lan", "block-ads"]);
        assert!(presets.iter().all(|p| p.enabled), "defaults ship enabled");
    }

    #[test]
    fn set_preset_toggles_enabled_flag() {
        let db = Db::open_in_memory().unwrap();
        set_preset(&db, "bypass-lan", false).unwrap();
        let presets = list_presets(&db).unwrap();
        assert!(!presets[0].enabled);
    }

    #[test]
    fn set_unknown_preset_is_not_found() {
        let db = Db::open_in_memory().unwrap();
        assert!(matches!(
            set_preset(&db, "ghost", true).unwrap_err(),
            StorageError::NotFound
        ));
    }

    #[test]
    fn insert_rule_rejects_blank_match_pattern() {
        let db = Db::open_in_memory().unwrap();
        let err = insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "  ".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));
    }

    #[test]
    fn select_rule_deselects_all_others() {
        let db = Db::open_in_memory().unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "a".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r2".into(),
                match_value: "b".into(),
                outbound: "Proxy".into(),
            },
        )
        .unwrap();
        select_rule(&db, "r1").unwrap();
        select_rule(&db, "r2").unwrap();
        let rules = list_rules(&db).unwrap();
        let selected: Vec<_> = rules
            .iter()
            .filter(|r| r.selected)
            .map(|r| r.id.clone())
            .collect();
        assert_eq!(selected, vec!["r2"]);
    }

    #[test]
    fn select_unknown_rule_leaves_selection_unchanged() {
        let db = Db::open_in_memory().unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "a".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
        select_rule(&db, "r1").unwrap();
        assert!(matches!(
            select_rule(&db, "ghost").unwrap_err(),
            StorageError::NotFound
        ));
        assert!(list_rules(&db).unwrap()[0].selected);
    }

    #[test]
    fn update_rule_outbound_rejects_unknown_rule() {
        let db = Db::open_in_memory().unwrap();
        assert!(matches!(
            update_rule_outbound(&db, "ghost", "Block").unwrap_err(),
            StorageError::NotFound
        ));
    }

    #[test]
    fn update_rule_outbound_rejects_a_value_outside_the_allowed_set() {
        let db = Db::open_in_memory().unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "a".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
        let err = update_rule_outbound(&db, "r1", "Teleport").unwrap_err();
        assert!(
            matches!(err, StorageError::Sqlite(_)),
            "the CHECK constraint should reject an unknown outbound"
        );
    }

    #[test]
    fn update_rule_patches_only_the_provided_fields() {
        let db = Db::open_in_memory().unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "a".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
        update_rule(
            &db,
            "r1",
            &RulePatch {
                match_value: Some("b.example.com"),
                outbound: None,
            },
        )
        .unwrap();
        let rule = list_rules(&db)
            .unwrap()
            .into_iter()
            .find(|r| r.id == "r1")
            .unwrap();
        assert_eq!(rule.match_value, "b.example.com");
        assert_eq!(
            rule.outbound, "Direct",
            "untouched field must survive a partial patch"
        );
    }

    #[test]
    fn update_rule_rejects_a_blank_match_pattern() {
        let db = Db::open_in_memory().unwrap();
        insert_rule(
            &db,
            &NewRoutingRule {
                id: "r1".into(),
                match_value: "a".into(),
                outbound: "Direct".into(),
            },
        )
        .unwrap();
        let err = update_rule(
            &db,
            "r1",
            &RulePatch {
                match_value: Some("   "),
                outbound: None,
            },
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::InvalidInput(_)));
    }

    #[test]
    fn update_rule_on_an_unknown_rule_is_not_found() {
        let db = Db::open_in_memory().unwrap();
        let err = update_rule(
            &db,
            "ghost",
            &RulePatch {
                match_value: None,
                outbound: Some("Block"),
            },
        )
        .unwrap_err();
        assert!(matches!(err, StorageError::NotFound));
    }
}
