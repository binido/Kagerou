use std::collections::HashMap;

use thiserror::Error;

use crate::storage::{groups, profiles, sources, Db, StorageError};
use crate::subscription::fetch::{self, FetchError};
use crate::subscription::model::ParsedOutbound;
use crate::subscription::{parse_subscription, SubscriptionError, Unsupported};
use crate::usecase::import::{self, ImportError, ImportOutcome, Imported, Pasted};

#[derive(Debug, Error)]
pub enum SubscriptionsError {
    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Import(#[from] ImportError),

    #[error(transparent)]
    Parse(#[from] SubscriptionError),

    #[error(transparent)]
    Fetch(#[from] FetchError),

    #[error("no group for this source")]
    NoGroup,

    #[error("a subscription URL must be an http(s) link")]
    NotASubscriptionUrl,
}

/// Replaces a subscription group's profiles with what the provider now
/// offers.
///
/// A profile whose key is still in the list keeps its id, so a selection,
/// and anything else pointing at it, survives the refresh. Anything the
/// provider dropped goes with it.
pub fn replace_group_profiles(
    db: &Db,
    source_id: &str,
    parsed: &[ParsedOutbound],
) -> Result<(), SubscriptionsError> {
    let group = groups::list_all(db)?
        .into_iter()
        .find(|g| g.source_id.as_deref() == Some(source_id))
        .ok_or(SubscriptionsError::NoGroup)?;

    let id_by_key: HashMap<String, String> = profiles::list_all(db)?
        .into_iter()
        .filter(|p| p.group_id == group.id)
        .map(|p| (p.key, p.id))
        .collect();

    for old_id in &group.profile_ids {
        let _ = profiles::delete(db, old_id);
    }
    for outbound in parsed {
        let mut profile =
            import::profile_from_outbound(outbound, &group.id, Some(source_id), "imported");
        if let Some(existing_id) = id_by_key.get(&profile.key) {
            profile.id = existing_id.clone();
        }
        profiles::insert(db, &profile)?;
    }

    let refreshed_at = sources::refreshed_now();
    sources::update(
        db,
        source_id,
        &sources::SourcePatch {
            name: None,
            value: None,
            status: Some("up-to-date"),
            last_refresh: Some(&refreshed_at),
        },
    )?;
    Ok(())
}

/// Fetches a subscription again and replaces what it produced last time.
/// A source that is not a URL has nothing to refresh from.
///
/// Returns what the provider offered that could not be imported. A body with
/// nothing importable fails in `parse_subscription`, before the group is touched.
pub async fn refresh(db: &Db, source_id: &str) -> Result<Vec<Unsupported>, SubscriptionsError> {
    let source = sources::get(db, source_id)?;
    if source.kind != "url" {
        return Ok(Vec::new());
    }
    let body = fetch::fetch(&source.value).await?.body;
    let parsed = parse_subscription(&body)?;
    replace_group_profiles(db, source_id, &parsed.outbounds)?;
    Ok(parsed.unsupported)
}

/// The one way VPNs get in: whatever was on the clipboard, or pasted by
/// hand. A URL that is already a subscription is refreshed rather than
/// added twice.
pub async fn import_text(db: &Db, text: &str) -> Result<Imported, SubscriptionsError> {
    let url = match import::classify(text)? {
        Pasted::Outbounds(parsed) => {
            return Ok(Imported {
                outcome: import::add_outbounds(db, &parsed.outbounds)?,
                unsupported: parsed.unsupported,
            })
        }
        Pasted::SubscriptionUrl(url) => url,
    };
    if let Some(group) = import::subscription_for_url(db, &url)? {
        let unsupported = match &group.source_id {
            Some(source_id) => refresh(db, source_id).await?,
            None => Vec::new(),
        };
        return Ok(Imported {
            outcome: ImportOutcome::SubscriptionRefreshed { group_id: group.id },
            unsupported,
        });
    }
    let fetched = fetch::fetch(&url).await?;
    let parsed = parse_subscription(&fetched.body)?;
    let name = import::subscription_name(fetched.title.as_deref(), &url);
    Ok(Imported {
        outcome: import::add_subscription(db, &url, &name, &parsed.outbounds)?,
        unsupported: parsed.unsupported,
    })
}

/// Renames a subscription, or points it at a different URL.
pub fn update_source(
    db: &Db,
    id: &str,
    name: Option<&str>,
    value: Option<&str>,
) -> Result<(), SubscriptionsError> {
    let value = value.map(str::trim);
    if value.is_some_and(|value| !import::is_subscription_url(value)) {
        return Err(SubscriptionsError::NotASubscriptionUrl);
    }
    sources::update(
        db,
        id,
        &sources::SourcePatch {
            name,
            value,
            status: None,
            last_refresh: None,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
