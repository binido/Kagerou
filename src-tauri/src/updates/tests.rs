
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
