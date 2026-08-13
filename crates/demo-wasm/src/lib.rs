//! Thin bounded WebAssembly adapter over `demo-core`.

use combinatorial_swarmability_demo_core::{
    ComparisonRunner, DemoCore, SemanticAction, FRAME_ROW_WIDTH, MEMBER_COUNT,
};
use wasm_bindgen::prelude::*;

/// Browser-facing owner of one deterministic demo instance.
#[wasm_bindgen]
pub struct DemoEngine {
    core: DemoCore,
}

/// Browser-facing owner of two isolated deterministic comparison lanes.
#[wasm_bindgen]
pub struct ComparisonEngine {
    runner: ComparisonRunner,
}

#[wasm_bindgen]
impl DemoEngine {
    /// Creates a paused engine from a decimal `u64` seed string.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error when the seed is invalid.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &str) -> Result<Self, JsValue> {
        let seed = parse_seed(seed)?;
        Ok(Self {
            core: DemoCore::new(seed),
        })
    }

    /// Dispatches one strict semantic-action JSON object.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error for invalid JSON or serialization.
    pub fn dispatch_json(&mut self, action_json: &str) -> Result<String, JsValue> {
        let action: SemanticAction = serde_json::from_str(action_json)
            .map_err(|_| JsValue::from_str("Invalid semantic action request."))?;
        serde_json::to_string(&self.core.dispatch(action))
            .map_err(|_| JsValue::from_str("Action receipt could not be serialized."))
    }

    /// Advances browser elapsed time through bounded fixed steps.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn advance(&mut self, elapsed_millis: f64) -> u32 {
        let bounded = if elapsed_millis.is_finite() && elapsed_millis > 0.0 {
            elapsed_millis.round().clamp(0.0, 250.0) as u32
        } else {
            0
        };
        self.core.advance_elapsed(bounded)
    }

    /// Returns one fixed-width row per member as a JavaScript `Float32Array`.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error when Matter rejects the payload.
    pub fn frame_rows(&self) -> Result<Vec<f32>, JsValue> {
        self.core
            .frame_rows()
            .map_err(|_| JsValue::from_str("Frame payload is unavailable."))
    }

    /// Returns concise semantic state JSON for DOM projection.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error when serialization fails.
    pub fn state_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.core.public_state())
            .map_err(|_| JsValue::from_str("State summary could not be serialized."))
    }

    /// Returns the strict bounded deterministic replay tape.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error when replay is unavailable.
    pub fn replay_json(&self) -> Result<String, JsValue> {
        self.core
            .replay_json()
            .map_err(|_| JsValue::from_str("Replay recording is unavailable."))
    }

    /// Replaces the active core with the state reconstructed from a replay tape.
    ///
    /// # Errors
    ///
    /// Returns a sanitized JavaScript error when the tape is invalid or damaged.
    pub fn load_replay_json(&mut self, replay_json: &str) -> Result<(), JsValue> {
        let restored = DemoCore::from_replay_json(replay_json)
            .map_err(|_| JsValue::from_str("Replay recording is invalid."))?;
        self.core = restored;
        Ok(())
    }

    /// Returns the fixed scene member count.
    #[must_use]
    pub fn member_count() -> usize {
        MEMBER_COUNT
    }

    /// Returns the number of floats in each frame row.
    #[must_use]
    pub fn frame_row_width() -> usize {
        FRAME_ROW_WIDTH
    }
}

#[wasm_bindgen]
impl ComparisonEngine {
    /// Creates a comparison runner from one strict immutable specification.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error for damaged schemas, seeds, or tape bindings.
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<Self, JsValue> {
        let runner = ComparisonRunner::from_spec_json(spec_json)
            .map_err(|_| JsValue::from_str("Invalid comparison specification."))?;
        Ok(Self { runner })
    }

    /// Applies one canonical normalized-input event to both lanes transactionally.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if a lane or lockstep invariant fails.
    pub fn step_event_json(&mut self) -> Result<String, JsValue> {
        self.runner
            .step_event_json()
            .map_err(|_| JsValue::from_str("Comparison step was rejected."))
    }

    /// Restores both isolated lanes to the canonical initial snapshot.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if canonical start equality cannot be restored.
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.runner
            .reset()
            .map_err(|_| JsValue::from_str("Comparison reset was rejected."))
    }

    /// Resets and replays the complete immutable normalized-input tape.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error if either lane or lockstep invariant fails.
    pub fn replay_all_json(&mut self) -> Result<String, JsValue> {
        self.runner
            .replay_all_json()
            .map_err(|_| JsValue::from_str("Comparison replay was rejected."))
    }

    /// Returns the current versioned comparison result and provenance.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error when the result cannot be projected.
    pub fn result_json(&self) -> Result<String, JsValue> {
        self.runner
            .result_json()
            .map_err(|_| JsValue::from_str("Comparison result is unavailable."))
    }

    /// Returns current renderer-neutral rows for the left isolated lane.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error when the lane frame cannot be projected.
    pub fn left_frame_rows(&self) -> Result<Vec<f32>, JsValue> {
        self.runner
            .left_frame_rows()
            .map_err(|_| JsValue::from_str("Left comparison frame is unavailable."))
    }

    /// Returns current renderer-neutral rows for the right isolated lane.
    ///
    /// # Errors
    ///
    /// Returns a sanitized error when the lane frame cannot be projected.
    pub fn right_frame_rows(&self) -> Result<Vec<f32>, JsValue> {
        self.runner
            .right_frame_rows()
            .map_err(|_| JsValue::from_str("Right comparison frame is unavailable."))
    }

    /// Returns the bounded event count for the selected canonical tape.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.runner.event_count()
    }
}

fn parse_seed(seed: &str) -> Result<u64, JsValue> {
    let trimmed = seed.trim();
    if trimmed.is_empty()
        || trimmed.len() > 20
        || !trimmed.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(JsValue::from_str(
            "Seed must be a decimal integer between 0 and 18446744073709551615.",
        ));
    }
    trimmed.parse::<u64>().map_err(|_| {
        JsValue::from_str("Seed must be a decimal integer between 0 and 18446744073709551615.")
    })
}
