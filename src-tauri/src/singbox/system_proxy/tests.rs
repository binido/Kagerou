use super::points_at_local_port;

#[test]
fn a_bare_address_on_our_port_is_ours() {
    assert!(points_at_local_port("127.0.0.1:2080", 2080));
}

#[test]
fn a_per_scheme_list_or_a_scheme_prefix_still_counts() {
    assert!(points_at_local_port(
        "http=127.0.0.1:2080;https=127.0.0.1:2080",
        2080
    ));
    assert!(points_at_local_port("http://127.0.0.1:2080", 2080));
    assert!(points_at_local_port("socks=127.0.0.1:2080", 2080));
}

#[test]
fn a_proxy_the_user_set_up_for_something_else_is_left_alone() {
    assert!(!points_at_local_port("127.0.0.1:20800", 2080));
    assert!(!points_at_local_port("127.0.0.1:8080", 2080));
    assert!(!points_at_local_port("corp-proxy.example:2080", 2080));
    assert!(!points_at_local_port("", 2080));
    assert!(!points_at_local_port(";;=", 2080));
}
