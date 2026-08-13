use core::fmt;
use std::collections::BTreeSet;

use rusty_matter_model::Vec3;
use rusty_matter_particles::{ParticleRenderPayload, ParticleSet, ParticleState};
use serde::{Deserialize, Serialize};

use crate::action::{
    ActionCode, ActionReceipt, BehaviorCounts, CollectiveBehavior, MemberSummary, PublicState,
    SemanticAction, TargetScope,
};
use crate::replay::{
    validate_replay_tape, ReplayEvent, ReplayRecorder, ReplayTape, MAX_REPLAY_JSON_BYTES,
};
use crate::rng::SplitMix64;

/// Number of members in the first public scene.
pub const MEMBER_COUNT: usize = 24;
/// Number of `f32` values in each Wasm frame row.
pub const FRAME_ROW_WIDTH: usize = 11;

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
struct DemoSnapshot {
    seed: u64,
    particles: ParticleSet,
    speed_offsets: Vec<f32>,
    behaviors: Vec<CollectiveBehavior>,
    scope: TargetScope,
    primary_member: Option<u16>,
    subgroup_members: Vec<u16>,
    running: bool,
    tick: u64,
    accumulator_millis: u32,
    state_revision: u64,
    selection_revision: u64,
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
    pub fn public_state(&self) -> PublicState {
        let targets = self.resolved_targets();
        let target_set = targets.iter().copied().collect::<BTreeSet<_>>();
        let subgroup_set = self
            .snapshot
            .subgroup_members
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
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
        self.snapshot = initial_snapshot(seed);
        self.snapshot.state_revision = next_state_revision;
        self.snapshot.selection_revision = next_selection_revision;
        self.accepted(
            ActionCode::Reset,
            "Reset to the current seed's paused initial state.".to_owned(),
            Vec::new(),
        )
    }

    fn restart_seed(&mut self, seed: u64) -> ActionReceipt {
        let next_state_revision = self.snapshot.state_revision.saturating_add(1);
        let next_selection_revision = self.snapshot.selection_revision.saturating_add(1);
        self.snapshot = initial_snapshot(seed);
        self.snapshot.state_revision = next_state_revision;
        self.snapshot.selection_revision = next_selection_revision;
        self.accepted(
            ActionCode::SeedRestarted,
            format!("Restarted with seed {seed} in a paused state."),
            Vec::new(),
        )
    }

    fn step_simulation(&mut self) {
        let source = self.snapshot.particles.particles.clone();
        let swarm_centroid = particle_centroid(&source);
        let mut next = source.clone();
        for (index, particle) in source.iter().enumerate() {
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

            let target_speed =
                (BASE_SPEED + self.snapshot.speed_offsets[index]).clamp(MIN_SPEED, MAX_SPEED);
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
            let (behavior_centroid, behavior_peer_count) =
                peer_behavior_centroid(&source, &self.snapshot.behaviors, behavior, index)
                    .unwrap_or((swarm_centroid, 1));
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
                    let peer_dispersion =
                        peer_dispersion_vector(&source, &self.snapshot.behaviors, behavior, index);
                    let dispersion = if peer_dispersion.length_squared() > f32::EPSILON {
                        peer_dispersion
                    } else {
                        particle.position - swarm_centroid
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
            let acceleration = behavior_acceleration
                + soft_boundary_acceleration(particle.position, particle.velocity, target_speed);

            let mut velocity = particle.velocity + acceleration * FIXED_STEP_SECONDS;
            velocity = normalized_or(velocity, particle.velocity) * target_speed;
            let mut position = particle.position + velocity * FIXED_STEP_SECONDS;
            contain_axis(&mut position.x, &mut velocity.x);
            contain_axis(&mut position.y, &mut velocity.y);
            position.z = 0.0;
            velocity.z = 0.0;

            next[index].position = position;
            next[index].velocity = velocity;
            next[index].age_seconds = particle.age_seconds + FIXED_STEP_SECONDS;
        }
        self.snapshot.tick = self.snapshot.tick.saturating_add(1);
        self.snapshot.particles.particles = next;
        self.snapshot.particles.time_seconds += FIXED_STEP_SECONDS;
        self.bump_state();
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

fn particle_centroid(particles: &[ParticleState]) -> Vec3 {
    let sum = particles.iter().fold(Vec3::ZERO, |centroid, particle| {
        centroid + particle.position
    });
    sum / MEMBER_COUNT_F32
}

fn peer_behavior_centroid(
    particles: &[ParticleState],
    behaviors: &[CollectiveBehavior],
    behavior: CollectiveBehavior,
    member_index: usize,
) -> Option<(Vec3, u16)> {
    let mut sum = Vec3::ZERO;
    let mut count = 0_u16;
    for (index, particle) in particles.iter().enumerate() {
        if index != member_index && behaviors[index] == behavior {
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
) -> Vec3 {
    let member = &particles[member_index];
    particles
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != member_index && behaviors[*index] == behavior)
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

fn soft_boundary_acceleration(position: Vec3, velocity: Vec3, target_speed: f32) -> Vec3 {
    if position.x.abs() <= SOFT_WORLD_LIMIT && position.y.abs() <= SOFT_WORLD_LIMIT {
        return Vec3::ZERO;
    }
    let desired = normalized_or(Vec3::ZERO - position, velocity) * target_speed;
    (desired - velocity) * 6.0
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
        scope: TargetScope::Member,
        primary_member: Some(0),
        subgroup_members: (0..DEFAULT_SUBGROUP_COUNT).map(member_id).collect(),
        running: false,
        tick: 0,
        accumulator_millis: 0,
        state_revision: 0,
        selection_revision: 0,
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

fn valid_member(member_id: u16) -> bool {
    usize::from(member_id) < MEMBER_COUNT
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
