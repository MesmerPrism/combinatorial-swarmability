use serde::{Deserialize, Serialize};

/// Target scope selected independently from input modality.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetScope {
    /// Apply an action to the primary selected member.
    Member,
    /// Apply an action to the current subgroup selection.
    Subgroup,
    /// Apply an action to every member.
    Swarm,
}

/// Collective steering rule assigned independently to each member.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectiveBehavior {
    /// Align with nearby headings while maintaining local spacing.
    Flock,
    /// Move toward members sharing the cohere rule while maintaining separation.
    Cohere,
    /// Move away from members sharing the disperse rule or from the swarm centre.
    Disperse,
}

/// Input-modality-free action accepted by the app-local reducer.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SemanticAction {
    /// Select which target scope subsequent actions use.
    SetScope {
        /// New target scope.
        scope: TargetScope,
    },
    /// Select one primary member.
    SelectMember {
        /// Stable zero-based member identifier.
        member_id: u16,
    },
    /// Add or remove one member from the subgroup.
    ToggleSubgroupMember {
        /// Stable zero-based member identifier.
        member_id: u16,
    },
    /// Remove every member from the subgroup selection.
    ClearSubgroup,
    /// Adjust the target members' preferred speed.
    AdjustSpeed {
        /// Signed speed change in world units per second.
        delta: f32,
        /// Selection revision against which the action was prepared.
        expected_selection_revision: u64,
    },
    /// Assign one collective steering rule to the resolved targets.
    SetBehavior {
        /// Collective steering rule to assign.
        behavior: CollectiveBehavior,
        /// Selection revision against which the action was prepared.
        expected_selection_revision: u64,
    },
    /// Begin fixed-step motion.
    Start,
    /// Pause fixed-step motion.
    Pause,
    /// Advance one fixed step while paused.
    Step,
    /// Restore the current seed's initial paused state.
    Reset,
    /// Restore a new seed's initial paused state.
    RestartSeed {
        /// New deterministic seed.
        seed: u64,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum SemanticActionWire {
    SetScope {
        scope: TargetScope,
    },
    SelectMember {
        member_id: u16,
    },
    ToggleSubgroupMember {
        member_id: u16,
    },
    ClearSubgroup {},
    AdjustSpeed {
        delta: f32,
        expected_selection_revision: u64,
    },
    SetBehavior {
        behavior: CollectiveBehavior,
        expected_selection_revision: u64,
    },
    Start {},
    Pause {},
    Step {},
    Reset {},
    RestartSeed {
        seed: u64,
    },
}

impl<'de> Deserialize<'de> for SemanticAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match SemanticActionWire::deserialize(deserializer)? {
            SemanticActionWire::SetScope { scope } => Self::SetScope { scope },
            SemanticActionWire::SelectMember { member_id } => Self::SelectMember { member_id },
            SemanticActionWire::ToggleSubgroupMember { member_id } => {
                Self::ToggleSubgroupMember { member_id }
            }
            SemanticActionWire::ClearSubgroup {} => Self::ClearSubgroup,
            SemanticActionWire::AdjustSpeed {
                delta,
                expected_selection_revision,
            } => Self::AdjustSpeed {
                delta,
                expected_selection_revision,
            },
            SemanticActionWire::SetBehavior {
                behavior,
                expected_selection_revision,
            } => Self::SetBehavior {
                behavior,
                expected_selection_revision,
            },
            SemanticActionWire::Start {} => Self::Start,
            SemanticActionWire::Pause {} => Self::Pause,
            SemanticActionWire::Step {} => Self::Step,
            SemanticActionWire::Reset {} => Self::Reset,
            SemanticActionWire::RestartSeed { seed } => Self::RestartSeed { seed },
        })
    }
}

/// Stable receipt code for a semantic action.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCode {
    /// Scope changed or was already selected.
    ScopeSet,
    /// Primary selection changed.
    MemberSelected,
    /// Subgroup membership changed.
    SubgroupChanged,
    /// Subgroup was cleared.
    SubgroupCleared,
    /// Preferred speed changed for all resolved targets.
    SpeedAdjusted,
    /// Collective steering rule changed for all resolved targets.
    BehaviorSet,
    /// Motion started.
    Started,
    /// Motion paused.
    Paused,
    /// One paused step completed.
    Stepped,
    /// Current seed was reset.
    Reset,
    /// New seed was installed.
    SeedRestarted,
    /// The requested member does not exist.
    InvalidMember,
    /// The requested scope currently resolves to no members.
    EmptySelection,
    /// The request was prepared against an older selection.
    StaleSelection,
    /// The speed delta was non-finite, zero, or outside the accepted bound.
    InvalidSpeedDelta,
    /// Single-step was requested while the simulation was running.
    StepRequiresPause,
}

/// Bounded result of applying one semantic action.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ActionReceipt {
    /// Whether the action was accepted.
    pub accepted: bool,
    /// Stable machine-readable outcome.
    pub code: ActionCode,
    /// Concise user-facing summary.
    pub summary: String,
    /// Members whose state changed.
    pub changed_member_ids: Vec<u16>,
    /// State revision after evaluation.
    pub state_revision: u64,
    /// Selection revision after evaluation.
    pub selection_revision: u64,
}

/// Concise member state for the semantic DOM surface.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MemberSummary {
    /// Stable member identifier.
    pub member_id: u16,
    /// Current speed in world units per second.
    pub speed: f32,
    /// Whether this is the primary member.
    pub primary_selected: bool,
    /// Whether this member belongs to the subgroup.
    pub subgroup_selected: bool,
    /// Whether the current scope targets this member.
    pub targeted: bool,
    /// Collective steering rule currently assigned to this member.
    pub behavior: CollectiveBehavior,
}

/// Count of members currently assigned to each collective steering rule.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BehaviorCounts {
    /// Members using local alignment and spacing.
    pub flock: usize,
    /// Members steering toward their cohere peers.
    pub cohere: usize,
    /// Members steering away from their disperse peers.
    pub disperse: usize,
}

/// Public state projected outside the high-rate canvas.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PublicState {
    /// Seed formatted as a decimal string for JavaScript safety.
    pub seed: String,
    /// Completed fixed-step count.
    pub tick: u64,
    /// Whether elapsed time currently advances the simulation.
    pub running: bool,
    /// Active target scope.
    pub scope: TargetScope,
    /// Primary selected member, if any.
    pub primary_member: Option<u16>,
    /// Sorted subgroup member identifiers.
    pub subgroup_members: Vec<u16>,
    /// Members resolved by the active scope.
    pub target_members: Vec<u16>,
    /// Current average particle speed.
    pub average_speed: f32,
    /// Current distribution of collective steering rules.
    pub behavior_counts: BehaviorCounts,
    /// Monotonic application-state revision.
    pub state_revision: u64,
    /// Monotonic selection-only revision.
    pub selection_revision: u64,
    /// Number of bounded events in the deterministic replay tape.
    pub replay_event_count: usize,
    /// Fixed simulation steps represented by explicit step actions and elapsed updates.
    pub replay_step_count: u64,
    /// Whether the current state can still be reproduced from its bounded tape.
    pub replay_available: bool,
    /// Per-member DOM summaries.
    pub members: Vec<MemberSummary>,
}
