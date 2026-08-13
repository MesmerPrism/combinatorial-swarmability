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

/// Direction of one app-local synthetic personal field.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldPolarity {
    /// Bend member trajectories toward the field source.
    Attract,
    /// Bend member trajectories away from the field source.
    Repel,
}

/// Explicit lifetime requested when placing a personal field.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum FieldLifetime {
    /// Remain active until removed or the scene is reset.
    Persistent,
    /// Expire after the requested number of fixed simulation steps.
    Expiring {
        /// Positive bounded fixed-step lifetime.
        steps: u32,
    },
}

#[derive(Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
enum FieldLifetimeWire {
    Persistent {},
    Expiring { steps: u32 },
}

impl<'de> Deserialize<'de> for FieldLifetime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match FieldLifetimeWire::deserialize(deserializer)? {
            FieldLifetimeWire::Persistent {} => Self::Persistent,
            FieldLifetimeWire::Expiring { steps } => Self::Expiring { steps },
        })
    }
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
    /// Place one bounded field with synthetic contributor provenance.
    PlaceField {
        /// Stable field identifier within the scene.
        field_id: u16,
        /// App-local synthetic contributor channel; never an account identity.
        contributor_id: u8,
        /// Horizontal position in normalized scene coordinates.
        x: f32,
        /// Vertical position in normalized scene coordinates.
        y: f32,
        /// Whether the field attracts or repels.
        polarity: FieldPolarity,
        /// Persistent or bounded expiring lifetime.
        lifetime: FieldLifetime,
    },
    /// Move an existing field without changing its provenance, polarity, or lifetime.
    MoveField {
        /// Stable field identifier within the scene.
        field_id: u16,
        /// New horizontal position in normalized scene coordinates.
        x: f32,
        /// New vertical position in normalized scene coordinates.
        y: f32,
    },
    /// Change one existing field's polarity.
    SetFieldPolarity {
        /// Stable field identifier within the scene.
        field_id: u16,
        /// New attract or repel direction.
        polarity: FieldPolarity,
    },
    /// Remove one existing field explicitly.
    RemoveField {
        /// Stable field identifier within the scene.
        field_id: u16,
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
    PlaceField {
        field_id: u16,
        contributor_id: u8,
        x: f32,
        y: f32,
        polarity: FieldPolarity,
        lifetime: FieldLifetime,
    },
    MoveField {
        field_id: u16,
        x: f32,
        y: f32,
    },
    SetFieldPolarity {
        field_id: u16,
        polarity: FieldPolarity,
    },
    RemoveField {
        field_id: u16,
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
            SemanticActionWire::PlaceField {
                field_id,
                contributor_id,
                x,
                y,
                polarity,
                lifetime,
            } => Self::PlaceField {
                field_id,
                contributor_id,
                x,
                y,
                polarity,
                lifetime,
            },
            SemanticActionWire::MoveField { field_id, x, y } => Self::MoveField { field_id, x, y },
            SemanticActionWire::SetFieldPolarity { field_id, polarity } => {
                Self::SetFieldPolarity { field_id, polarity }
            }
            SemanticActionWire::RemoveField { field_id } => Self::RemoveField { field_id },
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
    /// A bounded personal field was placed.
    FieldPlaced,
    /// An existing personal field was moved.
    FieldMoved,
    /// An existing personal field changed polarity.
    FieldPolaritySet,
    /// An existing personal field was removed.
    FieldRemoved,
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
    /// The requested field identifier is outside the bounded scene contract.
    InvalidFieldId,
    /// A field already uses the requested identifier.
    DuplicateField,
    /// The requested field does not exist.
    MissingField,
    /// The synthetic contributor channel is outside the app-local bound.
    InvalidContributor,
    /// A field position was non-finite or outside the normalized scene.
    InvalidFieldPosition,
    /// An expiring field requested a zero, excessive, or overflowing lifetime.
    InvalidFieldLifetime,
    /// The bounded scene already contains its maximum number of fields.
    FieldLimitReached,
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

/// Concise personal-field state for the semantic DOM surface.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FieldSummary {
    /// Stable scene-local field identifier.
    pub field_id: u16,
    /// App-local synthetic contributor channel.
    pub contributor_id: u8,
    /// Horizontal normalized scene position.
    pub x: f32,
    /// Vertical normalized scene position.
    pub y: f32,
    /// Current attract or repel direction.
    pub polarity: FieldPolarity,
    /// Absolute fixed-step expiry, or `None` for persistent fields.
    pub expires_at_tick: Option<u64>,
    /// Fixed steps remaining at the current tick, or `None` for persistent fields.
    pub remaining_steps: Option<u64>,
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
    /// Active additive personal fields in stable identifier order.
    pub fields: Vec<FieldSummary>,
    /// Number of app-local synthetic contributor channels currently represented.
    pub active_contributor_count: usize,
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
