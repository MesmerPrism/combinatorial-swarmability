use serde::{Deserialize, Serialize};

use crate::SemanticAction;

pub(crate) const REPLAY_SCHEMA: &str = "combinatorial.swarmability.replay.v1";
const MAX_REPLAY_EVENTS: usize = 4_096;
const MAX_REPLAY_STEPS: u64 = 50_000;
pub(crate) const MAX_REPLAY_JSON_BYTES: usize = 2_000_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReplayTape {
    pub(crate) schema: String,
    pub(crate) initial_seed: u64,
    pub(crate) events: Vec<ReplayEvent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ReplayEvent {
    Action { action: SemanticAction },
    AdvanceSteps { steps: u64 },
}

#[derive(Clone, Debug)]
pub(crate) struct ReplayRecorder {
    initial_seed: u64,
    events: Vec<ReplayEvent>,
    total_steps: u64,
    available: bool,
}

impl ReplayRecorder {
    pub(crate) fn new(initial_seed: u64) -> Self {
        Self {
            initial_seed,
            events: Vec::new(),
            total_steps: 0,
            available: true,
        }
    }

    pub(crate) fn unavailable(initial_seed: u64) -> Self {
        Self {
            initial_seed,
            events: Vec::new(),
            total_steps: 0,
            available: false,
        }
    }

    pub(crate) fn record_action(&mut self, action: SemanticAction) {
        if !self.available {
            return;
        }
        if matches!(&action, SemanticAction::Step) {
            let Some(total_steps) = self.total_steps.checked_add(1) else {
                self.available = false;
                return;
            };
            if total_steps > MAX_REPLAY_STEPS {
                self.available = false;
                return;
            }
            self.total_steps = total_steps;
        }
        if self.events.len() >= MAX_REPLAY_EVENTS {
            self.available = false;
            return;
        }
        self.events.push(ReplayEvent::Action { action });
    }

    pub(crate) fn record_advance(&mut self, steps: u32) {
        if !self.available || steps == 0 {
            return;
        }
        let steps = u64::from(steps);
        let Some(total_steps) = self.total_steps.checked_add(steps) else {
            self.available = false;
            return;
        };
        if total_steps > MAX_REPLAY_STEPS {
            self.available = false;
            return;
        }
        self.total_steps = total_steps;
        if let Some(ReplayEvent::AdvanceSteps { steps: recorded }) = self.events.last_mut() {
            *recorded += steps;
        } else if self.events.len() < MAX_REPLAY_EVENTS {
            self.events.push(ReplayEvent::AdvanceSteps { steps });
        } else {
            self.available = false;
        }
    }

    pub(crate) const fn available(&self) -> bool {
        self.available
    }

    pub(crate) fn event_count(&self) -> usize {
        self.events.len()
    }

    pub(crate) const fn total_steps(&self) -> u64 {
        self.total_steps
    }

    pub(crate) fn tape(&self) -> Option<ReplayTape> {
        self.available.then(|| ReplayTape {
            schema: REPLAY_SCHEMA.to_owned(),
            initial_seed: self.initial_seed,
            events: self.events.clone(),
        })
    }
}

pub(crate) fn validate_replay_tape(tape: &ReplayTape) -> Result<(), &'static str> {
    if tape.schema != REPLAY_SCHEMA {
        return Err("unsupported replay schema");
    }
    if tape.events.len() > MAX_REPLAY_EVENTS {
        return Err("replay event limit exceeded");
    }
    let mut total_steps = 0_u64;
    let mut previous_was_advance = false;
    for event in &tape.events {
        let steps = match event {
            ReplayEvent::Action {
                action: SemanticAction::Step,
            } => 1,
            ReplayEvent::Action { .. } => 0,
            ReplayEvent::AdvanceSteps { steps } => {
                if *steps == 0 {
                    return Err("replay advance count must be positive");
                }
                if previous_was_advance {
                    return Err("consecutive replay advances must be canonicalized");
                }
                *steps
            }
        };
        previous_was_advance = matches!(event, ReplayEvent::AdvanceSteps { .. });
        total_steps = total_steps
            .checked_add(steps)
            .ok_or("replay step count overflow")?;
        if total_steps > MAX_REPLAY_STEPS {
            return Err("replay step limit exceeded");
        }
    }
    Ok(())
}
