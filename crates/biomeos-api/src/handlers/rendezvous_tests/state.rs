use super::super::*;

// ========== RendezvousState Tests ==========

#[test]
fn test_rendezvous_state_creation() {
    let state = RendezvousState::new();
    assert!(!state.family_id.is_empty());
}

#[test]
fn test_rendezvous_state_clone() {
    let state = RendezvousState::new();
    let cloned = state.clone();
    assert_eq!(cloned.family_id, state.family_id);
}

#[tokio::test]
async fn test_clean_expired_removes_old_slots() {
    let state = RendezvousState::new();

    // Add an expired slot
    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "test".to_string(),
            node_hash: "node1".to_string(),
            lineage_hash: "lineage1".to_string(),
            created_at: 0,
            expires_at: 1, // Expired long ago
            connection_info: None,
        }],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert!(slots.is_empty());
}

#[tokio::test]
async fn test_clean_expired_keeps_valid_slots() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "valid".to_string(),
            node_hash: "node1".to_string(),
            lineage_hash: "lineage1".to_string(),
            created_at: now,
            expires_at: now + 300, // 5 minutes from now
            connection_info: None,
        }],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 1);
    assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
}

#[tokio::test]
async fn test_clean_expired_mixed_slots() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![
            RendezvousSlot {
                encrypted_beacon: "expired".to_string(),
                node_hash: "node1".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: 0,
                expires_at: 1, // Expired
                connection_info: None,
            },
            RendezvousSlot {
                encrypted_beacon: "valid".to_string(),
                node_hash: "node2".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: now,
                expires_at: now + 300, // Valid
                connection_info: None,
            },
        ],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 1);
    assert_eq!(slots["lineage1"].len(), 1);
    assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
}
#[tokio::test]
async fn test_multiple_lineage_groups() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;

    slots.insert(
        "family-a".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "beacon-a1".to_string(),
            node_hash: "node-a1".to_string(),
            lineage_hash: "family-a".to_string(),
            created_at: now,
            expires_at: now + 300,
            connection_info: None,
        }],
    );
    slots.insert(
        "family-b".to_string(),
        vec![
            RendezvousSlot {
                encrypted_beacon: "beacon-b1".to_string(),
                node_hash: "node-b1".to_string(),
                lineage_hash: "family-b".to_string(),
                created_at: now,
                expires_at: now + 300,
                connection_info: None,
            },
            RendezvousSlot {
                encrypted_beacon: "beacon-b2".to_string(),
                node_hash: "node-b2".to_string(),
                lineage_hash: "family-b".to_string(),
                created_at: now,
                expires_at: now + 300,
                connection_info: None,
            },
        ],
    );
    drop(slots);

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 2);
    assert_eq!(slots["family-a"].len(), 1);
    assert_eq!(slots["family-b"].len(), 2);
}

#[test]
fn test_rendezvous_slot_expiry_logic() {
    let slot = RendezvousSlot {
        encrypted_beacon: "x".to_string(),
        node_hash: "n".to_string(),
        lineage_hash: "l".to_string(),
        created_at: 100,
        expires_at: 400,
        connection_info: None,
    };
    assert!(slot.expires_at > slot.created_at);
    assert_eq!(slot.expires_at - slot.created_at, 300);
}
