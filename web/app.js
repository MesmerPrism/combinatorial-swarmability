import init, { DemoEngine } from "./pkg/demo_wasm.js?v=collective-behavior-v1";

const DEFAULT_SEED = "2026";
const SPEED_DELTA = 0.10;
const RELATION_RADIUS_SQUARED = 0.34 * 0.34;
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");

const canvas = document.querySelector("#swarm-canvas");
const context = canvas.getContext("2d", { alpha: false });
const status = document.querySelector("#action-status");
const controls = document.querySelector("#controls");
const memberControls = document.querySelector("#member-controls");
const seedInput = document.querySelector("#seed-input");

let engine;
let state;
let rows = new Float32Array();
let animationHandle = 0;
let previousTimestamp = 0;
let reducedTimestamp = 0;
let rowWidth = 0;

await init({
  module_or_path: new URL("./pkg/demo_wasm_bg.wasm?v=collective-behavior-v1", import.meta.url),
});
engine = new DemoEngine(DEFAULT_SEED);
rowWidth = DemoEngine.frame_row_width();
createMemberControls(DemoEngine.member_count());
bindControls();
await loadSyntheticCatalog();
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
    primary.addEventListener("click", () => {
      dispatch({ type: "select_member", member_id: memberId });
    });

    const groupLabel = document.createElement("label");
    const group = document.createElement("input");
    group.type = "checkbox";
    group.dataset.groupMemberId = String(memberId);
    group.setAttribute("aria-label", `Include member ${memberId + 1} in subgroup`);
    group.addEventListener("change", () => {
      dispatch({ type: "toggle_subgroup_member", member_id: memberId });
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
  document.querySelector("#start-button").addEventListener("click", () => {
    dispatch({ type: "start" });
  });
  document.querySelector("#pause-button").addEventListener("click", () => {
    dispatch({ type: "pause" });
  });
  document.querySelector("#step-button").addEventListener("click", () => {
    dispatch({ type: "step" });
  });
  document.querySelector("#reset-button").addEventListener("click", () => {
    dispatch({ type: "reset" });
  });
  document.querySelector("#slower-button").addEventListener("click", () => {
    adjustSpeed(-SPEED_DELTA);
  });
  document.querySelector("#faster-button").addEventListener("click", () => {
    adjustSpeed(SPEED_DELTA);
  });
  document.querySelector("#flock-button").addEventListener("click", () => {
    setBehavior("flock");
  });
  document.querySelector("#cohere-button").addEventListener("click", () => {
    setBehavior("cohere");
  });
  document.querySelector("#disperse-button").addEventListener("click", () => {
    setBehavior("disperse");
  });
  document.querySelector("#clear-subgroup-button").addEventListener("click", () => {
    dispatch({ type: "clear_subgroup" });
  });
  document.querySelector("#restart-button").addEventListener("click", restartSeed);

  controls.querySelectorAll('input[name="scope"]').forEach((input) => {
    input.addEventListener("change", () => {
      if (input.checked) {
        dispatch({ type: "set_scope", scope: input.value });
      }
    });
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
      adjustSpeed(SPEED_DELTA);
    } else if (event.key === "-" || event.key === "_") {
      event.preventDefault();
      adjustSpeed(-SPEED_DELTA);
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

function adjustSpeed(delta) {
  dispatch({
    type: "adjust_speed",
    delta,
    expected_selection_revision: state.selection_revision,
  });
}

function setBehavior(behavior) {
  dispatch({
    type: "set_behavior",
    behavior,
    expected_selection_revision: state.selection_revision,
  });
}

function restartSeed() {
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
    announce(`Restarted with seed ${seed}. Motion is paused.`);
  } catch {
    announce("Seed must be between 0 and 18446744073709551615.", true);
    seedInput.focus();
  }
}

function dispatch(action) {
  try {
    const receipt = JSON.parse(engine.dispatch_json(JSON.stringify(action)));
    refreshAll();
    announce(receipt.summary, !receipt.accepted);
    syncAnimation();
  } catch {
    announce("The action could not be applied safely.", true);
  }
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
  document.querySelector("#state-motion").textContent = state.running ? "Running" : "Paused";
  document.querySelector("#state-scope").textContent = scopeLabel(state.scope);
  document.querySelector("#state-targets").textContent = memberList(state.target_members);
  document.querySelector("#state-subgroup").textContent = memberList(state.subgroup_members);
  document.querySelector("#state-tick").textContent = String(state.tick);
  document.querySelector("#state-seed").textContent = state.seed;
  document.querySelector("#state-speed").textContent = state.average_speed.toFixed(3);
  document.querySelector("#state-behaviors").textContent = behaviorMixLabel();
  document.querySelector("#state-relations").textContent = String(relationCount());
  document.querySelector("#step-button").disabled = state.running;
  document.querySelector("#start-button").disabled = state.running;
  document.querySelector("#pause-button").disabled = !state.running;
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
  dispatch(
    event.shiftKey
      ? { type: "toggle_subgroup_member", member_id: memberId }
      : { type: "select_member", member_id: memberId }
  );
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

async function loadSyntheticCatalog() {
  try {
    const response = await fetch("./data/catalog.synthetic.json", { cache: "no-store" });
    if (!response.ok) {
      return;
    }
    const catalog = await response.json();
    const item = catalog.items?.[0];
    if (catalog.export_status === "synthetic_fixture" && item?.summary) {
      document.querySelector("#catalog-copy").textContent = `${item.summary} ${item.limitation}`;
    }
  } catch {
    // Static placeholder copy remains visible; no network or behavioral report is emitted.
  }
}
