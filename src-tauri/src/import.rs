//! What "add from clipboard" does with the text it is handed.
//!
//! The user used to pick "subscription URL" or "single key" before pasting,
//! and picking was the part people got wrong. The text decides now: an
//! http(s) URL is a subscription and gets a group that refreshes from it;
//! anything the subscription parser accepts is profiles, a lone one going to
//! Default and several becoming a static group of their own.
//!
//! Fetching the URL stays in `commands`, so everything here runs against text
//! and an in-memory database.

use std::collections::{HashMap, HashSet};

use base64::Engine;
use serde::Serialize;
use thiserror::Error;

use crate::storage::models::{NewProfile, NewProfileGroup, NewSource, ProfileGroup, Protocol};
use crate::storage::{groups, profiles, settings, sources, Db, StorageError};
use crate::subscription::model::ParsedOutbound;
use crate::subscription::{self, SubscriptionError};

const DEFAULT_GROUP_ID: &str = "default";
const IMPORTED_GROUP_LABEL: &str = "Imported";
const FALLBACK_SUBSCRIPTION_NAME: &str = "Subscription";

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Subscription(#[from] SubscriptionError),

    #[error("{0} is not a subscription group")]
    NotASubscription(String),

    #[error("switch to a VPN outside this subscription or disconnect before deleting it")]
    ActiveProfileInUse,
}

/// What a piece of pasted text turned out to be.
#[derive(Debug, PartialEq)]
pub enum Pasted {
    SubscriptionUrl(String),
    Outbounds(Vec<ParsedOutbound>),
}

pub fn is_subscription_url(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.contains(char::is_whitespace)
        && url::Url::parse(trimmed).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
}

pub fn classify(text: &str) -> Result<Pasted, ImportError> {
    let trimmed = text.trim();
    if is_subscription_url(trimmed) {
        return Ok(Pasted::SubscriptionUrl(trimmed.to_string()));
    }
    Ok(Pasted::Outbounds(subscription::parse_subscription(
        trimmed,
    )?))
}

/// What an import did, so the UI can say it in words.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ImportOutcome {
    #[serde(rename_all = "camelCase")]
    SubscriptionAdded { group_id: String, added: usize },
    /// The URL was already a subscription: it was refreshed instead of added
    /// a second time.
    #[serde(rename_all = "camelCase")]
    SubscriptionRefreshed { group_id: String },
    #[serde(rename_all = "camelCase")]
    ProfileAdded { profile_id: String, name: String },
    #[serde(rename_all = "camelCase")]
    GroupAdded {
        group_id: String,
        added: usize,
        skipped: usize,
    },
    /// A single key that is already in `group_id`.
    #[serde(rename_all = "camelCase")]
    AlreadyPresent { group_id: String },
    /// Several keys, every one of them already present.
    #[serde(rename_all = "camelCase")]
    NothingNew { skipped: usize },
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

/// Providers name a subscription in its `profile-title` header, often as
/// `base64:<text>` so that non-ASCII survives HTTP. Without one the host is
/// the most recognisable thing left.
pub fn subscription_name(profile_title: Option<&str>, url: &str) -> String {
    let from_header = profile_title
        .map(str::trim)
        .and_then(|title| match title.strip_prefix("base64:") {
            Some(encoded) => base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                .ok()
                .and_then(|bytes| String::from_utf8(bytes).ok()),
            None => Some(title.to_string()),
        })
        .map(|title| title.trim().to_string())
        .filter(|title| !title.is_empty());
    let from_host = || {
        url::Url::parse(url)
            .ok()
            .and_then(|url| {
                url.host_str()
                    .map(|host| host.trim_start_matches("www.").to_string())
            })
            .filter(|host| !host.is_empty())
    };
    from_header
        .or_else(from_host)
        .unwrap_or_else(|| FALLBACK_SUBSCRIPTION_NAME.to_string())
}

/// Subscriptions conventionally prefix the profile name with a flag emoji, and
/// a flag is just two regional-indicator code points that map 1:1 onto the
/// letters of the ISO 3166-1 alpha-2 country code. Names without one get an
/// empty region: the server hostname used to go here, but it says nothing
/// about location, so a blank is at least honest.
// ponytail: no geo-IP lookup; add one only if names stop carrying flags.
fn region_from_name(name: &str) -> String {
    fn letter(c: char) -> Option<char> {
        ('\u{1F1E6}'..='\u{1F1FF}')
            .contains(&c)
            .then(|| (b'A' + (c as u32 - 0x1F1E6) as u8) as char)
    }
    let mut chars = name.trim_start().chars();
    match (chars.next().and_then(letter), chars.next().and_then(letter)) {
        (Some(a), Some(b)) => format!("{a}{b}"),
        _ => String::new(),
    }
}

pub fn profile_from_outbound(
    outbound: &ParsedOutbound,
    group_id: &str,
    source_id: Option<&str>,
    origin: &str,
) -> NewProfile {
    let protocol = match outbound {
        ParsedOutbound::Vmess(_) => Protocol::VMess,
        ParsedOutbound::Vless(_) => Protocol::VLESS,
        ParsedOutbound::Trojan(_) => Protocol::Trojan,
        ParsedOutbound::Shadowsocks(_) => Protocol::Shadowsocks,
        ParsedOutbound::Hysteria2(_) => Protocol::Hysteria2,
        ParsedOutbound::Tuic(_) => Protocol::Tuic,
    };
    NewProfile {
        id: new_id(origin),
        name: outbound.name().to_string(),
        region: region_from_name(outbound.name()),
        protocol,
        origin: origin.to_string(),
        group_id: group_id.to_string(),
        source_id: source_id.map(str::to_string),
        key: subscription::to_uri(outbound),
    }
}

/// Two keys for the same server that differ only in their display name are
/// the same VPN. The name rides in the fragment for most schemes but inside
/// the base64 body for VMess, so it is blanked on the parsed form rather than
/// cut off the string.
fn identity(outbound: &ParsedOutbound) -> String {
    let mut unnamed = outbound.clone();
    match &mut unnamed {
        ParsedOutbound::Vmess(o) => o.name.clear(),
        ParsedOutbound::Vless(o) => o.name.clear(),
        ParsedOutbound::Trojan(o) => o.name.clear(),
        ParsedOutbound::Shadowsocks(o) => o.name.clear(),
        ParsedOutbound::Hysteria2(o) => o.name.clear(),
        ParsedOutbound::Tuic(o) => o.name.clear(),
    }
    subscription::to_uri(&unnamed)
}

fn stored_key_identity(key: &str) -> String {
    subscription::parse_uri(key)
        .map(|outbound| identity(&outbound))
        .unwrap_or_else(|_| key.trim().to_string())
}

fn next_imported_label(existing: &[ProfileGroup]) -> String {
    let taken: HashSet<&str> = existing.iter().map(|group| group.label.as_str()).collect();
    if !taken.contains(IMPORTED_GROUP_LABEL) {
        return IMPORTED_GROUP_LABEL.to_string();
    }
    let mut n = 2;
    loop {
        let label = format!("{IMPORTED_GROUP_LABEL} {n}");
        if !taken.contains(label.as_str()) {
            return label;
        }
        n += 1;
    }
}

/// The subscription group already fed by `url`, if there is one.
pub fn subscription_for_url(db: &Db, url: &str) -> Result<Option<ProfileGroup>, ImportError> {
    let source_ids: HashSet<String> = sources::list_all(db)?
        .into_iter()
        .filter(|source| source.kind == "url" && source.value == url)
        .map(|source| source.id)
        .collect();
    Ok(groups::list_all(db)?.into_iter().find(|group| {
        group.kind == "subscription"
            && group
                .source_id
                .as_ref()
                .is_some_and(|id| source_ids.contains(id))
    }))
}

pub fn add_subscription(
    db: &Db,
    url: &str,
    name: &str,
    outbounds: &[ParsedOutbound],
) -> Result<ImportOutcome, ImportError> {
    let source_id = new_id("source");
    let group_id = format!("subscription-{source_id}");
    sources::insert(
        db,
        &NewSource {
            id: source_id.clone(),
            name: name.to_string(),
            kind: "url".to_string(),
            value: url.to_string(),
            status: "up-to-date".to_string(),
            last_refresh: sources::refreshed_now(),
            origin_label: "Remote URL".to_string(),
        },
    )?;
    groups::insert(
        db,
        &NewProfileGroup {
            id: group_id.clone(),
            label: name.to_string(),
            kind: "subscription".to_string(),
            source_id: Some(source_id.clone()),
        },
    )?;
    for outbound in outbounds {
        profiles::insert(
            db,
            &profile_from_outbound(outbound, &group_id, Some(&source_id), "imported"),
        )?;
    }
    Ok(ImportOutcome::SubscriptionAdded {
        group_id,
        added: outbounds.len(),
    })
}

/// Pasted keys, as opposed to a subscription. Anything already present, in
/// any group, is skipped rather than added twice.
pub fn add_outbounds(db: &Db, outbounds: &[ParsedOutbound]) -> Result<ImportOutcome, ImportError> {
    let existing: HashMap<String, String> = profiles::list_all(db)?
        .into_iter()
        .map(|profile| (stored_key_identity(&profile.key), profile.group_id))
        .collect();

    if let [outbound] = outbounds {
        if let Some(group_id) = existing.get(&identity(outbound)) {
            return Ok(ImportOutcome::AlreadyPresent {
                group_id: group_id.clone(),
            });
        }
        let profile = profile_from_outbound(outbound, DEFAULT_GROUP_ID, None, "local");
        profiles::insert(db, &profile)?;
        return Ok(ImportOutcome::ProfileAdded {
            profile_id: profile.id,
            name: profile.name,
        });
    }

    let mut seen: HashSet<String> = existing.into_keys().collect();
    let fresh: Vec<&ParsedOutbound> = outbounds
        .iter()
        .filter(|outbound| seen.insert(identity(outbound)))
        .collect();
    let skipped = outbounds.len() - fresh.len();
    if fresh.is_empty() {
        return Ok(ImportOutcome::NothingNew { skipped });
    }

    let group_id = new_id("group");
    groups::insert(
        db,
        &NewProfileGroup {
            id: group_id.clone(),
            label: next_imported_label(&groups::list_all(db)?),
            kind: "custom".to_string(),
            source_id: None,
        },
    )?;
    for outbound in &fresh {
        profiles::insert(
            db,
            &profile_from_outbound(outbound, &group_id, None, "local"),
        )?;
    }
    Ok(ImportOutcome::GroupAdded {
        group_id,
        added: fresh.len(),
        skipped,
    })
}

/// Deletes a subscription together with its group and every VPN in it.
///
/// Refused while connected through one of those VPNs: pulling the active
/// profile out from under a running tunnel would leave the config pointing at
/// an outbound the database no longer knows. Disconnected, the active choice
/// is simply cleared.
pub fn remove_subscription(
    db: &Db,
    group_id: &str,
    active_profile_id: Option<&str>,
    connected: bool,
) -> Result<(), ImportError> {
    let group = groups::get(db, group_id)?;
    let source_id = match (group.kind.as_str(), group.source_id) {
        ("subscription", Some(source_id)) => source_id,
        _ => return Err(ImportError::NotASubscription(group_id.to_string())),
    };
    let holds_active =
        active_profile_id.is_some_and(|active| group.profile_ids.iter().any(|id| id == active));
    if holds_active && connected {
        return Err(ImportError::ActiveProfileInUse);
    }

    // The group first: deleting the source first would detach the group and
    // leave it behind if the second delete failed.
    groups::delete(db, group_id)?;
    sources::delete(db, &source_id)?;
    if holds_active {
        settings::set_active_profile_id(db, None)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
