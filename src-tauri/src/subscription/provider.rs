use base64::Engine;
use reqwest::header::HeaderMap;

use crate::storage::models::ProviderInfo;

/// Longer announcements are cut, so a broken panel cannot fill the window.
const ANNOUNCE_MAX_CHARS: usize = 500;

/// Providers write a date a century out, 2100 or 2126, to mean "never ends".
/// Nothing is paid for this far ahead, so such a date reads as no end date.
const OPEN_ENDED_AFTER_MS: i64 = 10 * 365 * 86_400_000;

/// Reads what the provider says about the subscription itself from the
/// headers Happ and v2rayN understand. A header that is missing or cannot be
/// read leaves its field empty. `now` is unix milliseconds.
pub fn provider_info(headers: &HeaderMap, now: i64) -> ProviderInfo {
    let header = |name: &str| headers.get(name).and_then(|value| value.to_str().ok());
    let mut info = header("subscription-userinfo")
        .map(parse_userinfo)
        .unwrap_or_default();
    info.expires_at = info
        .expires_at
        .filter(|&expires| expires <= now.saturating_add(OPEN_ENDED_AFTER_MS));
    info.announce = header("announce")
        .and_then(decode_header_text)
        .map(|text| text.chars().take(ANNOUNCE_MAX_CHARS).collect());
    info.support_url = header("support-url").and_then(web_link);
    info
}

/// Decodes a text header that may come as `base64:<text>`, the way providers
/// send non-ASCII through HTTP. Blank text counts as absent.
pub fn decode_header_text(value: &str) -> Option<String> {
    let value = value.trim();
    let text = match value.strip_prefix("base64:") {
        Some(encoded) => base64::engine::general_purpose::STANDARD
            .decode(encoded.trim())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())?,
        None => value.to_string(),
    };
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Parses `upload=1; download=2; total=3; expire=4`.
///
/// `total=0` means no limit and `expire=0` means no end date, so both read
/// as absent. `expire` is in unix seconds.
fn parse_userinfo(value: &str) -> ProviderInfo {
    let mut upload = None;
    let mut download = None;
    let mut total = None;
    let mut expire = None;
    for pair in value.split(';') {
        let Some((key, number)) = pair.split_once('=') else {
            continue;
        };
        let Ok(number) = number.trim().parse::<u64>() else {
            continue;
        };
        // SQLite stores a signed 64-bit integer.
        let number = number.min(i64::MAX as u64) as i64;
        match key.trim() {
            "upload" => upload = Some(number),
            "download" => download = Some(number),
            "total" => total = Some(number),
            "expire" => expire = Some(number),
            _ => {}
        }
    }
    let traffic_used = match (upload, download) {
        (None, None) => None,
        (up, down) => Some(up.unwrap_or(0).saturating_add(down.unwrap_or(0))),
    };
    ProviderInfo {
        traffic_used,
        traffic_total: total.filter(|&total| total > 0),
        expires_at: expire
            .filter(|&expire| expire > 0)
            .map(|seconds| seconds.saturating_mul(1000)),
        announce: None,
        support_url: None,
    }
}

/// The link opens in the system browser, so anything but http(s) is dropped.
fn web_link(value: &str) -> Option<String> {
    let url = url::Url::parse(value.trim()).ok()?;
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

#[cfg(test)]
mod tests;
