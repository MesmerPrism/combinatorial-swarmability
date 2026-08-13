import init, { DemoEngine } from "./pkg/demo_wasm.js?v=history-v1";

const DEFAULT_SEED = "2026";
const SPEED_DELTA = 0.10;
const RELATION_RADIUS_SQUARED = 0.34 * 0.34;
const MAX_CHECKPOINTS = 5;
const MAX_SESSION_HISTORY = 50;
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const canvas = document.querySelector("#swarm-canvas");
const context = canvas.getContext("2d", { alpha: false });
const status = document.querySelector("#action-status");
const controls = document.querySelector("#controls");
const memberControls = document.querySelector("#member-controls");
const seedInput = document.querySelector("#seed-input");
const checkpointNameInput = document.querySelector("#checkpoint-name");
const checkpointSelect = document.querySelector("#checkpoint-select");
const historyEvents = document.querySelector("#history-events");
const atlasFilters = document.querySelector("#atlas-filters");
const atlasList = document.querySelector("#atlas-list");
const atlasCount = document.querySelector("#atlas-count");

let engine;
let state;
let rows = new Float32Array();
let catalogEntries = [];
const savedCheckpoints = new Map();
const sessionHistory = [];
let selectedAtlasId = "";
let sessionHistorySequence = 0;
let animationHandle = 0;
let previousTimestamp = 0;
let reducedTimestamp = 0;
let rowWidth = 0;

await init({
  module_or_path: new URL("./pkg/demo_wasm_bg.wasm?v=history-v1", import.meta.url),
});
engine = new DemoEngine(DEFAULT_SEED);
rowWidth = DemoEngine.frame_row_width();
createMemberControls(DemoEngine.member_count());
bindControls();
await loadPublicCatalog();
refreshAll();

function createMemberControls(count) {
  const fragment = document.createDocumentFragment();
  for (let memberId = 0; memberId < count; memberId += 1) {
    const wrapper = document.createElement("div");
    wrapper.className = "member-control";

    const primary = document.createElement("button");
    primary.type = "button";
    primary.dataset.memberId = String(memberId);
    primary.textContent = String(memberId + 1);
    primary.setAttribute("aria-label", `Select member ${memberId + 1} as primary`);
    primary.setAttribute("aria-pressed", "false");
    primary.addEventListener("click", (event) => {
      dispatch(
        { type: "select_member", member_id: memberId },
        interactionTrace(event, `member-control.select(${memberId + 1})`)
      );
    });

    const groupLabel = document.createElement("label");
    const group = document.createElement("input");
    group.type = "checkbox";
    group.dataset.groupMemberId = String(memberId);
    group.setAttribute("aria-label", `Include member ${memberId + 1} in subgroup`);
    group.addEventListener("click", (event) => {
      dispatch(
        { type: "toggle_subgroup_member", member_id: memberId },
        interactionTrace(event, `subgroup.toggle(${memberId + 1})`)
      );
    });
    groupLabel.append(group);
    const behavior = document.createElement("abbr");
    behavior.dataset.behaviorMemberId = String(memberId);
    behavior.textContent = "F";
    behavior.title = "Flock";
    wrapper.append(primary, behavior, groupLabel);
    fragment.append(wrapper);
  }
  memberControls.append(fragment);
}

function bindControls() {
  document.querySelector("#start-button").addEventListener("click", (event) => {
    dispatch(
      { type: "start" },
      interactionTrace(event, "motion.start", "Run-state control; target scope is unchanged")
    );
  });
  document.querySelector("#pause-button").addEventListener("click", (event) => {
    dispatch(
      { type: "pause" },
      interactionTrace(event, "motion.pause", "Run-state control; target scope is unchanged")
    );
  });
  document.querySelector("#step-button").addEventListener("click", (event) => {
    dispatch(
      { type: "step" },
      interactionTrace(event, "motion.step", "One deterministic fixed step; target scope is unchanged")
    );
  });
  document.querySelector("#reset-button").addEventListener("click", (event) => {
    dispatch(
      { type: "reset" },
      interactionTrace(event, "history.reset", "Whole-scene reset to the active seed")
    );
  });
  document.querySelector("#slower-button").addEventListener("click", (event) => {
    adjustSpeed(-SPEED_DELTA, interactionTrace(event, "speed.delta(-0.10)"));
  });
  document.querySelector("#faster-button").addEventListener("click", (event) => {
    adjustSpeed(SPEED_DELTA, interactionTrace(event, "speed.delta(+0.10)"));
  });
  document.querySelector("#flock-button").addEventListener("click", (event) => {
    setBehavior("flock", interactionTrace(event, "collective-rule.flock"));
  });
  document.querySelector("#cohere-button").addEventListener("click", (event) => {
    setBehavior("cohere", interactionTrace(event, "collective-rule.cohere"));
  });
  document.querySelector("#disperse-button").addEventListener("click", (event) => {
    setBehavior("disperse", interactionTrace(event, "collective-rule.disperse"));
  });
  document.querySelector("#clear-subgroup-button").addEventListener("click", (event) => {
    dispatch(
      { type: "clear_subgroup" },
      interactionTrace(event, "subgroup.clear", "Subgroup definition changes; no swarm member behavior changes")
    );
  });
  document.querySelector("#restart-button").addEventListener("click", restartSeed);
  document.querySelector("#save-checkpoint-button").addEventListener("click", saveCheckpoint);
  document.querySelector("#retrieve-checkpoint-button").addEventListener("click", retrieveCheckpoint);
  document.querySelector("#replay-button").addEventListener("click", replayCurrentRun);
  checkpointSelect.addEventListener("change", () => {
    const checkpoint = savedCheckpoints.get(checkpointSelect.value);
    if (checkpoint) {
      checkpointNameInput.value = checkpoint.name;
    }
    updateHistoryControls();
  });

  controls.querySelectorAll('input[name="scope"]').forEach((input) => {
    input.addEventListener("click", (event) => {
      if (input.checked) {
        dispatch(
          { type: "set_scope", scope: input.value },
          interactionTrace(event, `scope.select(${input.value})`, "Target policy changes; swarm dynamics remain unchanged")
        );
      }
    });
  });

  atlasFilters.querySelectorAll("select[data-facet]").forEach((select) => {
    select.addEventListener("change", renderAtlasList);
  });
  document.querySelector("#clear-filters").addEventListener("click", () => {
    atlasFilters.reset();
    renderAtlasList();
    atlasFilters.querySelector("select").focus();
  });

  canvas.addEventListener("pointerup", selectNearestCanvasMember);
  document.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) {
      return;
    }
    const tagName = event.target instanceof Element ? event.target.tagName : "";
    if (tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT") {
      return;
    }
    if (event.key === "+" || event.key === "=") {
      event.preventDefault();
      adjustSpeed(SPEED_DELTA, {
        inputRoute: "keyboard-or-switch",
        normalizedInput: "speed.delta(+0.10)",
      });
    } else if (event.key === "-" || event.key === "_") {
      event.preventDefault();
      adjustSpeed(-SPEED_DELTA, {
        inputRoute: "keyboard-or-switch",
        normalizedInput: "speed.delta(-0.10)",
      });
    }
  });

  reducedMotion.addEventListener("change", () => {
    updateMotionMode();
    announce(
      reducedMotion.matches
        ? "Reduced-motion preference detected. Running updates are limited to four per second."
        : "Standard motion preference detected."
    );
  });
}

function interactionTrace(event, normalizedInput, policy = "") {
  return {
    inputRoute: event instanceof PointerEvent && event.detail > 0
      ? `${event.pointerType || "pointer"}`
      : event instanceof MouseEvent && event.detail > 0
        ? "pointer"
        : "keyboard-or-switch",
    normalizedInput,
    policy,
  };
}

function adjustSpeed(delta, trace) {
  dispatch({
    type: "adjust_speed",
    delta,
    expected_selection_revision: state.selection_revision,
  }, trace);
}

function setBehavior(behavior, trace) {
  dispatch({
    type: "set_behavior",
    behavior,
    expected_selection_revision: state.selection_revision,
  }, trace);
}

function saveCheckpoint(event) {
  const name = checkpointNameInput.value.trim();
  const trace = interactionTrace(
    event,
    "history.save",
    "Session-local memory only; saving pauses motion and changes no target authority"
  );
  if (!validCheckpointName(name)) {
    updateInfrastructureTrace(trace, { type: "save_state" }, "checkpoint_name_invalid", false, "name rejected");
    announce("Checkpoint name must contain 1–32 visible characters.", true);
    checkpointNameInput.focus();
    return;
  }
  if (!savedCheckpoints.has(name) && savedCheckpoints.size >= MAX_CHECKPOINTS) {
    updateInfrastructureTrace(trace, { type: "save_state", name }, "checkpoint_limit_reached", false, "5 of 5 saved");
    announce("Five checkpoints are already saved. Reuse an existing name to overwrite it.", true);
    checkpointNameInput.focus();
    return;
  }
  if (!pauseForHistory(event, "history.save")) {
    return;
  }
  try {
    const checkpoint = {
      name,
      replay: engine.replay_json(),
      seed: state.seed,
      tick: state.tick,
      eventCount: state.replay_event_count,
      stepCount: state.replay_step_count,
    };
    const overwritten = savedCheckpoints.has(name);
    savedCheckpoints.set(name, checkpoint);
    renderCheckpointOptions(name);
    updateInfrastructureTrace(
      trace,
      { type: "save_state", name },
      overwritten ? "checkpoint_overwritten" : "checkpoint_saved",
      true,
      `${checkpoint.eventCount} events, ${checkpoint.stepCount} steps, tick ${checkpoint.tick}`
    );
    announce(`${overwritten ? "Updated" : "Saved"} ${name} at tick ${checkpoint.tick}.`);
  } catch {
    updateInfrastructureTrace(trace, { type: "save_state", name }, "replay_unavailable", false, "tape export rejected");
    announce("This run can no longer be saved as a bounded replay.", true);
  }
}

function retrieveCheckpoint(event) {
  const checkpoint = savedCheckpoints.get(checkpointSelect.value);
  const trace = interactionTrace(
    event,
    "history.retrieve",
    "Replace the active deterministic core from one session-local saved tape"
  );
  if (!checkpoint) {
    updateInfrastructureTrace(trace, { type: "retrieve_state" }, "checkpoint_missing", false, "no saved selection");
    announce("Choose a saved checkpoint before retrieving it.", true);
    checkpointSelect.focus();
    return;
  }
  if (!pauseForHistory(event, "history.retrieve")) {
    return;
  }
  try {
    engine.load_replay_json(checkpoint.replay);
    previousTimestamp = 0;
    reducedTimestamp = 0;
    refreshAll();
    updateInfrastructureTrace(
      trace,
      { type: "retrieve_state", name: checkpoint.name },
      "checkpoint_retrieved",
      true,
      `${state.replay_event_count} events, tick ${state.tick}`
    );
    announce(`Retrieved ${checkpoint.name} at tick ${state.tick}.`);
  } catch {
    updateInfrastructureTrace(trace, { type: "retrieve_state", name: checkpoint.name }, "checkpoint_damaged", false, "tape rejected");
    announce("The saved checkpoint failed strict replay validation.", true);
  }
}

function replayCurrentRun(event) {
  const trace = interactionTrace(
    event,
    "history.replay",
    "Reconstruct the active state from its initial seed, semantic actions, and fixed-step counts"
  );
  if (!pauseForHistory(event, "history.replay")) {
    return;
  }
  try {
    const replay = engine.replay_json();
    engine.load_replay_json(replay);
    previousTimestamp = 0;
    reducedTimestamp = 0;
    refreshAll();
    updateInfrastructureTrace(
      trace,
      { type: "replay_actions" },
      "replay_completed",
      true,
      `${state.replay_event_count} events, ${state.replay_step_count} steps, tick ${state.tick}`
    );
    announce(`Replayed ${state.replay_event_count} events to tick ${state.tick}.`);
  } catch {
    updateInfrastructureTrace(trace, { type: "replay_actions" }, "replay_rejected", false, "tape rejected");
    announce("The current run failed strict replay validation.", true);
  }
}

function pauseForHistory(event, operation) {
  if (!state.running) {
    return true;
  }
  const receipt = dispatch(
    { type: "pause" },
    interactionTrace(event, `${operation}.pause`, "Pause before capturing or replacing deterministic state")
  );
  return receipt?.accepted === true;
}

function validCheckpointName(name) {
  return name.length >= 1 && name.length <= 32 && !/[\u0000-\u001f\u007f]/.test(name);
}

function restartSeed(event) {
  const seed = seedInput.value.trim();
  if (!/^\d{1,20}$/.test(seed)) {
    announce("Seed must contain between 1 and 20 decimal digits.", true);
    seedInput.focus();
    return;
  }
  try {
    engine.free();
    engine = new DemoEngine(seed);
    previousTimestamp = 0;
    reducedTimestamp = 0;
    refreshAll();
    updateInfrastructureTrace(
      interactionTrace(event, `history.restart-seed(${seed})`, "Whole-scene restart with deterministic seed"),
      { type: "restart_seed" },
      "seed_restarted",
      true,
      `state ${state.state_revision}, selection ${state.selection_revision}`
    );
    announce(`Restarted with seed ${seed}. Motion is paused.`);
  } catch {
    announce("Seed must be between 0 and 18446744073709551615.", true);
    seedInput.focus();
  }
}

function dispatch(action, trace = {}) {
  try {
    const receipt = JSON.parse(engine.dispatch_json(JSON.stringify(action)));
    refreshAll();
    updateActionTrace(trace, action, receipt);
    announce(receipt.summary, !receipt.accepted);
    syncAnimation();
    return receipt;
  } catch {
    announce("The action could not be applied safely.", true);
    return null;
  }
}

function updateActionTrace(trace, action, receipt) {
  const policy = trace.policy || `${scopeLabel(state.scope)} → ${memberList(state.target_members)}`;
  const accepted = receipt.accepted ? "accepted" : "rejected";
  const revision = `state ${receipt.state_revision}, selection ${receipt.selection_revision}`;
  renderActionTrace(
    { ...trace, policy },
    semanticActionLabel(action),
    `${receipt.code} · ${accepted} · ${revision}`
  );
}

function updateInfrastructureTrace(trace, action, code, accepted, detail) {
  renderActionTrace(
    trace,
    semanticActionLabel(action),
    `${code} · ${accepted ? "accepted" : "rejected"} · ${detail}`
  );
}

function renderActionTrace(trace, semanticAction, receipt) {
  document.querySelector("#trace-input-route").textContent = trace.inputRoute || "browser control";
  document.querySelector("#trace-normalized-input").textContent = trace.normalizedInput || semanticAction;
  document.querySelector("#trace-semantic-action").textContent = semanticAction;
  document.querySelector("#trace-policy").textContent = trace.policy || "No target policy change";
  document.querySelector("#trace-receipt").textContent = receipt;
  appendSessionHistory(trace, semanticAction, receipt);
}

function appendSessionHistory(trace, semanticAction, receipt) {
  sessionHistorySequence += 1;
  sessionHistory.push({
    sequence: sessionHistorySequence,
    semanticAction,
    route: trace.inputRoute || "browser control",
    normalizedInput: trace.normalizedInput || semanticAction,
    policy: trace.policy || "No target policy change",
    receipt,
  });
  if (sessionHistory.length > MAX_SESSION_HISTORY) {
    sessionHistory.shift();
  }
  renderSessionHistory();
}

function renderSessionHistory() {
  const fragment = document.createDocumentFragment();
  sessionHistory.forEach((entry) => {
    const item = document.createElement("li");
    const action = document.createElement("strong");
    const provenance = document.createElement("span");
    const policy = document.createElement("span");
    action.textContent = `${entry.sequence}. ${entry.semanticAction}`;
    provenance.textContent = `${entry.route} · ${entry.normalizedInput} · ${entry.receipt}`;
    policy.textContent = entry.policy;
    item.append(action, provenance, policy);
    fragment.append(item);
  });
  historyEvents.replaceChildren(fragment);
}

function semanticActionLabel(action) {
  const parameters = Object.entries(action)
    .filter(([key]) => key !== "type" && key !== "expected_selection_revision")
    .map(([key, value]) => `${key}=${value}`);
  return parameters.length > 0 ? `${action.type}(${parameters.join(", ")})` : action.type;
}

function refreshAll() {
  state = JSON.parse(engine.state_json());
  rows = engine.frame_rows();
  updateDomState();
  updateMemberControls();
  draw();
  syncAnimation();
}

function syncAnimation() {
  if (state.running && animationHandle === 0) {
    previousTimestamp = 0;
    animationHandle = requestAnimationFrame(animate);
  } else if (!state.running && animationHandle !== 0) {
    cancelAnimationFrame(animationHandle);
    animationHandle = 0;
  }
}

function animate(timestamp) {
  animationHandle = 0;
  if (!state.running) {
    return;
  }
  if (previousTimestamp === 0) {
    previousTimestamp = timestamp;
  }
  const elapsed = Math.min(timestamp - previousTimestamp, 100);
  previousTimestamp = timestamp;

  if (reducedMotion.matches) {
    if (timestamp - reducedTimestamp >= 250) {
      engine.advance(16);
      reducedTimestamp = timestamp;
      state = JSON.parse(engine.state_json());
      rows = engine.frame_rows();
      updateDomState(false);
      draw();
    }
  } else {
    engine.advance(elapsed);
    state = JSON.parse(engine.state_json());
    rows = engine.frame_rows();
    updateDomState(false);
    draw();
  }
  animationHandle = requestAnimationFrame(animate);
}

function updateDomState(updateControls = true) {
  const relationTotal = relationCount();
  const metrics = outcomeMetrics();
  document.querySelector("#state-motion").textContent = state.running ? "Running" : "Paused";
  document.querySelector("#state-scope").textContent = scopeLabel(state.scope);
  document.querySelector("#state-targets").textContent = memberList(state.target_members);
  document.querySelector("#state-subgroup").textContent = memberList(state.subgroup_members);
  document.querySelector("#state-tick").textContent = String(state.tick);
  document.querySelector("#state-seed").textContent = state.seed;
  document.querySelector("#state-speed").textContent = state.average_speed.toFixed(3);
  document.querySelector("#state-behaviors").textContent = behaviorMixLabel();
  document.querySelector("#state-relations").textContent = String(relationTotal);
  document.querySelector("#state-replay-events").textContent =
    `${state.replay_event_count} events / ${state.replay_step_count} steps`;
  document.querySelector("#state-checkpoint-count").textContent = `${savedCheckpoints.size} of ${MAX_CHECKPOINTS}`;
  document.querySelector("#metric-cohesion").textContent = metrics.cohesion.toFixed(3);
  document.querySelector("#metric-polarization").textContent = metrics.polarization.toFixed(3);
  document.querySelector("#metric-spacing").textContent = metrics.nearestSpacing.toFixed(3);
  document.querySelector("#metric-speed").textContent = state.average_speed.toFixed(3);
  document.querySelector("#metric-subgroup").textContent = String(state.subgroup_members.length);
  document.querySelector("#metric-relations").textContent = String(relationTotal);
  document.querySelector("#step-button").disabled = state.running;
  document.querySelector("#start-button").disabled = state.running;
  document.querySelector("#pause-button").disabled = !state.running;
  updateHistoryControls();
  canvas.setAttribute(
    "aria-label",
    `${state.running ? "Running" : "Paused"} synthetic swarm. ${scopeLabel(state.scope)} targets ${memberList(state.target_members)}. ${behaviorMixLabel()}.`
  );
  updateMotionMode();

  if (updateControls) {
    const activeScope = controls.querySelector(`input[name="scope"][value="${state.scope}"]`);
    if (activeScope) {
      activeScope.checked = true;
    }
    seedInput.value = state.seed;
  }
}

function renderCheckpointOptions(selectedName = checkpointSelect.value) {
  const fragment = document.createDocumentFragment();
  const empty = document.createElement("option");
  empty.value = "";
  empty.textContent = savedCheckpoints.size === 0 ? "No saved checkpoints" : "Choose a checkpoint";
  fragment.append(empty);
  savedCheckpoints.forEach((checkpoint) => {
    const option = document.createElement("option");
    option.value = checkpoint.name;
    option.textContent = `${checkpoint.name} · seed ${checkpoint.seed} · tick ${checkpoint.tick}`;
    fragment.append(option);
  });
  checkpointSelect.replaceChildren(fragment);
  checkpointSelect.value = savedCheckpoints.has(selectedName) ? selectedName : "";
  updateHistoryControls();
}

function updateHistoryControls() {
  document.querySelector("#retrieve-checkpoint-button").disabled = !savedCheckpoints.has(checkpointSelect.value);
  document.querySelector("#replay-button").disabled =
    !state?.replay_available || state.replay_event_count === 0;
  document.querySelector("#state-checkpoint-count").textContent = `${savedCheckpoints.size} of ${MAX_CHECKPOINTS}`;
}

function outcomeMetrics() {
  const projectedRows = [];
  forEachRow((row) => projectedRows.push(row));
  if (projectedRows.length < 2) {
    return { cohesion: 1, polarization: 1, nearestSpacing: 0 };
  }

  let pairDistanceTotal = 0;
  let pairCount = 0;
  let nearestDistanceTotal = 0;
  let headingX = 0;
  let headingY = 0;
  for (let firstIndex = 0; firstIndex < projectedRows.length; firstIndex += 1) {
    const first = projectedRows[firstIndex];
    let nearestDistance = Number.POSITIVE_INFINITY;
    const speed = Math.hypot(first[4], first[5]);
    if (speed > Number.EPSILON) {
      headingX += first[4] / speed;
      headingY += first[5] / speed;
    }
    for (let secondIndex = firstIndex + 1; secondIndex < projectedRows.length; secondIndex += 1) {
      const second = projectedRows[secondIndex];
      const distance = Math.hypot(first[1] - second[1], first[2] - second[2]);
      pairDistanceTotal += distance;
      pairCount += 1;
      nearestDistance = Math.min(nearestDistance, distance);
    }
    for (let secondIndex = 0; secondIndex < firstIndex; secondIndex += 1) {
      const second = projectedRows[secondIndex];
      nearestDistance = Math.min(nearestDistance, Math.hypot(first[1] - second[1], first[2] - second[2]));
    }
    nearestDistanceTotal += nearestDistance;
  }

  const meanPairDistance = pairDistanceTotal / pairCount;
  return {
    cohesion: 1 - Math.min(meanPairDistance / Math.sqrt(8), 1),
    polarization: Math.hypot(headingX, headingY) / projectedRows.length,
    nearestSpacing: nearestDistanceTotal / projectedRows.length,
  };
}

function updateMotionMode() {
  document.querySelector("#state-motion-mode").textContent = reducedMotion.matches
    ? "Reduced, 4 updates/s"
    : "Standard";
}

function updateMemberControls() {
  const subgroup = new Set(state.subgroup_members);
  const members = new Map(state.members.map((member) => [member.member_id, member]));
  memberControls.querySelectorAll("button[data-member-id]").forEach((button) => {
    const memberId = Number(button.dataset.memberId);
    button.setAttribute("aria-pressed", String(state.primary_member === memberId));
  });
  memberControls.querySelectorAll("input[data-group-member-id]").forEach((input) => {
    input.checked = subgroup.has(Number(input.dataset.groupMemberId));
  });
  memberControls.querySelectorAll("abbr[data-behavior-member-id]").forEach((label) => {
    const member = members.get(Number(label.dataset.behaviorMemberId));
    const behavior = behaviorLabel(member?.behavior);
    label.textContent = behavior.slice(0, 1);
    label.title = behavior;
  });
}

function selectNearestCanvasMember(event) {
  const bounds = canvas.getBoundingClientRect();
  const point = {
    x: ((event.clientX - bounds.left) / bounds.width) * 2 - 1,
    y: -(((event.clientY - bounds.top) / bounds.height) * 2 - 1),
  };
  let nearest = null;
  let nearestDistance = 0.18 * 0.18;
  forEachRow((row) => {
    const dx = row[1] - point.x;
    const dy = row[2] - point.y;
    const distance = dx * dx + dy * dy;
    if (distance < nearestDistance) {
      nearestDistance = distance;
      nearest = row[0];
    }
  });
  if (nearest === null) {
    announce("No member was close enough to that point.");
    return;
  }
  const memberId = Math.round(nearest);
  const action = event.shiftKey
    ? { type: "toggle_subgroup_member", member_id: memberId }
    : { type: "select_member", member_id: memberId };
  dispatch(action, {
    inputRoute: event.pointerType || "pointer",
    normalizedInput: event.shiftKey
      ? `canvas.subgroup-toggle(${memberId + 1})`
      : `canvas.select(${memberId + 1})`,
    policy: event.shiftKey
      ? "Subgroup definition changes; no swarm member behavior changes"
      : "Primary member changes; active scope policy determines later targets",
  });
}

function draw() {
  const width = canvas.width;
  const height = canvas.height;
  context.fillStyle = "#fbf8f2";
  context.fillRect(0, 0, width, height);
  drawFieldLines(width, height);
  const projectedRows = [];
  forEachRow((row) => projectedRows.push(row));
  const wholeSwarmTargeted = projectedRows.every((row) => row[9] > 0.5);
  if (wholeSwarmTargeted) {
    context.save();
    context.strokeStyle = "#463b69";
    context.lineWidth = 4;
    context.setLineDash([10, 8]);
    context.strokeRect(10, 10, width - 20, height - 20);
    context.restore();
  }
  drawRelations(projectedRows, width, height);
  projectedRows.forEach((row) => drawMember(row, width, height, !wholeSwarmTargeted));
}

function drawRelations(projectedRows, width, height) {
  relationEdges(projectedRows).forEach(([firstIndex, secondIndex]) => {
    const first = projectedRows[firstIndex];
    const second = projectedRows[secondIndex];
    const sharedBehavior = Math.round(first[10]) === Math.round(second[10])
      ? Math.round(first[10])
      : 0;
    context.save();
    context.strokeStyle = ["#cfc4b7", "#718069", "#a95f4d"][sharedBehavior];
    context.lineWidth = sharedBehavior === 0 ? 1.25 : 2;
    context.setLineDash(sharedBehavior === 2 ? [8, 6] : sharedBehavior === 0 ? [2, 7] : []);
    context.beginPath();
    context.moveTo(((first[1] + 1) / 2) * width, ((1 - first[2]) / 2) * height);
    context.lineTo(((second[1] + 1) / 2) * width, ((1 - second[2]) / 2) * height);
    context.stroke();
    context.restore();
  });
}

function drawFieldLines(width, height) {
  context.save();
  context.strokeStyle = "#ded5ca";
  context.lineWidth = 1;
  context.setLineDash([4, 10]);
  for (let index = 1; index < 6; index += 1) {
    const x = (width / 6) * index;
    const y = (height / 6) * index;
    context.beginPath();
    context.moveTo(x, 0);
    context.lineTo(x, height);
    context.stroke();
    context.beginPath();
    context.moveTo(0, y);
    context.lineTo(width, y);
    context.stroke();
  }
  context.restore();
}

function drawMember(row, width, height, showIndividualTarget) {
  const memberId = Math.round(row[0]);
  const x = ((row[1] + 1) / 2) * width;
  const y = ((1 - row[2]) / 2) * height;
  const radius = Math.max(7, row[3] * width * 0.62);
  const primary = row[7] > 0.5;
  const subgroup = row[8] > 0.5;
  const targeted = row[9] > 0.5;
  const behavior = Math.round(row[10]);

  context.save();
  context.translate(x, y);

  if (targeted && showIndividualTarget) {
    context.strokeStyle = "#463b69";
    context.lineWidth = 3;
    context.setLineDash([5, 4]);
    context.strokeRect(-radius - 8, -radius - 8, radius * 2 + 16, radius * 2 + 16);
  }
  if (subgroup) {
    context.strokeStyle = "#52654d";
    context.lineWidth = 3;
    context.setLineDash([]);
    context.strokeRect(-radius - 4, -radius - 4, radius * 2 + 8, radius * 2 + 8);
  }

  context.fillStyle = primary ? "#8a4638" : ["#d9a86c", "#9aae8f", "#d59b87"][behavior];
  context.strokeStyle = "#1e1713";
  context.lineWidth = primary ? 4 : 2;
  drawBehaviorShape(radius, behavior);
  context.fill();
  context.stroke();

  if (primary) {
    context.strokeStyle = "#fbf8f2";
    context.lineWidth = 2;
    context.beginPath();
    context.arc(0, 0, Math.max(2, radius - 5), 0, Math.PI * 2);
    context.stroke();
  }

  context.fillStyle = primary ? "#fbf8f2" : "#1e1713";
  context.font = "600 16px Aptos, Candara, sans-serif";
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(String(memberId + 1), 0, 1);
  context.restore();
}

function drawBehaviorShape(radius, behavior) {
  context.beginPath();
  if (behavior === 1) {
    for (let side = 0; side < 6; side += 1) {
      const angle = -Math.PI / 2 + (side * Math.PI) / 3;
      const x = Math.cos(angle) * radius;
      const y = Math.sin(angle) * radius;
      if (side === 0) {
        context.moveTo(x, y);
      } else {
        context.lineTo(x, y);
      }
    }
    context.closePath();
  } else if (behavior === 2) {
    context.moveTo(0, -radius);
    context.lineTo(radius, 0);
    context.lineTo(0, radius);
    context.lineTo(-radius, 0);
    context.closePath();
  } else {
    context.arc(0, 0, radius, 0, Math.PI * 2);
  }
}

function forEachRow(visitor) {
  for (let offset = 0; offset + rowWidth <= rows.length; offset += rowWidth) {
    visitor(rows.subarray(offset, offset + rowWidth));
  }
}

function relationCount() {
  const projectedRows = [];
  forEachRow((row) => projectedRows.push(row));
  return relationEdges(projectedRows).length;
}

function relationEdges(projectedRows) {
  const candidates = [];
  for (let firstIndex = 0; firstIndex < projectedRows.length; firstIndex += 1) {
    for (let secondIndex = firstIndex + 1; secondIndex < projectedRows.length; secondIndex += 1) {
      const dx = projectedRows[firstIndex][1] - projectedRows[secondIndex][1];
      const dy = projectedRows[firstIndex][2] - projectedRows[secondIndex][2];
      const distanceSquared = dx * dx + dy * dy;
      if (distanceSquared <= RELATION_RADIUS_SQUARED) {
        candidates.push([firstIndex, secondIndex, distanceSquared]);
      }
    }
  }
  candidates.sort((first, second) => first[2] - second[2]);
  const degrees = new Uint8Array(projectedRows.length);
  const edges = [];
  candidates.forEach(([firstIndex, secondIndex]) => {
    if (degrees[firstIndex] >= 3 || degrees[secondIndex] >= 3) {
      return;
    }
    degrees[firstIndex] += 1;
    degrees[secondIndex] += 1;
    edges.push([firstIndex, secondIndex]);
  });
  return edges;
}

function behaviorMixLabel() {
  const counts = state.behavior_counts;
  return [
    counts.flock > 0 ? `${counts.flock} flock` : "",
    counts.cohere > 0 ? `${counts.cohere} cohere` : "",
    counts.disperse > 0 ? `${counts.disperse} disperse` : "",
  ].filter(Boolean).join(", ");
}

function behaviorLabel(behavior) {
  return {
    flock: "Flock",
    cohere: "Cohere",
    disperse: "Disperse",
  }[behavior] ?? "Unknown";
}

function scopeLabel(scope) {
  return {
    member: "One member",
    subgroup: "Subgroup",
    swarm: "Whole swarm",
  }[scope] ?? "Unknown";
}

function memberList(members) {
  if (!members || members.length === 0) {
    return "None";
  }
  return members.map((member) => String(member + 1)).join(", ");
}

function announce(message, isError = false) {
  status.textContent = message;
  status.dataset.error = String(isError);
}

async function loadPublicCatalog() {
  try {
    const response = await fetch("./data/catalog.v1.json", { cache: "no-store" });
    if (!response.ok) {
      throw new Error("catalogue response was not successful");
    }
    const catalog = await response.json();
    if (catalog.schema !== "combinatorial.swarmability.public.catalog.v1" || !Array.isArray(catalog.items)) {
      throw new Error("catalogue schema was not recognized");
    }
    catalogEntries = [...catalog.items].sort((first, second) => first.display_order - second.display_order);
    populateAtlasFilters();
    selectedAtlasId = catalogEntries.find((entry) => entry.reconstruction.status === "implemented-reconstruction")?.public_id
      || catalogEntries[0]?.public_id
      || "";
    renderAtlasList();
    document.querySelector("#catalog-copy").textContent =
      `${catalogEntries.length} allowlisted public projections preserve source reports, evidence status, transfer limits, and app reconstruction claims as separate fields.`;
  } catch {
    atlasCount.textContent = "The public catalogue could not be loaded.";
    const failure = document.createElement("li");
    failure.textContent = "Catalogue unavailable. The interactive scope reconstruction remains usable below.";
    atlasList.replaceChildren(failure);
  }
}

function populateAtlasFilters() {
  atlasFilters.querySelectorAll("select[data-facet]").forEach((select) => {
    const facet = select.dataset.facet;
    const values = new Set();
    catalogEntries.forEach((entry) => {
      entry.facets[facet].forEach((value) => values.add(value));
    });
    [...values].sort().forEach((value) => {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = humanize(value);
      select.append(option);
    });
  });
}

function renderAtlasList() {
  const activeFilters = [...atlasFilters.querySelectorAll("select[data-facet]")]
    .filter((select) => select.value)
    .map((select) => ({ facet: select.dataset.facet, value: select.value }));
  const visibleEntries = catalogEntries.filter((entry) =>
    activeFilters.every(({ facet, value }) => entry.facets[facet].includes(value))
  );

  if (!visibleEntries.some((entry) => entry.public_id === selectedAtlasId)) {
    selectedAtlasId = visibleEntries[0]?.public_id || "";
  }
  atlasCount.textContent = `${visibleEntries.length} of ${catalogEntries.length} entries shown.`;

  const fragment = document.createDocumentFragment();
  visibleEntries.forEach((entry) => {
    const item = document.createElement("li");
    const button = document.createElement("button");
    const title = document.createElement("strong");
    const source = document.createElement("span");
    const statusLabel = document.createElement("span");
    button.type = "button";
    button.dataset.atlasId = entry.public_id;
    button.setAttribute("aria-pressed", String(entry.public_id === selectedAtlasId));
    title.textContent = entry.title;
    source.textContent = `${entry.source.system_or_study}, ${entry.source.year}`;
    statusLabel.textContent = humanize(entry.reconstruction.status);
    button.append(title, source, statusLabel);
    button.addEventListener("click", () => {
      selectedAtlasId = entry.public_id;
      renderAtlasList();
    });
    item.append(button);
    fragment.append(item);
  });

  if (visibleEntries.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No entries match every active filter.";
    fragment.append(empty);
  }
  atlasList.replaceChildren(fragment);
  renderAtlasDetail(catalogEntries.find((entry) => entry.public_id === selectedAtlasId));
}

function renderAtlasDetail(entry) {
  if (!entry) {
    setText("#atlas-detail-status", "No matching entry");
    setText("#atlas-detail-title", "Adjust the filters");
    setText("#atlas-detail-summary", "No public catalogue entry matches every active filter.");
    document.querySelector("#atlas-open-demo").hidden = true;
    return;
  }

  setText("#atlas-detail-status", humanize(entry.reconstruction.status));
  setText("#atlas-detail-title", entry.title);
  setText("#atlas-detail-summary", entry.reconstruction.summary);
  setText("#detail-input-expression", `${entry.reported.input_expression}. Route: ${entry.reported.input_routes}.`);
  setText("#detail-semantic-action", entry.reported.semantic_action);
  setText("#detail-controlled-quantity", `${entry.reported.controlled_quantity} (${entry.reported.parameter_exposure}).`);
  setText("#detail-scope-timing", `${entry.reported.target_scope}; ${entry.reported.temporal_mode}.`);
  setText("#detail-combination", `${entry.reported.human_configuration}; ${entry.reported.multi_user_combination}.`);
  setText("#detail-source", `${entry.source.system_or_study} (${entry.source.year}) · ${entry.source.source_id}`);
  setText(
    "#detail-evidence",
    `${humanize(entry.source.evidence_kind)} · ${humanize(entry.source.literature_status)} · ${humanize(entry.source.catalog_projection_status)} · checked ${entry.source.checked_on}`
  );
  setText("#detail-locus", entry.source.source_locus);
  setText("#detail-transfer", entry.reconstruction.transfer_boundary);
  setText("#detail-nonclaim", entry.reconstruction.does_not_claim);
  setText("#entry-trace-input", entry.reported.input_expression);
  setText("#entry-trace-normalized", entry.facets.input_routes.map(humanize).join(" / "));
  setText(
    "#entry-trace-action",
    `${entry.reported.semantic_action}; atlas actions: ${entry.reconstruction.semantic_actions.join(", ")}`
  );
  setText("#entry-trace-policy", `${entry.reported.target_scope}; ${entry.reported.multi_user_combination}`);
  setText("#entry-trace-effect", entry.reconstruction.effect);
  renderSourceLinks(entry.source);

  const implemented = entry.reconstruction.status === "implemented-reconstruction";
  document.querySelector("#atlas-open-demo").hidden = !implemented;
  setText(
    "#atlas-demo-state",
    implemented
      ? "Interactive now: use the reconstruction below and inspect its normalized input, semantic action, policy resolution, core receipt, and outcome metrics."
      : "Planned reconstruction: the public source and evidence card is available, but this mechanism is not enabled in the deterministic core yet."
  );
}

function renderSourceLinks(source) {
  const container = document.querySelector("#detail-links");
  const links = [
    ["Paper", source.paper_url],
    ["Project", source.project_url],
    ["Artifact", source.artifact_url],
  ].filter(([, url]) => url);
  const fragment = document.createDocumentFragment();
  links.forEach(([label, url], index) => {
    if (index > 0) {
      fragment.append(document.createTextNode(" · "));
    }
    const link = document.createElement("a");
    link.href = url;
    link.textContent = label;
    fragment.append(link);
  });
  container.replaceChildren(fragment);
}

function setText(selector, value) {
  document.querySelector(selector).textContent = value;
}

function humanize(value) {
  return String(value)
    .replaceAll("-", " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}
