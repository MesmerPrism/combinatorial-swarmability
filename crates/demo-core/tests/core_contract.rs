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
fn deterministic_replay_preserves_member_subgroup_and_swarm_actions() {
    let mut core = DemoCore::new(2026);

    let revision = core.public_state().selection_revision;
    let _ = core.dispatch(set_behavior(CollectiveBehavior::Cohere, revision));
    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Subgroup,
    });
    let revision = core.public_state().selection_revision;
    let _ = core.dispatch(adjust(0.15, revision));
    let _ = core.dispatch(SemanticAction::SetScope {
        scope: TargetScope::Swarm,
    });
    let revision = core.public_state().selection_revision;
    let _ = core.dispatch(set_behavior(CollectiveBehavior::Disperse, revision));
    let _ = core.dispatch(SemanticAction::Start);
    assert_eq!(core.advance_elapsed(128), 8);
    assert_eq!(core.advance_elapsed(48), 3);
    let _ = core.dispatch(SemanticAction::Pause);

    let expected_snapshot = core.snapshot_json().expect("snapshot serializes");
    let tape = core.replay_json().expect("replay tape serializes");
    let replayed = DemoCore::from_replay_json(&tape).expect("replay tape reconstructs");
    assert_eq!(
        expected_snapshot,
        replayed.snapshot_json().expect("snapshot serializes")
    );
    assert_eq!(
        tape,
        replayed.replay_json().expect("replay tape serializes")
    );
    let state = replayed.public_state();
    assert_eq!(state.scope, TargetScope::Swarm);
    assert_eq!(state.tick, 11);
    assert_eq!(state.behavior_counts.cohere, 0);
    assert_eq!(state.behavior_counts.disperse, MEMBER_COUNT);
    assert!(state.replay_available);
    assert_eq!(state.replay_step_count, 11);
    assert_eq!(state.replay_event_count, 8);
}

#[test]
fn deterministic_replay_rejects_damage_and_unbound_snapshots() {
    let unknown_field = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[],
        "private_commentary":"reject"
    }"#;
    assert!(DemoCore::from_replay_json(unknown_field).is_err());

    let advance_while_paused = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[{"kind":"advance_steps","steps":1}]
    }"#;
    assert!(DemoCore::from_replay_json(advance_while_paused).is_err());

    let rejected_action = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[{
            "kind":"action",
            "action":{"type":"adjust_speed","delta":0.1,"expected_selection_revision":99}
        }]
    }"#;
    assert!(DemoCore::from_replay_json(rejected_action).is_err());

    let zero_steps = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[{"kind":"advance_steps","steps":0}]
    }"#;
    assert!(DemoCore::from_replay_json(zero_steps).is_err());

    let consecutive_advances = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[
            {"kind":"action","action":{"type":"start"}},
            {"kind":"advance_steps","steps":1},
            {"kind":"advance_steps","steps":1},
            {"kind":"action","action":{"type":"pause"}}
        ]
    }"#;
    assert!(DemoCore::from_replay_json(consecutive_advances).is_err());

    let tape_ending_running = r#"{
        "schema":"combinatorial.swarmability.replay.v1",
        "initial_seed":2026,
        "events":[{"kind":"action","action":{"type":"start"}}]
    }"#;
    assert!(DemoCore::from_replay_json(tape_ending_running).is_err());

    let excessive_events = serde_json::json!({
        "schema": "combinatorial.swarmability.replay.v1",
        "initial_seed": 2026,
        "events": (0..4_097)
            .map(|_| serde_json::json!({"kind": "action", "action": {"type": "start"}}))
            .collect::<Vec<_>>()
    })
    .to_string();
    assert!(DemoCore::from_replay_json(&excessive_events).is_err());

    let excessive_bytes = " ".repeat(2_000_001);
    assert!(DemoCore::from_replay_json(&excessive_bytes).is_err());

    let snapshot = DemoCore::new(2026)
        .snapshot_json()
        .expect("snapshot serializes");
    let restored = DemoCore::from_snapshot_json(&snapshot).expect("snapshot restores");
    assert!(restored.replay_json().is_err());
    assert!(!restored.public_state().replay_available);

    let mut running = DemoCore::new(2026);
    let _ = running.dispatch(SemanticAction::Start);
    assert!(running.replay_json().is_err());
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
