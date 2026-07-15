// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{classify_method, MethodAccessLevel};

#[test]
fn health_methods_are_public() {
    assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
    assert_eq!(
        classify_method("health.liveness"),
        MethodAccessLevel::Public
    );
}

#[test]
fn identity_is_public() {
    assert_eq!(classify_method("identity.get"), MethodAccessLevel::Public);
}

#[test]
fn capabilities_list_is_public() {
    assert_eq!(
        classify_method("capabilities.list"),
        MethodAccessLevel::Public
    );
    assert_eq!(
        classify_method("capability.list"),
        MethodAccessLevel::Public
    );
}

#[test]
fn auth_introspection_is_public() {
    assert_eq!(classify_method("auth.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.mode"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.peer_info"), MethodAccessLevel::Public);
}

#[test]
fn lifecycle_status_is_public() {
    assert_eq!(
        classify_method("lifecycle.status"),
        MethodAccessLevel::Public
    );
}

#[test]
fn graph_methods_are_local_trusted() {
    assert_eq!(
        classify_method("graph.execute"),
        MethodAccessLevel::LocalTrusted
    );
    assert_eq!(
        classify_method("graph.save"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn composition_methods_are_local_trusted() {
    assert_eq!(
        classify_method("composition.deploy"),
        MethodAccessLevel::LocalTrusted
    );
    assert_eq!(
        classify_method("composition.status"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn deploy_methods_are_local_trusted() {
    assert_eq!(
        classify_method("deploy.start"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn non_orchestration_methods_are_protected() {
    assert_eq!(
        classify_method("capability.call"),
        MethodAccessLevel::Protected
    );
    assert_eq!(
        classify_method("neural_api.weight_health"),
        MethodAccessLevel::Protected
    );
}

#[test]
fn empty_method_is_protected() {
    assert_eq!(classify_method(""), MethodAccessLevel::Protected);
}
