// SPDX-License-Identifier: AGPL-3.0-or-later

//! BTSP family-scoped socket detection for composition broker routing.

use std::path::Path;

use biomeos_core::btsp_client;

#[test]
fn family_scoped_socket_detected() {
    let socket = Path::new("/run/membrane/beardog-west-001.sock");
    assert!(
        btsp_client::is_family_scoped_socket(socket),
        "beardog-west-001.sock is family-scoped"
    );
}

#[test]
fn non_family_socket_not_detected() {
    let socket = Path::new("/run/membrane/beardog.sock");
    assert!(
        !btsp_client::is_family_scoped_socket(socket),
        "beardog.sock is NOT family-scoped (dev mode)"
    );
}

#[test]
fn nestgate_family_scoped() {
    let socket = Path::new("/run/membrane/nestgate-alpha-42.sock");
    assert!(btsp_client::is_family_scoped_socket(socket));
}

#[test]
fn rhizocrypt_family_scoped() {
    let socket = Path::new("/run/membrane/rhizocrypt-nest-west.sock");
    assert!(btsp_client::is_family_scoped_socket(socket));
}

#[test]
fn sweetgrass_family_scoped() {
    let socket = Path::new("/run/membrane/sweetgrass-nest-west.sock");
    assert!(btsp_client::is_family_scoped_socket(socket));
}

#[test]
fn non_sock_extension_not_detected() {
    let path = Path::new("/run/membrane/beardog-west-001.pid");
    assert!(!btsp_client::is_family_scoped_socket(path));
}

#[test]
fn extract_family_id_from_socket() {
    let socket = Path::new("/run/membrane/nestgate-west-001.sock");
    let fid = btsp_client::extract_family_id(socket);
    assert_eq!(fid.as_deref(), Some("west-001"));
}

#[test]
fn development_mode_without_family_id() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
        ],
        || {
            let mode = btsp_client::security_mode();
            assert!(
                matches!(mode, btsp_client::SecurityMode::Development),
                "no FAMILY_ID → Development mode"
            );
        },
    );
}

#[test]
fn production_mode_with_family_id() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("west-001")),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            let mode = btsp_client::security_mode();
            assert!(
                matches!(mode, btsp_client::SecurityMode::Production { .. }),
                "FAMILY_ID set → Production mode"
            );
        },
    );
}

#[test]
fn btsp_enforce_off_without_family() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_BTSP_ENFORCE", None::<&str>),
        ],
        || {
            assert!(!btsp_client::btsp_enforce(), "no family → no enforcement");
        },
    );
}

#[test]
fn btsp_enforce_on_with_family() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("nest-001")),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_BTSP_ENFORCE", None::<&str>),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(
                btsp_client::btsp_enforce(),
                "family + no override → enforced"
            );
        },
    );
}

#[test]
fn btsp_enforce_override_disables() {
    temp_env::with_vars(
        [
            ("FAMILY_ID", Some("nest-001")),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_BTSP_ENFORCE", Some("0")),
            ("BIOMEOS_INSECURE", None::<&str>),
        ],
        || {
            assert!(
                !btsp_client::btsp_enforce(),
                "BIOMEOS_BTSP_ENFORCE=0 disables"
            );
        },
    );
}
