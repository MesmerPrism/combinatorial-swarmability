//! Deterministic simulation and semantic-action core for the browser demo.

mod action;
mod engine;
mod replay;
mod rng;

pub use action::{
    ActionCode, ActionReceipt, BehaviorCounts, CollectiveBehavior, FieldLifetime, FieldPolarity,
    FieldSummary, MemberSummary, PublicState, SemanticAction, TargetScope,
};
pub use engine::{
    DemoCore, DemoError, FRAME_ROW_WIDTH, MAX_FIELD_LIFETIME_STEPS, MAX_PERSONAL_FIELDS,
    MAX_SYNTHETIC_CONTRIBUTORS, MEMBER_COUNT,
};
