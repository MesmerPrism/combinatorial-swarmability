//! Deterministic simulation and semantic-action core for the browser demo.

mod action;
mod comparison;
mod engine;
mod replay;
mod rng;

pub use action::{
    ActionCode, ActionReceipt, BehaviorCounts, ClearanceMetrics, CollectiveBehavior,
    CollisionPolicy, DynamicsControlMode, DynamicsRates, ExecutionSettings, FieldLifetime,
    FieldPolarity, FieldSummary, GroupPartitionRule, GroupSummary, HandoffDecision, LeaseSummary,
    MemberSummary, NavigationFieldSummary, PublicState, ResolvedDynamics, SemanticAction,
    SemanticQualities, TargetScope,
};
pub use comparison::{ComparisonError, ComparisonRunner};
pub use engine::{
    DemoCore, DemoError, DEFAULT_DYNAMICS_RATES, DEFAULT_EXECUTION_SETTINGS,
    DEFAULT_FORMATION_SCALE, DEFAULT_SEMANTIC_QUALITIES, FRAME_ROW_WIDTH, MAX_ACCELERATION_LIMIT,
    MAX_ACTIVE_LEASES, MAX_BOUNDARY_STRENGTH, MAX_DYNAMICS_RATE, MAX_FIELD_LIFETIME_STEPS,
    MAX_FORMATION_SCALE, MAX_GROUPS, MAX_LEASE_LIFETIME_STEPS, MAX_NAVIGATION_FIELD_RADIUS,
    MAX_NAVIGATION_FIELD_STRENGTH, MAX_PERSONAL_FIELDS, MAX_SEMANTIC_QUALITY,
    MAX_SEPARATION_RADIUS, MAX_SEPARATION_WEIGHT, MAX_SPEED_LIMIT, MAX_SYNTHETIC_CONTRIBUTORS,
    MAX_SYNTHETIC_OPERATORS, MEMBER_COUNT, MIN_ACCELERATION_LIMIT, MIN_DYNAMICS_RATE,
    MIN_FORMATION_SCALE, MIN_NAVIGATION_FIELD_RADIUS, MIN_SEMANTIC_QUALITY, MIN_SEPARATION_RADIUS,
    MIN_SEPARATION_WEIGHT, MIN_SPEED_LIMIT,
};
