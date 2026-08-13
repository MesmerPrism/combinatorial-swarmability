use core::fmt;
use std::collections::BTreeSet;

use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleSet, ParticleState};
use serde::{Deserialize, Serialize};

use crate::action::{
    ActionCode, ActionReceipt, BehaviorCounts, CollectiveBehavior, DynamicsControlMode,
    DynamicsRates, FieldLifetime, FieldPolarity, FieldSummary, GroupPartitionRule, GroupSummary,
    HandoffDecision, LeaseSummary, MemberSummary, PublicState, ResolvedDynamics, SemanticAction,
    SemanticQualities, TargetScope,
};
use crate::replay::{
    validate_replay_tape, ReplayEvent, ReplayRecorder, ReplayTape, MAX_REPLAY_JSON_BYTES,
};
use crate::rng::SplitMix64;

/// Number of members in the first public scene.
pub const MEMBER_COUNT: usize = 24;
/// Number of `f32` values in each Wasm frame row.
pub const FRAME_ROW_WIDTH: usize = 12;
/// Maximum number of additive personal fields retained by one scene.
pub const MAX_PERSONAL_FIELDS: usize = 8;
/// Maximum number of app-local synthetic contributor channels.
pub const MAX_SYNTHETIC_CONTRIBUTORS: u8 = 4;
/// Maximum expiring-field lifetime in fixed simulation steps.
pub const MAX_FIELD_LIFETIME_STEPS: u32 = 1_200;
/// Minimum accepted app-owned collective-mode entry rate.
pub const MIN_DYNAMICS_RATE: f32 = 0.0;
/// Maximum accepted app-owned collective-mode entry rate.
pub const MAX_DYNAMICS_RATE: f32 = 1.0;
/// Disabled-by-default raw dynamics rates, preserving the established scene.
pub const DEFAULT_DYNAMICS_RATES: DynamicsRates = DynamicsRates {
    alignment: 0.0,
    cohesion: 0.0,
    separation: 0.0,
};
/// Minimum accepted semantic quality.
pub const MIN_SEMANTIC_QUALITY: f32 = 0.0;
/// Maximum accepted semantic quality.
pub const MAX_SEMANTIC_QUALITY: f32 = 1.0;
/// Midpoint defaults retained until a semantic action activates the profile.
pub const DEFAULT_SEMANTIC_QUALITIES: SemanticQualities = SemanticQualities {
    space: 0.5,
    time: 0.5,
    weight: 0.5,
    flow: 0.5,
};
/// Maximum number of canonical app-owned morphology groups.
pub const MAX_GROUPS: usize = 8;
/// Minimum accepted explicit formation-scale target.
pub const MIN_FORMATION_SCALE: f32 = 0.5;
/// Maximum accepted explicit formation-scale target.
pub const MAX_FORMATION_SCALE: f32 = 2.0;
/// Neutral formation-scale target that adds no group radial steering.
pub const DEFAULT_FORMATION_SCALE: f32 = 1.0;
/// Number of fixed app-local synthetic operator channels.
pub const MAX_SYNTHETIC_OPERATORS: u8 = 4;
/// Maximum simultaneously active per-member leases.
pub const MAX_ACTIVE_LEASES: usize = 8;
/// Maximum fixed-step lease lifetime accepted by the public reconstruction.
pub const MAX_LEASE_LIFETIME_STEPS: u32 = 600;

// App-owned qualitative interpolation endpoints. The source projection supplies
// directions and coupled variables, not portable coefficients.
const INDIRECT_ALIGNMENT_RATE: f32 = 0.15;
const DIRECT_ALIGNMENT_RATE: f32 = 0.85;
const INDIRECT_SEPARATION_RATE: f32 = 0.85;
const DIRECT_SEPARATION_RATE: f32 = 0.15;
const LOWER_WEIGHT_COHESION_RATE: f32 = 0.85;
const HIGHER_WEIGHT_COHESION_RATE: f32 = 0.15;
const SUSTAINED_SPEED_SCALE: f32 = 0.75;
const SUDDEN_SPEED_SCALE: f32 = 1.25;
const BOUND_FLOW_DAMPING: f32 = 0.75;
const FREE_FLOW_DAMPING: f32 = 0.15;
const BOUND_FLOW_JITTER: f32 = 0.0;
const FREE_FLOW_JITTER: f32 = 0.18;
const MAX_RESOLVED_DAMPING: f32 = BOUND_FLOW_DAMPING;
const MAX_RESOLVED_JITTER: f32 = FREE_FLOW_JITTER;

const DEFAULT_SUBGROUP_COUNT: usize = 6;
const MEMBER_COUNT_F32: f32 = 24.0;
const FIXED_STEP_MILLIS: u32 = 16;
const FIXED_STEP_SECONDS: f32 = 0.016;
const MAX_CATCH_UP_STEPS: u32 = 8;
const WORLD_LIMIT: f32 = 0.94;
const BASE_SPEED: f32 = 0.36;
const MIN_SPEED: f32 = 0.08;
const MAX_SPEED: f32 = 1.10;
const MAX_SPEED_OFFSET: f32 = 0.70;
const NEIGHBOR_RADIUS_SQUARED: f32 = 0.42 * 0.42;
const SEPARATION_RADIUS_SQUARED: f32 = 0.13 * 0.13;
const DISPERSE_RADIUS_SQUARED: f32 = 0.55 * 0.55;
const SOFT_WORLD_LIMIT: f32 = 0.78;
const FIELD_POSITION_LIMIT: f32 = 0.90;
const MAX_FIELD_ID: u16 = 63;
const FIELD_SOFTENING: f32 = 0.075;
const FIELD_ACCELERATION_SCALE: f32 = 0.22;
const MAX_FIELD_ACCELERATION_PER_SOURCE: f32 = 1.65;
const FORMATION_SCALE_ACCELERATION: f32 = 0.85;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct GroupState {
    group_id: u8,
    member_ids: Vec<u16>,
    formation_scale: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct FieldState {
    field_id: u16,
    contributor_id: u8,
    x: f32,
    y: f32,
    polarity: FieldPolarity,
    expires_at_tick: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct LeaseState {
    member_id: u16,
    holder_operator_id: u8,
    acquired_at_tick: u64,
    expires_at_tick: u64,
    pending_handoff_to: Option<u8>,
}

/// Error returned when a deterministic snapshot or Matter payload is invalid.
#[derive(Debug)]
pub enum DemoError {
    /// JSON serialization or deserialization failed.
    Json(serde_json::Error),
    /// Restored snapshot violates the fixed scene shape.
    InvalidSnapshot(&'static str),
    /// Replay input violates the bounded deterministic tape contract.
    InvalidReplay(&'static str),
    /// Rusty Matter rejected the particle state or render payload.
    Matter(String),
}

impl fmt::Display for DemoError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "snapshot JSON error: {error}"),
            Self::InvalidSnapshot(message) => write!(formatter, "invalid snapshot: {message}"),
            Self::InvalidReplay(message) => write!(formatter, "invalid replay: {message}"),
            Self::Matter(message) => write!(formatter, "Matter payload error: {message}"),
        }
    }
}

impl std::error::Error for DemoError {}

impl From<serde_json::Error> for DemoError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DemoSnapshot {
    seed: u64,
    particles: ParticleSet,
    speed_offsets: Vec<f32>,
    behaviors: Vec<CollectiveBehavior>,
    raw_dynamics_rates: DynamicsRates,
    dynamics_control_mode: DynamicsControlMode,
    semantic_qualities: SemanticQualities,
    resolved_dynamics: ResolvedDynamics,
    fields: Vec<FieldState>,
    groups: Vec<GroupState>,
    leases: Vec<LeaseState>,
    scope: TargetScope,
    primary_member: Option<u16>,
    subgroup_members: Vec<u16>,
    running: bool,
    tick: u64,
    accumulator_millis: u32,
    state_revision: u64,
    selection_revision: u64,
    morphology_revision: u64,
    authority_revision: u64,
}

/// Deterministic scene state and app-local semantic reducer.
#[derive(Clone, Debug)]
pub struct DemoCore {
    snapshot: DemoSnapshot,
    replay: ReplayRecorder,
}

impl DemoCore {
    /// Creates a deterministic paused scene for `seed`.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            snapshot: initial_snapshot(seed),
            replay: ReplayRecorder::new(seed),
        }
    }

    /// Restores a previously serialized deterministic snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when JSON or the fixed scene contract is invalid.
    pub fn from_snapshot_json(json: &str) -> Result<Self, DemoError> {
        let snapshot: DemoSnapshot = serde_json::from_str(json)?;
        validate_snapshot(&snapshot)?;
        let seed = snapshot.seed;
        Ok(Self {
            snapshot,
            replay: ReplayRecorder::unavailable(seed),
        })
    }

    /// Serializes all deterministic reducer and simulation state.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when serialization fails.
    pub fn snapshot_json(&self) -> Result<String, DemoError> {
        Ok(serde_json::to_string(&self.snapshot)?)
    }

    /// Serializes the bounded semantic-action and fixed-step replay tape.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when the tape exceeded its bounds or serialization fails.
    pub fn replay_json(&self) -> Result<String, DemoError> {
        if self.snapshot.running || self.snapshot.accumulator_millis != 0 {
            return Err(DemoError::InvalidReplay(
                "pause before exporting a replay tape",
            ));
        }
        let tape = self
            .replay
            .tape()
            .ok_or(DemoError::InvalidReplay("replay recording is unavailable"))?;
        Ok(serde_json::to_string(&tape)?)
    }

    /// Reconstructs a deterministic core from a strict bounded replay tape.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when the tape is malformed, out of bounds, or rejects an action.
    pub fn from_replay_json(json: &str) -> Result<Self, DemoError> {
        if json.len() > MAX_REPLAY_JSON_BYTES {
            return Err(DemoError::InvalidReplay("replay byte limit exceeded"));
        }
        let tape: ReplayTape = serde_json::from_str(json)?;
        validate_replay_tape(&tape).map_err(DemoError::InvalidReplay)?;
        let expected_tape = tape.clone();
        let mut core = Self::new(tape.initial_seed);
        for event in tape.events {
            match event {
                ReplayEvent::Action { action } => {
                    if !core.dispatch(action).accepted {
                        return Err(DemoError::InvalidReplay("recorded action was rejected"));
                    }
                }
                ReplayEvent::AdvanceSteps { steps } => core.replay_steps(steps)?,
            }
        }
        if core.snapshot.running || core.snapshot.accumulator_millis != 0 {
            return Err(DemoError::InvalidReplay("replay tape must end paused"));
        }
        if core.replay.tape().as_ref() != Some(&expected_tape) {
            return Err(DemoError::InvalidReplay("replayed tape did not round trip"));
        }
        Ok(core)
    }

    /// Applies one semantic action and returns a bounded receipt.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    #[allow(clippy::too_many_lines)] // One explicit reducer entry point keeps every action on one path.
    pub fn dispatch(&mut self, action: SemanticAction) -> ActionReceipt {
        let recorded_action = action.clone();
        let receipt = match action {
            SemanticAction::SetScope { scope } => self.set_scope(scope),
            SemanticAction::SelectMember { member_id } => self.select_member(member_id),
            SemanticAction::ToggleSubgroupMember { member_id } => {
                self.toggle_subgroup_member(member_id)
            }
            SemanticAction::ClearSubgroup => self.clear_subgroup(),
            SemanticAction::AdjustSpeed {
                delta,
                expected_selection_revision,
            } => self.adjust_speed(delta, expected_selection_revision),
            SemanticAction::SetBehavior {
                behavior,
                expected_selection_revision,
            } => self.set_behavior(behavior, expected_selection_revision),
            SemanticAction::SetAlignment { rate } => self.set_alignment(rate),
            SemanticAction::SetCohesion { rate } => self.set_cohesion(rate),
            SemanticAction::SetSeparation { rate } => self.set_separation(rate),
            SemanticAction::SetSpaceQuality { value } => self.set_space_quality(value),
            SemanticAction::SetTimeQuality { value } => self.set_time_quality(value),
            SemanticAction::SetWeightQuality { value } => self.set_weight_quality(value),
            SemanticAction::SetFlowQuality { value } => self.set_flow_quality(value),
            SemanticAction::ApplyComparisonRawMirror => self.apply_comparison_raw_mirror(),
            SemanticAction::SplitGroup {
                source_group_id,
                new_group_id,
                partition_rule,
                expected_morphology_revision,
            } => self.split_group(
                source_group_id,
                new_group_id,
                partition_rule,
                expected_morphology_revision,
            ),
            SemanticAction::MergeGroups {
                group_a_id,
                group_b_id,
                survivor_group_id,
                expected_morphology_revision,
            } => self.merge_groups(
                group_a_id,
                group_b_id,
                survivor_group_id,
                expected_morphology_revision,
            ),
            SemanticAction::SetFormationScale {
                group_id,
                scale,
                expected_morphology_revision,
            } => self.set_formation_scale(group_id, scale, expected_morphology_revision),
            SemanticAction::RequestLease {
                member_id,
                operator_id,
                lifetime_steps,
                expected_authority_revision,
            } => self.request_lease(
                member_id,
                operator_id,
                lifetime_steps,
                expected_authority_revision,
            ),
            SemanticAction::ReleaseLease {
                member_id,
                operator_id,
                expected_authority_revision,
            } => self.release_lease(member_id, operator_id, expected_authority_revision),
            SemanticAction::OfferLeaseHandoff {
                member_id,
                holder_operator_id,
                receiver_operator_id,
                expected_authority_revision,
            } => self.offer_lease_handoff(
                member_id,
                holder_operator_id,
                receiver_operator_id,
                expected_authority_revision,
            ),
            SemanticAction::ResolveLeaseHandoff {
                member_id,
                receiver_operator_id,
                decision,
                expected_authority_revision,
            } => self.resolve_lease_handoff(
                member_id,
                receiver_operator_id,
                decision,
                expected_authority_revision,
            ),
            SemanticAction::SetLeasedBehavior {
                member_id,
                operator_id,
                behavior,
                expected_authority_revision,
            } => self.set_leased_behavior(
                member_id,
                operator_id,
                behavior,
                expected_authority_revision,
            ),
            SemanticAction::PlaceField {
                field_id,
                contributor_id,
                x,
                y,
                polarity,
                lifetime,
            } => self.place_field(field_id, contributor_id, x, y, polarity, lifetime),
            SemanticAction::MoveField { field_id, x, y } => self.move_field(field_id, x, y),
            SemanticAction::SetFieldPolarity { field_id, polarity } => {
                self.set_field_polarity(field_id, polarity)
            }
            SemanticAction::RemoveField { field_id } => self.remove_field(field_id),
            SemanticAction::Start => self.start(),
            SemanticAction::Pause => self.pause(),
            SemanticAction::Step => self.step_action(),
            SemanticAction::Reset => self.reset(),
            SemanticAction::RestartSeed { seed } => self.restart_seed(seed),
        };
        if receipt.accepted {
            self.replay.record_action(recorded_action);
        }
        receipt
    }

    /// Advances elapsed time through bounded fixed steps when running.
    #[must_use]
    pub fn advance_elapsed(&mut self, elapsed_millis: u32) -> u32 {
        if !self.snapshot.running {
            return 0;
        }
        self.snapshot.accumulator_millis = self
            .snapshot
            .accumulator_millis
            .saturating_add(elapsed_millis.min(250));
        let mut completed = 0;
        while self.snapshot.accumulator_millis >= FIXED_STEP_MILLIS
            && completed < MAX_CATCH_UP_STEPS
        {
            self.step_simulation();
            self.snapshot.accumulator_millis -= FIXED_STEP_MILLIS;
            completed += 1;
        }
        if completed == MAX_CATCH_UP_STEPS {
            self.snapshot.accumulator_millis %= FIXED_STEP_MILLIS;
        }
        self.replay.record_advance(completed);
        completed
    }

    /// Returns the current semantic state for ordinary DOM presentation.
    #[must_use]
    #[allow(clippy::too_many_lines)] // One projection keeps the public state internally consistent.
    pub fn public_state(&self) -> PublicState {
        let targets = self.resolved_targets();
        let target_set = targets.iter().copied().collect::<BTreeSet<_>>();
        let subgroup_set = self
            .snapshot
            .subgroup_members
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let active_contributor_count = self
            .snapshot
            .fields
            .iter()
            .map(|field| field.contributor_id)
            .collect::<BTreeSet<_>>()
            .len();
        let fields = self
            .snapshot
            .fields
            .iter()
            .map(|field| FieldSummary {
                field_id: field.field_id,
                contributor_id: field.contributor_id,
                x: round_for_public(field.x),
                y: round_for_public(field.y),
                polarity: field.polarity,
                expires_at_tick: field.expires_at_tick,
                remaining_steps: field
                    .expires_at_tick
                    .map(|expiry| expiry.saturating_sub(self.snapshot.tick)),
            })
            .collect();
        let groups = self
            .snapshot
            .groups
            .iter()
            .map(|group| GroupSummary {
                group_id: group.group_id,
                member_ids: group.member_ids.clone(),
                formation_scale: round_for_public(group.formation_scale),
                formation_extent: round_for_public(group_extent(
                    group,
                    &self.snapshot.particles.particles,
                )),
            })
            .collect();
        let leases = self
            .snapshot
            .leases
            .iter()
            .map(|lease| LeaseSummary {
                member_id: lease.member_id,
                holder_operator_id: lease.holder_operator_id,
                acquired_at_tick: lease.acquired_at_tick,
                expires_at_tick: lease.expires_at_tick,
                remaining_steps: lease.expires_at_tick.saturating_sub(self.snapshot.tick),
                pending_handoff_to: lease.pending_handoff_to,
            })
            .collect();
        let mut speed_total = 0.0;
        let mut behavior_counts = BehaviorCounts {
            flock: 0,
            cohere: 0,
            disperse: 0,
        };
        let members = self
            .snapshot
            .particles
            .particles
            .iter()
            .enumerate()
            .map(|(index, particle)| {
                let member_id = member_id(index);
                let speed = round_for_public(particle.velocity.length());
                let behavior = self.snapshot.behaviors[index];
                speed_total += speed;
                match behavior {
                    CollectiveBehavior::Flock => behavior_counts.flock += 1,
                    CollectiveBehavior::Cohere => behavior_counts.cohere += 1,
                    CollectiveBehavior::Disperse => behavior_counts.disperse += 1,
                }
                MemberSummary {
                    member_id,
                    speed,
                    primary_selected: self.snapshot.primary_member == Some(member_id),
                    subgroup_selected: subgroup_set.contains(&member_id),
                    targeted: target_set.contains(&member_id),
                    behavior,
                    group_id: group_id_for_member(&self.snapshot.groups, member_id)
                        .unwrap_or_default(),
                    lease_holder_operator_id: lease_for_member(&self.snapshot.leases, member_id)
                        .map(|lease| lease.holder_operator_id),
                }
            })
            .collect();
        PublicState {
            seed: self.snapshot.seed.to_string(),
            tick: self.snapshot.tick,
            running: self.snapshot.running,
            scope: self.snapshot.scope,
            primary_member: self.snapshot.primary_member,
            subgroup_members: self.snapshot.subgroup_members.clone(),
            target_members: targets,
            average_speed: round_for_public(speed_total / MEMBER_COUNT_F32),
            behavior_counts,
            dynamics_rates: public_rates(self.snapshot.resolved_dynamics.rates),
            raw_dynamics_rates: public_rates(self.snapshot.raw_dynamics_rates),
            dynamics_control_mode: self.snapshot.dynamics_control_mode,
            semantic_qualities: public_qualities(self.snapshot.semantic_qualities),
            resolved_dynamics: public_resolved(self.snapshot.resolved_dynamics),
            groups,
            morphology_revision: self.snapshot.morphology_revision,
            leases,
            authority_revision: self.snapshot.authority_revision,
            fields,
            active_contributor_count,
            state_revision: self.snapshot.state_revision,
            selection_revision: self.snapshot.selection_revision,
            replay_event_count: self.replay.event_count(),
            replay_step_count: self.replay.total_steps(),
            replay_available: self.replay.available(),
            members,
        }
    }

    /// Builds the renderer-neutral Rusty Matter payload.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when Matter rejects the current particle set.
    pub fn render_payload(&self) -> Result<ParticleRenderPayload, DemoError> {
        ParticleRenderPayload::from_particle_set(
            format!("combinatorial-swarmability.frame.{}", self.snapshot.tick),
            &self.snapshot.particles,
        )
        .map_err(|error| DemoError::Matter(error.to_string()))
    }

    /// Projects Matter payload rows plus app-owned selection markers.
    ///
    /// # Errors
    ///
    /// Returns [`DemoError`] when Matter rejects the payload.
    pub fn frame_rows(&self) -> Result<Vec<f32>, DemoError> {
        let payload = self.render_payload()?;
        let targets = self.resolved_targets().into_iter().collect::<BTreeSet<_>>();
        let subgroup = self
            .snapshot
            .subgroup_members
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let mut rows = Vec::with_capacity(payload.samples.len() * FRAME_ROW_WIDTH);
        for (index, sample) in payload.samples.iter().enumerate() {
            let member_id = member_id(index);
            rows.extend_from_slice(&[
                f32::from(member_id),
                sample.position.x,
                sample.position.y,
                sample.radius,
                sample.velocity.x,
                sample.velocity.y,
                sample.speed,
                bool_marker(self.snapshot.primary_member == Some(member_id)),
                bool_marker(subgroup.contains(&member_id)),
                bool_marker(targets.contains(&member_id)),
                behavior_code(self.snapshot.behaviors[index]),
                f32::from(
                    group_id_for_member(&self.snapshot.groups, member_id).unwrap_or_default(),
                ),
            ]);
        }
        Ok(rows)
    }

    fn set_scope(&mut self, scope: TargetScope) -> ActionReceipt {
        if self.snapshot.scope != scope {
            self.snapshot.scope = scope;
            self.bump_selection();
        }
        self.accepted(
            ActionCode::ScopeSet,
            format!("Scope set to {scope:?}."),
            Vec::new(),
        )
    }

    fn select_member(&mut self, member_id: u16) -> ActionReceipt {
        if !valid_member(member_id) {
            return self.rejected(
                ActionCode::InvalidMember,
                "That member is not present in this scene.",
            );
        }
        if self.snapshot.primary_member != Some(member_id) {
            self.snapshot.primary_member = Some(member_id);
            self.bump_selection();
        }
        self.accepted(
            ActionCode::MemberSelected,
            format!("Member {} is the primary selection.", member_id + 1),
            Vec::new(),
        )
    }

    fn toggle_subgroup_member(&mut self, member_id: u16) -> ActionReceipt {
        if !valid_member(member_id) {
            return self.rejected(
                ActionCode::InvalidMember,
                "That member is not present in this scene.",
            );
        }
        match self.snapshot.subgroup_members.binary_search(&member_id) {
            Ok(index) => {
                self.snapshot.subgroup_members.remove(index);
            }
            Err(index) => {
                self.snapshot.subgroup_members.insert(index, member_id);
            }
        }
        self.bump_selection();
        self.accepted(
            ActionCode::SubgroupChanged,
            format!(
                "Member {} toggled in the subgroup; {} selected.",
                member_id + 1,
                self.snapshot.subgroup_members.len()
            ),
            Vec::new(),
        )
    }

    fn clear_subgroup(&mut self) -> ActionReceipt {
        if !self.snapshot.subgroup_members.is_empty() {
            self.snapshot.subgroup_members.clear();
            self.bump_selection();
        }
        self.accepted(
            ActionCode::SubgroupCleared,
            "Subgroup selection cleared.".to_owned(),
            Vec::new(),
        )
    }

    fn adjust_speed(&mut self, delta: f32, expected_selection_revision: u64) -> ActionReceipt {
        if expected_selection_revision != self.snapshot.selection_revision {
            return self.rejected(
                ActionCode::StaleSelection,
                "Selection changed before the speed action was applied.",
            );
        }
        if !delta.is_finite() || delta == 0.0 || delta.abs() > 0.5 {
            return self.rejected(
                ActionCode::InvalidSpeedDelta,
                "Speed change must be finite, non-zero, and at most 0.5.",
            );
        }
        let targets = self.resolved_targets();
        if targets.is_empty() {
            return self.rejected(
                ActionCode::EmptySelection,
                "The current scope has no selected members.",
            );
        }
        for member_id in &targets {
            let offset = &mut self.snapshot.speed_offsets[usize::from(*member_id)];
            *offset = (*offset + delta).clamp(-MAX_SPEED_OFFSET, MAX_SPEED_OFFSET);
        }
        self.bump_state();
        self.accepted(
            ActionCode::SpeedAdjusted,
            format!("Adjusted preferred speed for {} member(s).", targets.len()),
            targets,
        )
    }

    fn set_behavior(
        &mut self,
        behavior: CollectiveBehavior,
        expected_selection_revision: u64,
    ) -> ActionReceipt {
        if expected_selection_revision != self.snapshot.selection_revision {
            return self.rejected(
                ActionCode::StaleSelection,
                "Selection changed before the collective rule was applied.",
            );
        }
        let targets = self.resolved_targets();
        if targets.is_empty() {
            return self.rejected(
                ActionCode::EmptySelection,
                "The current scope has no selected members.",
            );
        }
        for member_id in &targets {
            self.snapshot.behaviors[usize::from(*member_id)] = behavior;
        }
        self.bump_state();
        self.accepted(
            ActionCode::BehaviorSet,
            format!(
                "Assigned {behavior:?} steering to {} member(s).",
                targets.len()
            ),
            targets,
        )
    }

    fn set_alignment(&mut self, rate: f32) -> ActionReceipt {
        if !valid_dynamics_rate(rate) {
            return self.rejected(
                ActionCode::InvalidDynamicsRate,
                "Alignment rate must be finite and between 0 and 1 transition per member-second.",
            );
        }
        self.snapshot.raw_dynamics_rates.alignment = rate;
        self.activate_raw_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::AlignmentSet,
            format!("Set the swarm-wide alignment-mode entry rate to {rate:.2}."),
            Vec::new(),
        )
    }

    fn set_cohesion(&mut self, rate: f32) -> ActionReceipt {
        if !valid_dynamics_rate(rate) {
            return self.rejected(
                ActionCode::InvalidDynamicsRate,
                "Cohesion rate must be finite and between 0 and 1 transition per member-second.",
            );
        }
        self.snapshot.raw_dynamics_rates.cohesion = rate;
        self.activate_raw_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::CohesionSet,
            format!("Set the swarm-wide cohesion-mode entry rate to {rate:.2}."),
            Vec::new(),
        )
    }

    fn set_separation(&mut self, rate: f32) -> ActionReceipt {
        if !valid_dynamics_rate(rate) {
            return self.rejected(
                ActionCode::InvalidDynamicsRate,
                "Separation rate must be finite and between 0 and 1 transition per member-second.",
            );
        }
        self.snapshot.raw_dynamics_rates.separation = rate;
        self.activate_raw_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::SeparationSet,
            format!("Set the swarm-wide separation-mode entry rate to {rate:.2}."),
            Vec::new(),
        )
    }

    fn set_space_quality(&mut self, value: f32) -> ActionReceipt {
        if !valid_semantic_quality(value) {
            return self.rejected(
                ActionCode::InvalidSemanticQuality,
                "Space quality must be finite and between 0 and 1.",
            );
        }
        self.snapshot.semantic_qualities.space = value;
        self.activate_semantic_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::SpaceQualitySet,
            format!("Set Space to {value:.2} and resolved the semantic dynamics vector."),
            Vec::new(),
        )
    }

    fn set_time_quality(&mut self, value: f32) -> ActionReceipt {
        if !valid_semantic_quality(value) {
            return self.rejected(
                ActionCode::InvalidSemanticQuality,
                "Time quality must be finite and between 0 and 1.",
            );
        }
        self.snapshot.semantic_qualities.time = value;
        self.activate_semantic_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::TimeQualitySet,
            format!("Set Time to {value:.2} and resolved the semantic dynamics vector."),
            Vec::new(),
        )
    }

    fn set_weight_quality(&mut self, value: f32) -> ActionReceipt {
        if !valid_semantic_quality(value) {
            return self.rejected(
                ActionCode::InvalidSemanticQuality,
                "Weight quality must be finite and between 0 and 1.",
            );
        }
        self.snapshot.semantic_qualities.weight = value;
        self.activate_semantic_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::WeightQualitySet,
            format!("Set Weight to {value:.2} and resolved the semantic dynamics vector."),
            Vec::new(),
        )
    }

    fn set_flow_quality(&mut self, value: f32) -> ActionReceipt {
        if !valid_semantic_quality(value) {
            return self.rejected(
                ActionCode::InvalidSemanticQuality,
                "Flow quality must be finite and between 0 and 1.",
            );
        }
        self.snapshot.semantic_qualities.flow = value;
        self.activate_semantic_dynamics();
        self.bump_state();
        self.accepted(
            ActionCode::FlowQualitySet,
            format!("Set Flow to {value:.2} and resolved the semantic dynamics vector."),
            Vec::new(),
        )
    }

    fn apply_comparison_raw_mirror(&mut self) -> ActionReceipt {
        let resolved = resolve_semantic_dynamics(DEFAULT_SEMANTIC_QUALITIES);
        self.snapshot.raw_dynamics_rates = resolved.rates;
        self.snapshot.dynamics_control_mode = DynamicsControlMode::ComparisonRawMirror;
        self.snapshot.resolved_dynamics = resolved;
        self.bump_state();
        self.accepted(
            ActionCode::ComparisonRawMirrorApplied,
            "Installed the fixed raw-vector mirror of the semantic midpoint profile.".to_owned(),
            Vec::new(),
        )
    }

    fn activate_raw_dynamics(&mut self) {
        self.snapshot.dynamics_control_mode = DynamicsControlMode::Raw;
        self.snapshot.resolved_dynamics = resolve_raw_dynamics(self.snapshot.raw_dynamics_rates);
    }

    fn activate_semantic_dynamics(&mut self) {
        self.snapshot.dynamics_control_mode = DynamicsControlMode::Semantic;
        self.snapshot.resolved_dynamics =
            resolve_semantic_dynamics(self.snapshot.semantic_qualities);
    }

    fn split_group(
        &mut self,
        source_group_id: u8,
        new_group_id: u8,
        partition_rule: GroupPartitionRule,
        expected_morphology_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_morphology(expected_morphology_revision) {
            return receipt;
        }
        let Ok(source_index) = self
            .snapshot
            .groups
            .binary_search_by_key(&source_group_id, |group| group.group_id)
        else {
            return self.rejected(ActionCode::MissingGroup, "The source group does not exist.");
        };
        if self.snapshot.groups.len() >= MAX_GROUPS {
            return self.rejected(
                ActionCode::GroupLimitReached,
                "The scene already contains eight canonical groups.",
            );
        }
        if self.snapshot.groups[source_index].member_ids.len() < 2 {
            return self.rejected(
                ActionCode::GroupCannotSplit,
                "A singleton group cannot be split into two non-empty groups.",
            );
        }
        let Some(canonical_new_id) = next_canonical_group_id(&self.snapshot.groups) else {
            return self.rejected(
                ActionCode::GroupLimitReached,
                "No canonical group identifier remains available.",
            );
        };
        if new_group_id != canonical_new_id {
            return self.rejected(
                ActionCode::NonCanonicalGroup,
                "A split must use the smallest currently unused canonical group identifier.",
            );
        }

        let source_members = self.snapshot.groups[source_index].member_ids.clone();
        let (retained_members, new_members) = partition_members(&source_members, partition_rule);
        let inherited_scale = self.snapshot.groups[source_index].formation_scale;
        self.snapshot.groups[source_index].member_ids = retained_members;
        let insertion = self
            .snapshot
            .groups
            .binary_search_by_key(&new_group_id, |group| group.group_id)
            .unwrap_or_else(|index| index);
        self.snapshot.groups.insert(
            insertion,
            GroupState {
                group_id: new_group_id,
                member_ids: new_members.clone(),
                formation_scale: inherited_scale,
            },
        );
        self.bump_morphology();
        self.accepted(
            ActionCode::GroupSplit,
            format!(
                "Split group {} by alternating member ID into canonical group {}.",
                source_group_id + 1,
                new_group_id + 1
            ),
            new_members,
        )
    }

    fn merge_groups(
        &mut self,
        first_group_id: u8,
        second_group_id: u8,
        survivor_group_id: u8,
        expected_morphology_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_morphology(expected_morphology_revision) {
            return receipt;
        }
        if first_group_id == second_group_id {
            return self.rejected(
                ActionCode::InvalidGroupOperation,
                "A merge requires two distinct groups.",
            );
        }
        let canonical_survivor = first_group_id.min(second_group_id);
        if survivor_group_id != canonical_survivor {
            return self.rejected(
                ActionCode::NonCanonicalGroup,
                "The lower participating group identifier must survive the merge.",
            );
        }
        let absorbed_group_id = first_group_id.max(second_group_id);
        let Ok(_) = self
            .snapshot
            .groups
            .binary_search_by_key(&canonical_survivor, |group| group.group_id)
        else {
            return self.rejected(
                ActionCode::MissingGroup,
                "The survivor group does not exist.",
            );
        };
        let Ok(absorbed_index) = self
            .snapshot
            .groups
            .binary_search_by_key(&absorbed_group_id, |group| group.group_id)
        else {
            return self.rejected(
                ActionCode::MissingGroup,
                "The absorbed group does not exist.",
            );
        };

        let absorbed_members = self.snapshot.groups[absorbed_index].member_ids.clone();
        self.snapshot.groups.remove(absorbed_index);
        let survivor_index = self
            .snapshot
            .groups
            .binary_search_by_key(&canonical_survivor, |group| group.group_id)
            .expect("validated survivor remains after removing a higher group ID");
        self.snapshot.groups[survivor_index]
            .member_ids
            .extend(absorbed_members.iter().copied());
        self.snapshot.groups[survivor_index]
            .member_ids
            .sort_unstable();
        self.bump_morphology();
        self.accepted(
            ActionCode::GroupsMerged,
            format!(
                "Merged groups {} and {}; canonical group {} retained its scale target.",
                first_group_id + 1,
                second_group_id + 1,
                canonical_survivor + 1
            ),
            absorbed_members,
        )
    }

    #[allow(clippy::float_cmp)] // Exact equality makes repeated canonical scale actions idempotent.
    fn set_formation_scale(
        &mut self,
        group_id: u8,
        scale: f32,
        expected_morphology_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_morphology(expected_morphology_revision) {
            return receipt;
        }
        if !valid_formation_scale(scale) {
            return self.rejected(
                ActionCode::InvalidFormationScale,
                "Formation scale must be finite and between 0.50 and 2.00.",
            );
        }
        let Ok(index) = self
            .snapshot
            .groups
            .binary_search_by_key(&group_id, |group| group.group_id)
        else {
            return self.rejected(ActionCode::MissingGroup, "That group does not exist.");
        };
        let members = self.snapshot.groups[index].member_ids.clone();
        if self.snapshot.groups[index].formation_scale != scale {
            self.snapshot.groups[index].formation_scale = scale;
            self.bump_morphology();
        }
        self.accepted(
            ActionCode::FormationScaleSet,
            format!("Set group {} formation scale to {scale:.2}.", group_id + 1),
            members,
        )
    }

    fn reject_stale_morphology(&self, expected_morphology_revision: u64) -> Option<ActionReceipt> {
        (expected_morphology_revision != self.snapshot.morphology_revision).then(|| {
            self.rejected(
                ActionCode::StaleMorphology,
                "Group state changed before the morphology action was applied.",
            )
        })
    }

    fn request_lease(
        &mut self,
        member_id: u16,
        operator_id: u8,
        lifetime_steps: u32,
        expected_authority_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_authority(expected_authority_revision) {
            return receipt;
        }
        if !valid_member(member_id) {
            return self.rejected(ActionCode::InvalidMember, "That member does not exist.");
        }
        if !valid_operator(operator_id) {
            return self.rejected(
                ActionCode::InvalidOperator,
                "That synthetic operator channel does not exist.",
            );
        }
        if lifetime_steps == 0 || lifetime_steps > MAX_LEASE_LIFETIME_STEPS {
            return self.rejected(
                ActionCode::InvalidLeaseLifetime,
                "Lease lifetime must be 1 to 600 deterministic fixed steps.",
            );
        }
        if self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
            .is_ok()
        {
            return self.rejected(
                ActionCode::LeaseAlreadyHeld,
                "That member already has an active lease.",
            );
        }
        if self.snapshot.leases.len() >= MAX_ACTIVE_LEASES {
            return self.rejected(
                ActionCode::LeaseLimitReached,
                "The scene already contains eight active leases.",
            );
        }
        let Some(expires_at_tick) = self.snapshot.tick.checked_add(u64::from(lifetime_steps))
        else {
            return self.rejected(
                ActionCode::InvalidLeaseLifetime,
                "Lease expiry overflowed the deterministic tick range.",
            );
        };
        let insertion = self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
            .unwrap_or_else(|index| index);
        self.snapshot.leases.insert(
            insertion,
            LeaseState {
                member_id,
                holder_operator_id: operator_id,
                acquired_at_tick: self.snapshot.tick,
                expires_at_tick,
                pending_handoff_to: None,
            },
        );
        self.bump_authority();
        self.accepted(
            ActionCode::LeaseAcquired,
            format!(
                "Synthetic operator {} acquired member {} for {lifetime_steps} fixed steps.",
                operator_id + 1,
                member_id + 1
            ),
            vec![member_id],
        )
    }

    fn release_lease(
        &mut self,
        member_id: u16,
        operator_id: u8,
        expected_authority_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_authority(expected_authority_revision) {
            return receipt;
        }
        if let Some(receipt) = self.reject_invalid_lease_actor(member_id, operator_id) {
            return receipt;
        }
        let Ok(index) = self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
        else {
            return self.rejected(
                ActionCode::MissingLease,
                "That member has no current unexpired lease.",
            );
        };
        if self.snapshot.leases[index].holder_operator_id != operator_id {
            return self.rejected(
                ActionCode::NotLeaseHolder,
                "Only the exact current holder may release this lease.",
            );
        }
        self.snapshot.leases.remove(index);
        self.bump_authority();
        self.accepted(
            ActionCode::LeaseReleased,
            format!(
                "Synthetic operator {} released member {}.",
                operator_id + 1,
                member_id + 1
            ),
            vec![member_id],
        )
    }

    fn offer_lease_handoff(
        &mut self,
        member_id: u16,
        holder_operator_id: u8,
        receiver_operator_id: u8,
        expected_authority_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_authority(expected_authority_revision) {
            return receipt;
        }
        if let Some(receipt) = self.reject_invalid_lease_actor(member_id, holder_operator_id) {
            return receipt;
        }
        if !valid_operator(receiver_operator_id) || receiver_operator_id == holder_operator_id {
            return self.rejected(
                ActionCode::InvalidHandoff,
                "A handoff receiver must be a valid operator distinct from the holder.",
            );
        }
        let Ok(index) = self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
        else {
            return self.rejected(
                ActionCode::MissingLease,
                "That member has no current unexpired lease.",
            );
        };
        let lease = &self.snapshot.leases[index];
        if lease.holder_operator_id != holder_operator_id {
            return self.rejected(
                ActionCode::NotLeaseHolder,
                "Only the exact current holder may offer this lease.",
            );
        }
        if lease.pending_handoff_to.is_some() {
            return self.rejected(
                ActionCode::HandoffAlreadyPending,
                "Resolve the existing handoff offer before making another.",
            );
        }
        self.snapshot.leases[index].pending_handoff_to = Some(receiver_operator_id);
        self.bump_authority();
        self.accepted(
            ActionCode::LeaseHandoffOffered,
            format!(
                "Synthetic operator {} offered member {} to operator {}; expiry is unchanged.",
                holder_operator_id + 1,
                member_id + 1,
                receiver_operator_id + 1
            ),
            vec![member_id],
        )
    }

    fn resolve_lease_handoff(
        &mut self,
        member_id: u16,
        receiver_operator_id: u8,
        decision: HandoffDecision,
        expected_authority_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_authority(expected_authority_revision) {
            return receipt;
        }
        if let Some(receipt) = self.reject_invalid_lease_actor(member_id, receiver_operator_id) {
            return receipt;
        }
        let Ok(index) = self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
        else {
            return self.rejected(
                ActionCode::MissingLease,
                "That member has no current unexpired lease.",
            );
        };
        if self.snapshot.leases[index].pending_handoff_to != Some(receiver_operator_id) {
            return self.rejected(
                ActionCode::MissingHandoff,
                "No pending handoff names that member and receiver.",
            );
        }
        self.snapshot.leases[index].pending_handoff_to = None;
        let (code, summary) = match decision {
            HandoffDecision::Accept => {
                self.snapshot.leases[index].holder_operator_id = receiver_operator_id;
                (
                    ActionCode::LeaseHandoffAccepted,
                    format!(
                        "Synthetic operator {} accepted member {}; the original expiry remains.",
                        receiver_operator_id + 1,
                        member_id + 1
                    ),
                )
            }
            HandoffDecision::Decline => (
                ActionCode::LeaseHandoffDeclined,
                format!(
                    "Synthetic operator {} declined member {}; the current holder remains.",
                    receiver_operator_id + 1,
                    member_id + 1
                ),
            ),
        };
        self.bump_authority();
        self.accepted(code, summary, vec![member_id])
    }

    fn set_leased_behavior(
        &mut self,
        member_id: u16,
        operator_id: u8,
        behavior: CollectiveBehavior,
        expected_authority_revision: u64,
    ) -> ActionReceipt {
        if let Some(receipt) = self.reject_stale_authority(expected_authority_revision) {
            return receipt;
        }
        if let Some(receipt) = self.reject_invalid_lease_actor(member_id, operator_id) {
            return receipt;
        }
        let Ok(index) = self
            .snapshot
            .leases
            .binary_search_by_key(&member_id, |lease| lease.member_id)
        else {
            return self.rejected(
                ActionCode::MissingLease,
                "That member has no current unexpired lease.",
            );
        };
        if self.snapshot.leases[index].holder_operator_id != operator_id {
            return self.rejected(
                ActionCode::NotLeaseHolder,
                "Only the exact current holder may use this lease.",
            );
        }
        self.snapshot.behaviors[usize::from(member_id)] = behavior;
        self.bump_authority();
        self.accepted(
            ActionCode::LeasedBehaviorSet,
            format!(
                "Synthetic operator {} set member {} to {} through its current lease.",
                operator_id + 1,
                member_id + 1,
                behavior_label(behavior)
            ),
            vec![member_id],
        )
    }

    fn reject_stale_authority(&self, expected_authority_revision: u64) -> Option<ActionReceipt> {
        (expected_authority_revision != self.snapshot.authority_revision).then(|| {
            self.rejected(
                ActionCode::StaleAuthority,
                "Lease authority changed before this operation was applied.",
            )
        })
    }

    fn reject_invalid_lease_actor(&self, member_id: u16, operator_id: u8) -> Option<ActionReceipt> {
        if !valid_member(member_id) {
            Some(self.rejected(ActionCode::InvalidMember, "That member does not exist."))
        } else if !valid_operator(operator_id) {
            Some(self.rejected(
                ActionCode::InvalidOperator,
                "That synthetic operator channel does not exist.",
            ))
        } else {
            None
        }
    }

    fn place_field(
        &mut self,
        field_id: u16,
        contributor_id: u8,
        x: f32,
        y: f32,
        polarity: FieldPolarity,
        lifetime: FieldLifetime,
    ) -> ActionReceipt {
        if field_id > MAX_FIELD_ID {
            return self.rejected(
                ActionCode::InvalidFieldId,
                "Field identifiers must remain inside the bounded scene range.",
            );
        }
        if contributor_id >= MAX_SYNTHETIC_CONTRIBUTORS {
            return self.rejected(
                ActionCode::InvalidContributor,
                "Synthetic contributor channel is outside the app-local range.",
            );
        }
        if !valid_field_position(x, y) {
            return self.rejected(
                ActionCode::InvalidFieldPosition,
                "Field position must be finite and inside the normalized scene.",
            );
        }
        if self
            .snapshot
            .fields
            .binary_search_by_key(&field_id, |field| field.field_id)
            .is_ok()
        {
            return self.rejected(
                ActionCode::DuplicateField,
                "A field already uses that scene-local identifier.",
            );
        }
        if self.snapshot.fields.len() >= MAX_PERSONAL_FIELDS {
            return self.rejected(
                ActionCode::FieldLimitReached,
                "The bounded scene already contains eight personal fields.",
            );
        }
        let expires_at_tick = match lifetime {
            FieldLifetime::Persistent => None,
            FieldLifetime::Expiring { steps } if steps > 0 && steps <= MAX_FIELD_LIFETIME_STEPS => {
                self.snapshot.tick.checked_add(u64::from(steps))
            }
            FieldLifetime::Expiring { .. } => {
                return self.rejected(
                    ActionCode::InvalidFieldLifetime,
                    "Expiring fields require a positive bounded fixed-step lifetime.",
                );
            }
        };
        if matches!(lifetime, FieldLifetime::Expiring { .. }) && expires_at_tick.is_none() {
            return self.rejected(
                ActionCode::InvalidFieldLifetime,
                "Field expiry overflowed the deterministic tick range.",
            );
        }
        let insertion = self
            .snapshot
            .fields
            .binary_search_by_key(&field_id, |field| field.field_id)
            .unwrap_or_else(|index| index);
        self.snapshot.fields.insert(
            insertion,
            FieldState {
                field_id,
                contributor_id,
                x,
                y,
                polarity,
                expires_at_tick,
            },
        );
        self.bump_state();
        self.accepted(
            ActionCode::FieldPlaced,
            format!(
                "Placed field {} for synthetic contributor {}.",
                field_id + 1,
                contributor_id + 1
            ),
            Vec::new(),
        )
    }

    fn move_field(&mut self, field_id: u16, x: f32, y: f32) -> ActionReceipt {
        if !valid_field_position(x, y) {
            return self.rejected(
                ActionCode::InvalidFieldPosition,
                "Field position must be finite and inside the normalized scene.",
            );
        }
        let Ok(index) = self
            .snapshot
            .fields
            .binary_search_by_key(&field_id, |field| field.field_id)
        else {
            return self.rejected(ActionCode::MissingField, "That field is not active.");
        };
        self.snapshot.fields[index].x = x;
        self.snapshot.fields[index].y = y;
        self.bump_state();
        self.accepted(
            ActionCode::FieldMoved,
            format!("Moved field {}.", field_id + 1),
            Vec::new(),
        )
    }

    fn set_field_polarity(&mut self, field_id: u16, polarity: FieldPolarity) -> ActionReceipt {
        let Ok(index) = self
            .snapshot
            .fields
            .binary_search_by_key(&field_id, |field| field.field_id)
        else {
            return self.rejected(ActionCode::MissingField, "That field is not active.");
        };
        self.snapshot.fields[index].polarity = polarity;
        self.bump_state();
        self.accepted(
            ActionCode::FieldPolaritySet,
            format!("Set field {} to {polarity:?}.", field_id + 1),
            Vec::new(),
        )
    }

    fn remove_field(&mut self, field_id: u16) -> ActionReceipt {
        let Ok(index) = self
            .snapshot
            .fields
            .binary_search_by_key(&field_id, |field| field.field_id)
        else {
            return self.rejected(ActionCode::MissingField, "That field is not active.");
        };
        self.snapshot.fields.remove(index);
        self.bump_state();
        self.accepted(
            ActionCode::FieldRemoved,
            format!("Removed field {}.", field_id + 1),
            Vec::new(),
        )
    }

    fn start(&mut self) -> ActionReceipt {
        if !self.snapshot.running {
            self.snapshot.running = true;
            self.snapshot.accumulator_millis = 0;
            self.bump_state();
        }
        self.accepted(
            ActionCode::Started,
            "Motion started.".to_owned(),
            Vec::new(),
        )
    }

    fn pause(&mut self) -> ActionReceipt {
        if self.snapshot.running {
            self.snapshot.running = false;
            self.snapshot.accumulator_millis = 0;
            self.bump_state();
        }
        self.accepted(ActionCode::Paused, "Motion paused.".to_owned(), Vec::new())
    }

    fn step_action(&mut self) -> ActionReceipt {
        if self.snapshot.running {
            return self.rejected(
                ActionCode::StepRequiresPause,
                "Pause before advancing one step.",
            );
        }
        self.step_simulation();
        self.accepted(
            ActionCode::Stepped,
            "Advanced one fixed step.".to_owned(),
            Vec::new(),
        )
    }

    fn reset(&mut self) -> ActionReceipt {
        let seed = self.snapshot.seed;
        let next_state_revision = self.snapshot.state_revision.saturating_add(1);
        let next_selection_revision = self.snapshot.selection_revision.saturating_add(1);
        let next_morphology_revision = self.snapshot.morphology_revision.saturating_add(1);
        let next_authority_revision = self.snapshot.authority_revision.saturating_add(1);
        self.snapshot = initial_snapshot(seed);
        self.snapshot.state_revision = next_state_revision;
        self.snapshot.selection_revision = next_selection_revision;
        self.snapshot.morphology_revision = next_morphology_revision;
        self.snapshot.authority_revision = next_authority_revision;
        self.accepted(
            ActionCode::Reset,
            "Reset to the current seed's paused initial state.".to_owned(),
            Vec::new(),
        )
    }

    fn restart_seed(&mut self, seed: u64) -> ActionReceipt {
        let next_state_revision = self.snapshot.state_revision.saturating_add(1);
        let next_selection_revision = self.snapshot.selection_revision.saturating_add(1);
        let next_morphology_revision = self.snapshot.morphology_revision.saturating_add(1);
        let next_authority_revision = self.snapshot.authority_revision.saturating_add(1);
        self.snapshot = initial_snapshot(seed);
        self.snapshot.state_revision = next_state_revision;
        self.snapshot.selection_revision = next_selection_revision;
        self.snapshot.morphology_revision = next_morphology_revision;
        self.snapshot.authority_revision = next_authority_revision;
        self.accepted(
            ActionCode::SeedRestarted,
            format!("Restarted with seed {seed} in a paused state."),
            Vec::new(),
        )
    }

    #[allow(clippy::too_many_lines)] // The deterministic integration path remains singular and inspectable.
    fn step_simulation(&mut self) {
        let source = self.snapshot.particles.particles.clone();
        let fields = self.snapshot.fields.clone();
        let groups = self.snapshot.groups.clone();
        let resolved = self.snapshot.resolved_dynamics;
        let next_tick = self.snapshot.tick.saturating_add(1);
        let mut next = source.clone();
        for (index, particle) in source.iter().enumerate() {
            let morphology_group = group_for_member(&groups, member_id(index))
                .expect("every live member belongs to exactly one validated group");
            let morphology_centroid = group_centroid(morphology_group, &source);
            let mut neighbor_count = 0_u16;
            let mut alignment = Vec3::ZERO;
            let mut cohesion = Vec3::ZERO;
            let mut separation = Vec3::ZERO;
            for (other_index, other) in source.iter().enumerate() {
                if index == other_index {
                    continue;
                }
                let away = particle.position - other.position;
                let distance_squared = away.length_squared();
                if distance_squared <= f32::EPSILON || distance_squared > NEIGHBOR_RADIUS_SQUARED {
                    continue;
                }
                neighbor_count += 1;
                alignment = alignment + other.velocity;
                cohesion = cohesion + other.position;
                if distance_squared < SEPARATION_RADIUS_SQUARED {
                    separation = separation + away / distance_squared.max(0.000_1);
                }
            }

            let target_speed = ((BASE_SPEED + self.snapshot.speed_offsets[index])
                * resolved.speed_scale)
                .clamp(MIN_SPEED, MAX_SPEED);
            let mut flock_acceleration = Vec3::ZERO;
            if neighbor_count > 0 {
                let inverse_count = 1.0 / f32::from(neighbor_count);
                alignment = alignment * inverse_count;
                cohesion = cohesion * inverse_count;
                flock_acceleration = (alignment - particle.velocity) * 0.68
                    + (cohesion - particle.position) * 0.46
                    + separation * 0.028;
            }

            let behavior = self.snapshot.behaviors[index];
            let (behavior_centroid, behavior_peer_count) = peer_behavior_centroid(
                &source,
                &self.snapshot.behaviors,
                behavior,
                index,
                &morphology_group.member_ids,
            )
            .unwrap_or((morphology_centroid, 1));
            let behavior_acceleration = match behavior {
                CollectiveBehavior::Flock => flock_acceleration,
                CollectiveBehavior::Cohere => {
                    let centroid_offset = behavior_centroid - particle.position;
                    let settle_radius =
                        (f32::from(behavior_peer_count).sqrt() * 0.085).clamp(0.18, 0.42);
                    let collective_steering = if centroid_offset.length() > settle_radius {
                        normalized_or(centroid_offset, particle.velocity) * target_speed
                            - particle.velocity
                    } else {
                        Vec3::ZERO
                    };
                    flock_acceleration * 0.52 + collective_steering * 1.7 + separation * 0.11
                }
                CollectiveBehavior::Disperse => {
                    let peer_dispersion = peer_dispersion_vector(
                        &source,
                        &self.snapshot.behaviors,
                        behavior,
                        index,
                        &morphology_group.member_ids,
                    );
                    let dispersion = if peer_dispersion.length_squared() > f32::EPSILON {
                        peer_dispersion
                    } else {
                        particle.position - morphology_centroid
                    };
                    let collective_steering = if dispersion.length_squared() > f32::EPSILON {
                        normalized_or(dispersion, particle.velocity) * target_speed
                            - particle.velocity
                    } else {
                        Vec3::ZERO
                    };
                    flock_acceleration * 0.38 + collective_steering * 2.4 + separation * 0.055
                }
            };
            let steering_response = 1.0 - resolved.damping * 0.6;
            let controlled_acceleration = behavior_acceleration
                + personal_field_acceleration(particle.position, &fields)
                + formation_scale_acceleration(particle.position, morphology_group, &source);
            let jitter = deterministic_jitter(
                self.snapshot.seed,
                next_tick,
                u64::try_from(index).unwrap_or_default(),
                resolved.jitter,
            );
            let acceleration = controlled_acceleration * steering_response
                + jitter
                + soft_boundary_acceleration(particle.position, particle.velocity, target_speed);

            let (position, velocity) = integrate_particle(particle, acceleration, target_speed);

            next[index].position = position;
            next[index].velocity = velocity;
            next[index].age_seconds = particle.age_seconds + FIXED_STEP_SECONDS;
        }
        self.apply_dynamics_transitions(next_tick);
        self.snapshot.tick = next_tick;
        let tick = self.snapshot.tick;
        self.snapshot
            .fields
            .retain(|field| field.expires_at_tick.map_or(true, |expiry| expiry > tick));
        let lease_count_before_expiry = self.snapshot.leases.len();
        self.snapshot
            .leases
            .retain(|lease| lease.expires_at_tick > tick);
        if self.snapshot.leases.len() != lease_count_before_expiry {
            self.snapshot.authority_revision = self.snapshot.authority_revision.saturating_add(1);
        }
        self.snapshot.particles.particles = next;
        self.snapshot.particles.time_seconds += FIXED_STEP_SECONDS;
        self.bump_state();
    }

    fn apply_dynamics_transitions(&mut self, tick: u64) {
        let rates = self.snapshot.resolved_dynamics.rates;
        let total_rate = rates.alignment + rates.cohesion + rates.separation;
        if total_rate <= f32::EPSILON {
            return;
        }
        let transition_probability = total_rate * FIXED_STEP_SECONDS;
        for (index, behavior) in self.snapshot.behaviors.iter_mut().enumerate() {
            let member = u64::try_from(index).unwrap_or_default();
            let event_draw = deterministic_unit(self.snapshot.seed, tick, member, 0);
            if event_draw >= transition_probability {
                continue;
            }
            let mode_draw = deterministic_unit(self.snapshot.seed, tick, member, 1) * total_rate;
            *behavior = if mode_draw < rates.alignment {
                CollectiveBehavior::Flock
            } else if mode_draw < rates.alignment + rates.cohesion {
                CollectiveBehavior::Cohere
            } else {
                CollectiveBehavior::Disperse
            };
        }
    }

    fn replay_steps(&mut self, steps: u64) -> Result<(), DemoError> {
        if !self.snapshot.running {
            return Err(DemoError::InvalidReplay(
                "advance steps require a running simulation",
            ));
        }
        let steps = u32::try_from(steps)
            .map_err(|_| DemoError::InvalidReplay("replay advance count is too large"))?;
        for _ in 0..steps {
            self.step_simulation();
        }
        self.replay.record_advance(steps);
        Ok(())
    }

    fn resolved_targets(&self) -> Vec<u16> {
        match self.snapshot.scope {
            TargetScope::Member => self.snapshot.primary_member.into_iter().collect(),
            TargetScope::Subgroup => self.snapshot.subgroup_members.clone(),
            TargetScope::Swarm => (0..MEMBER_COUNT).map(member_id).collect(),
        }
    }

    fn bump_state(&mut self) {
        self.snapshot.state_revision = self.snapshot.state_revision.saturating_add(1);
    }

    fn bump_selection(&mut self) {
        self.snapshot.selection_revision = self.snapshot.selection_revision.saturating_add(1);
        self.bump_state();
    }

    fn bump_morphology(&mut self) {
        self.snapshot.morphology_revision = self.snapshot.morphology_revision.saturating_add(1);
        self.bump_state();
    }

    fn bump_authority(&mut self) {
        self.snapshot.authority_revision = self.snapshot.authority_revision.saturating_add(1);
        self.bump_state();
    }

    fn accepted(
        &self,
        code: ActionCode,
        summary: String,
        changed_member_ids: Vec<u16>,
    ) -> ActionReceipt {
        ActionReceipt {
            accepted: true,
            code,
            summary,
            changed_member_ids,
            state_revision: self.snapshot.state_revision,
            selection_revision: self.snapshot.selection_revision,
            morphology_revision: self.snapshot.morphology_revision,
            authority_revision: self.snapshot.authority_revision,
        }
    }

    fn rejected(&self, code: ActionCode, summary: &str) -> ActionReceipt {
        ActionReceipt {
            accepted: false,
            code,
            summary: summary.to_owned(),
            changed_member_ids: Vec::new(),
            state_revision: self.snapshot.state_revision,
            selection_revision: self.snapshot.selection_revision,
            morphology_revision: self.snapshot.morphology_revision,
            authority_revision: self.snapshot.authority_revision,
        }
    }
}

fn member_id(index: usize) -> u16 {
    debug_assert!(index < MEMBER_COUNT);
    u16::try_from(index).unwrap_or_default()
}

const fn bool_marker(value: bool) -> f32 {
    if value {
        1.0
    } else {
        0.0
    }
}

const fn behavior_code(behavior: CollectiveBehavior) -> f32 {
    match behavior {
        CollectiveBehavior::Flock => 0.0,
        CollectiveBehavior::Cohere => 1.0,
        CollectiveBehavior::Disperse => 2.0,
    }
}

fn peer_behavior_centroid(
    particles: &[ParticleState],
    behaviors: &[CollectiveBehavior],
    behavior: CollectiveBehavior,
    member_index: usize,
    group_members: &[u16],
) -> Option<(Vec3, u16)> {
    let mut sum = Vec3::ZERO;
    let mut count = 0_u16;
    for (index, particle) in particles.iter().enumerate() {
        if index != member_index
            && behaviors[index] == behavior
            && group_members.binary_search(&member_id(index)).is_ok()
        {
            sum = sum + particle.position;
            count += 1;
        }
    }
    (count > 0).then(|| (sum / f32::from(count), count))
}

fn peer_dispersion_vector(
    particles: &[ParticleState],
    behaviors: &[CollectiveBehavior],
    behavior: CollectiveBehavior,
    member_index: usize,
    group_members: &[u16],
) -> Vec3 {
    let member = &particles[member_index];
    particles
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            *index != member_index
                && behaviors[*index] == behavior
                && group_members.binary_search(&member_id(*index)).is_ok()
        })
        .fold(Vec3::ZERO, |dispersion, (_, peer)| {
            let away = member.position - peer.position;
            let distance_squared = away.length_squared();
            if distance_squared > f32::EPSILON && distance_squared < DISPERSE_RADIUS_SQUARED {
                dispersion + away / distance_squared.max(0.000_1)
            } else {
                dispersion
            }
        })
}

fn group_for_member(groups: &[GroupState], member_id: u16) -> Option<&GroupState> {
    groups
        .iter()
        .find(|group| group.member_ids.binary_search(&member_id).is_ok())
}

fn group_id_for_member(groups: &[GroupState], member_id: u16) -> Option<u8> {
    group_for_member(groups, member_id).map(|group| group.group_id)
}

fn group_centroid(group: &GroupState, particles: &[ParticleState]) -> Vec3 {
    let sum = group
        .member_ids
        .iter()
        .fold(Vec3::ZERO, |centroid, member| {
            centroid + particles[usize::from(*member)].position
        });
    sum / f32::from(u16::try_from(group.member_ids.len()).unwrap_or(1))
}

fn group_extent(group: &GroupState, particles: &[ParticleState]) -> f32 {
    let centroid = group_centroid(group, particles);
    group
        .member_ids
        .iter()
        .map(|member| (particles[usize::from(*member)].position - centroid).length())
        .fold(0.0, f32::max)
}

fn formation_scale_acceleration(
    position: Vec3,
    group: &GroupState,
    particles: &[ParticleState],
) -> Vec3 {
    let scale_offset = group.formation_scale - DEFAULT_FORMATION_SCALE;
    if scale_offset.abs() <= f32::EPSILON || group.member_ids.len() < 2 {
        return Vec3::ZERO;
    }
    let radial = position - group_centroid(group, particles);
    if radial.length_squared() <= f32::EPSILON {
        return Vec3::ZERO;
    }
    normalized_or(radial, Vec3::ZERO) * scale_offset * FORMATION_SCALE_ACCELERATION
}

fn next_canonical_group_id(groups: &[GroupState]) -> Option<u8> {
    (0..u8::try_from(MAX_GROUPS).unwrap_or_default())
        .find(|candidate| groups.iter().all(|group| group.group_id != *candidate))
}

fn partition_members(
    member_ids: &[u16],
    partition_rule: GroupPartitionRule,
) -> (Vec<u16>, Vec<u16>) {
    match partition_rule {
        GroupPartitionRule::AlternatingMemberId => member_ids.iter().copied().enumerate().fold(
            (Vec::new(), Vec::new()),
            |mut partition, (index, member)| {
                if index % 2 == 0 {
                    partition.0.push(member);
                } else {
                    partition.1.push(member);
                }
                partition
            },
        ),
    }
}

fn personal_field_acceleration(position: Vec3, fields: &[FieldState]) -> Vec3 {
    fields.iter().fold(Vec3::ZERO, |sum, field| {
        let source = Vec3::new(field.x, field.y, 0.0);
        let offset = source - position;
        let distance_squared = offset.length_squared() + FIELD_SOFTENING * FIELD_SOFTENING;
        let direction = match field.polarity {
            FieldPolarity::Attract => offset,
            FieldPolarity::Repel => Vec3::ZERO - offset,
        };
        let magnitude =
            (FIELD_ACCELERATION_SCALE / distance_squared).min(MAX_FIELD_ACCELERATION_PER_SOURCE);
        sum + normalized_or(direction, Vec3::ZERO) * magnitude
    })
}

fn soft_boundary_acceleration(position: Vec3, velocity: Vec3, target_speed: f32) -> Vec3 {
    if position.x.abs() <= SOFT_WORLD_LIMIT && position.y.abs() <= SOFT_WORLD_LIMIT {
        return Vec3::ZERO;
    }
    let desired = normalized_or(Vec3::ZERO - position, velocity) * target_speed;
    (desired - velocity) * 6.0
}

fn valid_field_position(x: f32, y: f32) -> bool {
    x.is_finite()
        && y.is_finite()
        && x.abs() <= FIELD_POSITION_LIMIT
        && y.abs() <= FIELD_POSITION_LIMIT
}

fn valid_dynamics_rate(rate: f32) -> bool {
    rate.is_finite() && (MIN_DYNAMICS_RATE..=MAX_DYNAMICS_RATE).contains(&rate)
}

fn valid_semantic_quality(value: f32) -> bool {
    value.is_finite() && (MIN_SEMANTIC_QUALITY..=MAX_SEMANTIC_QUALITY).contains(&value)
}

fn valid_semantic_qualities(qualities: SemanticQualities) -> bool {
    valid_semantic_quality(qualities.space)
        && valid_semantic_quality(qualities.time)
        && valid_semantic_quality(qualities.weight)
        && valid_semantic_quality(qualities.flow)
}

fn valid_resolved_dynamics(resolved: ResolvedDynamics) -> bool {
    valid_dynamics_rate(resolved.rates.alignment)
        && valid_dynamics_rate(resolved.rates.cohesion)
        && valid_dynamics_rate(resolved.rates.separation)
        && resolved.speed_scale.is_finite()
        && (SUSTAINED_SPEED_SCALE..=SUDDEN_SPEED_SCALE).contains(&resolved.speed_scale)
        && resolved.damping.is_finite()
        && (0.0..=MAX_RESOLVED_DAMPING).contains(&resolved.damping)
        && resolved.jitter.is_finite()
        && (0.0..=MAX_RESOLVED_JITTER).contains(&resolved.jitter)
}

fn interpolate(start: f32, end: f32, value: f32) -> f32 {
    (end - start).mul_add(value, start)
}

fn resolve_raw_dynamics(rates: DynamicsRates) -> ResolvedDynamics {
    ResolvedDynamics {
        rates,
        speed_scale: 1.0,
        damping: 0.0,
        jitter: 0.0,
    }
}

fn resolve_semantic_dynamics(qualities: SemanticQualities) -> ResolvedDynamics {
    ResolvedDynamics {
        rates: DynamicsRates {
            alignment: interpolate(
                INDIRECT_ALIGNMENT_RATE,
                DIRECT_ALIGNMENT_RATE,
                qualities.space,
            ),
            cohesion: interpolate(
                LOWER_WEIGHT_COHESION_RATE,
                HIGHER_WEIGHT_COHESION_RATE,
                qualities.weight,
            ),
            separation: interpolate(
                INDIRECT_SEPARATION_RATE,
                DIRECT_SEPARATION_RATE,
                qualities.space,
            ),
        },
        speed_scale: interpolate(SUSTAINED_SPEED_SCALE, SUDDEN_SPEED_SCALE, qualities.time),
        damping: interpolate(BOUND_FLOW_DAMPING, FREE_FLOW_DAMPING, qualities.flow),
        jitter: interpolate(BOUND_FLOW_JITTER, FREE_FLOW_JITTER, qualities.flow),
    }
}

fn public_rates(rates: DynamicsRates) -> DynamicsRates {
    DynamicsRates {
        alignment: round_for_public(rates.alignment),
        cohesion: round_for_public(rates.cohesion),
        separation: round_for_public(rates.separation),
    }
}

fn public_qualities(qualities: SemanticQualities) -> SemanticQualities {
    SemanticQualities {
        space: round_for_public(qualities.space),
        time: round_for_public(qualities.time),
        weight: round_for_public(qualities.weight),
        flow: round_for_public(qualities.flow),
    }
}

fn public_resolved(resolved: ResolvedDynamics) -> ResolvedDynamics {
    ResolvedDynamics {
        rates: public_rates(resolved.rates),
        speed_scale: round_for_public(resolved.speed_scale),
        damping: round_for_public(resolved.damping),
        jitter: round_for_public(resolved.jitter),
    }
}

fn deterministic_unit(seed: u64, tick: u64, member: u64, stream: u64) -> f32 {
    let mut value = seed
        ^ tick.wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ member.wrapping_mul(0xbf58_476d_1ce4_e5b9)
        ^ stream.wrapping_mul(0x94d0_49bb_1331_11eb);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    let bits = u16::try_from(value >> 48).unwrap_or_default();
    f32::from(bits) / (f32::from(u16::MAX) + 1.0)
}

fn deterministic_jitter(seed: u64, tick: u64, member: u64, amplitude: f32) -> Vec3 {
    if amplitude <= f32::EPSILON {
        return Vec3::ZERO;
    }
    let x = deterministic_unit(seed, tick, member, 2).mul_add(2.0, -1.0);
    let y = deterministic_unit(seed, tick, member, 3).mul_add(2.0, -1.0);
    normalized_or(Vec3::new(x, y, 0.0), Vec3::ZERO) * amplitude
}

fn initial_snapshot(seed: u64) -> DemoSnapshot {
    let mut rng = SplitMix64::new(seed);
    let mut particles =
        ParticleSet::with_capacity("combinatorial-swarmability.scene", MEMBER_COUNT);
    for index in 0..MEMBER_COUNT {
        let position = Vec3::new(
            rng.signed_unit_f32() * 0.78,
            rng.signed_unit_f32() * 0.78,
            0.0,
        );
        let direction = normalized_or(
            Vec3::new(rng.signed_unit_f32(), rng.signed_unit_f32(), 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        );
        let mut particle = ParticleState::new(
            format!("member-{index:02}"),
            position,
            0.035 + f32::from(member_id(index) % 3) * 0.004,
        );
        particle.velocity = direction * BASE_SPEED;
        particles.push(particle);
    }
    DemoSnapshot {
        seed,
        particles,
        speed_offsets: vec![0.0; MEMBER_COUNT],
        behaviors: vec![CollectiveBehavior::Flock; MEMBER_COUNT],
        raw_dynamics_rates: DEFAULT_DYNAMICS_RATES,
        dynamics_control_mode: DynamicsControlMode::Raw,
        semantic_qualities: DEFAULT_SEMANTIC_QUALITIES,
        resolved_dynamics: resolve_raw_dynamics(DEFAULT_DYNAMICS_RATES),
        fields: Vec::new(),
        groups: vec![GroupState {
            group_id: 0,
            member_ids: (0..MEMBER_COUNT).map(member_id).collect(),
            formation_scale: DEFAULT_FORMATION_SCALE,
        }],
        leases: Vec::new(),
        scope: TargetScope::Member,
        primary_member: Some(0),
        subgroup_members: (0..DEFAULT_SUBGROUP_COUNT).map(member_id).collect(),
        running: false,
        tick: 0,
        accumulator_millis: 0,
        state_revision: 0,
        selection_revision: 0,
        morphology_revision: 0,
        authority_revision: 0,
    }
}

fn validate_snapshot(snapshot: &DemoSnapshot) -> Result<(), DemoError> {
    snapshot
        .particles
        .validate()
        .map_err(|error| DemoError::Matter(error.to_string()))?;
    if snapshot.particles.particles.len() != MEMBER_COUNT {
        return Err(DemoError::InvalidSnapshot("unexpected member count"));
    }
    if snapshot.speed_offsets.len() != MEMBER_COUNT
        || snapshot
            .speed_offsets
            .iter()
            .any(|value| !value.is_finite() || value.abs() > MAX_SPEED_OFFSET)
    {
        return Err(DemoError::InvalidSnapshot("invalid speed offsets"));
    }
    if snapshot.behaviors.len() != MEMBER_COUNT {
        return Err(DemoError::InvalidSnapshot(
            "unexpected collective-behavior count",
        ));
    }
    validate_snapshot_groups(snapshot)?;
    validate_snapshot_dynamics(snapshot)?;
    validate_snapshot_leases(snapshot)?;
    if snapshot.fields.len() > MAX_PERSONAL_FIELDS {
        return Err(DemoError::InvalidSnapshot("too many personal fields"));
    }
    let mut previous_field_id = None;
    for field in &snapshot.fields {
        if field.field_id > MAX_FIELD_ID {
            return Err(DemoError::InvalidSnapshot(
                "field identifier is out of range",
            ));
        }
        if previous_field_id.is_some_and(|previous| previous >= field.field_id) {
            return Err(DemoError::InvalidSnapshot(
                "personal fields must be sorted and unique",
            ));
        }
        if field.contributor_id >= MAX_SYNTHETIC_CONTRIBUTORS {
            return Err(DemoError::InvalidSnapshot(
                "synthetic contributor is out of range",
            ));
        }
        if !valid_field_position(field.x, field.y) {
            return Err(DemoError::InvalidSnapshot("field position is invalid"));
        }
        if field
            .expires_at_tick
            .is_some_and(|expiry| expiry <= snapshot.tick)
        {
            return Err(DemoError::InvalidSnapshot("field expiry is stale"));
        }
        previous_field_id = Some(field.field_id);
    }
    if snapshot
        .primary_member
        .is_some_and(|member| !valid_member(member))
        || snapshot
            .subgroup_members
            .iter()
            .copied()
            .any(|member| !valid_member(member))
    {
        return Err(DemoError::InvalidSnapshot(
            "selection contains unknown member",
        ));
    }
    let mut normalized = snapshot.subgroup_members.clone();
    normalized.sort_unstable();
    normalized.dedup();
    if normalized != snapshot.subgroup_members {
        return Err(DemoError::InvalidSnapshot(
            "subgroup selection must be sorted and unique",
        ));
    }
    if snapshot.accumulator_millis >= FIXED_STEP_MILLIS {
        return Err(DemoError::InvalidSnapshot("accumulator is out of range"));
    }
    Ok(())
}

fn validate_snapshot_groups(snapshot: &DemoSnapshot) -> Result<(), DemoError> {
    if snapshot.groups.is_empty() || snapshot.groups.len() > MAX_GROUPS {
        return Err(DemoError::InvalidSnapshot("invalid morphology group count"));
    }
    let mut previous_group_id = None;
    let mut all_members = BTreeSet::new();
    for group in &snapshot.groups {
        if usize::from(group.group_id) >= MAX_GROUPS
            || previous_group_id.is_some_and(|previous| previous >= group.group_id)
        {
            return Err(DemoError::InvalidSnapshot(
                "morphology groups must have sorted unique canonical IDs",
            ));
        }
        if group.member_ids.is_empty() || !valid_formation_scale(group.formation_scale) {
            return Err(DemoError::InvalidSnapshot(
                "morphology group roster or scale is invalid",
            ));
        }
        let mut normalized = group.member_ids.clone();
        normalized.sort_unstable();
        normalized.dedup();
        if normalized != group.member_ids
            || group
                .member_ids
                .iter()
                .copied()
                .any(|member| !valid_member(member) || !all_members.insert(member))
        {
            return Err(DemoError::InvalidSnapshot(
                "morphology membership must be sorted, unique, and conserved",
            ));
        }
        previous_group_id = Some(group.group_id);
    }
    if all_members.len() != MEMBER_COUNT {
        return Err(DemoError::InvalidSnapshot(
            "every member must belong to exactly one morphology group",
        ));
    }
    Ok(())
}

fn validate_snapshot_leases(snapshot: &DemoSnapshot) -> Result<(), DemoError> {
    if snapshot.leases.len() > MAX_ACTIVE_LEASES {
        return Err(DemoError::InvalidSnapshot("too many active leases"));
    }
    let mut previous_member_id = None;
    for lease in &snapshot.leases {
        if !valid_member(lease.member_id)
            || previous_member_id.is_some_and(|previous| previous >= lease.member_id)
        {
            return Err(DemoError::InvalidSnapshot(
                "leases must use sorted unique canonical member IDs",
            ));
        }
        if !valid_operator(lease.holder_operator_id)
            || lease.pending_handoff_to.is_some_and(|receiver| {
                !valid_operator(receiver) || receiver == lease.holder_operator_id
            })
        {
            return Err(DemoError::InvalidSnapshot(
                "lease holder or handoff receiver is invalid",
            ));
        }
        if lease.acquired_at_tick > snapshot.tick || lease.expires_at_tick <= snapshot.tick {
            return Err(DemoError::InvalidSnapshot(
                "lease acquisition or expiry tick is stale",
            ));
        }
        if lease.expires_at_tick.saturating_sub(lease.acquired_at_tick)
            > u64::from(MAX_LEASE_LIFETIME_STEPS)
        {
            return Err(DemoError::InvalidSnapshot(
                "lease lifetime exceeds the app-local bound",
            ));
        }
        previous_member_id = Some(lease.member_id);
    }
    Ok(())
}

fn validate_snapshot_dynamics(snapshot: &DemoSnapshot) -> Result<(), DemoError> {
    if !valid_dynamics_rate(snapshot.raw_dynamics_rates.alignment)
        || !valid_dynamics_rate(snapshot.raw_dynamics_rates.cohesion)
        || !valid_dynamics_rate(snapshot.raw_dynamics_rates.separation)
    {
        return Err(DemoError::InvalidSnapshot(
            "raw dynamics rate is outside the accepted range",
        ));
    }
    if !valid_semantic_qualities(snapshot.semantic_qualities) {
        return Err(DemoError::InvalidSnapshot(
            "semantic quality is outside the accepted range",
        ));
    }
    if !valid_resolved_dynamics(snapshot.resolved_dynamics) {
        return Err(DemoError::InvalidSnapshot(
            "resolved dynamics vector is outside the accepted range",
        ));
    }
    let expected_resolved = match snapshot.dynamics_control_mode {
        DynamicsControlMode::Raw => resolve_raw_dynamics(snapshot.raw_dynamics_rates),
        DynamicsControlMode::Semantic => resolve_semantic_dynamics(snapshot.semantic_qualities),
        DynamicsControlMode::ComparisonRawMirror => {
            let expected = resolve_semantic_dynamics(DEFAULT_SEMANTIC_QUALITIES);
            if snapshot.raw_dynamics_rates != expected.rates {
                return Err(DemoError::InvalidSnapshot(
                    "comparison raw mirror rates do not match the fixed profile",
                ));
            }
            expected
        }
    };
    if snapshot.resolved_dynamics != expected_resolved {
        return Err(DemoError::InvalidSnapshot(
            "resolved dynamics vector does not match its control owner",
        ));
    }
    Ok(())
}

fn integrate_particle(
    particle: &ParticleState,
    acceleration: Vec3,
    target_speed: f32,
) -> (Vec3, Vec3) {
    let mut velocity = particle.velocity + acceleration * FIXED_STEP_SECONDS;
    velocity = normalized_or(velocity, particle.velocity) * target_speed;
    let mut position = particle.position + velocity * FIXED_STEP_SECONDS;
    contain_axis(&mut position.x, &mut velocity.x);
    contain_axis(&mut position.y, &mut velocity.y);
    position.z = 0.0;
    velocity.z = 0.0;
    (position, velocity)
}

fn valid_member(member_id: u16) -> bool {
    usize::from(member_id) < MEMBER_COUNT
}

fn valid_operator(operator_id: u8) -> bool {
    operator_id < MAX_SYNTHETIC_OPERATORS
}

fn lease_for_member(leases: &[LeaseState], member_id: u16) -> Option<&LeaseState> {
    leases
        .binary_search_by_key(&member_id, |lease| lease.member_id)
        .ok()
        .map(|index| &leases[index])
}

const fn behavior_label(behavior: CollectiveBehavior) -> &'static str {
    match behavior {
        CollectiveBehavior::Flock => "Flock",
        CollectiveBehavior::Cohere => "Cohere",
        CollectiveBehavior::Disperse => "Disperse",
    }
}

fn valid_formation_scale(scale: f32) -> bool {
    scale.is_finite() && (MIN_FORMATION_SCALE..=MAX_FORMATION_SCALE).contains(&scale)
}

fn normalized_or(value: Vec3, fallback: Vec3) -> Vec3 {
    let length = value.length();
    if length > 0.000_01 {
        value / length
    } else {
        let fallback_length = fallback.length();
        if fallback_length > 0.000_01 {
            fallback / fallback_length
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        }
    }
}

fn contain_axis(position: &mut f32, velocity: &mut f32) {
    if *position < -WORLD_LIMIT {
        *position = -WORLD_LIMIT;
        *velocity = velocity.abs();
    } else if *position > WORLD_LIMIT {
        *position = WORLD_LIMIT;
        *velocity = -velocity.abs();
    }
}

fn round_for_public(value: f32) -> f32 {
    (value * 1_000.0).round() / 1_000.0
}
