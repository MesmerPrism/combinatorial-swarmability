//! Deterministic simulation and semantic-action core for the browser demo.

mod action;
mod engine;
mod replay;
mod rng;

pub use action::{
    ActionCode, ActionReceipt, BehaviorCounts, CollectiveBehavior, DynamicsControlMode,
    DynamicsRates, FieldLifetime, FieldPolarity, FieldSummary, MemberSummary, PublicState,
    ResolvedDynamics, SemanticAction, SemanticQualities, TargetScope,
};
pub use engine::{
    DemoCore, DemoError, DEFAULT_DYNAMICS_RATES, DEFAULT_SEMANTIC_QUALITIES, FRAME_ROW_WIDTH,
    MAX_DYNAMICS_RATE, MAX_FIELD_LIFETIME_STEPS, MAX_PERSONAL_FIELDS, MAX_SEMANTIC_QUALITY,
    MAX_SYNTHETIC_CONTRIBUTORS, MEMBER_COUNT, MIN_DYNAMICS_RATE, MIN_SEMANTIC_QUALITY,
};
