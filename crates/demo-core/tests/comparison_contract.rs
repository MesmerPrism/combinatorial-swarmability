//! Integration coverage for isolated deterministic comparison scenarios.

use combinatorial_swarmability_demo_core::{
    ComparisonRunner, DemoCore, SemanticAction, FRAME_ROW_WIDTH,
};
use serde_json::{json, Value};

const SPEC_SCHEMA: &str = "combinatorial.swarmability.comparison-spec.v1";
const TAPE_SCHEMA: &str = "combinatorial.swarmability.normalized-input-tape.v1";

fn spec(scenario: &str, tape_id: &str, seed: &str) -> String {
    json!({
        "schema": SPEC_SCHEMA,
        "scenario_id": scenario,
        "seed": seed,
        "left_seed": seed,
        "right_seed": seed,
        "input_tape": {
            "schema": TAPE_SCHEMA,
            "tape_id": tape_id,
        },
    })
    .to_string()
}

fn result(runner: &ComparisonRunner) -> Value {
    serde_json::from_str(&runner.result_json().expect("result serializes")).expect("result parses")
}

fn replay(runner: &mut ComparisonRunner) -> Value {
    serde_json::from_str(&runner.replay_all_json().expect("comparison replays"))
        .expect("replay result parses")
}

#[test]
fn strict_specs_reject_mismatched_schema_seed_tape_and_unknown_fields() {
    let valid = spec(
        "raw_semantic_equivalent",
        "raw-semantic-midpoint.v1",
        "2026",
    );
    assert!(ComparisonRunner::from_spec_json(&valid).is_ok());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["schema"] = json!("combinatorial.swarmability.comparison-spec.v0");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["input_tape"]["schema"] = json!("damaged");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["input_tape"]["tape_id"] = json!("superposition-lease.v1");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["right_seed"] = json!("2027");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["seed"] = json!("02");
    damaged["left_seed"] = json!("02");
    damaged["right_seed"] = json!("02");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let mut damaged: Value = serde_json::from_str(&valid).expect("spec parses");
    damaged["private_session"] = json!("reject");
    assert!(ComparisonRunner::from_spec_json(&damaged.to_string()).is_err());

    let oversized = format!("{}{}", valid, " ".repeat(4_096));
    assert!(ComparisonRunner::from_spec_json(&oversized).is_err());
}

#[test]
fn canonical_start_same_spec_and_replay_are_exact() {
    let request = spec(
        "raw_semantic_equivalent",
        "raw-semantic-midpoint.v1",
        "3030",
    );
    let mut first = ComparisonRunner::from_spec_json(&request).expect("runner builds");
    let mut second = ComparisonRunner::from_spec_json(&request).expect("runner builds");
    let initial = result(&first);
    assert_eq!(initial, result(&second));
    assert_eq!(initial["cursor"], 0);
    assert_eq!(initial["invariants"]["canonical_start_equal"], true);
    assert_eq!(initial["invariants"]["shared_seed"], true);
    assert_eq!(initial["invariants"]["lockstep_tick"], true);
    assert_eq!(initial["left"]["state"], initial["right"]["state"]);
    assert_eq!(
        first.left_frame_rows().expect("left frame"),
        first.right_frame_rows().expect("right frame")
    );

    let first_result = replay(&mut first);
    let second_result = replay(&mut second);
    assert_eq!(first_result, second_result);
    assert_eq!(first_result["complete"], true);
    assert_eq!(first_result["left"]["metrics"]["tick"], 40);
    assert_eq!(first_result["right"]["metrics"]["tick"], 40);

    first.reset().expect("reset succeeds");
    assert_eq!(result(&first), initial);
    assert_eq!(replay(&mut first), first_result);
}

#[test]
fn equivalent_raw_and_semantic_vectors_produce_equal_frames_and_metrics() {
    let mut runner = ComparisonRunner::from_spec_json(&spec(
        "raw_semantic_equivalent",
        "raw-semantic-midpoint.v1",
        "4040",
    ))
    .expect("runner builds");
    runner.step_event_json().expect("configuration applies");
    let configured = result(&runner);
    assert_eq!(configured["vector_relation"], "equivalent");
    assert_eq!(
        configured["left"]["state"]["resolved_dynamics"],
        configured["right"]["state"]["resolved_dynamics"]
    );
    assert_eq!(
        configured["left"]["state"]["dynamics_control_mode"],
        "comparison_raw_mirror"
    );
    assert_eq!(
        configured["right"]["state"]["dynamics_control_mode"],
        "semantic"
    );
    assert_eq!(
        configured["left"]["descriptor"]["catalogue_entry_id"],
        "raw-dynamics-parameters"
    );
    assert_eq!(
        configured["right"]["descriptor"]["catalogue_entry_id"],
        "semantic-laban-dynamics"
    );

    let final_result = replay(&mut runner);
    assert_eq!(final_result["vector_relation"], "equivalent");
    assert_eq!(
        runner.left_frame_rows().expect("left frame"),
        runner.right_frame_rows().expect("right frame")
    );
    for metric in [
        "cohesion",
        "polarization",
        "mean_spacing",
        "average_speed",
        "group_count",
        "mean_formation_extent",
        "active_fields",
        "active_leases",
    ] {
        assert!(
            final_result["delta_right_minus_left"][metric]
                .as_f64()
                .expect("numeric delta")
                .abs()
                < f64::EPSILON
        );
    }
    assert_eq!(
        final_result["delta_right_minus_left"]["behavior_distribution"],
        json!({"flock": 0, "cohere": 0, "disperse": 0})
    );
}

#[test]
fn intentionally_different_profiles_preserve_start_and_change_only_lane_configuration() {
    let mut runner = ComparisonRunner::from_spec_json(&spec(
        "raw_semantic_contrast",
        "raw-semantic-contrast.v1",
        "5050",
    ))
    .expect("runner builds");
    let initial = result(&runner);
    assert_eq!(initial["left"]["state"], initial["right"]["state"]);

    runner.step_event_json().expect("configuration applies");
    let configured = result(&runner);
    assert_eq!(configured["vector_relation"], "intentionally_different");
    assert_ne!(
        configured["left"]["state"]["resolved_dynamics"],
        configured["right"]["state"]["resolved_dynamics"]
    );
    assert_eq!(configured["left"]["state"]["fields"], json!([]));
    assert_eq!(configured["right"]["state"]["fields"], json!([]));
    assert_eq!(configured["left"]["state"]["leases"], json!([]));
    assert_eq!(configured["right"]["state"]["leases"], json!([]));
    assert_eq!(configured["left"]["state"]["tick"], 0);
    assert_eq!(configured["right"]["state"]["tick"], 0);

    let final_result = replay(&mut runner);
    assert_eq!(final_result["complete"], true);
    assert_ne!(
        final_result["left"]["metrics"]["behavior_distribution"],
        final_result["right"]["metrics"]["behavior_distribution"]
    );
    assert_ne!(final_result["delta_right_minus_left"]["mean_spacing"], 0);
    assert_eq!(final_result["delta_right_minus_left"]["tick"], 0);
}

#[test]
fn superposition_and_lease_show_distinct_policy_outcomes_without_cross_lane_state() {
    let mut runner = ComparisonRunner::from_spec_json(&spec(
        "superposition_lease",
        "superposition-lease.v1",
        "6060",
    ))
    .expect("runner builds");
    runner.step_event_json().expect("first influence applies");
    let first = result(&runner);
    assert_eq!(
        first["left"]["state"]["fields"].as_array().unwrap().len(),
        1
    );
    assert_eq!(first["left"]["state"]["leases"], json!([]));
    assert_eq!(first["right"]["state"]["fields"], json!([]));
    assert_eq!(
        first["right"]["state"]["leases"].as_array().unwrap().len(),
        1
    );
    assert_eq!(first["left"]["state"]["behavior_counts"]["cohere"], 0);
    assert_eq!(first["right"]["state"]["behavior_counts"]["cohere"], 1);

    runner.step_event_json().expect("second influence resolves");
    let second = result(&runner);
    assert_eq!(
        second["left"]["state"]["fields"].as_array().unwrap().len(),
        2
    );
    assert_eq!(
        second["right"]["state"]["leases"].as_array().unwrap().len(),
        1
    );
    let right_receipts = second["right"]["trace"][1]["receipts"]
        .as_array()
        .expect("right receipts");
    assert_eq!(right_receipts.len(), 2);
    assert_eq!(right_receipts[0]["accepted"], false);
    assert_eq!(right_receipts[0]["code"], "lease_already_held");
    assert_eq!(right_receipts[1]["accepted"], false);
    assert_eq!(right_receipts[1]["code"], "not_lease_holder");

    let final_result = replay(&mut runner);
    assert_eq!(final_result["delta_right_minus_left"]["active_fields"], -2);
    assert_eq!(final_result["delta_right_minus_left"]["active_leases"], 1);
    assert_eq!(final_result["delta_right_minus_left"]["tick"], 0);
    assert_eq!(final_result["invariants"]["lockstep_tick"], true);
    assert_eq!(final_result["invariants"]["isolated_mutable_cores"], true);
    assert_eq!(
        final_result["invariants"]["ordinary_atlas_state_touched"],
        false
    );
}

#[test]
fn comparison_does_not_mutate_an_ordinary_atlas_core() {
    let mut ordinary = DemoCore::new(7070);
    assert!(ordinary.dispatch(SemanticAction::Step).accepted);
    let ordinary_before = ordinary.snapshot_json().expect("ordinary snapshot");
    let ordinary_frame = ordinary.frame_rows().expect("ordinary frame");

    let mut runner = ComparisonRunner::from_spec_json(&spec(
        "superposition_lease",
        "superposition-lease.v1",
        "7070",
    ))
    .expect("runner builds");
    replay(&mut runner);

    assert_eq!(
        ordinary.snapshot_json().expect("ordinary snapshot"),
        ordinary_before
    );
    assert_eq!(
        ordinary.frame_rows().expect("ordinary frame"),
        ordinary_frame
    );
}

#[test]
fn tape_results_traces_and_frame_rows_remain_bounded_and_well_formed() {
    for (scenario, tape) in [
        ("raw_semantic_equivalent", "raw-semantic-midpoint.v1"),
        ("raw_semantic_contrast", "raw-semantic-contrast.v1"),
        ("superposition_lease", "superposition-lease.v1"),
    ] {
        let mut runner =
            ComparisonRunner::from_spec_json(&spec(scenario, tape, "8080")).expect("runner builds");
        assert!(runner.event_count() <= 16);
        let final_json = runner.replay_all_json().expect("comparison replays");
        assert!(final_json.len() < 500_000);
        let final_result: Value = serde_json::from_str(&final_json).expect("result parses");
        assert!(final_result["left"]["trace"].as_array().unwrap().len() <= 16);
        assert!(final_result["right"]["trace"].as_array().unwrap().len() <= 16);
        assert_eq!(
            runner.left_frame_rows().expect("left frame").len() % FRAME_ROW_WIDTH,
            0
        );
        assert_eq!(
            runner.right_frame_rows().expect("right frame").len() % FRAME_ROW_WIDTH,
            0
        );
        assert_eq!(
            final_result["metric_definitions"].as_array().unwrap().len(),
            10
        );
    }
}
