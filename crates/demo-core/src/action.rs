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

/// Deterministic rule used to partition one canonical morphology group.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupPartitionRule {
    /// Sort members by stable ID; source retains even ordinals and new group receives odd ordinals.
    AlternatingMemberId,
}

/// Explicit receiver response to a pending app-local lease handoff.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HandoffDecision {
    /// Receiver consents and becomes holder for the lease's remaining lifetime.
    Accept,
    /// Receiver declines; the current holder and original expiry remain in place.
    Decline,
}

/// App-owned rates for entering the three existing collective steering modes.
///
/// These values are a bounded technical reconstruction of endogenous controller
/// rates, not a reproduction of the source system's robot-state model.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DynamicsRates {
    /// Per-member, per-second rate for entering the alignment-oriented flock mode.
    pub alignment: f32,
    /// Per-member, per-second rate for entering the cohesion-oriented cohere mode.
    pub cohesion: f32,
    /// Per-member, per-second rate for entering the separation-oriented disperse mode.
    pub separation: f32,
}

/// Which app-owned control surface currently owns the resolved dynamics vector.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DynamicsControlMode {
    /// Explicit raw rate actions own the effective vector.
    Raw,
    /// Space, Time, Weight, and Flow compile into the effective vector.
    Semantic,
    /// One fixed comparison-only raw vector that mirrors the semantic midpoint exactly.
    ComparisonRawMirror,
}

/// Bounded semantic movement-quality values.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticQualities {
    /// Indirect (0) to Direct (1).
    pub space: f32,
    /// Sustained (0) to Sudden (1).
    pub time: f32,
    /// Lower/compressed (0) to higher (1) value used by the source mapping.
    pub weight: f32,
    /// Bound (0) to Free (1).
    pub flow: f32,
}

/// Complete app-owned vector consumed by the one deterministic simulation path.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedDynamics {
    /// Effective rates for entering Flock, Cohere, and Disperse modes.
    pub rates: DynamicsRates,
    /// Global multiplier applied after member-level preferred-speed adjustments.
    pub speed_scale: f32,
    /// Bounded reduction in steering response.
    pub damping: f32,
    /// Bounded deterministic heading perturbation amplitude.
    pub jitter: f32,
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
    /// Set the swarm-wide rate for entering the alignment-oriented flock mode.
    SetAlignment {
        /// Bounded transitions per member-second.
        rate: f32,
    },
    /// Set the swarm-wide rate for entering the cohesion-oriented cohere mode.
    SetCohesion {
        /// Bounded transitions per member-second.
        rate: f32,
    },
    /// Set the swarm-wide rate for entering the separation-oriented disperse mode.
    SetSeparation {
        /// Bounded transitions per member-second.
        rate: f32,
    },
    /// Set the bounded semantic Space quality and resolve the complete raw vector.
    SetSpaceQuality {
        /// Indirect (0) to Direct (1).
        value: f32,
    },
    /// Set the bounded semantic Time quality and resolve the complete raw vector.
    SetTimeQuality {
        /// Sustained (0) to Sudden (1).
        value: f32,
    },
    /// Set the bounded semantic Weight quality and resolve the complete raw vector.
    SetWeightQuality {
        /// Lower/compressed (0) to higher (1) mapped value.
        value: f32,
    },
    /// Set the bounded semantic Flow quality and resolve the complete raw vector.
    SetFlowQuality {
        /// Bound (0) to Free (1).
        value: f32,
    },
    /// Install the one fixed raw-vector mirror used to prove comparison equivalence.
    ///
    /// This is deliberately not generic parameter plumbing: the profile has no
    /// caller-supplied coefficients and is available only as atlas infrastructure.
    ApplyComparisonRawMirror,
    /// Split one canonical morphology group by an explicit deterministic rule.
    SplitGroup {
        /// Existing source group whose identity and scale target remain in place.
        source_group_id: u8,
        /// Smallest currently unused canonical group identifier.
        new_group_id: u8,
        /// Explicit deterministic member partition.
        partition_rule: GroupPartitionRule,
        /// Morphology revision against which the operation was prepared.
        expected_morphology_revision: u64,
    },
    /// Merge two exact morphology groups into the canonical lower-ID survivor.
    MergeGroups {
        /// First participating group; operand order does not affect the result.
        group_a_id: u8,
        /// Second participating group; must differ from the first.
        group_b_id: u8,
        /// Canonical survivor, required to be the lower participating ID.
        survivor_group_id: u8,
        /// Morphology revision against which the operation was prepared.
        expected_morphology_revision: u64,
    },
    /// Set one group's explicit formation-scale target without changing dynamics parameters.
    SetFormationScale {
        /// Existing canonical group identifier.
        group_id: u8,
        /// Bounded multiplier around the neutral default of 1.
        scale: f32,
        /// Morphology revision against which the operation was prepared.
        expected_morphology_revision: u64,
    },
    /// Acquire one unheld member for a bounded fixed-step lifetime.
    RequestLease {
        /// Canonical member identifier.
        member_id: u16,
        /// App-local synthetic operator channel requesting authority.
        operator_id: u8,
        /// Positive bounded lifetime measured only in deterministic fixed steps.
        lifetime_steps: u32,
        /// Authority revision against which the request was prepared.
        expected_authority_revision: u64,
    },
    /// Release one lease explicitly; only its exact current holder may release it.
    ReleaseLease {
        /// Canonical member identifier.
        member_id: u16,
        /// Exact current synthetic holder.
        operator_id: u8,
        /// Authority revision against which the release was prepared.
        expected_authority_revision: u64,
    },
    /// Offer a held lease to one explicit distinct receiver.
    OfferLeaseHandoff {
        /// Canonical member identifier.
        member_id: u16,
        /// Exact current synthetic holder consenting to the offer.
        holder_operator_id: u8,
        /// Explicit distinct synthetic receiver.
        receiver_operator_id: u8,
        /// Authority revision against which the offer was prepared.
        expected_authority_revision: u64,
    },
    /// Accept or decline one exact pending lease handoff as its named receiver.
    ResolveLeaseHandoff {
        /// Canonical member identifier.
        member_id: u16,
        /// Exact pending synthetic receiver.
        receiver_operator_id: u8,
        /// Explicit receiver consent or refusal.
        decision: HandoffDecision,
        /// Authority revision against which the response was prepared.
        expected_authority_revision: u64,
    },
    /// Use a held lease to assign one behavior to its exact canonical member.
    SetLeasedBehavior {
        /// Canonical member identifier.
        member_id: u16,
        /// Exact current synthetic holder.
        operator_id: u8,
        /// Collective steering rule to assign to this member only.
        behavior: CollectiveBehavior,
        /// Authority revision against which the use was prepared.
        expected_authority_revision: u64,
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
    SetAlignment {
        rate: f32,
    },
    SetCohesion {
        rate: f32,
    },
    SetSeparation {
        rate: f32,
    },
    SetSpaceQuality {
        value: f32,
    },
    SetTimeQuality {
        value: f32,
    },
    SetWeightQuality {
        value: f32,
    },
    SetFlowQuality {
        value: f32,
    },
    ApplyComparisonRawMirror {},
    SplitGroup {
        source_group_id: u8,
        new_group_id: u8,
        partition_rule: GroupPartitionRule,
        expected_morphology_revision: u64,
    },
    MergeGroups {
        group_a_id: u8,
        group_b_id: u8,
        survivor_group_id: u8,
        expected_morphology_revision: u64,
    },
    SetFormationScale {
        group_id: u8,
        scale: f32,
        expected_morphology_revision: u64,
    },
    RequestLease {
        member_id: u16,
        operator_id: u8,
        lifetime_steps: u32,
        expected_authority_revision: u64,
    },
    ReleaseLease {
        member_id: u16,
        operator_id: u8,
        expected_authority_revision: u64,
    },
    OfferLeaseHandoff {
        member_id: u16,
        holder_operator_id: u8,
        receiver_operator_id: u8,
        expected_authority_revision: u64,
    },
    ResolveLeaseHandoff {
        member_id: u16,
        receiver_operator_id: u8,
        decision: HandoffDecision,
        expected_authority_revision: u64,
    },
    SetLeasedBehavior {
        member_id: u16,
        operator_id: u8,
        behavior: CollectiveBehavior,
        expected_authority_revision: u64,
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
    #[allow(clippy::too_many_lines)] // Strict wire mapping is intentionally explicit and centralized.
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
            SemanticActionWire::SetAlignment { rate } => Self::SetAlignment { rate },
            SemanticActionWire::SetCohesion { rate } => Self::SetCohesion { rate },
            SemanticActionWire::SetSeparation { rate } => Self::SetSeparation { rate },
            SemanticActionWire::SetSpaceQuality { value } => Self::SetSpaceQuality { value },
            SemanticActionWire::SetTimeQuality { value } => Self::SetTimeQuality { value },
            SemanticActionWire::SetWeightQuality { value } => Self::SetWeightQuality { value },
            SemanticActionWire::SetFlowQuality { value } => Self::SetFlowQuality { value },
            SemanticActionWire::ApplyComparisonRawMirror {} => Self::ApplyComparisonRawMirror,
            SemanticActionWire::SplitGroup {
                source_group_id,
                new_group_id,
                partition_rule,
                expected_morphology_revision,
            } => Self::SplitGroup {
                source_group_id,
                new_group_id,
                partition_rule,
                expected_morphology_revision,
            },
            SemanticActionWire::MergeGroups {
                group_a_id,
                group_b_id,
                survivor_group_id,
                expected_morphology_revision,
            } => Self::MergeGroups {
                group_a_id,
                group_b_id,
                survivor_group_id,
                expected_morphology_revision,
            },
            SemanticActionWire::SetFormationScale {
                group_id,
                scale,
                expected_morphology_revision,
            } => Self::SetFormationScale {
                group_id,
                scale,
                expected_morphology_revision,
            },
            SemanticActionWire::RequestLease {
                member_id,
                operator_id,
                lifetime_steps,
                expected_authority_revision,
            } => Self::RequestLease {
                member_id,
                operator_id,
                lifetime_steps,
                expected_authority_revision,
            },
            SemanticActionWire::ReleaseLease {
                member_id,
                operator_id,
                expected_authority_revision,
            } => Self::ReleaseLease {
                member_id,
                operator_id,
                expected_authority_revision,
            },
            SemanticActionWire::OfferLeaseHandoff {
                member_id,
                holder_operator_id,
                receiver_operator_id,
                expected_authority_revision,
            } => Self::OfferLeaseHandoff {
                member_id,
                holder_operator_id,
                receiver_operator_id,
                expected_authority_revision,
            },
            SemanticActionWire::ResolveLeaseHandoff {
                member_id,
                receiver_operator_id,
                decision,
                expected_authority_revision,
            } => Self::ResolveLeaseHandoff {
                member_id,
                receiver_operator_id,
                decision,
                expected_authority_revision,
            },
            SemanticActionWire::SetLeasedBehavior {
                member_id,
                operator_id,
                behavior,
                expected_authority_revision,
            } => Self::SetLeasedBehavior {
                member_id,
                operator_id,
                behavior,
                expected_authority_revision,
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
    /// Swarm-wide alignment-mode entry rate changed.
    AlignmentSet,
    /// Swarm-wide cohesion-mode entry rate changed.
    CohesionSet,
    /// Swarm-wide separation-mode entry rate changed.
    SeparationSet,
    /// Semantic Space quality changed and resolved into the raw vector.
    SpaceQualitySet,
    /// Semantic Time quality changed and resolved into the raw vector.
    TimeQualitySet,
    /// Semantic Weight quality changed and resolved into the raw vector.
    WeightQualitySet,
    /// Semantic Flow quality changed and resolved into the raw vector.
    FlowQualitySet,
    /// The comparison-only explicit raw vector was installed.
    ComparisonRawMirrorApplied,
    /// One canonical morphology group was split.
    GroupSplit,
    /// Two exact morphology groups were merged into their canonical survivor.
    GroupsMerged,
    /// One explicit formation-scale target changed.
    FormationScaleSet,
    /// One previously unheld canonical member was leased.
    LeaseAcquired,
    /// One current holder explicitly released its lease.
    LeaseReleased,
    /// One current holder offered its lease to an explicit receiver.
    LeaseHandoffOffered,
    /// The explicit receiver accepted and became the current holder.
    LeaseHandoffAccepted,
    /// The explicit receiver declined and the current holder remained unchanged.
    LeaseHandoffDeclined,
    /// The current holder used its lease to change one member's behavior.
    LeasedBehaviorSet,
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
    /// A raw dynamics rate was non-finite or outside its explicit range.
    InvalidDynamicsRate,
    /// A semantic quality was non-finite or outside the normalized range.
    InvalidSemanticQuality,
    /// The operation was prepared against an older morphology state.
    StaleMorphology,
    /// The requested canonical morphology group does not exist.
    MissingGroup,
    /// A split or merge contains a duplicate or impossible group operand.
    InvalidGroupOperation,
    /// The requested new or surviving group identity is not canonical.
    NonCanonicalGroup,
    /// The source group cannot be partitioned while conserving non-empty groups.
    GroupCannotSplit,
    /// The bounded scene already contains its maximum number of morphology groups.
    GroupLimitReached,
    /// A formation scale was non-finite or outside its explicit range.
    InvalidFormationScale,
    /// The operation was prepared against an older app-local authority state.
    StaleAuthority,
    /// The synthetic operator channel is outside the app-local bound.
    InvalidOperator,
    /// The requested fixed-step lease lifetime is zero, excessive, or overflowing.
    InvalidLeaseLifetime,
    /// The bounded scene already contains its maximum number of active leases.
    LeaseLimitReached,
    /// The requested member already has an active lease.
    LeaseAlreadyHeld,
    /// The requested member has no current unexpired lease.
    MissingLease,
    /// The supplied synthetic operator is not the exact current holder.
    NotLeaseHolder,
    /// A handoff receiver must be valid and distinct from the current holder.
    InvalidHandoff,
    /// The lease already has a pending handoff offer.
    HandoffAlreadyPending,
    /// No exact pending handoff exists for this member and receiver.
    MissingHandoff,
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
    /// Morphology revision after evaluation.
    pub morphology_revision: u64,
    /// App-local authority revision after evaluation.
    pub authority_revision: u64,
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
    /// Canonical morphology group containing this member.
    pub group_id: u8,
    /// Current app-local synthetic lease holder, if any.
    pub lease_holder_operator_id: Option<u8>,
}

/// One canonical morphology group's complete public roster and observed extent.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GroupSummary {
    /// Stable bounded group identifier.
    pub group_id: u8,
    /// Sorted conserved member identifiers.
    pub member_ids: Vec<u16>,
    /// Explicit app-owned formation-scale target.
    pub formation_scale: f32,
    /// Maximum observed member distance from the current group centroid.
    pub formation_extent: f32,
}

/// One active app-local simulated authority lease.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LeaseSummary {
    /// Canonical leased member identifier.
    pub member_id: u16,
    /// Current app-local synthetic operator holder.
    pub holder_operator_id: u8,
    /// Fixed tick at which the lease was originally acquired.
    pub acquired_at_tick: u64,
    /// Exclusive fixed tick at which the lease expires.
    pub expires_at_tick: u64,
    /// Fixed steps remaining at the current tick.
    pub remaining_steps: u64,
    /// Explicit pending receiver, if the current holder offered a handoff.
    pub pending_handoff_to: Option<u8>,
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
    /// Effective collective-mode entry rates retained for the existing raw view.
    pub dynamics_rates: DynamicsRates,
    /// Raw rate values retained when semantic control owns the effective vector.
    pub raw_dynamics_rates: DynamicsRates,
    /// Current owner of the effective raw dynamics vector.
    pub dynamics_control_mode: DynamicsControlMode,
    /// Current semantic values, whether or not semantic mode is active.
    pub semantic_qualities: SemanticQualities,
    /// Complete effective vector consumed by deterministic stepping.
    pub resolved_dynamics: ResolvedDynamics,
    /// Canonical morphology groups in stable identifier order.
    pub groups: Vec<GroupSummary>,
    /// Monotonic revision fencing split, merge, and rescale operations.
    pub morphology_revision: u64,
    /// Active per-member leases in canonical member-ID order.
    pub leases: Vec<LeaseSummary>,
    /// Monotonic app-local authority revision fencing every lease mutation and use.
    pub authority_revision: u64,
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
