//! Thin bounded WebAssembly adapter over `demo-core`.

use combinatorial_swarmability_demo_core::{
    DemoCore, SemanticAction, FRAME_ROW_WIDTH, MEMBER_COUNT,
};
use wasm_bindgen::prelude::*;

/// Browser-facing owner of one deterministic demo instance.
#[wasm_bindgen]
pub struct DemoEngine {
    core: DemoCore,
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
