//! Checks GitHub for a newer release than the one running.
//!
//! Deliberately silent on failure: an update check that cannot reach the
//! network is not something to put in front of the user, and until the
//! project cuts its first release the endpoint answers 404, which is a
//! perfectly ordinary "nothing newer" rather than an error.

use std::time::Duration;

use semver::Version;
use serde::{Deserialize, Serialize};

const RELEASES_ENDPOINT: &str = "https://api.github.com/repos/binido/Kagerou/releases/latest";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// What the sidebar needs to offer the update: the version and where to get it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
}

/// Compares a release tag against the running version. Tags are conventionally
/// prefixed with `v`; anything that isn't a semver version is ignored rather
/// than guessed at.
fn newer_than(current: &Version, release: &GithubRelease) -> Option<UpdateInfo> {
    let tag = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);
    let candidate = Version::parse(tag).ok()?;
    (candidate > *current).then(|| UpdateInfo {
        version: candidate.to_string(),
        url: release.html_url.clone(),
    })
}

/// `None` covers every uninteresting outcome alike: no releases yet, nothing
/// newer, no network, a tag we can't read.
pub async fn check(current: &Version) -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        // GitHub rejects requests without one.
        .user_agent(concat!("Kagerou/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;

    let response = tokio::time::timeout(REQUEST_TIMEOUT, client.get(RELEASES_ENDPOINT).send())
        .await
        .ok()?
        .ok()?;
    if !response.status().is_success() {
        return None;
    }

    let release: GithubRelease = tokio::time::timeout(REQUEST_TIMEOUT, response.json())
        .await
        .ok()?
        .ok()?;
    newer_than(current, &release)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> GithubRelease {
        GithubRelease {
            tag_name: tag.to_string(),
            html_url: format!("https://github.com/binido/Kagerou/releases/tag/{tag}"),
        }
    }

    fn current() -> Version {
        Version::parse("0.2.0").unwrap()
    }

    #[test]
    fn a_higher_tag_is_an_update() {
        let found = newer_than(&current(), &release("v0.3.0")).unwrap();
        assert_eq!(found.version, "0.3.0");
        assert_eq!(
            found.url,
            "https://github.com/binido/Kagerou/releases/tag/v0.3.0"
        );
    }

    #[test]
    fn the_v_prefix_is_optional() {
        assert!(newer_than(&current(), &release("0.3.0")).is_some());
    }

    #[test]
    fn the_running_version_and_older_ones_are_not_updates() {
        assert_eq!(newer_than(&current(), &release("v0.2.0")), None);
        assert_eq!(newer_than(&current(), &release("v0.1.9")), None);
        assert_eq!(newer_than(&current(), &release("v0.2.0-alpha.1")), None);
    }

    #[test]
    fn a_prerelease_of_a_higher_version_still_counts() {
        let found = newer_than(&current(), &release("v0.3.0-alpha.1")).unwrap();
        assert_eq!(found.version, "0.3.0-alpha.1");
    }

    #[test]
    fn an_unreadable_tag_is_ignored_rather_than_guessed_at() {
        assert_eq!(newer_than(&current(), &release("nightly")), None);
        assert_eq!(newer_than(&current(), &release("v1.2")), None);
        assert_eq!(newer_than(&current(), &release("")), None);
    }

    /// The version Tauri hands us is 0.0.0 until the first release is cut, so
    /// every real tag must read as newer — including a pre-release.
    #[test]
    fn anything_released_beats_the_unreleased_default() {
        let unreleased = Version::parse("0.0.0").unwrap();
        assert!(newer_than(&unreleased, &release("v0.1.0")).is_some());
        assert!(newer_than(&unreleased, &release("v0.1.0-alpha.1")).is_some());
    }
}
