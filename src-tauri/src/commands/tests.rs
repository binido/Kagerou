use super::test_core_is_idle;
use std::time::{Duration, Instant};

#[test]
fn a_test_core_is_idle_only_after_the_timeout_has_passed() {
    let now = Instant::now();
    let timeout = Duration::from_secs(30);

    assert!(!test_core_is_idle(
        Some(now - Duration::from_secs(5)),
        now,
        timeout
    ));
    assert!(test_core_is_idle(
        Some(now - Duration::from_secs(31)),
        now,
        timeout
    ));
    assert!(
        test_core_is_idle(None, now, timeout),
        "a core with no recorded test outlived whatever started it"
    );
}
