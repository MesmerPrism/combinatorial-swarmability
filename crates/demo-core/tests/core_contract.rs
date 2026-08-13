//! Integration coverage for the deterministic core and Matter handoff.

use combinatorial_swarmability_demo_core::{
    ActionCode, CollectiveBehavior, DemoCore, SemanticAction, TargetScope, FRAME_ROW_WIDTH,
    MEMBER_COUNT,
};
use sha2::{Digest, Sha256};

fn adjust(delta: f32, revision: u64) -> SemanticAction {
    SemanticAction::AdjustSpeed {
        delta,
        expected_selection_revision: revision,
    }
}

fn set_behavior(behavior: CollectiveBehavior, revision: u64) -> SemanticAction {
    SemanticAction::SetBehavior {
        behavior,
        expected_selection_revision: revision,
    }
}

#[test]
fn same_speed_action_resolves_member_subgroup_and_swarm_scopes() {
    let mut core = DemoCore::new(7);
    let revision = core.public_state().selection_revision;
    let member = core.dispatch(adjust(0.1, revision));
    assert!(member.accepted);
    assert_eq!(member.changed_member_ids, vec![0]);

    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Subgroup,
    });
    let revision = core.public_state().selection_revision;
    let subgroup = core.dispatch(adjust(0.1, revision));
    assert_eq!(subgroup.changed_member_ids, vec![0, 1, 2, 3, 4, 5]);

    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Swarm,
    });
    let revision = core.public_state().selection_revision;
    let swarm = core.dispatch(adjust(0.1, revision));
    assert_eq!(swarm.changed_member_ids.len(), MEMBER_COUNT);
}

#[test]
fn same_collective_rule_resolves_member_subgroup_and_swarm_scopes() {
    let mut core = DemoCore::new(17);
    let revision = core.public_state().selection_revision;
    let member = core.dispatch(set_behavior(CollectiveBehavior::Cohere, revision));
    assert_eq!(member.changed_member_ids, vec![0]);
    assert_eq!(core.public_state().behavior_counts.cohere, 1);

    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Subgroup,
    });
    let revision = core.public_state().selection_revision;
    let subgroup = core.dispatch(set_behavior(CollectiveBehavior::Disperse, revision));
    assert_eq!(subgroup.changed_member_ids, vec![0, 1, 2, 3, 4, 5]);
    assert_eq!(core.public_state().behavior_counts.disperse, 6);

    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Swarm,
    });
    let revision = core.public_state().selection_revision;
    let swarm = core.dispatch(set_behavior(CollectiveBehavior::Flock, revision));
    assert_eq!(swarm.changed_member_ids.len(), MEMBER_COUNT);
    assert_eq!(core.public_state().behavior_counts.flock, MEMBER_COUNT);
}

#[test]
fn invalid_empty_and_stale_selections_fail_closed() {
    let mut core = DemoCore::new(11);
    let invalid = core.dispatch(SemanticAction::SelectMember { member_id: 99 });
    assert!(!invalid.accepted);
    assert_eq!(invalid.code, ActionCode::InvalidMember);

    let stale_revision = core.public_state().selection_revision;
    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Subgroup,
    });
    let stale = core.dispatch(adjust(0.1, stale_revision));
    assert!(!stale.accepted);
    assert_eq!(stale.code, ActionCode::StaleSelection);

    let _ = core.dispatch(SemanticAction::ClearSubgroup);
    let revision = core.public_state().selection_revision;
    let empty = core.dispatch(adjust(0.1, revision));
    assert!(!empty.accepted);
    assert_eq!(empty.code, ActionCode::EmptySelection);

    let empty_behavior = core.dispatch(set_behavior(CollectiveBehavior::Cohere, revision));
    assert!(!empty_behavior.accepted);
    assert_eq!(empty_behavior.code, ActionCode::EmptySelection);
}

#[test]
fn collective_rules_change_pairwise_swarm_structure() {
    let mut cohere = DemoCore::new(303);
    let initial_distance = mean_pair_distance(&cohere.frame_rows().expect("frame rows project"));
    let _ = cohere.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Swarm,
    });
    let revision = cohere.public_state().selection_revision;
    let _ = cohere.dispatch(set_behavior(CollectiveBehavior::Cohere, revision));
    let _ = cohere.dispatch(SemanticAction::Start);
    for _ in 0..20 {
        let _ = cohere.advance_elapsed(128);
    }
    let cohere_distance = mean_pair_distance(&cohere.frame_rows().expect("frame rows project"));

    let mut disperse = DemoCore::new(303);
    let _ = disperse.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Swarm,
    });
    let revision = disperse.public_state().selection_revision;
    let _ = disperse.dispatch(set_behavior(CollectiveBehavior::Disperse, revision));
    let _ = disperse.dispatch(SemanticAction::Start);
    for _ in 0..20 {
        let _ = disperse.advance_elapsed(128);
    }
    let disperse_distance = mean_pair_distance(&disperse.frame_rows().expect("frame rows project"));

    assert!(cohere_distance < initial_distance * 0.90);
    assert!(disperse_distance > initial_distance * 1.05);
    assert!(disperse_distance > cohere_distance * 1.20);
}

#[test]
fn pause_step_reset_and_seeded_restart_are_deterministic() {
    let mut core = DemoCore::new(23);
    assert_eq!(core.advance_elapsed(160), 0);
    let _ = core.dispatch(SemanticAction::Start);
    assert_eq!(core.advance_elapsed(160), 8);
    assert_eq!(core.public_state().tick, 8);
    let running_step = core.dispatch(SemanticAction::Step);
    assert_eq!(running_step.code, ActionCode::StepRequiresPause);
    let _ = core.dispatch(SemanticAction::Pause);
    let _ = core.dispatch(SemanticAction::Step);
    assert_eq!(core.public_state().tick, 9);
    let _ = core.dispatch(SemanticAction::Reset);
    assert_eq!(core.public_state().tick, 0);
    assert!(!core.public_state().running);

    let _ = core.dispatch(SemanticAction::RestartSeed { seed: 44 });
    let restarted = core.snapshot_json().expect("snapshot serializes");
    let fresh = DemoCore::new(44)
        .snapshot_json()
        .expect("snapshot serializes");
    let mut restarted_value: serde_json::Value =
        serde_json::from_str(&restarted).expect("valid JSON");
    let fresh_value: serde_json::Value = serde_json::from_str(&fresh).expect("valid JSON");
    restarted_value["state_revision"] = fresh_value["state_revision"].clone();
    restarted_value["selection_revision"] = fresh_value["selection_revision"].clone();
    assert_eq!(restarted_value, fresh_value);
}

#[test]
fn actions_and_snapshots_round_trip_strictly() {
    let action = SemanticAction::AdjustSpeed {
        delta: -0.125,
        expected_selection_revision: 17,
    };
    let encoded = serde_json::to_string(&action).expect("action serializes");
    let decoded: SemanticAction = serde_json::from_str(&encoded).expect("action deserializes");
    assert_eq!(decoded, action);
    assert!(
        serde_json::from_str::<SemanticAction>(r#"{"type":"start","unexpected":true}"#).is_err()
    );

    let mut core = DemoCore::new(101);
    let _ = core.dispatch(SemanticAction::Start);
    let _ = core.advance_elapsed(64);
    let _ = core.dispatch(SemanticAction::Pause);
    let snapshot = core.snapshot_json().expect("snapshot serializes");
    let restored = DemoCore::from_snapshot_json(&snapshot).expect("snapshot restores");
    assert_eq!(
        snapshot,
        restored.snapshot_json().expect("snapshot serializes")
    );
}

#[test]
fn matter_payload_and_frame_rows_preserve_scene_identity() {
    let core = DemoCore::new(5);
    let payload = core.render_payload().expect("Matter payload validates");
    assert_eq!(payload.samples.len(), MEMBER_COUNT);
    assert_eq!(payload.source_set_id, "combinatorial-swarmability.scene");
    let rows = core.frame_rows().expect("frame rows project");
    assert_eq!(rows.len(), MEMBER_COUNT * FRAME_ROW_WIDTH);
}

#[test]
fn seeded_golden_hash_matches_fixture() {
    let actions_json = include_str!("../../../tests/fixtures/action-sequence.json");
    let actions: Vec<SemanticAction> =
        serde_json::from_str(actions_json).expect("fixture action sequence parses");
    let mut core = DemoCore::new(2026);
    for action in actions {
        let _ = core.dispatch(action);
    }
    let _ = core.advance_elapsed(96);
    let snapshot = core.snapshot_json().expect("snapshot serializes");
    let actual = format!("{:x}", Sha256::digest(snapshot.as_bytes()));
    let expected = include_str!("../../../tests/fixtures/deterministic-seed-2026.sha256").trim();
    assert_eq!(actual, expected);
}

fn mean_pair_distance(rows: &[f32]) -> f32 {
    let positions = rows
        .chunks_exact(FRAME_ROW_WIDTH)
        .map(|row| (row[1], row[2]))
        .collect::<Vec<_>>();
    let mut total = 0.0;
    let mut pairs = 0_u16;
    for (index, first) in positions.iter().enumerate() {
        for second in &positions[index + 1..] {
            total += ((first.0 - second.0).powi(2) + (first.1 - second.1).powi(2)).sqrt();
            pairs += 1;
        }
    }
    total / f32::from(pairs)
}
