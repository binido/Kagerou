use super::*;
use crate::storage::models::{NewProfile, Protocol};
use crate::usecase::events::test_support::RecordedEvents;

fn core() -> TestCore {
    TestCore::new(SidecarLauncher {
        binary_path: "/nonexistent/sing-box".into(),
        run_dir: std::env::temp_dir(),
        system_proxy_port: 2081,
    })
}

fn db_with(ids: &[&str]) -> Db {
    let db = Db::open_in_memory().unwrap();
    for id in ids {
        profiles::insert(
            &db,
            &NewProfile {
                id: (*id).into(),
                name: (*id).into(),
                region: "r".into(),
                protocol: Protocol::VLESS,
                origin: "local".into(),
                group_id: "default".into(),
                source_id: None,
                key: format!("vless://uuid@1.2.3.4:443#{id}"),
            },
        )
        .unwrap();
    }
    db
}

fn good(millis: u32) -> Measured {
    Measured::Outcome(TestOutcome::Latency { millis })
}

fn progress(events: &RecordedEvents) -> Vec<(String, TestOutcome, usize, usize)> {
    events
        .all()
        .into_iter()
        .filter_map(|event| match event {
            AppEvent::TestProgress(p) => Some((p.profile_id, p.result.outcome, p.done, p.total)),
            _ => None,
        })
        .collect()
}

fn finished(events: &RecordedEvents) -> Option<TestFinished> {
    events.all().into_iter().find_map(|event| match event {
        AppEvent::TestFinished(f) => Some(f),
        _ => None,
    })
}

#[tokio::test]
async fn a_run_reports_every_profile_in_order_and_then_finishes() {
    let db = db_with(&["a", "b"]);
    let core = core();
    let events = RecordedEvents::default();
    let cancel = begin_run(&core).unwrap();

    run_group(
        &db,
        &core,
        &events,
        cancel,
        vec!["a".into(), "b".into()],
        |_| async { good(42) },
    )
    .await;

    assert_eq!(
        progress(&events),
        vec![
            ("a".to_string(), TestOutcome::Latency { millis: 42 }, 1, 2),
            ("b".to_string(), TestOutcome::Latency { millis: 42 }, 2, 2),
        ]
    );
    assert_eq!(
        finished(&events),
        Some(TestFinished {
            done: 2,
            total: 2,
            cancelled: false
        })
    );
    assert_eq!(
        profiles::get(&db, "a").unwrap().url.outcome,
        TestOutcome::Latency { millis: 42 }
    );
}

#[tokio::test]
async fn cancelling_stops_the_run_and_says_so() {
    let db = db_with(&["a", "b", "c"]);
    let core = core();
    let events = RecordedEvents::default();
    let cancel = begin_run(&core).unwrap();
    core.cancel_run();

    run_group(
        &db,
        &core,
        &events,
        cancel,
        vec!["a".into(), "b".into(), "c".into()],
        |_| async { good(42) },
    )
    .await;

    assert!(progress(&events).is_empty(), "nothing was measured");
    assert_eq!(
        finished(&events),
        Some(TestFinished {
            done: 0,
            total: 3,
            cancelled: true
        })
    );
}

#[tokio::test]
async fn a_core_that_would_not_come_up_leaves_the_stored_result_alone() {
    let db = db_with(&["a"]);
    profiles::set_test_outcome(&db, "a", TestOutcome::Latency { millis: 120 }).unwrap();
    let core = core();
    let events = RecordedEvents::default();
    let cancel = begin_run(&core).unwrap();

    run_group(&db, &core, &events, cancel, vec!["a".into()], |_| async {
        Measured::CoreUnavailable
    })
    .await;

    assert_eq!(
        progress(&events),
        vec![("a".to_string(), TestOutcome::NotTested, 1, 1)],
        "the profile is reported as untested, not as failing"
    );
    assert_eq!(
        profiles::get(&db, "a").unwrap().url.outcome,
        TestOutcome::Latency { millis: 120 },
        "the run's own problem must not overwrite what the profile last measured"
    );
}

#[tokio::test]
async fn a_finished_run_frees_the_slot_for_the_next_one() {
    let db = db_with(&["a"]);
    let core = core();
    let events = RecordedEvents::default();
    let cancel = begin_run(&core).unwrap();
    assert!(
        begin_run(&core).is_err(),
        "a second run must not point the selector at a profile the first one is measuring"
    );

    run_group(&db, &core, &events, cancel, vec!["a".into()], |_| async {
        good(10)
    })
    .await;

    assert!(begin_run(&core).is_ok());
}

#[test]
fn a_core_is_idle_only_after_the_timeout_has_passed() {
    let now = Instant::now();
    let timeout = Duration::from_secs(30);

    assert!(!is_idle(Some(now - Duration::from_secs(5)), now, timeout));
    assert!(is_idle(Some(now - Duration::from_secs(31)), now, timeout));
    assert!(
        is_idle(None, now, timeout),
        "a core with no recorded request outlived whatever started it"
    );
}
