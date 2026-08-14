//! Integration coverage for the deterministic core and Matter handoff.

use combinatorial_swarmability_demo_core::{
    ActionCode, CollectiveBehavior, CollisionPolicy, DemoCore, DynamicsControlMode, FieldLifetime,
    FieldPolarity, GroupPartitionRule, HandoffDecision, PublicState, SemanticAction, TargetScope,
    DEFAULT_DYNAMICS_RATES, DEFAULT_EXECUTION_SETTINGS, DEFAULT_FORMATION_SCALE,
    DEFAULT_SEMANTIC_QUALITIES, FRAME_ROW_WIDTH, MAX_ACCELERATION_LIMIT, MAX_ACTIVE_LEASES,
    MAX_DYNAMICS_RATE, MAX_FIELD_LIFETIME_STEPS, MAX_FORMATION_SCALE, MAX_GROUPS,
    MAX_LEASE_LIFETIME_STEPS, MAX_PERSONAL_FIELDS, MAX_SEMANTIC_QUALITY, MAX_SPEED_LIMIT,
    MAX_SYNTHETIC_OPERATORS, MEMBER_COUNT, MIN_ACCELERATION_LIMIT, MIN_FORMATION_SCALE,
    MIN_SPEED_LIMIT,
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

fn place_field(
    field_id: u16,
    contributor_id: u8,
    x: f32,
    y: f32,
    polarity: FieldPolarity,
    lifetime: FieldLifetime,
) -> SemanticAction {
    SemanticAction::PlaceField {
        field_id,
        contributor_id,
        x,
        y,
        polarity,
        lifetime,
    }
}

fn set_alignment(rate: f32) -> SemanticAction {
    SemanticAction::SetAlignment { rate }
}

fn set_cohesion(rate: f32) -> SemanticAction {
    SemanticAction::SetCohesion { rate }
}

fn set_separation(rate: f32) -> SemanticAction {
    SemanticAction::SetSeparation { rate }
}

fn set_space(value: f32) -> SemanticAction {
    SemanticAction::SetSpaceQuality { value }
}

fn set_time(value: f32) -> SemanticAction {
    SemanticAction::SetTimeQuality { value }
}

fn set_weight(value: f32) -> SemanticAction {
    SemanticAction::SetWeightQuality { value }
}

fn set_flow(value: f32) -> SemanticAction {
    SemanticAction::SetFlowQuality { value }
}

fn split_group(source_group_id: u8, new_group_id: u8, revision: u64) -> SemanticAction {
    SemanticAction::SplitGroup {
        source_group_id,
        new_group_id,
        partition_rule: GroupPartitionRule::AlternatingMemberId,
        expected_morphology_revision: revision,
    }
}

fn merge_groups(
    first_group_id: u8,
    second_group_id: u8,
    survivor_group_id: u8,
    revision: u64,
) -> SemanticAction {
    SemanticAction::MergeGroups {
        group_a_id: first_group_id,
        group_b_id: second_group_id,
        survivor_group_id,
        expected_morphology_revision: revision,
    }
}

fn set_formation_scale(group_id: u8, scale: f32, revision: u64) -> SemanticAction {
    SemanticAction::SetFormationScale {
        group_id,
        scale,
        expected_morphology_revision: revision,
    }
}

fn request_lease(
    member_id: u16,
    operator_id: u8,
    lifetime_steps: u32,
    revision: u64,
) -> SemanticAction {
    SemanticAction::RequestLease {
        member_id,
        operator_id,
        lifetime_steps,
        expected_authority_revision: revision,
    }
}

fn release_lease(member_id: u16, operator_id: u8, revision: u64) -> SemanticAction {
    SemanticAction::ReleaseLease {
        member_id,
        operator_id,
        expected_authority_revision: revision,
    }
}

fn offer_handoff(member_id: u16, holder: u8, receiver: u8, revision: u64) -> SemanticAction {
    SemanticAction::OfferLeaseHandoff {
        member_id,
        holder_operator_id: holder,
        receiver_operator_id: receiver,
        expected_authority_revision: revision,
    }
}

fn resolve_handoff(
    member_id: u16,
    receiver: u8,
    decision: HandoffDecision,
    revision: u64,
) -> SemanticAction {
    SemanticAction::ResolveLeaseHandoff {
        member_id,
        receiver_operator_id: receiver,
        decision,
        expected_authority_revision: revision,
    }
}

fn set_leased_behavior(
    member_id: u16,
    operator_id: u8,
    behavior: CollectiveBehavior,
    revision: u64,
) -> SemanticAction {
    SemanticAction::SetLeasedBehavior {
        member_id,
        operator_id,
        behavior,
        expected_authority_revision: revision,
    }
}

fn run_steps(core: &mut DemoCore, batches: usize) {
    assert!(core.dispatch(SemanticAction::Start).accepted);
    for _ in 0..batches {
        assert_eq!(core.advance_elapsed(128), 8);
    }
    assert!(core.dispatch(SemanticAction::Pause).accepted);
}

fn configure_boundary_pressure(core: &mut DemoCore, policy: CollisionPolicy) {
    assert!(
        core.dispatch(SemanticAction::SetCollisionPolicy { policy })
            .accepted
    );
    assert!(
        core.dispatch(SemanticAction::SetSeparationWeight { value: 0.0 })
            .accepted
    );
    assert!(
        core.dispatch(SemanticAction::SetBoundaryStrength { value: 0.0 })
            .accepted
    );
    assert!(
        core.dispatch(SemanticAction::SetNavigationField {
            x: 0.0,
            y: 0.0,
            direction_x: 1.0,
            direction_y: 0.0,
            radius: 1.2,
            strength: 3.0,
        })
        .accepted
    );
    assert!(
        core.dispatch(SemanticAction::SetScope {
            scope: TargetScope::Swarm,
        })
        .accepted
    );
    let revision = core.public_state().selection_revision;
    assert!(core.dispatch(adjust(0.5, revision)).accepted);
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
    restarted_value["morphology_revision"] = fresh_value["morphology_revision"].clone();
    restarted_value["authority_revision"] = fresh_value["authority_revision"].clone();
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
fn additive_personal_fields_are_order_independent_and_same_seed_comparable() {
    let field_two = place_field(
        2,
        0,
        0.72,
        0.18,
        FieldPolarity::Attract,
        FieldLifetime::Persistent,
    );
    let field_nine = place_field(
        9,
        1,
        -0.62,
        -0.24,
        FieldPolarity::Repel,
        FieldLifetime::Persistent,
    );

    let mut first = DemoCore::new(707);
    assert!(first.dispatch(field_nine.clone()).accepted);
    assert!(first.dispatch(field_two.clone()).accepted);
    let _ = first.dispatch(SemanticAction::Start);
    for _ in 0..20 {
        assert_eq!(first.advance_elapsed(128), 8);
    }
    let _ = first.dispatch(SemanticAction::Pause);

    let mut reverse = DemoCore::new(707);
    assert!(reverse.dispatch(field_two).accepted);
    assert!(reverse.dispatch(field_nine).accepted);
    let _ = reverse.dispatch(SemanticAction::Start);
    for _ in 0..20 {
        assert_eq!(reverse.advance_elapsed(128), 8);
    }
    let _ = reverse.dispatch(SemanticAction::Pause);

    assert_eq!(
        first.snapshot_json().expect("snapshot serializes"),
        reverse.snapshot_json().expect("snapshot serializes")
    );
    assert_eq!(first.public_state().fields.len(), 2);
    assert_eq!(first.public_state().active_contributor_count, 2);

    let mut baseline = DemoCore::new(707);
    let _ = baseline.dispatch(SemanticAction::Start);
    for _ in 0..20 {
        let _ = baseline.advance_elapsed(128);
    }
    let _ = baseline.dispatch(SemanticAction::Pause);
    let field_centroid = centroid_x(&first.frame_rows().expect("field rows project"));
    let baseline_centroid = centroid_x(&baseline.frame_rows().expect("baseline rows project"));
    assert!((field_centroid - baseline_centroid).abs() > 0.02);

    let tape = first.replay_json().expect("field replay serializes");
    let replayed = DemoCore::from_replay_json(&tape).expect("field replay reconstructs");
    assert_eq!(
        first.snapshot_json().expect("snapshot serializes"),
        replayed.snapshot_json().expect("snapshot serializes")
    );
}

#[test]
fn field_expiry_removal_replay_and_reset_share_one_state_path() {
    let mut core = DemoCore::new(808);
    assert!(
        core.dispatch(place_field(
            0,
            0,
            0.4,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .accepted
    );
    assert_eq!(
        core.dispatch(SemanticAction::MoveField {
            field_id: 0,
            x: 0.55,
            y: 0.1,
        })
        .code,
        ActionCode::FieldMoved
    );
    assert_eq!(
        core.dispatch(SemanticAction::SetFieldPolarity {
            field_id: 0,
            polarity: FieldPolarity::Repel,
        })
        .code,
        ActionCode::FieldPolaritySet
    );
    assert!(
        core.dispatch(place_field(
            1,
            1,
            -0.4,
            0.0,
            FieldPolarity::Repel,
            FieldLifetime::Expiring { steps: 2 },
        ))
        .accepted
    );
    assert_eq!(core.public_state().fields[1].remaining_steps, Some(2));
    let _ = core.dispatch(SemanticAction::Step);
    assert_eq!(core.public_state().fields[1].remaining_steps, Some(1));
    let _ = core.dispatch(SemanticAction::Step);
    assert_eq!(core.public_state().fields.len(), 1);

    let removed = core.dispatch(SemanticAction::RemoveField { field_id: 0 });
    assert_eq!(removed.code, ActionCode::FieldRemoved);
    assert!(core.public_state().fields.is_empty());
    assert!(
        core.dispatch(place_field(
            3,
            2,
            0.0,
            0.5,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .accepted
    );
    let _ = core.dispatch(SemanticAction::Reset);
    assert!(core.public_state().fields.is_empty());
    assert_eq!(core.public_state().tick, 0);

    let tape = core.replay_json().expect("field lifecycle serializes");
    let replayed = DemoCore::from_replay_json(&tape).expect("field lifecycle replays");
    assert_eq!(
        core.snapshot_json().expect("snapshot serializes"),
        replayed.snapshot_json().expect("snapshot serializes")
    );
}

#[test]
fn invalid_personal_field_inputs_fail_closed() {
    let mut core = DemoCore::new(909);
    assert_eq!(
        core.dispatch(place_field(
            64,
            0,
            0.0,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .code,
        ActionCode::InvalidFieldId
    );
    assert_eq!(
        core.dispatch(place_field(
            0,
            4,
            0.0,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .code,
        ActionCode::InvalidContributor
    );
    assert_eq!(
        core.dispatch(place_field(
            0,
            0,
            f32::NAN,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .code,
        ActionCode::InvalidFieldPosition
    );
    assert_eq!(
        core.dispatch(place_field(
            0,
            0,
            0.0,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Expiring { steps: 0 },
        ))
        .code,
        ActionCode::InvalidFieldLifetime
    );
    assert_eq!(
        core.dispatch(place_field(
            0,
            0,
            0.0,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Expiring {
                steps: MAX_FIELD_LIFETIME_STEPS + 1,
            },
        ))
        .code,
        ActionCode::InvalidFieldLifetime
    );

    assert_eq!(
        core.dispatch(SemanticAction::MoveField {
            field_id: 99,
            x: 0.0,
            y: 0.0,
        })
        .code,
        ActionCode::MissingField
    );
}

#[test]
fn personal_field_count_and_damaged_actions_fail_closed() {
    let mut core = DemoCore::new(910);
    for field_id in 0..MAX_PERSONAL_FIELDS {
        assert!(
            core.dispatch(place_field(
                u16::try_from(field_id).expect("field ID fits"),
                u8::try_from(field_id % 4).expect("contributor fits"),
                0.0,
                0.0,
                FieldPolarity::Attract,
                FieldLifetime::Persistent,
            ))
            .accepted
        );
    }
    assert_eq!(
        core.dispatch(place_field(
            8,
            0,
            0.0,
            0.0,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .code,
        ActionCode::FieldLimitReached
    );

    let damaged_action = r#"{
        "type":"place_field",
        "field_id":1,
        "contributor_id":0,
        "x":0.0,
        "y":0.0,
        "polarity":"attract",
        "lifetime":{"mode":"persistent","private_identity":"reject"}
    }"#;
    assert!(serde_json::from_str::<SemanticAction>(damaged_action).is_err());
    let damaged_replay = format!(
        r#"{{
            "schema":"combinatorial.swarmability.replay.v1",
            "initial_seed":910,
            "events":[{{"kind":"action","action":{damaged_action}}}]
        }}"#
    );
    assert!(DemoCore::from_replay_json(&damaged_replay).is_err());
}

#[test]
fn raw_dynamics_rates_are_order_independent_and_same_seed_measurable() {
    let mut first = DemoCore::new(1_515);
    assert!(first.dispatch(set_alignment(0.15)).accepted);
    assert!(first.dispatch(set_cohesion(0.75)).accepted);
    assert!(first.dispatch(set_separation(0.10)).accepted);
    run_steps(&mut first, 40);

    let mut reverse = DemoCore::new(1_515);
    assert!(reverse.dispatch(set_separation(0.10)).accepted);
    assert!(reverse.dispatch(set_cohesion(0.75)).accepted);
    assert!(reverse.dispatch(set_alignment(0.15)).accepted);
    run_steps(&mut reverse, 40);

    assert_eq!(
        first.snapshot_json().expect("snapshot serializes"),
        reverse.snapshot_json().expect("snapshot serializes")
    );
    let distribution = first.public_state().behavior_counts;
    assert_eq!(
        distribution.flock + distribution.cohere + distribution.disperse,
        MEMBER_COUNT
    );
    assert!(distribution.cohere > distribution.flock);
    assert!(distribution.cohere > distribution.disperse);

    let mut cohesion_only = DemoCore::new(1_515);
    assert!(cohesion_only.dispatch(set_cohesion(1.0)).accepted);
    run_steps(&mut cohesion_only, 40);

    let mut separation_only = DemoCore::new(1_515);
    assert!(separation_only.dispatch(set_separation(1.0)).accepted);
    run_steps(&mut separation_only, 40);

    assert!(cohesion_only.public_state().behavior_counts.cohere >= 20);
    assert!(separation_only.public_state().behavior_counts.disperse >= 20);
    let cohesion_rows = cohesion_only.frame_rows().expect("cohesion rows project");
    let separation_rows = separation_only
        .frame_rows()
        .expect("separation rows project");
    assert!(mean_pair_distance(&cohesion_rows) < mean_pair_distance(&separation_rows));
    assert!((polarization(&cohesion_rows) - polarization(&separation_rows)).abs() > 0.02);

    let replay = first.replay_json().expect("raw dynamics replay serializes");
    let replayed = DemoCore::from_replay_json(&replay).expect("raw dynamics replay reconstructs");
    assert_eq!(
        first.snapshot_json().expect("snapshot serializes"),
        replayed.snapshot_json().expect("snapshot serializes")
    );
}

#[test]
fn raw_dynamics_checkpoint_reset_and_other_mechanisms_share_one_state() {
    let mut core = DemoCore::new(1_516);
    assert!(core.dispatch(set_alignment(0.2)).accepted);
    assert!(core.dispatch(set_cohesion(0.6)).accepted);
    assert!(core.dispatch(set_separation(0.2)).accepted);
    assert!(
        core.dispatch(SemanticAction::SetScope {
            scope: TargetScope::Swarm
        })
        .accepted
    );
    let selection_revision = core.public_state().selection_revision;
    assert!(
        core.dispatch(set_behavior(CollectiveBehavior::Cohere, selection_revision))
            .accepted
    );
    assert!(
        core.dispatch(place_field(
            4,
            2,
            0.35,
            -0.25,
            FieldPolarity::Attract,
            FieldLifetime::Expiring { steps: 500 },
        ))
        .accepted
    );
    run_steps(&mut core, 12);

    let checkpoint = core
        .snapshot_json()
        .expect("combined checkpoint serializes");
    let restored = DemoCore::from_snapshot_json(&checkpoint).expect("combined checkpoint restores");
    assert_eq!(
        checkpoint,
        restored
            .snapshot_json()
            .expect("restored snapshot serializes")
    );
    assert_eq!(restored.public_state().fields.len(), 1);
    assert_eq!(
        restored.public_state().dynamics_rates,
        core.public_state().dynamics_rates
    );

    let replay = core.replay_json().expect("combined replay serializes");
    let replayed = DemoCore::from_replay_json(&replay).expect("combined replay reconstructs");
    assert_eq!(
        checkpoint,
        replayed
            .snapshot_json()
            .expect("replayed snapshot serializes")
    );

    assert!(core.dispatch(SemanticAction::Reset).accepted);
    assert_eq!(core.public_state().dynamics_rates, DEFAULT_DYNAMICS_RATES);
    assert!(core.public_state().fields.is_empty());
    assert_eq!(core.public_state().behavior_counts.flock, MEMBER_COUNT);
    assert_eq!(core.public_state().tick, 0);
}

#[test]
fn invalid_raw_dynamics_rates_and_damaged_state_fail_closed() {
    let mut core = DemoCore::new(1_517);
    for action in [
        set_alignment(-0.01),
        set_cohesion(MAX_DYNAMICS_RATE + 0.01),
        set_separation(f32::NAN),
    ] {
        let before = core.snapshot_json().expect("snapshot serializes");
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, ActionCode::InvalidDynamicsRate);
        assert_eq!(before, core.snapshot_json().expect("snapshot serializes"));
    }

    assert!(serde_json::from_str::<SemanticAction>(
        r#"{"type":"set_alignment","rate":0.5,"arbitrary_parameter":"reject"}"#
    )
    .is_err());

    let mut damaged: serde_json::Value =
        serde_json::from_str(&core.snapshot_json().expect("snapshot serializes"))
            .expect("snapshot parses");
    damaged["raw_dynamics_rates"]["cohesion"] = serde_json::json!(1.5);
    assert!(DemoCore::from_snapshot_json(&damaged.to_string()).is_err());
}

#[test]
#[allow(clippy::float_cmp)] // Defaults are a serialized, exact public contract.
fn semantic_quality_defaults_bounds_and_damaged_vectors_fail_closed() {
    let mut core = DemoCore::new(2_020);
    let initial = core.public_state();
    assert_eq!(initial.dynamics_control_mode, DynamicsControlMode::Raw);
    assert_eq!(initial.semantic_qualities, DEFAULT_SEMANTIC_QUALITIES);
    assert_eq!(initial.raw_dynamics_rates, DEFAULT_DYNAMICS_RATES);
    assert_eq!(initial.resolved_dynamics.rates, DEFAULT_DYNAMICS_RATES);
    assert_eq!(initial.resolved_dynamics.speed_scale, 1.0);
    assert_eq!(initial.resolved_dynamics.damping, 0.0);
    assert_eq!(initial.resolved_dynamics.jitter, 0.0);

    for action in [
        set_space(-0.01),
        set_time(MAX_SEMANTIC_QUALITY + 0.01),
        set_weight(f32::NAN),
        set_flow(f32::INFINITY),
    ] {
        let before = core.snapshot_json().expect("snapshot serializes");
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, ActionCode::InvalidSemanticQuality);
        assert_eq!(before, core.snapshot_json().expect("snapshot serializes"));
    }

    assert!(serde_json::from_str::<SemanticAction>(
        r#"{"type":"set_space_quality","value":0.5,"camera_pose":"reject"}"#
    )
    .is_err());

    assert!(core.dispatch(set_space(0.7)).accepted);
    let snapshot = core.snapshot_json().expect("snapshot serializes");
    let mut damaged_quality: serde_json::Value =
        serde_json::from_str(&snapshot).expect("snapshot parses");
    damaged_quality["semantic_qualities"]["flow"] = serde_json::json!(1.2);
    assert!(DemoCore::from_snapshot_json(&damaged_quality.to_string()).is_err());

    let mut damaged_resolution: serde_json::Value =
        serde_json::from_str(&snapshot).expect("snapshot parses");
    damaged_resolution["resolved_dynamics"]["speed_scale"] = serde_json::json!(1.01);
    assert!(DemoCore::from_snapshot_json(&damaged_resolution.to_string()).is_err());
}

#[test]
#[allow(clippy::float_cmp)] // Qualitative-profile endpoints are an exact public contract.
fn semantic_translation_preserves_reported_directions_and_app_owned_endpoints() {
    let mut indirect = DemoCore::new(2_021);
    assert!(indirect.dispatch(set_space(0.0)).accepted);
    let indirect_vector = indirect.public_state().resolved_dynamics;
    let mut direct = DemoCore::new(2_021);
    assert!(direct.dispatch(set_space(1.0)).accepted);
    let direct_vector = direct.public_state().resolved_dynamics;
    assert_eq!(indirect_vector.rates.alignment, 0.15);
    assert_eq!(direct_vector.rates.alignment, 0.85);
    assert_eq!(indirect_vector.rates.separation, 0.85);
    assert_eq!(direct_vector.rates.separation, 0.15);
    assert!(direct_vector.rates.alignment > indirect_vector.rates.alignment);
    assert!(direct_vector.rates.separation < indirect_vector.rates.separation);

    let mut sustained = DemoCore::new(2_021);
    assert!(sustained.dispatch(set_time(0.0)).accepted);
    let mut sudden = DemoCore::new(2_021);
    assert!(sudden.dispatch(set_time(1.0)).accepted);
    assert_eq!(sustained.public_state().resolved_dynamics.speed_scale, 0.75);
    assert_eq!(sudden.public_state().resolved_dynamics.speed_scale, 1.25);

    let mut lower_weight = DemoCore::new(2_021);
    assert!(lower_weight.dispatch(set_weight(0.0)).accepted);
    let mut higher_weight = DemoCore::new(2_021);
    assert!(higher_weight.dispatch(set_weight(1.0)).accepted);
    assert_eq!(
        lower_weight.public_state().resolved_dynamics.rates.cohesion,
        0.85
    );
    assert_eq!(
        higher_weight
            .public_state()
            .resolved_dynamics
            .rates
            .cohesion,
        0.15
    );

    let mut bound = DemoCore::new(2_021);
    assert!(bound.dispatch(set_flow(0.0)).accepted);
    let mut free = DemoCore::new(2_021);
    assert!(free.dispatch(set_flow(1.0)).accepted);
    assert_eq!(bound.public_state().resolved_dynamics.damping, 0.75);
    assert_eq!(free.public_state().resolved_dynamics.damping, 0.15);
    assert_eq!(bound.public_state().resolved_dynamics.jitter, 0.0);
    assert_eq!(free.public_state().resolved_dynamics.jitter, 0.18);
}

#[test]
fn semantic_order_raw_inspection_checkpoint_replay_and_reset_are_exact() {
    let ordered_actions = [
        set_space(0.8),
        set_time(0.7),
        set_weight(0.2),
        set_flow(0.9),
    ];
    let reverse_actions = [
        set_flow(0.9),
        set_weight(0.2),
        set_time(0.7),
        set_space(0.8),
    ];

    let mut ordered = DemoCore::new(2_022);
    for action in ordered_actions {
        assert!(ordered.dispatch(action).accepted);
    }
    assert!(
        ordered
            .dispatch(SemanticAction::SetScope {
                scope: TargetScope::Swarm,
            })
            .accepted
    );
    assert!(
        ordered
            .dispatch(place_field(
                5,
                1,
                -0.3,
                0.2,
                FieldPolarity::Repel,
                FieldLifetime::Persistent,
            ))
            .accepted
    );
    run_steps(&mut ordered, 24);

    let mut reverse = DemoCore::new(2_022);
    for action in reverse_actions {
        assert!(reverse.dispatch(action).accepted);
    }
    assert!(
        reverse
            .dispatch(SemanticAction::SetScope {
                scope: TargetScope::Swarm,
            })
            .accepted
    );
    assert!(
        reverse
            .dispatch(place_field(
                5,
                1,
                -0.3,
                0.2,
                FieldPolarity::Repel,
                FieldLifetime::Persistent,
            ))
            .accepted
    );
    run_steps(&mut reverse, 24);

    assert_eq!(
        ordered.public_state().dynamics_control_mode,
        DynamicsControlMode::Semantic
    );
    assert_eq!(
        ordered.public_state().resolved_dynamics,
        reverse.public_state().resolved_dynamics
    );
    assert_eq!(
        ordered.snapshot_json().expect("snapshot serializes"),
        reverse.snapshot_json().expect("snapshot serializes")
    );

    let before_inspection = ordered.snapshot_json().expect("snapshot serializes");
    for _ in 0..100 {
        let state = ordered.public_state();
        assert_eq!(state.dynamics_rates, state.resolved_dynamics.rates);
    }
    assert_eq!(
        before_inspection,
        ordered.snapshot_json().expect("snapshot serializes")
    );

    let checkpoint = ordered.snapshot_json().expect("checkpoint serializes");
    let restored = DemoCore::from_snapshot_json(&checkpoint).expect("checkpoint restores");
    assert_eq!(
        checkpoint,
        restored.snapshot_json().expect("snapshot serializes")
    );
    let tape = ordered.replay_json().expect("semantic replay serializes");
    let replayed = DemoCore::from_replay_json(&tape).expect("semantic replay reconstructs");
    assert_eq!(
        checkpoint,
        replayed.snapshot_json().expect("snapshot serializes")
    );

    assert!(ordered.dispatch(SemanticAction::Reset).accepted);
    let reset = ordered.public_state();
    assert_eq!(reset.dynamics_control_mode, DynamicsControlMode::Raw);
    assert_eq!(reset.semantic_qualities, DEFAULT_SEMANTIC_QUALITIES);
    assert_eq!(reset.resolved_dynamics.rates, DEFAULT_DYNAMICS_RATES);
    assert!(reset.fields.is_empty());
}

#[test]
fn semantic_profiles_change_same_seed_distribution_speed_spacing_and_polarization() {
    let mut compact = DemoCore::new(2_023);
    for action in [
        set_space(0.0),
        set_time(0.0),
        set_weight(0.0),
        set_flow(0.0),
    ] {
        assert!(compact.dispatch(action).accepted);
    }
    run_steps(&mut compact, 40);

    let mut expansive = DemoCore::new(2_023);
    for action in [
        set_space(1.0),
        set_time(1.0),
        set_weight(1.0),
        set_flow(1.0),
    ] {
        assert!(expansive.dispatch(action).accepted);
    }
    run_steps(&mut expansive, 40);

    let compact_state = compact.public_state();
    let expansive_state = expansive.public_state();
    assert!(compact_state.average_speed < expansive_state.average_speed);
    assert!(compact_state.behavior_counts.cohere > expansive_state.behavior_counts.cohere);
    assert!(compact_state.behavior_counts.disperse > expansive_state.behavior_counts.disperse);
    assert!(expansive_state.behavior_counts.flock > compact_state.behavior_counts.flock);

    let compact_rows = compact.frame_rows().expect("compact rows project");
    let expansive_rows = expansive.frame_rows().expect("expansive rows project");
    let spacing_delta =
        (mean_pair_distance(&compact_rows) - mean_pair_distance(&expansive_rows)).abs();
    let polarization_delta = (polarization(&compact_rows) - polarization(&expansive_rows)).abs();
    assert!(spacing_delta > 0.005, "spacing delta was {spacing_delta}");
    assert!(
        polarization_delta > 0.01,
        "polarization delta was {polarization_delta}"
    );
}

#[test]
#[allow(clippy::float_cmp)] // Canonical serialized scale values are exact public contracts.
fn morphology_split_merge_and_identity_are_canonical_and_conservative() {
    let mut core = DemoCore::new(2_021);
    let initial = core.public_state();
    assert_eq!(initial.morphology_revision, 0);
    assert_eq!(initial.groups.len(), 1);
    assert_eq!(initial.groups[0].group_id, 0);
    assert_eq!(
        initial.groups[0].member_ids,
        (0..member_count_u16()).collect::<Vec<_>>()
    );
    assert_eq!(initial.groups[0].formation_scale, DEFAULT_FORMATION_SCALE);

    let first = core.dispatch(split_group(0, 1, 0));
    assert!(first.accepted);
    assert_eq!(first.code, ActionCode::GroupSplit);
    assert_eq!(first.morphology_revision, 1);
    let split = core.public_state();
    assert_eq!(
        split.groups[0].member_ids,
        (0..member_count_u16()).step_by(2).collect::<Vec<_>>()
    );
    assert_eq!(
        split.groups[1].member_ids,
        (1..member_count_u16()).step_by(2).collect::<Vec<_>>()
    );
    assert_conserved_membership(&split);

    assert!(core.dispatch(set_formation_scale(0, 0.8, 1)).accepted);
    assert!(core.dispatch(set_formation_scale(1, 1.6, 2)).accepted);
    let before_merge = core.public_state();
    let mut reversed_operands = core.clone();
    let forward = core.dispatch(merge_groups(0, 1, 0, 3));
    let reversed = reversed_operands.dispatch(merge_groups(1, 0, 0, 3));
    assert!(forward.accepted && reversed.accepted);
    assert_eq!(
        core.snapshot_json().expect("snapshot serializes"),
        reversed_operands
            .snapshot_json()
            .expect("snapshot serializes")
    );
    let merged = core.public_state();
    assert_eq!(merged.groups.len(), 1);
    assert_eq!(merged.groups[0].group_id, 0);
    assert_eq!(merged.groups[0].formation_scale, 0.8);
    assert_ne!(
        before_merge.groups[1].formation_scale,
        merged.groups[0].formation_scale
    );
    assert_conserved_membership(&merged);
}

#[test]
fn morphology_rejects_stale_impossible_and_out_of_range_operations() {
    let mut core = DemoCore::new(2_102);
    let before = core.snapshot_json().expect("snapshot serializes");
    for (action, code) in [
        (split_group(9, 1, 0), ActionCode::MissingGroup),
        (split_group(0, 2, 0), ActionCode::NonCanonicalGroup),
        (merge_groups(0, 0, 0, 0), ActionCode::InvalidGroupOperation),
        (merge_groups(0, 1, 0, 0), ActionCode::MissingGroup),
        (
            set_formation_scale(0, MIN_FORMATION_SCALE - 0.01, 0),
            ActionCode::InvalidFormationScale,
        ),
        (
            set_formation_scale(0, MAX_FORMATION_SCALE + 0.01, 0),
            ActionCode::InvalidFormationScale,
        ),
        (
            set_formation_scale(0, f32::NAN, 0),
            ActionCode::InvalidFormationScale,
        ),
    ] {
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, code);
        assert_eq!(before, core.snapshot_json().expect("snapshot serializes"));
    }

    assert!(core.dispatch(split_group(0, 1, 0)).accepted);
    assert_eq!(
        core.dispatch(set_formation_scale(0, 1.2, 0)).code,
        ActionCode::StaleMorphology
    );
    assert_eq!(
        core.dispatch(merge_groups(0, 1, 1, 1)).code,
        ActionCode::NonCanonicalGroup
    );

    let mut singleton = DemoCore::new(2_103);
    for new_group_id in 1..=5 {
        let revision = singleton.public_state().morphology_revision;
        assert!(
            singleton
                .dispatch(split_group(0, new_group_id, revision))
                .accepted
        );
    }
    assert_eq!(singleton.public_state().groups[0].member_ids.len(), 1);
    let singleton_revision = singleton.public_state().morphology_revision;
    assert_eq!(
        singleton
            .dispatch(split_group(0, 6, singleton_revision))
            .code,
        ActionCode::GroupCannotSplit
    );

    let mut bounded = DemoCore::new(2_104);
    for new_group_id in 1..u8::try_from(MAX_GROUPS).expect("group count fits in a u8") {
        let state = bounded.public_state();
        let source = state
            .groups
            .iter()
            .filter(|group| group.member_ids.len() >= 2)
            .max_by_key(|group| group.member_ids.len())
            .expect("a splittable group remains");
        assert!(
            bounded
                .dispatch(split_group(
                    source.group_id,
                    new_group_id,
                    state.morphology_revision,
                ))
                .accepted
        );
    }
    assert_eq!(bounded.public_state().groups.len(), MAX_GROUPS);
    let bounded_state = bounded.public_state();
    assert_eq!(
        bounded
            .dispatch(split_group(
                bounded_state.groups[0].group_id,
                7,
                bounded_state.morphology_revision,
            ))
            .code,
        ActionCode::GroupLimitReached
    );

    assert!(serde_json::from_str::<SemanticAction>(
        r#"{"type":"split_group","source_group_id":0,"new_group_id":1,"partition_rule":"alternating_member_id","expected_morphology_revision":0,"implicit_visual_group":true}"#
    )
    .is_err());
}

#[test]
fn damaged_morphology_snapshots_fail_closed() {
    let mut core = DemoCore::new(2_104);
    assert!(core.dispatch(split_group(0, 1, 0)).accepted);
    let snapshot = core.snapshot_json().expect("snapshot serializes");

    let mut empty: serde_json::Value = serde_json::from_str(&snapshot).expect("snapshot parses");
    empty["groups"] = serde_json::json!([]);
    assert!(DemoCore::from_snapshot_json(&empty.to_string()).is_err());

    let mut scale: serde_json::Value = serde_json::from_str(&snapshot).expect("snapshot parses");
    scale["groups"][0]["formation_scale"] = serde_json::json!(2.1);
    assert!(DemoCore::from_snapshot_json(&scale.to_string()).is_err());

    let mut group_id: serde_json::Value = serde_json::from_str(&snapshot).expect("snapshot parses");
    group_id["groups"][0]["group_id"] = serde_json::json!(8);
    assert!(DemoCore::from_snapshot_json(&group_id.to_string()).is_err());

    let mut duplicate: serde_json::Value =
        serde_json::from_str(&snapshot).expect("snapshot parses");
    duplicate["groups"][0]["member_ids"] = serde_json::json!([0, 0, 2, 4]);
    assert!(DemoCore::from_snapshot_json(&duplicate.to_string()).is_err());

    let mut missing: serde_json::Value = serde_json::from_str(&snapshot).expect("snapshot parses");
    missing["groups"][0]["member_ids"] = serde_json::json!([0]);
    assert!(DemoCore::from_snapshot_json(&missing.to_string()).is_err());
}

#[test]
#[allow(clippy::float_cmp)] // Neutral raw dynamics values are exact public contracts.
fn morphology_coexists_with_scope_fields_dynamics_checkpoint_replay_and_reset() {
    let mut core = DemoCore::new(2_105);
    assert!(
        core.dispatch(SemanticAction::SetScope {
            scope: TargetScope::Subgroup,
        })
        .accepted
    );
    assert!(
        core.dispatch(place_field(
            3,
            2,
            0.25,
            -0.3,
            FieldPolarity::Attract,
            FieldLifetime::Persistent,
        ))
        .accepted
    );
    assert!(core.dispatch(set_space(0.8)).accepted);
    assert!(core.dispatch(set_flow(0.7)).accepted);
    let before = core.public_state();

    assert!(core.dispatch(split_group(0, 1, 0)).accepted);
    assert!(core.dispatch(set_formation_scale(1, 1.7, 1)).accepted);
    let after = core.public_state();
    assert_eq!(after.scope, before.scope);
    assert_eq!(after.subgroup_members, before.subgroup_members);
    assert_eq!(after.fields, before.fields);
    assert_eq!(
        after.active_contributor_count,
        before.active_contributor_count
    );
    assert_eq!(after.dynamics_control_mode, before.dynamics_control_mode);
    assert_eq!(after.semantic_qualities, before.semantic_qualities);
    assert_eq!(after.resolved_dynamics, before.resolved_dynamics);
    assert_eq!(after.selection_revision, before.selection_revision);

    let groups_before_raw = after.groups.clone();
    assert!(core.dispatch(set_alignment(0.35)).accepted);
    let raw = core.public_state();
    assert_eq!(raw.groups, groups_before_raw);
    assert_eq!(raw.dynamics_control_mode, DynamicsControlMode::Raw);
    assert_eq!(raw.resolved_dynamics.speed_scale, 1.0);

    run_steps(&mut core, 16);
    let checkpoint = core.snapshot_json().expect("checkpoint serializes");
    let restored = DemoCore::from_snapshot_json(&checkpoint).expect("checkpoint restores");
    assert_eq!(
        checkpoint,
        restored.snapshot_json().expect("snapshot serializes")
    );
    let replay = core.replay_json().expect("replay serializes");
    let replayed = DemoCore::from_replay_json(&replay).expect("replay reconstructs");
    assert_eq!(
        checkpoint,
        replayed.snapshot_json().expect("snapshot serializes")
    );

    let revision_before_reset = core.public_state().morphology_revision;
    assert!(core.dispatch(SemanticAction::Reset).accepted);
    let reset = core.public_state();
    assert_eq!(reset.groups.len(), 1);
    assert_eq!(reset.groups[0].member_ids.len(), MEMBER_COUNT);
    assert_eq!(reset.groups[0].formation_scale, DEFAULT_FORMATION_SCALE);
    assert_eq!(reset.morphology_revision, revision_before_reset + 1);
}

#[test]
fn formation_scale_changes_same_seed_extent_without_rewriting_dynamics() {
    let mut compact = DemoCore::new(2_106);
    let mut expanded = DemoCore::new(2_106);
    for core in [&mut compact, &mut expanded] {
        assert!(core.dispatch(split_group(0, 1, 0)).accepted);
        assert!(core.dispatch(set_time(0.7)).accepted);
    }
    let compact_dynamics = compact.public_state().resolved_dynamics;
    let expanded_dynamics = expanded.public_state().resolved_dynamics;
    assert!(
        compact
            .dispatch(set_formation_scale(0, MIN_FORMATION_SCALE, 1))
            .accepted
    );
    assert!(
        expanded
            .dispatch(set_formation_scale(0, MAX_FORMATION_SCALE, 1))
            .accepted
    );
    assert_eq!(compact.public_state().resolved_dynamics, compact_dynamics);
    assert_eq!(expanded.public_state().resolved_dynamics, expanded_dynamics);

    run_steps(&mut compact, 60);
    run_steps(&mut expanded, 60);
    let compact_state = compact.public_state();
    let expanded_state = expanded.public_state();
    let extent_delta =
        expanded_state.groups[0].formation_extent - compact_state.groups[0].formation_extent;
    assert!(
        extent_delta > 0.04,
        "formation extent delta was {extent_delta}"
    );
    assert_ne!(
        compact.frame_rows().unwrap(),
        expanded.frame_rows().unwrap()
    );
    assert_conserved_membership(&compact_state);
    assert_conserved_membership(&expanded_state);
}

#[test]
fn lease_state_machine_requires_holder_and_explicit_receiver_consent() {
    let mut core = DemoCore::new(2_301);
    let acquired = core.dispatch(request_lease(4, 0, 8, 0));
    assert!(acquired.accepted);
    assert_eq!(acquired.code, ActionCode::LeaseAcquired);
    assert_eq!(acquired.authority_revision, 1);
    let lease = &core.public_state().leases[0];
    assert_eq!(lease.member_id, 4);
    assert_eq!(lease.holder_operator_id, 0);
    assert_eq!(lease.remaining_steps, 8);

    assert_eq!(
        core.dispatch(request_lease(4, 0, 8, 1)).code,
        ActionCode::LeaseAlreadyHeld
    );
    assert_eq!(
        core.dispatch(release_lease(4, 0, 0)).code,
        ActionCode::StaleAuthority
    );
    assert_eq!(
        core.dispatch(release_lease(4, 1, 1)).code,
        ActionCode::NotLeaseHolder
    );
    assert!(
        core.dispatch(set_leased_behavior(4, 0, CollectiveBehavior::Disperse, 1))
            .accepted
    );
    assert_eq!(
        core.public_state().members[4].behavior,
        CollectiveBehavior::Disperse
    );

    assert!(core.dispatch(offer_handoff(4, 0, 1, 2)).accepted);
    assert_eq!(core.public_state().leases[0].pending_handoff_to, Some(1));
    assert_eq!(
        core.dispatch(offer_handoff(4, 0, 2, 3)).code,
        ActionCode::HandoffAlreadyPending
    );
    assert_eq!(
        core.dispatch(resolve_handoff(4, 2, HandoffDecision::Accept, 3))
            .code,
        ActionCode::MissingHandoff
    );
    let expiry = core.public_state().leases[0].expires_at_tick;
    assert!(
        core.dispatch(resolve_handoff(4, 1, HandoffDecision::Decline, 3))
            .accepted
    );
    assert_eq!(core.public_state().leases[0].holder_operator_id, 0);
    assert!(core.dispatch(offer_handoff(4, 0, 1, 4)).accepted);
    let accepted = core.dispatch(resolve_handoff(4, 1, HandoffDecision::Accept, 5));
    assert_eq!(accepted.code, ActionCode::LeaseHandoffAccepted);
    let lease = &core.public_state().leases[0];
    assert_eq!(lease.holder_operator_id, 1);
    assert_eq!(lease.expires_at_tick, expiry);
    assert_eq!(lease.pending_handoff_to, None);
    assert_eq!(
        core.dispatch(release_lease(4, 0, 6)).code,
        ActionCode::NotLeaseHolder
    );
    assert!(core.dispatch(release_lease(4, 1, 6)).accepted);
    assert!(core.public_state().leases.is_empty());
}

#[test]
fn lease_expiry_uses_exact_fixed_step_boundary_and_fences_replayed_use() {
    let mut core = DemoCore::new(2_302);
    let acquire = request_lease(3, 2, 3, 0);
    assert!(core.dispatch(acquire.clone()).accepted);
    assert_eq!(core.dispatch(acquire).code, ActionCode::StaleAuthority);
    assert!(core.dispatch(SemanticAction::Step).accepted);
    assert!(core.dispatch(SemanticAction::Step).accepted);
    assert_eq!(core.public_state().leases[0].remaining_steps, 1);
    let before_expiry_revision = core.public_state().authority_revision;
    assert!(core.dispatch(SemanticAction::Step).accepted);
    let expired = core.public_state();
    assert!(expired.leases.is_empty());
    assert_eq!(expired.authority_revision, before_expiry_revision + 1);
    assert_eq!(
        core.dispatch(set_leased_behavior(
            3,
            2,
            CollectiveBehavior::Cohere,
            expired.authority_revision,
        ))
        .code,
        ActionCode::MissingLease
    );

    let replay = core.replay_json().expect("expired lease replay serializes");
    let replayed = DemoCore::from_replay_json(&replay).expect("expired lease replay restores");
    assert_eq!(
        core.snapshot_json().expect("snapshot serializes"),
        replayed.snapshot_json().expect("snapshot serializes")
    );
}

#[test]
fn invalid_lease_actions_and_damaged_snapshots_fail_closed() {
    let mut core = DemoCore::new(2_303);
    let initial = core.snapshot_json().expect("snapshot serializes");
    for (action, code) in [
        (request_lease(24, 0, 10, 0), ActionCode::InvalidMember),
        (
            request_lease(0, MAX_SYNTHETIC_OPERATORS, 10, 0),
            ActionCode::InvalidOperator,
        ),
        (request_lease(0, 0, 0, 0), ActionCode::InvalidLeaseLifetime),
        (
            request_lease(0, 0, MAX_LEASE_LIFETIME_STEPS + 1, 0),
            ActionCode::InvalidLeaseLifetime,
        ),
        (release_lease(0, 0, 0), ActionCode::MissingLease),
    ] {
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, code);
        assert_eq!(initial, core.snapshot_json().expect("snapshot serializes"));
    }

    assert!(core.dispatch(request_lease(0, 0, 10, 0)).accepted);
    assert_eq!(
        core.dispatch(offer_handoff(0, 0, 0, 1)).code,
        ActionCode::InvalidHandoff
    );
    assert_eq!(
        core.dispatch(offer_handoff(0, 0, MAX_SYNTHETIC_OPERATORS, 1))
            .code,
        ActionCode::InvalidHandoff
    );

    let snapshot = core.snapshot_json().expect("snapshot serializes");
    for mutation in ["member", "holder", "receiver", "expiry", "duplicate"] {
        let mut damaged: serde_json::Value =
            serde_json::from_str(&snapshot).expect("snapshot parses");
        match mutation {
            "member" => damaged["leases"][0]["member_id"] = serde_json::json!(24),
            "holder" => damaged["leases"][0]["holder_operator_id"] = serde_json::json!(4),
            "receiver" => damaged["leases"][0]["pending_handoff_to"] = serde_json::json!(0),
            "expiry" => damaged["leases"][0]["expires_at_tick"] = serde_json::json!(0),
            "duplicate" => {
                let lease = damaged["leases"][0].clone();
                damaged["leases"].as_array_mut().unwrap().push(lease);
            }
            _ => unreachable!(),
        }
        assert!(DemoCore::from_snapshot_json(&damaged.to_string()).is_err());
    }
    assert!(serde_json::from_str::<SemanticAction>(
        r#"{"type":"request_lease","member_id":0,"operator_id":0,"lifetime_steps":10,"expected_authority_revision":0,"account_id":"private"}"#
    )
    .is_err());
}

#[test]
fn lease_caps_and_distinct_member_request_order_are_deterministic() {
    let mut ordered = DemoCore::new(2_304);
    let mut reversed = DemoCore::new(2_304);
    for (revision, member) in (0..MAX_ACTIVE_LEASES).enumerate() {
        assert!(
            ordered
                .dispatch(request_lease(
                    u16::try_from(member).unwrap(),
                    u8::try_from(member % usize::from(MAX_SYNTHETIC_OPERATORS)).unwrap(),
                    20,
                    u64::try_from(revision).unwrap(),
                ))
                .accepted
        );
    }
    for (revision, member) in (0..MAX_ACTIVE_LEASES).rev().enumerate() {
        assert!(
            reversed
                .dispatch(request_lease(
                    u16::try_from(member).unwrap(),
                    u8::try_from(member % usize::from(MAX_SYNTHETIC_OPERATORS)).unwrap(),
                    20,
                    u64::try_from(revision).unwrap(),
                ))
                .accepted
        );
    }
    assert_eq!(
        ordered.public_state().leases,
        reversed.public_state().leases
    );
    assert_eq!(
        ordered
            .dispatch(request_lease(
                u16::try_from(MAX_ACTIVE_LEASES).unwrap(),
                0,
                20,
                u64::try_from(MAX_ACTIVE_LEASES).unwrap(),
            ))
            .code,
        ActionCode::LeaseLimitReached
    );
}

#[test]
fn leases_survive_morphology_and_preserve_unrelated_mechanism_state() {
    let mut core = DemoCore::new(2_305);
    assert!(
        core.dispatch(SemanticAction::SetScope {
            scope: TargetScope::Subgroup,
        })
        .accepted
    );
    assert!(
        core.dispatch(place_field(
            0,
            3,
            0.3,
            0.2,
            FieldPolarity::Repel,
            FieldLifetime::Persistent,
        ))
        .accepted
    );
    assert!(core.dispatch(set_space(0.8)).accepted);
    assert!(core.dispatch(request_lease(1, 0, 40, 0)).accepted);
    let before_morphology = core.public_state();
    assert!(core.dispatch(split_group(0, 1, 0)).accepted);
    assert!(core.dispatch(set_formation_scale(1, 1.4, 1)).accepted);
    assert!(core.dispatch(merge_groups(1, 0, 0, 2)).accepted);
    let after_morphology = core.public_state();
    assert_eq!(after_morphology.leases, before_morphology.leases);
    assert_eq!(after_morphology.authority_revision, 1);
    assert_eq!(after_morphology.scope, before_morphology.scope);
    assert_eq!(
        after_morphology.subgroup_members,
        before_morphology.subgroup_members
    );
    assert_eq!(after_morphology.fields, before_morphology.fields);
    assert_eq!(
        after_morphology.semantic_qualities,
        before_morphology.semantic_qualities
    );
    assert_eq!(
        after_morphology.resolved_dynamics,
        before_morphology.resolved_dynamics
    );

    assert!(
        core.dispatch(set_leased_behavior(1, 0, CollectiveBehavior::Cohere, 1))
            .accepted
    );
    let checkpoint = core.snapshot_json().expect("checkpoint serializes");
    let restored = DemoCore::from_snapshot_json(&checkpoint).expect("checkpoint restores");
    assert_eq!(restored.public_state().leases[0].remaining_steps, 40);
    let replay = core.replay_json().expect("replay serializes");
    let replayed = DemoCore::from_replay_json(&replay).expect("replay restores");
    assert_eq!(
        checkpoint,
        replayed.snapshot_json().expect("snapshot serializes")
    );

    let authority_before_reset = core.public_state().authority_revision;
    assert!(core.dispatch(SemanticAction::Reset).accepted);
    let reset = core.public_state();
    assert!(reset.leases.is_empty());
    assert_eq!(reset.authority_revision, authority_before_reset + 1);
    assert!(reset.fields.is_empty());
    assert_eq!(reset.groups.len(), 1);
}

#[test]
fn same_seed_lease_actions_produce_exact_behavior_and_motion() {
    let mut first = DemoCore::new(2_306);
    let mut second = DemoCore::new(2_306);
    for core in [&mut first, &mut second] {
        assert!(core.dispatch(request_lease(2, 1, 60, 0)).accepted);
        assert!(
            core.dispatch(set_leased_behavior(2, 1, CollectiveBehavior::Disperse, 1))
                .accepted
        );
        run_steps(core, 24);
    }
    assert_eq!(
        first.snapshot_json().expect("snapshot serializes"),
        second.snapshot_json().expect("snapshot serializes")
    );
    assert_eq!(first.frame_rows().unwrap(), second.frame_rows().unwrap());
}

#[test]
fn execution_controls_have_explicit_defaults_bounds_and_strict_damage_rejection() {
    let mut core = DemoCore::new(8080);
    let initial = core.public_state();
    assert_eq!(initial.execution_settings, DEFAULT_EXECUTION_SETTINGS);
    assert_eq!(
        initial.execution_settings.collision_policy,
        CollisionPolicy::CollisionFree
    );
    assert_eq!(initial.clearance_metrics.overlap_pair_count, 0);
    assert!(initial.clearance_metrics.minimum_surface_clearance >= 0.0);

    for action in [
        SemanticAction::SetSpeedLimit {
            value: MIN_SPEED_LIMIT,
        },
        SemanticAction::SetSpeedLimit {
            value: MAX_SPEED_LIMIT,
        },
        SemanticAction::SetAccelerationLimit {
            value: MIN_ACCELERATION_LIMIT,
        },
        SemanticAction::SetAccelerationLimit {
            value: MAX_ACCELERATION_LIMIT,
        },
        SemanticAction::SetSeparationRadius { value: 0.08 },
        SemanticAction::SetSeparationRadius { value: 0.30 },
        SemanticAction::SetSeparationWeight { value: 3.0 },
        SemanticAction::SetBoundaryStrength { value: 12.0 },
    ] {
        assert!(core.dispatch(action).accepted);
    }

    for action in [
        SemanticAction::SetSpeedLimit { value: f32::NAN },
        SemanticAction::SetSpeedLimit { value: 1.51 },
        SemanticAction::SetAccelerationLimit { value: 0.49 },
        SemanticAction::SetSeparationRadius { value: 0.31 },
        SemanticAction::SetSeparationWeight { value: -0.01 },
        SemanticAction::SetBoundaryStrength { value: 12.01 },
    ] {
        let before = core.snapshot_json().expect("snapshot serializes");
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, ActionCode::InvalidExecutionSetting);
        assert_eq!(core.snapshot_json().expect("snapshot serializes"), before);
    }

    for action in [
        SemanticAction::SetNavigationField {
            x: 0.0,
            y: 0.0,
            direction_x: 0.0,
            direction_y: 0.0,
            radius: 0.5,
            strength: 1.0,
        },
        SemanticAction::SetNavigationField {
            x: 0.0,
            y: 0.0,
            direction_x: f32::INFINITY,
            direction_y: 0.0,
            radius: 0.5,
            strength: 1.0,
        },
        SemanticAction::SetNavigationField {
            x: 0.0,
            y: 0.0,
            direction_x: 1.0,
            direction_y: 0.0,
            radius: 1.21,
            strength: 1.0,
        },
    ] {
        let receipt = core.dispatch(action);
        assert!(!receipt.accepted);
        assert_eq!(receipt.code, ActionCode::InvalidNavigationField);
    }

    let mut damaged: serde_json::Value =
        serde_json::from_str(&core.snapshot_json().expect("snapshot serializes"))
            .expect("snapshot parses");
    damaged["execution_settings"]["speed_limit"] = serde_json::json!(99.0);
    assert!(DemoCore::from_snapshot_json(&damaged.to_string()).is_err());
    let action = serde_json::json!({
        "type": "set_collision_policy",
        "policy": "collision_free",
        "hidden_override": true
    });
    assert!(serde_json::from_value::<SemanticAction>(action).is_err());
}

#[test]
fn collision_free_projection_prevents_overlap_under_the_same_pressure_tape() {
    let mut soft = DemoCore::new(9090);
    let mut collision_free = DemoCore::new(9090);
    configure_boundary_pressure(&mut soft, CollisionPolicy::SoftAvoidance);
    configure_boundary_pressure(&mut collision_free, CollisionPolicy::CollisionFree);
    run_steps(&mut soft, 12);
    run_steps(&mut collision_free, 12);

    let soft_state = soft.public_state();
    let hard_state = collision_free.public_state();
    assert_eq!(soft_state.tick, hard_state.tick);
    assert!(soft_state.clearance_metrics.overlap_pair_count > 0);
    assert!(soft_state.clearance_metrics.minimum_surface_clearance < 0.0);
    assert_eq!(hard_state.clearance_metrics.overlap_pair_count, 0);
    assert!(hard_state.clearance_metrics.minimum_surface_clearance >= 0.0);
    assert!(hard_state.clearance_metrics.total_intervention_count > 0);
    assert!(hard_state.clearance_metrics.contact_tick_count > 0);
    assert_eq!(soft_state.clearance_metrics.total_intervention_count, 0);
    assert_ne!(
        soft.frame_rows().unwrap(),
        collision_free.frame_rows().unwrap()
    );
}

#[test]
fn navigation_field_replay_reset_and_collision_policy_share_one_state_path() {
    let mut core = DemoCore::new(10_010);
    configure_boundary_pressure(&mut core, CollisionPolicy::CollisionFree);
    let configured = core
        .snapshot_json()
        .expect("configured snapshot serializes");
    run_steps(&mut core, 4);
    let final_snapshot = core.snapshot_json().expect("final snapshot serializes");
    let tape = core.replay_json().expect("replay serializes");
    let replayed = DemoCore::from_replay_json(&tape).expect("replay reconstructs");
    assert_eq!(replayed.snapshot_json().unwrap(), final_snapshot);
    assert_eq!(replayed.frame_rows().unwrap(), core.frame_rows().unwrap());

    let clear = core.dispatch(SemanticAction::ClearNavigationField);
    assert!(clear.accepted);
    assert_eq!(clear.code, ActionCode::NavigationFieldCleared);
    assert!(core.public_state().navigation_field.is_none());
    assert_eq!(
        core.public_state().execution_settings.collision_policy,
        CollisionPolicy::CollisionFree
    );

    assert!(core.dispatch(SemanticAction::Reset).accepted);
    let reset = core.public_state();
    assert_eq!(reset.execution_settings, DEFAULT_EXECUTION_SETTINGS);
    assert!(reset.navigation_field.is_none());
    assert_eq!(reset.clearance_metrics.total_intervention_count, 0);
    assert_ne!(core.snapshot_json().unwrap(), configured);
}

#[test]
fn collision_free_discs_survive_dense_cohesion_fields_and_boundaries() {
    let mut core = DemoCore::new(11_011);
    assert!(
        core.dispatch(SemanticAction::SetScope {
            scope: TargetScope::Swarm,
        })
        .accepted
    );
    let revision = core.public_state().selection_revision;
    assert!(
        core.dispatch(set_behavior(CollectiveBehavior::Cohere, revision))
            .accepted
    );
    for field_id in 0..MAX_PERSONAL_FIELDS {
        assert!(
            core.dispatch(place_field(
                u16::try_from(field_id).unwrap(),
                u8::try_from(field_id % 4).unwrap(),
                0.0,
                0.0,
                FieldPolarity::Attract,
                FieldLifetime::Persistent,
            ))
            .accepted
        );
    }
    assert!(
        core.dispatch(SemanticAction::SetNavigationField {
            x: 0.0,
            y: 0.0,
            direction_x: 1.0,
            direction_y: 0.0,
            radius: 1.2,
            strength: 3.0,
        })
        .accepted
    );
    run_steps(&mut core, 40);
    let state = core.public_state();
    assert_eq!(state.clearance_metrics.overlap_pair_count, 0);
    assert!(state.clearance_metrics.minimum_surface_clearance >= 0.0);
    assert!(state.clearance_metrics.total_intervention_count > 0);
    assert_eq!(state.fields.len(), MAX_PERSONAL_FIELDS);
    assert!(state.navigation_field.is_some());
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

fn assert_conserved_membership(state: &PublicState) {
    assert!(!state.groups.is_empty());
    assert!(state.groups.len() <= MAX_GROUPS);
    assert!(state
        .groups
        .windows(2)
        .all(|groups| groups[0].group_id < groups[1].group_id));

    let mut members = Vec::with_capacity(MEMBER_COUNT);
    for group in &state.groups {
        assert!(!group.member_ids.is_empty());
        assert!(group
            .member_ids
            .windows(2)
            .all(|members| members[0] < members[1]));
        members.extend_from_slice(&group.member_ids);
    }
    members.sort_unstable();
    assert_eq!(members, (0..member_count_u16()).collect::<Vec<_>>());
}

fn member_count_u16() -> u16 {
    u16::try_from(MEMBER_COUNT).expect("bounded member count fits in a u16")
}

fn centroid_x(rows: &[f32]) -> f32 {
    rows.chunks_exact(FRAME_ROW_WIDTH)
        .map(|row| row[1])
        .sum::<f32>()
        / 24.0
}

fn polarization(rows: &[f32]) -> f32 {
    let (heading_x, heading_y) =
        rows.chunks_exact(FRAME_ROW_WIDTH)
            .fold((0.0, 0.0), |(x, y), row| {
                let speed = row[4].hypot(row[5]);
                if speed <= f32::EPSILON {
                    (x, y)
                } else {
                    (x + row[4] / speed, y + row[5] / speed)
                }
            });
    heading_x.hypot(heading_y) / 24.0
}
