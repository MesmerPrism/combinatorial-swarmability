use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    ActionReceipt, BehaviorCounts, CollectiveBehavior, CollisionPolicy, DemoCore, DemoError,
    FieldLifetime, FieldPolarity, PublicState, SemanticAction, TargetScope, FRAME_ROW_WIDTH,
};

const SPEC_SCHEMA: &str = "combinatorial.swarmability.comparison-spec.v1";
const TAPE_SCHEMA: &str = "combinatorial.swarmability.normalized-input-tape.v1";
const RESULT_SCHEMA: &str = "combinatorial.swarmability.comparison-result.v1";
const STEP_RECEIPT_SCHEMA: &str = "combinatorial.swarmability.comparison-step-receipt.v1";
const MAX_SPEC_JSON_BYTES: usize = 4_096;
const MAX_RESULT_JSON_BYTES: usize = 500_000;
const MAX_INPUT_EVENTS: usize = 16;
const MAX_ACTION_RECEIPTS_PER_LANE: usize = 64;
const COMPARISON_STEPS: usize = 40;
const STEPS_PER_EVENT: usize = 5;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ScenarioId {
    RawSemanticEquivalent,
    RawSemanticContrast,
    SuperpositionLease,
    SoftCollisionFree,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct InputTapeBinding {
    schema: String,
    tape_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ComparisonSpec {
    schema: String,
    scenario_id: ScenarioId,
    seed: String,
    left_seed: String,
    right_seed: String,
    input_tape: InputTapeBinding,
}

#[derive(Clone, Copy, Debug)]
enum InputOperation {
    ConfigureDynamics,
    ConfigureCollision,
    FirstInfluence,
    SecondInfluence,
    Advance { steps: usize },
}

#[derive(Clone, Copy, Debug)]
struct NormalizedInputEvent {
    normalized_input: &'static str,
    semantic_request: &'static str,
    operation: InputOperation,
}

#[derive(Clone, Debug, Serialize)]
struct LaneTrace {
    sequence: usize,
    input_route: &'static str,
    normalized_input: String,
    semantic_request: String,
    semantic_actions: Vec<String>,
    policy: String,
    receipts: Vec<ActionReceipt>,
    tick_before: u64,
    tick_after: u64,
    state_revision_before: u64,
    state_revision_after: u64,
}

#[derive(Clone, Debug, Serialize)]
struct LaneDescriptor {
    lane_id: &'static str,
    label: &'static str,
    mapping_id: &'static str,
    configuration_id: &'static str,
    catalogue_entry_id: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct LaneMetrics {
    tick: u64,
    cohesion: f32,
    polarization: f32,
    mean_spacing: f32,
    average_speed: f32,
    group_count: usize,
    group_sizes: Vec<usize>,
    mean_formation_extent: f32,
    behavior_distribution: BehaviorCounts,
    active_fields: usize,
    active_leases: usize,
    minimum_surface_clearance: f32,
    overlap_pair_count: usize,
    near_miss_pair_count: usize,
    total_collision_interventions: u64,
    contact_tick_count: u64,
}

#[derive(Clone, Debug, Serialize)]
struct BehaviorDelta {
    flock: i32,
    cohere: i32,
    disperse: i32,
}

#[derive(Clone, Debug, Serialize)]
struct MetricDelta {
    tick: i64,
    cohesion: f32,
    polarization: f32,
    mean_spacing: f32,
    average_speed: f32,
    group_count: i32,
    mean_formation_extent: f32,
    behavior_distribution: BehaviorDelta,
    active_fields: i32,
    active_leases: i32,
    minimum_surface_clearance: f32,
    overlap_pair_count: i32,
    near_miss_pair_count: i32,
    total_collision_interventions: i64,
    contact_tick_count: i64,
}

#[derive(Clone, Debug, Serialize)]
struct LaneResult {
    descriptor: LaneDescriptor,
    state: PublicState,
    metrics: LaneMetrics,
    trace: Vec<LaneTrace>,
    final_state_provenance: String,
}

#[derive(Clone, Debug, Serialize)]
#[allow(clippy::struct_excessive_bools)] // These are independently inspectable invariant receipts.
struct ComparisonInvariants {
    canonical_start_equal: bool,
    shared_seed: bool,
    shared_input_tape: bool,
    lockstep_tick: bool,
    isolated_mutable_cores: bool,
    ordinary_atlas_state_touched: bool,
}

#[derive(Clone, Debug, Serialize)]
struct MetricDefinition {
    metric_id: &'static str,
    label: &'static str,
    unit: &'static str,
    definition: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct ComparisonResult {
    schema: &'static str,
    spec: ComparisonSpec,
    cursor: usize,
    event_count: usize,
    complete: bool,
    vector_relation: &'static str,
    invariants: ComparisonInvariants,
    left: LaneResult,
    right: LaneResult,
    delta_right_minus_left: MetricDelta,
    metric_definitions: Vec<MetricDefinition>,
    claim_boundary: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct ComparisonStepReceipt {
    schema: &'static str,
    accepted: bool,
    code: &'static str,
    event_index: Option<usize>,
    cursor: usize,
    event_count: usize,
    normalized_input: Option<String>,
    left_tick: u64,
    right_tick: u64,
    lockstep_tick: bool,
}

/// Error returned when a comparison specification, lane, or result is invalid.
#[derive(Debug)]
pub enum ComparisonError {
    /// Strict JSON serialization or deserialization failed.
    Json(serde_json::Error),
    /// The immutable comparison specification is unsupported or inconsistent.
    InvalidSpec(&'static str),
    /// One deterministic core rejected a snapshot or frame projection.
    Core(DemoError),
    /// A cross-lane invariant failed before a transactional step could commit.
    Invariant(&'static str),
}

impl fmt::Display for ComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "comparison JSON error: {error}"),
            Self::InvalidSpec(message) => write!(formatter, "invalid comparison spec: {message}"),
            Self::Core(error) => write!(formatter, "comparison core error: {error}"),
            Self::Invariant(message) => write!(formatter, "comparison invariant failed: {message}"),
        }
    }
}

impl std::error::Error for ComparisonError {}

impl From<serde_json::Error> for ComparisonError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<DemoError> for ComparisonError {
    fn from(value: DemoError) -> Self {
        Self::Core(value)
    }
}

/// Two isolated deterministic scenarios bound to one canonical seed and input tape.
#[derive(Clone, Debug)]
pub struct ComparisonRunner {
    spec: ComparisonSpec,
    seed: u64,
    events: Vec<NormalizedInputEvent>,
    cursor: usize,
    left: DemoCore,
    right: DemoCore,
    left_trace: Vec<LaneTrace>,
    right_trace: Vec<LaneTrace>,
    canonical_start_equal: bool,
}

impl ComparisonRunner {
    /// Constructs a runner from one strict, versioned immutable comparison request.
    ///
    /// # Errors
    ///
    /// Rejects unknown fields, unsupported schemas or tape IDs, invalid decimal
    /// seeds, and any left/right seed binding that differs from the canonical seed.
    pub fn from_spec_json(json: &str) -> Result<Self, ComparisonError> {
        if json.len() > MAX_SPEC_JSON_BYTES {
            return Err(ComparisonError::InvalidSpec("spec byte limit exceeded"));
        }
        let spec: ComparisonSpec = serde_json::from_str(json)?;
        let seed = validate_spec(&spec)?;
        Self::build(spec, seed)
    }

    /// Applies one canonical normalized-input event transactionally to both lanes.
    ///
    /// # Errors
    ///
    /// Leaves both lanes unchanged when either mapping or a lockstep invariant fails.
    pub fn step_event_json(&mut self) -> Result<String, ComparisonError> {
        let receipt = self.step_event()?;
        Ok(serde_json::to_string(&receipt)?)
    }

    /// Restores both lanes to independent clones of the exact canonical start.
    ///
    /// # Errors
    ///
    /// Rejects if the reconstructed lane snapshots are not byte-identical.
    pub fn reset(&mut self) -> Result<(), ComparisonError> {
        let canonical = DemoCore::new(self.seed);
        let left = canonical.clone();
        let right = canonical;
        let equal = snapshots_equal(&left, &right)?;
        if !equal {
            return Err(ComparisonError::Invariant(
                "reset lanes do not share one canonical initial snapshot",
            ));
        }
        self.left = left;
        self.right = right;
        self.left_trace.clear();
        self.right_trace.clear();
        self.cursor = 0;
        self.canonical_start_equal = true;
        Ok(())
    }

    /// Resets and deterministically replays the complete normalized-input tape.
    ///
    /// # Errors
    ///
    /// Fails closed on any lane rejection or cross-lane drift.
    pub fn replay_all_json(&mut self) -> Result<String, ComparisonError> {
        self.reset()?;
        while self.cursor < self.events.len() {
            self.step_event()?;
        }
        self.result_json()
    }

    /// Serializes the current versioned comparison result and lane provenance.
    ///
    /// # Errors
    ///
    /// Rejects an unexpectedly oversized result or a failed frame projection.
    pub fn result_json(&self) -> Result<String, ComparisonError> {
        let result = self.result()?;
        let json = serde_json::to_string(&result)?;
        if json.len() > MAX_RESULT_JSON_BYTES {
            return Err(ComparisonError::Invariant("result byte limit exceeded"));
        }
        Ok(json)
    }

    /// Returns the current left-lane renderer-neutral frame rows.
    ///
    /// # Errors
    ///
    /// Returns an error when Rusty Matter rejects the lane projection.
    pub fn left_frame_rows(&self) -> Result<Vec<f32>, ComparisonError> {
        Ok(self.left.frame_rows()?)
    }

    /// Returns the current right-lane renderer-neutral frame rows.
    ///
    /// # Errors
    ///
    /// Returns an error when Rusty Matter rejects the lane projection.
    pub fn right_frame_rows(&self) -> Result<Vec<f32>, ComparisonError> {
        Ok(self.right.frame_rows()?)
    }

    /// Returns the fixed bounded event count for the selected canonical tape.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    fn build(spec: ComparisonSpec, seed: u64) -> Result<Self, ComparisonError> {
        let events = canonical_events(spec.scenario_id);
        if events.is_empty() || events.len() > MAX_INPUT_EVENTS {
            return Err(ComparisonError::Invariant(
                "canonical input tape is empty or exceeds its bound",
            ));
        }
        let canonical = DemoCore::new(seed);
        let left = canonical.clone();
        let right = canonical;
        let canonical_start_equal = snapshots_equal(&left, &right)?;
        if !canonical_start_equal {
            return Err(ComparisonError::Invariant(
                "lanes do not share one canonical initial snapshot",
            ));
        }
        Ok(Self {
            spec,
            seed,
            events,
            cursor: 0,
            left,
            right,
            left_trace: Vec::new(),
            right_trace: Vec::new(),
            canonical_start_equal,
        })
    }

    fn step_event(&mut self) -> Result<ComparisonStepReceipt, ComparisonError> {
        if self.cursor >= self.events.len() {
            let left_tick = self.left.public_state().tick;
            let right_tick = self.right.public_state().tick;
            return Ok(ComparisonStepReceipt {
                schema: STEP_RECEIPT_SCHEMA,
                accepted: true,
                code: "tape_complete",
                event_index: None,
                cursor: self.cursor,
                event_count: self.events.len(),
                normalized_input: None,
                left_tick,
                right_tick,
                lockstep_tick: left_tick == right_tick,
            });
        }

        ensure_lockstep(&self.left, &self.right, self.seed)?;
        let event = self.events[self.cursor];
        let mut next_left = self.left.clone();
        let mut next_right = self.right.clone();
        let left_trace = apply_event(
            self.spec.scenario_id,
            true,
            self.cursor,
            event,
            &mut next_left,
        )?;
        let right_trace = apply_event(
            self.spec.scenario_id,
            false,
            self.cursor,
            event,
            &mut next_right,
        )?;
        ensure_lockstep(&next_left, &next_right, self.seed)?;

        let next_receipt_count = self
            .left_trace
            .iter()
            .map(|trace| trace.receipts.len())
            .sum::<usize>()
            .saturating_add(left_trace.receipts.len());
        let right_receipt_count = self
            .right_trace
            .iter()
            .map(|trace| trace.receipts.len())
            .sum::<usize>()
            .saturating_add(right_trace.receipts.len());
        if next_receipt_count > MAX_ACTION_RECEIPTS_PER_LANE
            || right_receipt_count > MAX_ACTION_RECEIPTS_PER_LANE
        {
            return Err(ComparisonError::Invariant(
                "lane action-receipt limit exceeded",
            ));
        }

        self.left = next_left;
        self.right = next_right;
        self.left_trace.push(left_trace);
        self.right_trace.push(right_trace);
        let event_index = self.cursor;
        self.cursor += 1;
        let left_tick = self.left.public_state().tick;
        let right_tick = self.right.public_state().tick;
        Ok(ComparisonStepReceipt {
            schema: STEP_RECEIPT_SCHEMA,
            accepted: true,
            code: "event_applied",
            event_index: Some(event_index),
            cursor: self.cursor,
            event_count: self.events.len(),
            normalized_input: Some(event.normalized_input.to_owned()),
            left_tick,
            right_tick,
            lockstep_tick: left_tick == right_tick,
        })
    }

    fn result(&self) -> Result<ComparisonResult, ComparisonError> {
        ensure_lockstep(&self.left, &self.right, self.seed)?;
        let left_state = self.left.public_state();
        let right_state = self.right.public_state();
        let left_metrics = lane_metrics(&self.left, &left_state)?;
        let right_metrics = lane_metrics(&self.right, &right_state)?;
        let vector_relation = match self.spec.scenario_id {
            ScenarioId::SuperpositionLease | ScenarioId::SoftCollisionFree => "not_applicable",
            _ if self.cursor == 0 => "pending_configuration",
            _ if left_state.resolved_dynamics == right_state.resolved_dynamics => "equivalent",
            _ => "intentionally_different",
        };
        Ok(ComparisonResult {
            schema: RESULT_SCHEMA,
            spec: self.spec.clone(),
            cursor: self.cursor,
            event_count: self.events.len(),
            complete: self.cursor == self.events.len(),
            vector_relation,
            invariants: ComparisonInvariants {
                canonical_start_equal: self.canonical_start_equal,
                shared_seed: left_state.seed == right_state.seed
                    && left_state.seed == self.seed.to_string(),
                shared_input_tape: true,
                lockstep_tick: left_state.tick == right_state.tick,
                isolated_mutable_cores: true,
                ordinary_atlas_state_touched: false,
            },
            left: LaneResult {
                descriptor: lane_descriptor(self.spec.scenario_id, true),
                final_state_provenance: final_provenance(&left_state),
                state: left_state,
                metrics: left_metrics.clone(),
                trace: self.left_trace.clone(),
            },
            right: LaneResult {
                descriptor: lane_descriptor(self.spec.scenario_id, false),
                final_state_provenance: final_provenance(&right_state),
                state: right_state,
                metrics: right_metrics.clone(),
                trace: self.right_trace.clone(),
            },
            delta_right_minus_left: metric_delta(&left_metrics, &right_metrics),
            metric_definitions: metric_definitions(),
            claim_boundary: "Deterministic reconstruction outcome for this exact seed, normalized input tape, and app-owned configuration; not a research result or evidence of live multi-user behavior.",
        })
    }
}

fn validate_spec(spec: &ComparisonSpec) -> Result<u64, ComparisonError> {
    if spec.schema != SPEC_SCHEMA {
        return Err(ComparisonError::InvalidSpec("unsupported spec schema"));
    }
    if spec.input_tape.schema != TAPE_SCHEMA {
        return Err(ComparisonError::InvalidSpec(
            "unsupported normalized-input tape schema",
        ));
    }
    if spec.input_tape.tape_id != expected_tape_id(spec.scenario_id) {
        return Err(ComparisonError::InvalidSpec(
            "input tape does not match the selected scenario",
        ));
    }
    let seed = parse_seed(&spec.seed)?;
    if spec.left_seed != spec.seed || spec.right_seed != spec.seed {
        return Err(ComparisonError::InvalidSpec(
            "left and right lane seeds must match the canonical seed",
        ));
    }
    Ok(seed)
}

fn parse_seed(seed: &str) -> Result<u64, ComparisonError> {
    if seed.is_empty()
        || seed.len() > 20
        || !seed.bytes().all(|byte| byte.is_ascii_digit())
        || (seed.len() > 1 && seed.starts_with('0'))
    {
        return Err(ComparisonError::InvalidSpec(
            "seed must be one canonical decimal u64 string",
        ));
    }
    seed.parse::<u64>()
        .map_err(|_| ComparisonError::InvalidSpec("seed must be one canonical decimal u64 string"))
}

const fn expected_tape_id(scenario: ScenarioId) -> &'static str {
    match scenario {
        ScenarioId::RawSemanticEquivalent => "raw-semantic-midpoint.v1",
        ScenarioId::RawSemanticContrast => "raw-semantic-contrast.v1",
        ScenarioId::SuperpositionLease => "superposition-lease.v1",
        ScenarioId::SoftCollisionFree => "soft-collision-free.v1",
    }
}

fn canonical_events(scenario: ScenarioId) -> Vec<NormalizedInputEvent> {
    let mut events = Vec::with_capacity(10);
    match scenario {
        ScenarioId::RawSemanticEquivalent | ScenarioId::RawSemanticContrast => {
            events.push(NormalizedInputEvent {
                normalized_input: "movement-quality.profile.apply",
                semantic_request: "Apply the selected movement-dynamics profile",
                operation: InputOperation::ConfigureDynamics,
            });
        }
        ScenarioId::SuperpositionLease => {
            events.push(NormalizedInputEvent {
                normalized_input: "operator-a.member-1.cohere-influence",
                semantic_request:
                    "Synthetic Operator A requests cohere-oriented influence on Member 1",
                operation: InputOperation::FirstInfluence,
            });
            events.push(NormalizedInputEvent {
                normalized_input: "operator-b.member-1.disperse-influence",
                semantic_request: "Synthetic Operator B requests simultaneous disperse-oriented influence on Member 1",
                operation: InputOperation::SecondInfluence,
            });
        }
        ScenarioId::SoftCollisionFree => {
            events.push(NormalizedInputEvent {
                normalized_input: "navigation-field.rightward-boundary-pressure",
                semantic_request:
                    "Apply the same navigation field and pace while changing only overlap policy",
                operation: InputOperation::ConfigureCollision,
            });
        }
    }
    for _ in 0..(COMPARISON_STEPS / STEPS_PER_EVENT) {
        events.push(NormalizedInputEvent {
            normalized_input: "fixed-step.advance(5)",
            semantic_request: "Advance both deterministic scenarios by five fixed steps",
            operation: InputOperation::Advance {
                steps: STEPS_PER_EVENT,
            },
        });
    }
    events
}

fn apply_event(
    scenario: ScenarioId,
    is_left: bool,
    sequence: usize,
    event: NormalizedInputEvent,
    core: &mut DemoCore,
) -> Result<LaneTrace, ComparisonError> {
    let before = core.public_state();
    let (actions, receipts, policy) = match event.operation {
        InputOperation::ConfigureDynamics => apply_dynamics_configuration(scenario, is_left, core)?,
        InputOperation::ConfigureCollision => apply_collision_configuration(is_left, core)?,
        InputOperation::FirstInfluence => apply_first_influence(is_left, core)?,
        InputOperation::SecondInfluence => apply_second_influence(is_left, core)?,
        InputOperation::Advance { steps } => apply_advance(steps, core)?,
    };
    let after = core.public_state();
    Ok(LaneTrace {
        sequence: sequence + 1,
        input_route: "deterministic_replay",
        normalized_input: event.normalized_input.to_owned(),
        semantic_request: event.semantic_request.to_owned(),
        semantic_actions: actions,
        policy,
        receipts,
        tick_before: before.tick,
        tick_after: after.tick,
        state_revision_before: before.state_revision,
        state_revision_after: after.state_revision,
    })
}

fn apply_dynamics_configuration(
    scenario: ScenarioId,
    is_left: bool,
    core: &mut DemoCore,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    match (scenario, is_left) {
        (ScenarioId::RawSemanticEquivalent, true) => dispatch_actions(
            core,
            vec![SemanticAction::ApplyComparisonRawMirror],
            &[true],
            "Explicit fixed raw-vector mirror; no source coefficient is invented and no second simulation path is selected.",
        ),
        (ScenarioId::RawSemanticEquivalent, false) => dispatch_actions(
            core,
            semantic_profile(0.5, 0.5, 0.5, 0.5),
            &[true, true, true, true],
            "Space, Time, Weight, and Flow compile through the documented app-owned interpolation into the established core vector.",
        ),
        (ScenarioId::RawSemanticContrast, true) => dispatch_actions(
            core,
            vec![
                SemanticAction::SetAlignment { rate: 0.10 },
                SemanticAction::SetCohesion { rate: 0.20 },
                SemanticAction::SetSeparation { rate: 0.85 },
            ],
            &[true, true, true],
            "Three accepted source-bound raw rates own the vector; speed scale, damping, and jitter retain neutral raw defaults.",
        ),
        (ScenarioId::RawSemanticContrast, false) => dispatch_actions(
            core,
            semantic_profile(0.9, 0.8, 0.2, 0.9),
            &[true, true, true, true],
            "The intentionally contrasting semantic profile compiles through the same core-owned interpolation.",
        ),
        (ScenarioId::SuperpositionLease | ScenarioId::SoftCollisionFree, _) => Err(ComparisonError::Invariant(
            "authority scenario received a dynamics event",
        )),
    }
}

fn apply_collision_configuration(
    is_left: bool,
    core: &mut DemoCore,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    let policy = if is_left {
        CollisionPolicy::SoftAvoidance
    } else {
        CollisionPolicy::CollisionFree
    };
    let (mut actions, mut receipts, _) = dispatch_actions(
        core,
        vec![
            SemanticAction::SetCollisionPolicy { policy },
            SemanticAction::SetSeparationWeight { value: 0.0 },
            SemanticAction::SetBoundaryStrength { value: 0.0 },
            SemanticAction::SetNavigationField {
                x: 0.0,
                y: 0.0,
                direction_x: 1.0,
                direction_y: 0.0,
                radius: 1.2,
                strength: 3.0,
            },
            SemanticAction::SetScope {
                scope: TargetScope::Swarm,
            },
        ],
        &[true, true, true, true, true],
        "Both lanes receive the same app-owned navigation region and steering controls.",
    )?;
    let revision = core.public_state().selection_revision;
    let (pace_actions, pace_receipts, _) = dispatch_actions(
        core,
        vec![SemanticAction::AdjustSpeed {
            delta: 0.5,
            expected_selection_revision: revision,
        }],
        &[true],
        "The same whole-swarm pace request is applied after scope resolution.",
    )?;
    actions.extend(pace_actions);
    receipts.extend(pace_receipts);
    Ok((
        actions,
        receipts,
        if is_left {
            "Steering-only soft avoidance measures overlap but does not correct it."
        } else {
            "Collision-free execution adds anticipatory steering and deterministic disc projection after the same core integration."
        }
        .to_owned(),
    ))
}

fn semantic_profile(space: f32, time: f32, weight: f32, flow: f32) -> Vec<SemanticAction> {
    vec![
        SemanticAction::SetSpaceQuality { value: space },
        SemanticAction::SetTimeQuality { value: time },
        SemanticAction::SetWeightQuality { value: weight },
        SemanticAction::SetFlowQuality { value: flow },
    ]
}

fn apply_first_influence(
    is_left: bool,
    core: &mut DemoCore,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    if is_left {
        dispatch_actions(
            core,
            vec![SemanticAction::PlaceField {
                field_id: 0,
                contributor_id: 0,
                x: -0.55,
                y: 0.0,
                polarity: FieldPolarity::Attract,
                lifetime: FieldLifetime::Persistent,
            }],
            &[true],
            "Additive superposition accepts the first bounded synthetic contributor field without exclusive ownership.",
        )
    } else {
        let request = SemanticAction::RequestLease {
            member_id: 0,
            operator_id: 0,
            lifetime_steps: 120,
            expected_authority_revision: core.public_state().authority_revision,
        };
        let (mut actions, mut receipts, _) = dispatch_actions(
            core,
            vec![request],
            &[true],
            "Exclusive lease authority first resolves the holder before member mutation.",
        )?;
        let behavior = SemanticAction::SetLeasedBehavior {
            member_id: 0,
            operator_id: 0,
            behavior: CollectiveBehavior::Cohere,
            expected_authority_revision: core.public_state().authority_revision,
        };
        let (next_actions, next_receipts, _) = dispatch_actions(
            core,
            vec![behavior],
            &[true],
            "Exclusive lease authority first resolves the holder before member mutation.",
        )?;
        actions.extend(next_actions);
        receipts.extend(next_receipts);
        Ok((
            actions,
            receipts,
            "Exclusive member policy acquires Operator A's fixed-step lease, then accepts only its holder-gated cohere request."
                .to_owned(),
        ))
    }
}

fn apply_second_influence(
    is_left: bool,
    core: &mut DemoCore,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    if is_left {
        dispatch_actions(
            core,
            vec![SemanticAction::PlaceField {
                field_id: 1,
                contributor_id: 1,
                x: 0.55,
                y: 0.0,
                polarity: FieldPolarity::Repel,
                lifetime: FieldLifetime::Persistent,
            }],
            &[true],
            "Additive superposition retains both fields in stable ID order; neither synthetic contributor excludes the other.",
        )
    } else {
        let revision = core.public_state().authority_revision;
        dispatch_actions(
            core,
            vec![
                SemanticAction::RequestLease {
                    member_id: 0,
                    operator_id: 1,
                    lifetime_steps: 120,
                    expected_authority_revision: revision,
                },
                SemanticAction::SetLeasedBehavior {
                    member_id: 0,
                    operator_id: 1,
                    behavior: CollectiveBehavior::Disperse,
                    expected_authority_revision: revision,
                },
            ],
            &[false, false],
            "Exclusive member policy rejects Operator B while Operator A's unexpired lease remains current; no hidden arbitration occurs.",
        )
    }
}

fn apply_advance(
    steps: usize,
    core: &mut DemoCore,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    if steps == 0 || steps > STEPS_PER_EVENT {
        return Err(ComparisonError::Invariant(
            "comparison advance is outside its canonical bound",
        ));
    }
    let actions = (0..steps).map(|_| SemanticAction::Step).collect::<Vec<_>>();
    dispatch_actions(
        core,
        actions,
        &vec![true; steps],
        "Both isolated cores advance by the same explicit fixed-step count; elapsed wall time is not consulted.",
    )
}

fn dispatch_actions(
    core: &mut DemoCore,
    actions: Vec<SemanticAction>,
    expected_acceptance: &[bool],
    policy: &str,
) -> Result<(Vec<String>, Vec<ActionReceipt>, String), ComparisonError> {
    if actions.len() != expected_acceptance.len() {
        return Err(ComparisonError::Invariant(
            "action acceptance expectation is malformed",
        ));
    }
    let mut labels = Vec::with_capacity(actions.len());
    let mut receipts = Vec::with_capacity(actions.len());
    for (action, expected) in actions.into_iter().zip(expected_acceptance.iter().copied()) {
        labels.push(serde_json::to_string(&action)?);
        let receipt = core.dispatch(action);
        if receipt.accepted != expected {
            return Err(ComparisonError::Invariant(
                "lane action differed from its immutable expected policy outcome",
            ));
        }
        receipts.push(receipt);
    }
    Ok((labels, receipts, policy.to_owned()))
}

fn ensure_lockstep(left: &DemoCore, right: &DemoCore, seed: u64) -> Result<(), ComparisonError> {
    let left_state = left.public_state();
    let right_state = right.public_state();
    if left_state.seed != seed.to_string() || right_state.seed != seed.to_string() {
        return Err(ComparisonError::Invariant("lane seed drift detected"));
    }
    if left_state.tick != right_state.tick {
        return Err(ComparisonError::Invariant("lane tick drift detected"));
    }
    Ok(())
}

fn snapshots_equal(left: &DemoCore, right: &DemoCore) -> Result<bool, ComparisonError> {
    Ok(left.snapshot_json()? == right.snapshot_json()?)
}

const fn lane_descriptor(scenario: ScenarioId, is_left: bool) -> LaneDescriptor {
    match (scenario, is_left) {
        (ScenarioId::RawSemanticEquivalent, true) => LaneDescriptor {
            lane_id: "lane_a",
            label: "Raw vector mirror",
            mapping_id: "raw_dynamics_parameters",
            configuration_id: "raw-semantic-midpoint-mirror.v1",
            catalogue_entry_id: "raw-dynamics-parameters",
        },
        (ScenarioId::RawSemanticEquivalent, false) => LaneDescriptor {
            lane_id: "lane_b",
            label: "Semantic midpoint qualities",
            mapping_id: "semantic_laban_qualities",
            configuration_id: "semantic-midpoint.v1",
            catalogue_entry_id: "semantic-laban-dynamics",
        },
        (ScenarioId::RawSemanticContrast, true) => LaneDescriptor {
            lane_id: "lane_a",
            label: "Raw separation-biased rates",
            mapping_id: "raw_dynamics_parameters",
            configuration_id: "raw-separation-biased.v1",
            catalogue_entry_id: "raw-dynamics-parameters",
        },
        (ScenarioId::RawSemanticContrast, false) => LaneDescriptor {
            lane_id: "lane_b",
            label: "Semantic direct/sudden/free profile",
            mapping_id: "semantic_laban_qualities",
            configuration_id: "semantic-contrast.v1",
            catalogue_entry_id: "semantic-laban-dynamics",
        },
        (ScenarioId::SuperpositionLease, true) => LaneDescriptor {
            lane_id: "lane_a",
            label: "Additive superposition",
            mapping_id: "additive_personal_fields",
            configuration_id: "two-contributor-superposition.v1",
            catalogue_entry_id: "additive-personal-fields",
        },
        (ScenarioId::SuperpositionLease, false) => LaneDescriptor {
            lane_id: "lane_b",
            label: "Exclusive member lease",
            mapping_id: "exclusive_member_lease",
            configuration_id: "two-operator-exclusive-lease.v1",
            catalogue_entry_id: "lease-expiry-and-handoff",
        },
        (ScenarioId::SoftCollisionFree, true) => LaneDescriptor {
            lane_id: "lane_a",
            label: "Soft avoidance; overlap permitted",
            mapping_id: "navigation_field_soft_avoidance",
            configuration_id: "rightward-field-soft-overlap.v1",
            catalogue_entry_id: "navigation-field",
        },
        (ScenarioId::SoftCollisionFree, false) => LaneDescriptor {
            lane_id: "lane_b",
            label: "Collision-free discs",
            mapping_id: "navigation_field_collision_free",
            configuration_id: "rightward-field-collision-free.v1",
            catalogue_entry_id: "navigation-field",
        },
    }
}

fn lane_metrics(core: &DemoCore, state: &PublicState) -> Result<LaneMetrics, ComparisonError> {
    let rows = core.frame_rows()?;
    let member_rows = rows.chunks_exact(FRAME_ROW_WIDTH).collect::<Vec<_>>();
    if member_rows.len() < 2 {
        return Err(ComparisonError::Invariant(
            "comparison metric projection requires at least two members",
        ));
    }
    let mut pair_distance_total = 0.0_f32;
    let mut pair_count = 0_usize;
    let mut heading_x = 0.0_f32;
    let mut heading_y = 0.0_f32;
    for (first_index, first) in member_rows.iter().enumerate() {
        let speed = first[4].hypot(first[5]);
        if speed > f32::EPSILON {
            heading_x += first[4] / speed;
            heading_y += first[5] / speed;
        }
        for second in member_rows.iter().skip(first_index + 1) {
            pair_distance_total += (first[1] - second[1]).hypot(first[2] - second[2]);
            pair_count += 1;
        }
    }
    let mean_spacing = pair_distance_total / bounded_count_f32(pair_count)?;
    let member_count = bounded_count_f32(member_rows.len())?;
    let mean_formation_extent = state
        .groups
        .iter()
        .map(|group| group.formation_extent)
        .sum::<f32>()
        / bounded_count_f32(state.groups.len())?;
    Ok(LaneMetrics {
        tick: state.tick,
        cohesion: round_metric(1.0 - (mean_spacing / 8.0_f32.sqrt()).min(1.0)),
        polarization: round_metric(heading_x.hypot(heading_y) / member_count),
        mean_spacing: round_metric(mean_spacing),
        average_speed: state.average_speed,
        group_count: state.groups.len(),
        group_sizes: state
            .groups
            .iter()
            .map(|group| group.member_ids.len())
            .collect(),
        mean_formation_extent: round_metric(mean_formation_extent),
        behavior_distribution: state.behavior_counts,
        active_fields: state.fields.len(),
        active_leases: state.leases.len(),
        minimum_surface_clearance: state.clearance_metrics.minimum_surface_clearance,
        overlap_pair_count: state.clearance_metrics.overlap_pair_count,
        near_miss_pair_count: state.clearance_metrics.near_miss_pair_count,
        total_collision_interventions: state.clearance_metrics.total_intervention_count,
        contact_tick_count: state.clearance_metrics.contact_tick_count,
    })
}

fn bounded_count_f32(value: usize) -> Result<f32, ComparisonError> {
    u16::try_from(value).map(f32::from).map_err(|_| {
        ComparisonError::Invariant("comparison metric count exceeds its numeric bound")
    })
}

fn metric_delta(left: &LaneMetrics, right: &LaneMetrics) -> MetricDelta {
    MetricDelta {
        tick: signed_delta_u64(left.tick, right.tick),
        cohesion: round_metric(right.cohesion - left.cohesion),
        polarization: round_metric(right.polarization - left.polarization),
        mean_spacing: round_metric(right.mean_spacing - left.mean_spacing),
        average_speed: round_metric(right.average_speed - left.average_speed),
        group_count: signed_delta_usize(left.group_count, right.group_count),
        mean_formation_extent: round_metric(
            right.mean_formation_extent - left.mean_formation_extent,
        ),
        behavior_distribution: BehaviorDelta {
            flock: signed_delta_usize(
                left.behavior_distribution.flock,
                right.behavior_distribution.flock,
            ),
            cohere: signed_delta_usize(
                left.behavior_distribution.cohere,
                right.behavior_distribution.cohere,
            ),
            disperse: signed_delta_usize(
                left.behavior_distribution.disperse,
                right.behavior_distribution.disperse,
            ),
        },
        active_fields: signed_delta_usize(left.active_fields, right.active_fields),
        active_leases: signed_delta_usize(left.active_leases, right.active_leases),
        minimum_surface_clearance: round_metric(
            right.minimum_surface_clearance - left.minimum_surface_clearance,
        ),
        overlap_pair_count: signed_delta_usize(left.overlap_pair_count, right.overlap_pair_count),
        near_miss_pair_count: signed_delta_usize(
            left.near_miss_pair_count,
            right.near_miss_pair_count,
        ),
        total_collision_interventions: signed_delta_u64(
            left.total_collision_interventions,
            right.total_collision_interventions,
        ),
        contact_tick_count: signed_delta_u64(left.contact_tick_count, right.contact_tick_count),
    }
}

fn signed_delta_usize(left: usize, right: usize) -> i32 {
    i32::try_from(right).unwrap_or(i32::MAX) - i32::try_from(left).unwrap_or(i32::MAX)
}

fn signed_delta_u64(left: u64, right: u64) -> i64 {
    i64::try_from(right).unwrap_or(i64::MAX) - i64::try_from(left).unwrap_or(i64::MAX)
}

fn round_metric(value: f32) -> f32 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn final_provenance(state: &PublicState) -> String {
    format!(
        "seed {}; tick {}; state revision {}; selection revision {}; morphology revision {}; authority revision {}; replay events {}; replay steps {}",
        state.seed,
        state.tick,
        state.state_revision,
        state.selection_revision,
        state.morphology_revision,
        state.authority_revision,
        state.replay_event_count,
        state.replay_step_count
    )
}

fn metric_definitions() -> Vec<MetricDefinition> {
    vec![
        MetricDefinition {
            metric_id: "tick",
            label: "Fixed tick",
            unit: "fixed steps",
            definition: "Completed deterministic core steps; both lanes must remain equal.",
        },
        MetricDefinition {
            metric_id: "cohesion",
            label: "Cohesion index",
            unit: "unitless 0–1",
            definition: "One minus mean pair spacing divided by the normalized scene diagonal, clamped to 0–1.",
        },
        MetricDefinition {
            metric_id: "polarization",
            label: "Polarization",
            unit: "unitless 0–1",
            definition: "Magnitude of the mean normalized member heading.",
        },
        MetricDefinition {
            metric_id: "mean_spacing",
            label: "Mean pair spacing",
            unit: "normalized scene units",
            definition: "Arithmetic mean of every unique member-to-member distance.",
        },
        MetricDefinition {
            metric_id: "average_speed",
            label: "Average speed",
            unit: "normalized scene units per second",
            definition: "Arithmetic mean of current member velocity magnitudes.",
        },
        MetricDefinition {
            metric_id: "group_count",
            label: "Group count and sizes",
            unit: "groups / members",
            definition: "Canonical morphology groups and their conserved member counts.",
        },
        MetricDefinition {
            metric_id: "mean_formation_extent",
            label: "Mean formation extent",
            unit: "normalized scene units",
            definition: "Mean across groups of the maximum member distance from each group centroid.",
        },
        MetricDefinition {
            metric_id: "behavior_distribution",
            label: "Behavior distribution",
            unit: "members (F/C/D)",
            definition: "Counts assigned to Flock, Cohere, and Disperse in that order.",
        },
        MetricDefinition {
            metric_id: "active_fields",
            label: "Field provenance",
            unit: "active fields",
            definition: "Bounded app-local fields retained with synthetic contributor and polarity provenance.",
        },
        MetricDefinition {
            metric_id: "active_leases",
            label: "Lease provenance",
            unit: "active leases",
            definition: "Current app-local per-member leases with holder, expiry, and revision provenance.",
        },
        MetricDefinition {
            metric_id: "minimum_surface_clearance",
            label: "Minimum surface clearance",
            unit: "normalized scene units",
            definition: "Smallest signed edge-to-edge distance between any rendered disc pair; negative means overlap.",
        },
        MetricDefinition {
            metric_id: "overlap_pair_count",
            label: "Overlapping pairs",
            unit: "disc pairs",
            definition: "Unique rendered-disc pairs with negative surface clearance at this tick.",
        },
        MetricDefinition {
            metric_id: "near_miss_pair_count",
            label: "Near-miss pairs",
            unit: "disc pairs",
            definition: "Non-overlapping pairs with no more than 0.03 scene units of surface clearance.",
        },
        MetricDefinition {
            metric_id: "total_collision_interventions",
            label: "Collision interventions",
            unit: "pair corrections",
            definition: "Deterministic post-integration pair corrections since the canonical start.",
        },
        MetricDefinition {
            metric_id: "contact_tick_count",
            label: "Tentative-contact ticks",
            unit: "fixed steps",
            definition: "Fixed steps whose tentative integrated state contained at least one overlapping pair.",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unequal_lane_starts_are_rejected() {
        let left = DemoCore::new(1);
        let right = DemoCore::new(2);
        assert!(!snapshots_equal(&left, &right).expect("snapshots compare"));
        assert!(ensure_lockstep(&left, &right, 1).is_err());
    }
}
