#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

#[test]
fn primal_id_valid() {
    assert!(PrimalId::new("beardog-local").is_ok());
    assert!(PrimalId::new("songbird_v2").is_ok());
    assert!(PrimalId::new("tower123").is_ok());
}

#[test]
fn primal_id_invalid() {
    assert!(PrimalId::new("").is_err());
    assert!(PrimalId::new("has spaces").is_err());
    assert!(PrimalId::new("has/slash").is_err());
}

#[test]
fn primal_id_error_types() {
    assert!(matches!(PrimalId::new(""), Err(IdError::Empty)));
    assert!(matches!(
        PrimalId::new("bad@char"),
        Err(IdError::InvalidCharacters)
    ));
}

#[test]
fn primal_id_unchecked() {
    let id = PrimalId::new_unchecked("trusted");
    assert_eq!(id.as_str(), "trusted");
}

#[test]
fn primal_id_serialization() {
    let id = PrimalId::new("beardog").expect("valid");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, r#""beardog""#);
    let loaded: PrimalId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.as_str(), "beardog");
}

#[test]
fn primal_id_into_string() {
    let id = PrimalId::new("test").expect("valid");
    let s: String = id.into_string();
    assert_eq!(s, "test");
}

#[test]
fn primal_id_display() {
    let id = PrimalId::new("beardog").expect("valid");
    assert_eq!(format!("{id}"), "beardog");
}

#[test]
fn primal_id_as_ref() {
    let id = PrimalId::new("x").expect("valid");
    let s: &str = id.as_ref();
    assert_eq!(s, "x");
}

#[test]
fn family_id_display() {
    let family = FamilyId::new("iidn");
    assert_eq!(format!("{family}"), "iidn");
}

#[test]
fn family_id_generate() {
    let id = FamilyId::generate();
    assert!(!id.as_str().is_empty());
    assert_eq!(id.as_str().len(), 8);
}

#[test]
fn family_id_from_env() {
    let id = FamilyId::from_env_or_override(Some("env-family"));
    assert!(id.is_some());
    assert_eq!(id.unwrap().as_str(), "env-family");
}

#[test]
fn tower_id_display() {
    let tower = TowerId::new("tower-alpha");
    assert_eq!(format!("{tower}"), "tower-alpha");
}

#[test]
fn session_id_from_uuid() {
    let uuid = uuid::Uuid::new_v4();
    let session = SessionId::from_uuid(uuid);
    assert_eq!(session.uuid(), &uuid);
}

#[test]
fn session_id_default() {
    let _ = SessionId::default();
}

#[test]
fn session_id_unique() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();
    assert_ne!(id1, id2);
}

#[test]
fn endpoint_valid() {
    assert!(Endpoint::new("http://localhost:9000").is_ok());
    assert!(Endpoint::new("https://192.0.2.10:8080").is_ok());
}

#[test]
fn endpoint_invalid() {
    assert!(Endpoint::new("not-a-url").is_err());
    assert!(Endpoint::new("").is_err());
}

#[test]
fn endpoint_join() {
    let base = Endpoint::new("http://localhost:9000").expect("valid");
    let api = base.join("api/v1/health").expect("join");
    assert_eq!(api.as_str(), "http://localhost:9000/api/v1/health");
}

#[test]
fn endpoint_url() {
    let ep = Endpoint::new("http://example.com").expect("valid");
    assert_eq!(ep.url().host_str(), Some("example.com"));
}

#[test]
fn endpoint_serialization() {
    let ep = Endpoint::new("http://localhost:8080").expect("valid");
    let json = serde_json::to_string(&ep).expect("serialize");
    let loaded: Endpoint = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.as_str(), ep.as_str());
}

#[test]
fn id_error_display() {
    let e = IdError::Empty;
    assert!(e.to_string().contains("empty"));
    let e2 = IdError::InvalidCharacters;
    assert!(e2.to_string().contains("invalid"));
}

#[test]
fn primal_id_from_conversion() {
    let id = PrimalId::new("beardog").expect("valid");
    let s: String = id.into_string();
    let id2 = PrimalId::new(&s).expect("valid");
    assert_eq!(id2.as_str(), "beardog");
}

#[test]
fn primal_id_from_into_string() {
    let id = PrimalId::new("test-id").expect("valid");
    let s: String = id.into_string();
    assert_eq!(s, "test-id");
}

#[test]
fn primal_id_deserialize_invalid() {
    let result: Result<PrimalId, _> = serde_json::from_str(r#""bad@char""#);
    assert!(result.is_err());
}

#[test]
fn primal_id_deserialize_empty() {
    let result: Result<PrimalId, _> = serde_json::from_str(r#""""#);
    assert!(result.is_err());
}

#[test]
fn family_id_from_conversion() {
    let family = FamilyId::new("iidn");
    let s: String = family.into_string();
    assert_eq!(s, "iidn");
}

#[test]
fn family_id_serialization() {
    let family = FamilyId::new("test-family");
    let json = serde_json::to_string(&family).expect("serialize");
    assert_eq!(json, r#""test-family""#);
    let loaded: FamilyId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.as_str(), "test-family");
}

#[test]
fn family_id_as_ref() {
    let family = FamilyId::new("ref-test");
    let s: &str = family.as_ref();
    assert_eq!(s, "ref-test");
}

#[test]
fn family_id_get_or_create_from_env() {
    let id = FamilyId::get_or_create_with(Some("env-created"));
    assert_eq!(id.as_str(), "env-created");
}

#[test]
fn tower_id_serialization() {
    let tower = TowerId::new("tower-1");
    let json = serde_json::to_string(&tower).expect("serialize");
    assert_eq!(json, r#""tower-1""#);
    let loaded: TowerId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(loaded.as_str(), "tower-1");
}

#[test]
fn tower_id_from_conversion() {
    let tower = TowerId::new("tower-alpha");
    let s: String = tower.into();
    assert_eq!(s, "tower-alpha");
}

#[test]
fn tower_id_as_str() {
    let tower = TowerId::new("tower-x");
    assert_eq!(tower.as_str(), "tower-x");
}

#[test]
fn session_id_display() {
    let uuid = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let session = SessionId::from_uuid(uuid);
    let s = format!("{session}");
    assert!(s.contains("550e8400"));
}

#[test]
fn session_id_serialization() {
    let session = SessionId::new();
    let json = serde_json::to_string(&session).expect("serialize");
    let loaded: SessionId = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(session.uuid(), loaded.uuid());
}

#[test]
fn endpoint_display() {
    let ep = Endpoint::new("http://example.com").expect("valid");
    let s = format!("{ep}");
    assert!(s.contains("example.com"));
}

#[test]
fn endpoint_as_ref() {
    let ep = Endpoint::new("http://localhost:8080").expect("valid");
    let s: &str = ep.as_ref();
    assert!(s.starts_with("http://"));
}

#[test]
fn id_error_invalid_url() {
    let err = Endpoint::new("not-a-valid-url").unwrap_err();
    let id_err: IdError = err.into();
    assert!(format!("{id_err}").to_lowercase().contains("url"));
}
