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

/// Reports a GitHub release newer than `current`, if there is one.
///
/// `None` covers every uninteresting outcome alike - no releases yet, nothing
/// newer, no network, a tag that will not parse. Until the project cuts its
/// first release the endpoint answers 404, which is an ordinary "nothing
/// newer" rather than a failure, and an unreachable network is not something
/// to put in front of the user either.
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
mod tests;
