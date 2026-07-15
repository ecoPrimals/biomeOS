// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_domains::*;

#[test]
fn test_capability_to_provider_runtime_overrides_bootstrap() {
    clear_runtime_capability_registry();

    register_capability_provider("security", "live-security-primal");
    assert_eq!(
        capability_to_provider("security"),
        Some("live-security-primal".into())
    );
    assert_eq!(
        capability_to_provider_fallback("security"),
        Some("beardog")
    );

    clear_runtime_capability_registry();
}

#[test]
fn test_capability_to_provider_runtime_prefix_from_dotted_capability() {
    clear_runtime_capability_registry();

    register_capability_provider("crypto.sign", "live-crypto-primal");
    assert_eq!(
        capability_to_provider("crypto.encrypt"),
        Some("live-crypto-primal".into())
    );

    clear_runtime_capability_registry();
}

#[test]
fn test_register_capability_provider_ignores_empty() {
    clear_runtime_capability_registry();

    register_capability_provider("", "beardog");
    register_capability_provider("crypto", "");
    assert_eq!(capability_to_provider("crypto"), Some("beardog".into()));

    clear_runtime_capability_registry();
}

#[test]
fn test_capability_domain_struct_access() {
    let domain = &BOOTSTRAP_CAPABILITY_HINTS[0];
    assert_eq!(domain.provider, "beardog");
    assert!(domain.capabilities.contains(&"security"));
    assert!(domain.capabilities.contains(&"crypto"));
}

#[test]
fn test_capability_to_provider_unknown() {
    assert_eq!(capability_to_provider_fallback("unknown"), None);
    assert_eq!(capability_to_provider_fallback("random.capability"), None);
    assert_eq!(capability_to_provider_fallback(""), None);
}
