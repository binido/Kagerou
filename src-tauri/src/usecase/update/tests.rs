use super::*;

#[test]
fn a_packaged_build_can_install_updates() {
    for bundle in [
        BundleType::App,
        BundleType::Nsis,
        BundleType::Msi,
        BundleType::AppImage,
        BundleType::Deb,
        BundleType::Rpm,
    ] {
        assert!(installable(false, Some(bundle)));
    }
}

#[test]
fn the_portable_exe_carries_no_bundle_type_and_cannot() {
    assert!(!installable(false, None));
}

#[test]
fn a_dev_build_never_installs_even_where_the_os_reports_a_bundle() {
    assert!(!installable(true, Some(BundleType::App)));
}
