use super::worth_auto_connecting;

#[test]
fn auto_connect_needs_both_a_profile_to_use_and_profiles_to_pick_from() {
    assert!(worth_auto_connecting("p1", 3));
    assert!(
        !worth_auto_connecting("", 3),
        "nothing was selected last session"
    );
    assert!(
        !worth_auto_connecting("p1", 0),
        "the selected profile is gone, so there is nothing to connect to"
    );
}
