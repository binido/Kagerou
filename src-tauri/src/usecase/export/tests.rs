use super::*;
use crate::storage::models::{Protocol, TestOutcome};

const TOKYO: &str = "vless://uuid-a@1.2.3.4:443?security=tls&type=tcp#Tokyo";
const VMESS: &str = "vmess://eyJ2IjoiMiIsInBzIjoiT2xkIiwiYWRkIjoiMS4yLjMuNCIsInBvcnQiOiI0NDMiLCJpZCI6InV1aWQiLCJhaWQiOiIwIiwibmV0IjoidGNwIn0=";

fn profile(name: &str, key: &str) -> Profile {
    Profile {
        id: "p1".into(),
        name: name.into(),
        region: String::new(),
        protocol: Protocol::VLESS,
        origin: "local".into(),
        group_id: "default".into(),
        source_id: None,
        selected: false,
        url: TestOutcome::NotTested.into(),
        key: key.into(),
    }
}

#[test]
fn a_renamed_profile_is_shared_under_its_new_name() {
    let link = share_link(&profile("Home", TOKYO));
    let parsed = subscription::parse_uri(&link).unwrap();
    assert_eq!(parsed.name(), "Home");

    let mut original = subscription::parse_uri(TOKYO).unwrap();
    original.set_name("Home");
    assert_eq!(parsed, original, "only the name changes");
}

#[test]
fn a_vmess_name_is_replaced_inside_the_encoded_body() {
    let link = share_link(&profile("New", VMESS));
    assert_eq!(subscription::parse_uri(&link).unwrap().name(), "New");
}

#[test]
fn a_key_that_no_longer_parses_is_shared_as_stored() {
    assert_eq!(share_link(&profile("X", " not a link ")), "not a link");
}

#[test]
fn a_qr_code_is_an_svg_of_the_share_link() {
    let db = Db::open_in_memory().unwrap();
    profiles::insert(
        &db,
        &crate::storage::models::NewProfile {
            id: "p1".into(),
            name: "Tokyo".into(),
            region: String::new(),
            protocol: Protocol::VLESS,
            origin: "local".into(),
            group_id: "default".into(),
            source_id: None,
            key: TOKYO.into(),
        },
    )
    .unwrap();
    let svg = qr_svg(&db, "p1").unwrap();
    assert!(svg.contains("<svg"), "{svg}");
    assert!(matches!(
        qr_svg(&db, "missing").unwrap_err(),
        ExportError::Storage(StorageError::NotFound)
    ));
}

#[test]
fn a_missing_profile_fails_the_whole_export() {
    let db = Db::open_in_memory().unwrap();
    assert!(matches!(
        share_links(&db, &["missing".into()]).unwrap_err(),
        ExportError::Storage(StorageError::NotFound)
    ));
}

#[test]
fn the_suggested_file_name_is_safe_on_every_desktop() {
    assert_eq!(suggested_file_name("Work: EU/US?"), "Work_ EU_US_.txt");
    assert_eq!(suggested_file_name("   "), "kagerou.txt");
    assert_eq!(suggested_file_name("Работа"), "Работа.txt");
}
