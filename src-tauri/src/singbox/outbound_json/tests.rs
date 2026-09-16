use super::*;
use crate::subscription::model::{Hysteria2Outbound, VlessOutbound};

#[test]
fn vless_with_reality_includes_the_reality_block() {
    let parsed = ParsedOutbound::Vless(VlessOutbound {
        name: "n".into(),
        server: "example.com".into(),
        port: 443,
        uuid: "uuid".into(),
        flow: Some("xtls-rprx-vision".into()),
        network: "tcp".into(),
        tls: true,
        sni: Some("cdn.example.com".into()),
        ws_path: None,
        ws_host: None,
        grpc_service_name: None,
        reality_public_key: Some("pbk".into()),
        reality_short_id: Some("sid".into()),
        fingerprint: None,
    });
    let json = to_singbox_outbound(&parsed, "profile-1");
    assert_eq!(json["type"], "vless");
    assert_eq!(json["tag"], "profile-1");
    assert_eq!(json["tls"]["reality"]["public_key"], "pbk");
    assert_eq!(json["tls"]["reality"]["short_id"], "sid");
    assert_eq!(json["flow"], "xtls-rprx-vision");
    assert_eq!(json["tls"]["utls"]["fingerprint"], "chrome");
    assert!(
        json.get("transport").is_none(),
        "tcp network should not emit a transport block"
    );
}

#[test]
fn a_plaintext_vless_outbound_omits_the_tls_block_entirely() {
    let parsed = ParsedOutbound::Vless(VlessOutbound {
        name: "n".into(),
        server: "127.0.0.1".into(),
        port: 18443,
        uuid: "uuid".into(),
        flow: None,
        network: "tcp".into(),
        tls: false,
        sni: None,
        ws_path: None,
        ws_host: None,
        grpc_service_name: None,
        reality_public_key: None,
        reality_short_id: None,
        fingerprint: None,
    });
    let json = to_singbox_outbound(&parsed, "profile-1");
    assert!(
        json.get("tls").is_none(),
        "sing-box panics on a disabled-but-present tls block: {json}"
    );
}

#[test]
fn a_grpc_service_name_reaches_the_transport_block() {
    let with_service = transport_block("grpc", None, None, Some("svc")).unwrap();
    assert_eq!(with_service["type"], "grpc");
    assert_eq!(with_service["service_name"], "svc");

    // Links without one are common and valid: the key must be absent
    // rather than an empty string.
    let without = transport_block("grpc", None, None, None).unwrap();
    assert_eq!(without["type"], "grpc");
    assert!(without.get("service_name").is_none());
}

#[test]
fn hysteria2_includes_obfs_only_when_present() {
    let with_obfs = ParsedOutbound::Hysteria2(Hysteria2Outbound {
        name: "n".into(),
        server: "s".into(),
        port: 443,
        password: "pw".into(),
        sni: None,
        insecure: true,
        obfs: Some("salamander".into()),
        obfs_password: Some("op".into()),
    });
    let json = to_singbox_outbound(&with_obfs, "t");
    assert_eq!(json["obfs"]["type"], "salamander");
    assert_eq!(json["tls"]["insecure"], true);

    let without_obfs = ParsedOutbound::Hysteria2(Hysteria2Outbound {
        name: "n".into(),
        server: "s".into(),
        port: 443,
        password: "pw".into(),
        sni: None,
        insecure: false,
        obfs: None,
        obfs_password: None,
    });
    let json = to_singbox_outbound(&without_obfs, "t");
    assert!(json.get("obfs").is_none());
}
