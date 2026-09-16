//! What a routing rule's single match string means.
//!
//! `classify_match` is total: anything that is not `localhost` and does not
//! parse as an address falls through to `domain_suffix` verbatim. That is what
//! the generator wants — it must always produce something — but it is useless
//! as an answer to "did the user type a sensible pattern?", because it says
//! "domain suffix" just as cheerfully for `*.example.com` or a pasted URL,
//! and those match nothing at all while sitting in the UI looking configured.
//!
//! So plausibility lives here as its own notion, next to the classifier rather
//! than mirrored in TypeScript: a second parser on a traffic path would drift
//! from this one silently, and silent is the expensive failure in this repo.

use std::net::IpAddr;

use serde::Serialize;

pub enum Matcher {
    Domain(String),
    DomainSuffix(String),
    IpCidr(String),
}

pub fn classify_match(raw: &str) -> Matcher {
    if raw.eq_ignore_ascii_case("localhost") {
        return Matcher::Domain(raw.to_string());
    }
    if let Some((addr, _prefix)) = raw.split_once('/') {
        if addr.parse::<IpAddr>().is_ok() {
            return Matcher::IpCidr(raw.to_string());
        }
    }
    if raw.parse::<IpAddr>().is_ok() {
        let cidr = match raw.parse::<IpAddr>().unwrap() {
            IpAddr::V4(_) => format!("{raw}/32"),
            IpAddr::V6(_) => format!("{raw}/128"),
        };
        return Matcher::IpCidr(cidr);
    }
    Matcher::DomainSuffix(raw.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchKind {
    Domain,
    DomainSuffix,
    IpCidr,
}

/// Codes, not sentences: the wording belongs in the locale files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatchWarning {
    Wildcard,
    Url,
    NonAscii,
    InvalidPrefix,
    InvalidDomain,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchAnalysis {
    /// What should be stored, which is not always what was typed.
    pub normalized: String,
    pub kind: MatchKind,
    pub warning: Option<MatchWarning>,
}

/// A leading dot is how people write a suffix, and it is exactly what
/// `domain_suffix` already means, so it is dropped rather than carried into
/// the config where it would match nothing.
pub fn normalize(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed.strip_prefix('.').unwrap_or(trimmed).to_string()
}

pub fn analyze(raw: &str) -> MatchAnalysis {
    let normalized = normalize(raw);
    let (kind, cidr_prefix) = match classify_match(&normalized) {
        Matcher::Domain(_) => (MatchKind::Domain, None),
        Matcher::DomainSuffix(_) => (MatchKind::DomainSuffix, None),
        // Only a slash the user typed carries a prefix to check; a bare
        // address gets /32 or /128 from the classifier and cannot be wrong.
        Matcher::IpCidr(_) => (
            MatchKind::IpCidr,
            normalized.split_once('/').map(|(addr, prefix)| {
                let bits = match addr.parse::<IpAddr>() {
                    Ok(IpAddr::V6(_)) => 128,
                    _ => 32,
                };
                prefix.parse::<u32>().map(|p| p <= bits).unwrap_or(false)
            }),
        ),
    };

    let warning = match kind {
        MatchKind::IpCidr => match cidr_prefix {
            Some(false) => Some(MatchWarning::InvalidPrefix),
            _ => None,
        },
        _ => domain_warning(&normalized),
    };

    MatchAnalysis {
        normalized,
        kind,
        warning,
    }
}

/// Ordered from most specific to least: a wildcard and a pasted URL are the
/// two mistakes worth naming, and a generic "this is not a domain" is the
/// fallback that would otherwise swallow them.
fn domain_warning(value: &str) -> Option<MatchWarning> {
    if value.eq_ignore_ascii_case("localhost") {
        return None;
    }
    if value.contains('*') {
        return Some(MatchWarning::Wildcard);
    }
    if value.contains("://") || value.contains('/') {
        return Some(MatchWarning::Url);
    }
    if !value.is_ascii() {
        return Some(MatchWarning::NonAscii);
    }
    if is_plausible_domain(value) {
        None
    } else {
        Some(MatchWarning::InvalidDomain)
    }
}

fn is_plausible_domain(value: &str) -> bool {
    if !value.contains('.') {
        return false;
    }
    value.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    })
}

#[cfg(test)]
mod tests;
