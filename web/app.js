import init, { DemoEngine } from "./pkg/demo_wasm.js";

const DEFAULT_SEED = "2026";
const ROW_WIDTH = 10;
const SPEED_DELTA = 0.10;
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

await init();
engine = new DemoEngine(DEFAULT_SEED);
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
    wrapper.append(primary, groupLabel);
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
  document.querySelector("#step-button").disabled = state.running;
  document.querySelector("#start-button").disabled = state.running;
  document.querySelector("#pause-button").disabled = !state.running;
  canvas.setAttribute(
    "aria-label",
    `${state.running ? "Running" : "Paused"} synthetic swarm. ${scopeLabel(state.scope)} targets ${memberList(state.target_members)}.`
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
  memberControls.querySelectorAll("button[data-member-id]").forEach((button) => {
    const memberId = Number(button.dataset.memberId);
    button.setAttribute("aria-pressed", String(state.primary_member === memberId));
  });
  memberControls.querySelectorAll("input[data-group-member-id]").forEach((input) => {
    input.checked = subgroup.has(Number(input.dataset.groupMemberId));
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
  forEachRow((row) => drawMember(row, width, height));
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

function drawMember(row, width, height) {
  const memberId = Math.round(row[0]);
  const x = ((row[1] + 1) / 2) * width;
  const y = ((1 - row[2]) / 2) * height;
  const radius = Math.max(7, row[3] * width * 0.62);
  const primary = row[7] > 0.5;
  const subgroup = row[8] > 0.5;
  const targeted = row[9] > 0.5;

  context.save();
  context.translate(x, y);

  if (targeted) {
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

  context.fillStyle = primary ? "#8a4638" : "#d9a86c";
  context.strokeStyle = "#1e1713";
  context.lineWidth = primary ? 4 : 2;
  context.beginPath();
  context.arc(0, 0, radius, 0, Math.PI * 2);
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

function forEachRow(visitor) {
  for (let offset = 0; offset + ROW_WIDTH <= rows.length; offset += ROW_WIDTH) {
    visitor(rows.subarray(offset, offset + ROW_WIDTH));
  }
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

