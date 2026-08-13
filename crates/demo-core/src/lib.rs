//! Deterministic simulation and semantic-action core for the browser demo.

mod action;
mod engine;
mod rng;

pub use action::{
    ActionCode, ActionReceipt, BehaviorCounts, CollectiveBehavior, MemberSummary, PublicState,
    SemanticAction, TargetScope,
};
pub use engine::{DemoCore, DemoError, FRAME_ROW_WIDTH, MEMBER_COUNT};
