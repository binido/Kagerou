use reqwest::header::{HeaderMap, HeaderValue};

use super::*;

/// 2026-09-23.
const NOW: i64 = 1_790_121_600_000;

fn provider_info_at(headers: &HeaderMap) -> ProviderInfo {
    provider_info(headers, NOW)
}

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in pairs {
        map.insert(*name, HeaderValue::from_str(value).unwrap());
    }
    map
}

#[test]
fn reads_usage_limit_and_expiry() {
    let info = provider_info_at(&headers(&[(
        "subscription-userinfo",
        "upload=1000; download=2000; total=10737418240; expire=1767225600",
    )]));
    assert_eq!(info.traffic_used, Some(3000));
    assert_eq!(info.traffic_total, Some(10_737_418_240));
    assert_eq!(info.expires_at, Some(1_767_225_600_000));
}

#[test]
fn zero_total_and_zero_expiry_mean_no_limit_and_no_end() {
    let info = provider_info_at(&headers(&[(
        "subscription-userinfo",
        "upload=0; download=0; total=0; expire=0",
    )]));
    assert_eq!(info.traffic_used, Some(0));
    assert_eq!(info.traffic_total, None);
    assert_eq!(info.expires_at, None);
}

#[test]
fn garbage_in_the_usage_header_is_skipped_pair_by_pair() {
    let info = provider_info_at(&headers(&[(
        "subscription-userinfo",
        "upload=; download=12.5; total=abc; expire=-1; nonsense; extra=7; download=40",
    )]));
    assert_eq!(info.traffic_used, Some(40));
    assert_eq!(info.traffic_total, None);
    assert_eq!(info.expires_at, None);
}

#[test]
fn a_number_past_what_sqlite_holds_is_capped() {
    let info = provider_info_at(&headers(&[(
        "subscription-userinfo",
        "upload=18446744073709551615; download=18446744073709551615; expire=18446744073709551615",
    )]));
    assert_eq!(info.traffic_used, Some(i64::MAX));
    // Far past any real end date, so it also reads as none.
    assert_eq!(info.expires_at, None);
}

#[test]
fn no_headers_means_nothing_known() {
    assert_eq!(provider_info_at(&HeaderMap::new()), ProviderInfo::default());
}

#[test]
fn the_announcement_is_decoded_trimmed_and_cut() {
    // "Привет" in base64.
    let info = provider_info_at(&headers(&[("announce", "base64:0J/RgNC40LLQtdGC")]));
    assert_eq!(info.announce.as_deref(), Some("Привет"));

    let long = "a".repeat(2000);
    let info = provider_info_at(&headers(&[("announce", &long)]));
    assert_eq!(info.announce.map(|a| a.len()), Some(ANNOUNCE_MAX_CHARS));

    let info = provider_info_at(&headers(&[("announce", "base64:%%%")]));
    assert_eq!(info.announce, None);
    let info = provider_info_at(&headers(&[("announce", "   ")]));
    assert_eq!(info.announce, None);
}

#[test]
fn only_a_web_link_is_kept_as_the_support_url() {
    let info = provider_info_at(&headers(&[("support-url", " https://t.me/provider ")]));
    assert_eq!(info.support_url.as_deref(), Some("https://t.me/provider"));

    for rejected in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "tg://resolve",
        "not a url",
    ] {
        let info = provider_info_at(&headers(&[("support-url", rejected)]));
        assert_eq!(info.support_url, None, "{rejected}");
    }
}

#[test]
fn an_end_date_a_century_out_means_no_end_date() {
    let info = provider_info_at(&headers(&[(
        "subscription-userinfo",
        "upload=0; download=0; total=0; expire=4939401600",
    )]));
    assert_eq!(info.expires_at, None);

    // Nine years out is still a date.
    let info = provider_info_at(&headers(&[("subscription-userinfo", "expire=2074118400")]));
    assert_eq!(info.expires_at, Some(2_074_118_400_000));
}
