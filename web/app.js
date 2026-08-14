import init, { ComparisonEngine, DemoEngine } from "./pkg/demo_wasm.js?v=clearance-v1";

const DEFAULT_SEED = "2026";
const SPEED_DELTA = 0.10;
const RELATION_RADIUS_SQUARED = 0.34 * 0.34;
const MAX_CHECKPOINTS = 5;
const MAX_SESSION_HISTORY = 50;
const MAX_PERSONAL_FIELDS = 8;
const MAX_MORPHOLOGY_GROUPS = 8;
const MAX_ACTIVE_LEASES = 8;
const COMPARISON_SPEC_SCHEMA = "combinatorial.swarmability.comparison-spec.v1";
const COMPARISON_TAPE_SCHEMA = "combinatorial.swarmability.normalized-input-tape.v1";
const COMPARISON_STEP_INTERVAL = 320;
const COMPARISON_REDUCED_STEP_INTERVAL = 700;
const COMPARISON_SCENARIOS = {
  raw_semantic_equivalent: "raw-semantic-midpoint.v1",
  raw_semantic_contrast: "raw-semantic-contrast.v1",
  superposition_lease: "superposition-lease.v1",
  soft_collision_free: "soft-collision-free.v1",
};
const CONTRIBUTOR_LABELS = ["A", "B", "C", "D"];
const OPERATOR_LABELS = ["A", "B", "C", "D"];
const CONTRIBUTOR_COLORS = ["#3f6f85", "#a85d45", "#657a46", "#755b9b"];
const GROUP_COLORS = ["#315d6c", "#9b563f", "#5d753c", "#705296", "#936d22", "#276d65", "#8a4968", "#57698f"];
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
const fieldContributor = document.querySelector("#field-contributor");
const fieldPolarity = document.querySelector("#field-polarity");
const fieldLifetime = document.querySelector("#field-lifetime");
const fieldX = document.querySelector("#field-x");
const fieldY = document.querySelector("#field-y");
const fieldSelect = document.querySelector("#field-select");
const dynamicsAlignment = document.querySelector("#dynamics-alignment");
const dynamicsCohesion = document.querySelector("#dynamics-cohesion");
const dynamicsSeparation = document.querySelector("#dynamics-separation");
const executionPolicy = document.querySelector("#execution-policy");
const executionSeparationWeight = document.querySelector("#execution-separation-weight");
const executionSeparationRadius = document.querySelector("#execution-separation-radius");
const executionSpeedLimit = document.querySelector("#execution-speed-limit");
const executionAccelerationLimit = document.querySelector("#execution-acceleration-limit");
const executionBoundaryStrength = document.querySelector("#execution-boundary-strength");
const navigationX = document.querySelector("#navigation-x");
const navigationY = document.querySelector("#navigation-y");
const navigationDirection = document.querySelector("#navigation-direction");
const navigationRadius = document.querySelector("#navigation-radius");
const navigationStrength = document.querySelector("#navigation-strength");
const semanticSpace = document.querySelector("#semantic-space");
const semanticTime = document.querySelector("#semantic-time");
const semanticWeight = document.querySelector("#semantic-weight");
const semanticFlow = document.querySelector("#semantic-flow");
const splitSourceGroup = document.querySelector("#split-source-group");
const mergeFirstGroup = document.querySelector("#merge-first-group");
const mergeSecondGroup = document.querySelector("#merge-second-group");
const scaleGroup = document.querySelector("#scale-group");
const formationScale = document.querySelector("#formation-scale");
const leaseOperator = document.querySelector("#lease-operator");
const leaseMember = document.querySelector("#lease-member");
const leaseReceiver = document.querySelector("#lease-receiver");
const leaseLifetime = document.querySelector("#lease-lifetime");
const leasedBehavior = document.querySelector("#leased-behavior");
const atlasFilters = document.querySelector("#atlas-filters");
const atlasList = document.querySelector("#atlas-list");
const atlasCount = document.querySelector("#atlas-count");
const demonstration = document.querySelector("#demonstration");
const comparisonSection = document.querySelector("#comparison");
const comparisonScenario = document.querySelector("#comparison-scenario");
const comparisonSeed = document.querySelector("#comparison-seed");
const comparisonStatus = document.querySelector("#comparison-status");
const comparisonLeftCanvas = document.querySelector("#comparison-left-canvas");
const comparisonRightCanvas = document.querySelector("#comparison-right-canvas");

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
let comparisonEngine = null;
let comparisonResult = null;
let comparisonModeActive = false;
let comparisonRunning = false;
let comparisonAnimationHandle = 0;
let comparisonPreviousTimestamp = 0;
let ordinaryStateBeforeComparison = "";
let ordinaryRowsBeforeComparison = [];

await init({
  module_or_path: new URL("./pkg/demo_wasm_bg.wasm?v=clearance-v1", import.meta.url),
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
  document.querySelector("#open-comparison-button").addEventListener("click", openComparisonMode);
  document.querySelector("#nav-comparison-link").addEventListener("click", (event) => {
    event.preventDefault();
    openComparisonMode();
  });
  document.querySelector("#close-comparison-button").addEventListener("click", closeComparisonMode);
  document.querySelector("#comparison-start-button").addEventListener("click", startComparison);
  document.querySelector("#comparison-pause-button").addEventListener("click", pauseComparison);
  document.querySelector("#comparison-step-button").addEventListener("click", stepComparisonEvent);
  document.querySelector("#comparison-reset-button").addEventListener("click", resetComparison);
  document.querySelector("#comparison-replay-button").addEventListener("click", replayComparison);
  comparisonScenario.addEventListener("change", initializeComparison);
  comparisonSeed.addEventListener("change", initializeComparison);
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
  document.querySelector("#place-field-button").addEventListener("click", placePersonalField);
  document.querySelector("#move-field-button").addEventListener("click", moveSelectedField);
  document.querySelector("#polarity-field-button").addEventListener("click", setSelectedFieldPolarity);
  document.querySelector("#remove-field-button").addEventListener("click", removeSelectedField);
  fieldX.addEventListener("input", updateFieldOutputs);
  fieldY.addEventListener("input", updateFieldOutputs);
  fieldSelect.addEventListener("change", () => {
    syncSelectedFieldEditor();
    updateFieldControls();
  });
  bindDynamicsSlider(dynamicsAlignment, "set_alignment", "alignment");
  bindDynamicsSlider(dynamicsCohesion, "set_cohesion", "cohesion");
  bindDynamicsSlider(dynamicsSeparation, "set_separation", "separation");
  executionPolicy.addEventListener("change", (event) => {
    dispatch(
      { type: "set_collision_policy", policy: executionPolicy.value },
      interactionTrace(
        event,
        `execution.collision-policy(${executionPolicy.value})`,
        "Whole-scene app-owned collision policy; steering core and input route remain unchanged"
      )
    );
  });
  bindExecutionSlider(executionSeparationWeight, "set_separation_weight", "separation-weight");
  bindExecutionSlider(executionSeparationRadius, "set_separation_radius", "separation-radius");
  bindExecutionSlider(executionSpeedLimit, "set_speed_limit", "speed-limit");
  bindExecutionSlider(executionAccelerationLimit, "set_acceleration_limit", "acceleration-limit");
  bindExecutionSlider(executionBoundaryStrength, "set_boundary_strength", "boundary-strength");
  [navigationX, navigationY, navigationDirection, navigationRadius, navigationStrength]
    .forEach((slider) => {
      slider.addEventListener("input", updateNavigationOutputs);
      slider.addEventListener("pointerdown", (event) => {
        slider.dataset.inputRoute = event.pointerType || "pointer";
      });
      slider.addEventListener("keydown", () => {
        slider.dataset.inputRoute = "keyboard-or-switch";
      });
    });
  document.querySelector("#apply-navigation-button").addEventListener("click", applyNavigationField);
  document.querySelector("#clear-navigation-button").addEventListener("click", clearNavigationField);
  bindSemanticSlider(semanticSpace, "set_space_quality", "space");
  bindSemanticSlider(semanticTime, "set_time_quality", "time");
  bindSemanticSlider(semanticWeight, "set_weight_quality", "weight");
  bindSemanticSlider(semanticFlow, "set_flow_quality", "flow");
  document.querySelector("#split-group-button").addEventListener("click", splitSelectedGroup);
  document.querySelector("#merge-groups-button").addEventListener("click", mergeSelectedGroups);
  document.querySelector("#set-formation-scale-button").addEventListener("click", setSelectedFormationScale);
  splitSourceGroup.addEventListener("change", updateMorphologyControls);
  mergeFirstGroup.addEventListener("change", updateMorphologyControls);
  mergeSecondGroup.addEventListener("change", updateMorphologyControls);
  scaleGroup.addEventListener("change", syncFormationScaleEditor);
  formationScale.addEventListener("input", updateFormationScaleOutput);
  document.querySelector("#request-lease-button").addEventListener("click", requestSelectedLease);
  document.querySelector("#release-lease-button").addEventListener("click", releaseSelectedLease);
  document.querySelector("#offer-handoff-button").addEventListener("click", offerSelectedHandoff);
  document.querySelector("#accept-handoff-button").addEventListener("click", (event) => resolveSelectedHandoff(event, "accept"));
  document.querySelector("#decline-handoff-button").addEventListener("click", (event) => resolveSelectedHandoff(event, "decline"));
  document.querySelector("#use-lease-button").addEventListener("click", useSelectedLease);
  [leaseOperator, leaseMember, leaseReceiver].forEach((select) => {
    select.addEventListener("change", updateLeaseControls);
  });
  leaseLifetime.addEventListener("input", updateLeaseLifetimeOutput);
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
    if (comparisonModeActive || event.defaultPrevented || event.altKey || event.ctrlKey || event.metaKey) {
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

function bindDynamicsSlider(slider, actionType, parameter) {
  slider.addEventListener("pointerdown", (event) => {
    slider.dataset.inputRoute = event.pointerType || "pointer";
  });
  slider.addEventListener("keydown", () => {
    slider.dataset.inputRoute = "keyboard-or-switch";
  });
  slider.addEventListener("input", () => {
    updateDynamicsOutput(parameter, Number(slider.value));
  });
  slider.addEventListener("change", () => {
    const rate = Number(slider.value);
    dispatch(
      { type: actionType, rate },
      {
        inputRoute: slider.dataset.inputRoute || "synthetic-slider",
        normalizedInput: `dynamics.${parameter}.rate(${rate.toFixed(2)})`,
        policy: "Raw controls take authority over the one resolved vector; swarm-wide deterministic transition selection; no multi-user combination",
      }
    );
    delete slider.dataset.inputRoute;
  });
}

function bindSemanticSlider(slider, actionType, quality) {
  slider.addEventListener("pointerdown", (event) => {
    slider.dataset.inputRoute = event.pointerType || "pointer";
  });
  slider.addEventListener("keydown", () => {
    slider.dataset.inputRoute = "keyboard-or-switch";
  });
  slider.addEventListener("input", () => {
    updateSemanticOutput(quality, Number(slider.value));
  });
  slider.addEventListener("change", () => {
    const value = Number(slider.value);
    dispatch(
      { type: actionType, value },
      {
        inputRoute: slider.dataset.inputRoute || "synthetic-slider",
        normalizedInput: `semantic-dynamics.${quality}(${value.toFixed(2)})`,
        policy: "Semantic controls take authority over one app-owned resolved vector; swarm-wide effect; no camera or multi-user authority",
      }
    );
    delete slider.dataset.inputRoute;
  });
}

function bindExecutionSlider(slider, actionType, setting) {
  slider.addEventListener("pointerdown", (event) => {
    slider.dataset.inputRoute = event.pointerType || "pointer";
  });
  slider.addEventListener("keydown", () => {
    slider.dataset.inputRoute = "keyboard-or-switch";
  });
  slider.addEventListener("input", () => {
    updateExecutionOutput(setting, Number(slider.value));
  });
  slider.addEventListener("change", () => {
    const value = Number(slider.value);
    dispatch(
      { type: actionType, value },
      {
        inputRoute: slider.dataset.inputRoute || "synthetic-slider",
        normalizedInput: `execution.${setting}(${value.toFixed(2)})`,
        policy: "Whole-scene app-owned kinematic or clearance setting; same deterministic simulation path",
      }
    );
    delete slider.dataset.inputRoute;
  });
}

function updateExecutionOutput(setting, value) {
  document.querySelector(`#execution-${setting}-value`).textContent = value.toFixed(2);
}

function updateExecutionControls() {
  const settings = state.execution_settings;
  executionPolicy.value = settings.collision_policy;
  const controlsBySetting = {
    "separation-weight": [executionSeparationWeight, settings.separation_weight],
    "separation-radius": [executionSeparationRadius, settings.separation_radius],
    "speed-limit": [executionSpeedLimit, settings.speed_limit],
    "acceleration-limit": [executionAccelerationLimit, settings.acceleration_limit],
    "boundary-strength": [executionBoundaryStrength, settings.boundary_strength],
  };
  Object.entries(controlsBySetting).forEach(([setting, [slider, value]]) => {
    slider.value = String(value);
    updateExecutionOutput(setting, value);
  });
  document.querySelector("#clear-navigation-button").disabled = state.navigation_field === null;
}

function updateNavigationOutputs() {
  document.querySelector("#navigation-x-value").textContent = Number(navigationX.value).toFixed(2);
  document.querySelector("#navigation-y-value").textContent = Number(navigationY.value).toFixed(2);
  document.querySelector("#navigation-direction-value").textContent = `${Number(navigationDirection.value)}°`;
  document.querySelector("#navigation-radius-value").textContent = Number(navigationRadius.value).toFixed(2);
  document.querySelector("#navigation-strength-value").textContent = Number(navigationStrength.value).toFixed(2);
}

function applyNavigationField(event) {
  const angleDegrees = Number(navigationDirection.value);
  const angleRadians = angleDegrees * Math.PI / 180;
  dispatch(
    {
      type: "set_navigation_field",
      x: Number(navigationX.value),
      y: Number(navigationY.value),
      direction_x: Math.cos(angleRadians),
      direction_y: Math.sin(angleRadians),
      radius: Number(navigationRadius.value),
      strength: Number(navigationStrength.value),
    },
    interactionTrace(
      event,
      `navigation-field.apply(${angleDegrees}deg)`,
      "Directional field affects members inside its region; local behavior and collision policy remain independently active"
    )
  );
}

function clearNavigationField(event) {
  dispatch(
    { type: "clear_navigation_field" },
    interactionTrace(
      event,
      "navigation-field.clear",
      "Navigation influence is removed; collision, dynamics, morphology, fields, and leases remain intact"
    )
  );
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

function splitSelectedGroup(event) {
  const sourceGroupId = Number(splitSourceGroup.value);
  const newGroupId = nextCanonicalGroupId();
  if (!Number.isInteger(sourceGroupId) || newGroupId === null) {
    announce("No canonical split is available.", true);
    return;
  }
  dispatch(
    {
      type: "split_group",
      source_group_id: sourceGroupId,
      new_group_id: newGroupId,
      partition_rule: "alternating_member_id",
      expected_morphology_revision: state.morphology_revision,
    },
    interactionTrace(
      event,
      `morphology.split(group-${sourceGroupId + 1}, alternating-member-id)`,
      `Group ${sourceGroupId + 1} retains its identity and scale; the smallest unused canonical ID becomes Group ${newGroupId + 1}`
    )
  );
}

function mergeSelectedGroups(event) {
  const firstGroupId = Number(mergeFirstGroup.value);
  const secondGroupId = Number(mergeSecondGroup.value);
  if (!Number.isInteger(firstGroupId) || !Number.isInteger(secondGroupId) || firstGroupId === secondGroupId) {
    announce("Choose two distinct groups to merge.", true);
    return;
  }
  const survivorGroupId = Math.min(firstGroupId, secondGroupId);
  dispatch(
    {
      type: "merge_groups",
      group_a_id: firstGroupId,
      group_b_id: secondGroupId,
      survivor_group_id: survivorGroupId,
      expected_morphology_revision: state.morphology_revision,
    },
    interactionTrace(
      event,
      `morphology.merge(group-${firstGroupId + 1}, group-${secondGroupId + 1})`,
      `Lower canonical ID Group ${survivorGroupId + 1} survives and retains its scale; membership is conserved`
    )
  );
}

function setSelectedFormationScale(event) {
  const groupId = Number(scaleGroup.value);
  const scale = Number(formationScale.value);
  dispatch(
    {
      type: "set_formation_scale",
      group_id: groupId,
      scale,
      expected_morphology_revision: state.morphology_revision,
    },
    interactionTrace(
      event,
      `morphology.scale(group-${groupId + 1}, ${scale.toFixed(2)})`,
      `Only Group ${groupId + 1}'s formation-scale target changes; dynamics, fields, scope, and provenance remain unchanged`
    )
  );
}

function nextCanonicalGroupId() {
  const activeIds = new Set(state.groups.map((group) => group.group_id));
  for (let groupId = 0; groupId < MAX_MORPHOLOGY_GROUPS; groupId += 1) {
    if (!activeIds.has(groupId)) {
      return groupId;
    }
  }
  return null;
}

function replaceGroupOptions(select, previousValue) {
  const fragment = document.createDocumentFragment();
  state.groups.forEach((group) => {
    const option = document.createElement("option");
    option.value = String(group.group_id);
    option.textContent = `Group ${group.group_id + 1} · ${group.member_ids.length} members`;
    fragment.append(option);
  });
  select.replaceChildren(fragment);
  select.value = state.groups.some((group) => String(group.group_id) === previousValue)
    ? previousValue
    : String(state.groups[0].group_id);
}

function updateMorphologyControls() {
  const splitValue = splitSourceGroup.value;
  const firstValue = mergeFirstGroup.value;
  const secondValue = mergeSecondGroup.value;
  const scaleValue = scaleGroup.value;
  replaceGroupOptions(splitSourceGroup, splitValue);
  replaceGroupOptions(mergeFirstGroup, firstValue);
  replaceGroupOptions(mergeSecondGroup, secondValue);
  replaceGroupOptions(scaleGroup, scaleValue);

  if (state.groups.length > 1 && mergeFirstGroup.value === mergeSecondGroup.value) {
    const alternative = state.groups.find((group) => String(group.group_id) !== mergeFirstGroup.value);
    mergeSecondGroup.value = String(alternative.group_id);
  }

  const newGroupId = nextCanonicalGroupId();
  document.querySelector("#split-new-group").textContent = newGroupId === null
    ? "Maximum reached"
    : `Group ${newGroupId + 1}`;
  const splitGroup = state.groups.find((group) => String(group.group_id) === splitSourceGroup.value);
  document.querySelector("#split-group-button").disabled =
    newGroupId === null || !splitGroup || splitGroup.member_ids.length < 2;

  const distinctMerge = state.groups.length > 1 && mergeFirstGroup.value !== mergeSecondGroup.value;
  document.querySelector("#merge-groups-button").disabled = !distinctMerge;
  document.querySelector("#merge-survivor-group").textContent = distinctMerge
    ? `Group ${Math.min(Number(mergeFirstGroup.value), Number(mergeSecondGroup.value)) + 1}`
    : "Choose two groups";
  syncFormationScaleEditor();
}

function syncFormationScaleEditor() {
  const group = state.groups.find((candidate) => String(candidate.group_id) === scaleGroup.value);
  if (group) {
    formationScale.value = String(group.formation_scale);
    updateFormationScaleOutput();
  }
}

function updateFormationScaleOutput() {
  document.querySelector("#formation-scale-value").textContent = Number(formationScale.value).toFixed(2);
}

function requestSelectedLease(event) {
  const memberId = Number(leaseMember.value);
  const operatorId = Number(leaseOperator.value);
  const lifetimeSteps = Number(leaseLifetime.value);
  dispatch(
    {
      type: "request_lease",
      member_id: memberId,
      operator_id: operatorId,
      lifetime_steps: lifetimeSteps,
      expected_authority_revision: state.authority_revision,
    },
    interactionTrace(
      event,
      `lease.request(member-${memberId + 1}, operator-${OPERATOR_LABELS[operatorId]}, ${lifetimeSteps}-steps)`,
      "Acquire only if the canonical member is unheld and the active-lease cap has room; fixed-step expiry only"
    )
  );
}

function releaseSelectedLease(event) {
  const memberId = Number(leaseMember.value);
  const operatorId = Number(leaseOperator.value);
  dispatch(
    {
      type: "release_lease",
      member_id: memberId,
      operator_id: operatorId,
      expected_authority_revision: state.authority_revision,
    },
    interactionTrace(
      event,
      `lease.release(member-${memberId + 1}, operator-${OPERATOR_LABELS[operatorId]})`,
      "Only the exact current holder may release; a pending offer grants no authority"
    )
  );
}

function offerSelectedHandoff(event) {
  const memberId = Number(leaseMember.value);
  const holderId = Number(leaseOperator.value);
  const receiverId = Number(leaseReceiver.value);
  dispatch(
    {
      type: "offer_lease_handoff",
      member_id: memberId,
      holder_operator_id: holderId,
      receiver_operator_id: receiverId,
      expected_authority_revision: state.authority_revision,
    },
    interactionTrace(
      event,
      `lease.handoff-offer(member-${memberId + 1}, ${OPERATOR_LABELS[holderId]}→${OPERATOR_LABELS[receiverId]})`,
      "Exact holder consent creates one pending named-receiver offer; holder and original expiry remain unchanged"
    )
  );
}

function resolveSelectedHandoff(event, decision) {
  const memberId = Number(leaseMember.value);
  const receiverId = Number(leaseReceiver.value);
  dispatch(
    {
      type: "resolve_lease_handoff",
      member_id: memberId,
      receiver_operator_id: receiverId,
      decision,
      expected_authority_revision: state.authority_revision,
    },
    interactionTrace(
      event,
      `lease.handoff-${decision}(member-${memberId + 1}, receiver-${OPERATOR_LABELS[receiverId]})`,
      decision === "accept"
        ? "Only the exact named receiver may explicitly accept; the original fixed-step expiry remains"
        : "Only the exact named receiver may decline; current holder and original expiry remain"
    )
  );
}

function useSelectedLease(event) {
  const memberId = Number(leaseMember.value);
  const operatorId = Number(leaseOperator.value);
  const behavior = leasedBehavior.value;
  dispatch(
    {
      type: "set_leased_behavior",
      member_id: memberId,
      operator_id: operatorId,
      behavior,
      expected_authority_revision: state.authority_revision,
    },
    interactionTrace(
      event,
      `lease.use(member-${memberId + 1}, operator-${OPERATOR_LABELS[operatorId]}, ${behavior})`,
      "Exact current holder may assign one behavior to this canonical member; target scope remains independent"
    )
  );
}

function selectedLease() {
  const memberId = Number(leaseMember.value);
  return state.leases.find((lease) => lease.member_id === memberId);
}

function updateLeaseLifetimeOutput() {
  document.querySelector("#lease-lifetime-value").textContent = leaseLifetime.value;
}

function updateLeaseControls() {
  const selectedMember = leaseMember.value;
  if (leaseMember.options.length === 0) {
    const fragment = document.createDocumentFragment();
    state.members.forEach((member) => {
      const option = document.createElement("option");
      option.value = String(member.member_id);
      option.textContent = `Member ${member.member_id + 1}`;
      fragment.append(option);
    });
    leaseMember.append(fragment);
  }
  if (selectedMember && state.members.some((member) => String(member.member_id) === selectedMember)) {
    leaseMember.value = selectedMember;
  }
  const lease = selectedLease();
  const operatorId = Number(leaseOperator.value);
  const receiverId = Number(leaseReceiver.value);
  const isHolder = lease?.holder_operator_id === operatorId;
  const isReceiver = lease?.pending_handoff_to === receiverId;
  document.querySelector("#request-lease-button").disabled = Boolean(lease) || state.leases.length >= MAX_ACTIVE_LEASES;
  document.querySelector("#release-lease-button").disabled = !isHolder;
  document.querySelector("#offer-handoff-button").disabled =
    !isHolder || receiverId === operatorId || lease.pending_handoff_to !== null;
  document.querySelector("#accept-handoff-button").disabled = !isReceiver;
  document.querySelector("#decline-handoff-button").disabled = !isReceiver;
  document.querySelector("#use-lease-button").disabled = !isHolder;

  const memberLabel = `Member ${Number(leaseMember.value) + 1}`;
  const operatorLabel = `Operator ${OPERATOR_LABELS[operatorId]}`;
  let reason = `${memberLabel} is unheld; ${operatorLabel} may request it.`;
  if (lease) {
    reason = `${memberLabel} is held by Operator ${OPERATOR_LABELS[lease.holder_operator_id]} for ${lease.remaining_steps} more fixed steps.`;
    if (lease.pending_handoff_to !== null) {
      reason += ` Operator ${OPERATOR_LABELS[lease.pending_handoff_to]} must explicitly accept or decline the pending offer.`;
    } else if (!isHolder) {
      reason += ` ${operatorLabel} cannot release, offer, or use it.`;
    }
  } else if (state.leases.length >= MAX_ACTIVE_LEASES) {
    reason = `The eight-lease cap is full; release or await expiry before requesting ${memberLabel}.`;
  }
  document.querySelector("#lease-command-reason").textContent = reason;
  updateLeaseLifetimeOutput();
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

function updateDynamicsOutput(parameter, rate) {
  document.querySelector(`#dynamics-${parameter}-value`).textContent = rate.toFixed(2);
}

function updateDynamicsControls() {
  const controlsByParameter = {
    alignment: dynamicsAlignment,
    cohesion: dynamicsCohesion,
    separation: dynamicsSeparation,
  };
  Object.entries(controlsByParameter).forEach(([parameter, slider]) => {
    const rate = state.raw_dynamics_rates[parameter];
    slider.value = String(rate);
    updateDynamicsOutput(parameter, rate);
  });
}

function updateSemanticOutput(quality, value) {
  document.querySelector(`#semantic-${quality}-value`).textContent = value.toFixed(2);
}

function updateSemanticControls() {
  const controlsByQuality = {
    space: semanticSpace,
    time: semanticTime,
    weight: semanticWeight,
    flow: semanticFlow,
  };
  Object.entries(controlsByQuality).forEach(([quality, slider]) => {
    const value = state.semantic_qualities[quality];
    slider.value = String(value);
    updateSemanticOutput(quality, value);
  });
}

function dynamicsModeLabel() {
  return state.dynamics_control_mode === "semantic" ? "Semantic qualities" : "Raw controls";
}

function renderResolvedDynamics() {
  const vector = state.resolved_dynamics;
  document.querySelector("#resolved-control-mode").textContent = dynamicsModeLabel();
  document.querySelector("#resolved-alignment").textContent = vector.rates.alignment.toFixed(2);
  document.querySelector("#resolved-cohesion").textContent = vector.rates.cohesion.toFixed(2);
  document.querySelector("#resolved-separation").textContent = vector.rates.separation.toFixed(2);
  document.querySelector("#resolved-speed-scale").textContent = vector.speed_scale.toFixed(2);
  document.querySelector("#resolved-damping").textContent = vector.damping.toFixed(2);
  document.querySelector("#resolved-jitter").textContent = vector.jitter.toFixed(2);
}

function placePersonalField(event) {
  if (state.fields.length >= MAX_PERSONAL_FIELDS) {
    announce("Eight personal fields are already active. Remove one before placing another.", true);
    fieldSelect.focus();
    return;
  }
  const fieldId = nextFieldId();
  const action = {
    type: "place_field",
    field_id: fieldId,
    contributor_id: Number(fieldContributor.value),
    x: Number(fieldX.value),
    y: Number(fieldY.value),
    polarity: fieldPolarity.value,
    lifetime: fieldLifetime.value === "persistent"
      ? { mode: "persistent" }
      : { mode: "expiring", steps: Number(fieldLifetime.value) },
  };
  const receipt = dispatch(
    action,
    interactionTrace(
      event,
      `field.place(${fieldId + 1})`,
      "Field scope; app-local synthetic contributors combine by additive superposition"
    )
  );
  if (receipt?.accepted) {
    fieldSelect.value = String(fieldId);
    updateFieldControls();
  }
}

function moveSelectedField(event) {
  const fieldId = selectedFieldId();
  if (fieldId === null) {
    announce("Choose an active field before moving it.", true);
    fieldSelect.focus();
    return;
  }
  dispatch(
    { type: "move_field", field_id: fieldId, x: Number(fieldX.value), y: Number(fieldY.value) },
    interactionTrace(
      event,
      `field.move(${fieldId + 1})`,
      "Field scope; contributor, polarity, lifetime, and additive policy remain unchanged"
    )
  );
}

function setSelectedFieldPolarity(event) {
  const fieldId = selectedFieldId();
  if (fieldId === null) {
    announce("Choose an active field before changing its polarity.", true);
    fieldSelect.focus();
    return;
  }
  dispatch(
    { type: "set_field_polarity", field_id: fieldId, polarity: fieldPolarity.value },
    interactionTrace(
      event,
      `field.polarity(${fieldPolarity.value})`,
      "Field scope; polarity changes without changing provenance, lifetime, or superposition policy"
    )
  );
}

function removeSelectedField(event) {
  const fieldId = selectedFieldId();
  if (fieldId === null) {
    announce("Choose an active field before removing it.", true);
    fieldSelect.focus();
    return;
  }
  dispatch(
    { type: "remove_field", field_id: fieldId },
    interactionTrace(
      event,
      `field.remove(${fieldId + 1})`,
      "Field scope; remove one additive contribution without changing other contributors"
    )
  );
}

function nextFieldId() {
  const used = new Set(state.fields.map((field) => field.field_id));
  for (let fieldId = 0; fieldId <= 63; fieldId += 1) {
    if (!used.has(fieldId)) {
      return fieldId;
    }
  }
  return 0;
}

function selectedFieldId() {
  if (fieldSelect.value === "") {
    return null;
  }
  const fieldId = Number(fieldSelect.value);
  return state.fields.some((field) => field.field_id === fieldId) ? fieldId : null;
}

function updateFieldOutputs() {
  document.querySelector("#field-x-value").textContent = Number(fieldX.value).toFixed(2);
  document.querySelector("#field-y-value").textContent = Number(fieldY.value).toFixed(2);
}

function syncSelectedFieldEditor() {
  const fieldId = selectedFieldId();
  const field = state.fields.find((candidate) => candidate.field_id === fieldId);
  if (!field) {
    return;
  }
  fieldContributor.value = String(field.contributor_id);
  fieldPolarity.value = field.polarity;
  fieldX.value = String(field.x);
  fieldY.value = String(field.y);
  updateFieldOutputs();
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
  const morphologyBefore = isMorphologyAction(action) ? morphologySummary(state) : "";
  const authorityBefore = authoritySummary(state);
  const authorityRevisionBefore = state.authority_revision;
  try {
    const receipt = JSON.parse(engine.dispatch_json(JSON.stringify(action)));
    refreshAll();
    updateActionTrace(trace, action, receipt);
    if (morphologyBefore) {
      updateMorphologyTrace(morphologyBefore, trace, action, receipt);
    }
    if (isLeaseAction(action) || state.authority_revision !== authorityRevisionBefore) {
      updateLeaseTrace(authorityBefore, trace, action, receipt);
    }
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
  const revision = `state ${receipt.state_revision}, selection ${receipt.selection_revision}, morphology ${receipt.morphology_revision}, authority ${receipt.authority_revision}`;
  renderActionTrace(
    { ...trace, policy },
    semanticActionLabel(action),
    `${receipt.code} · ${accepted} · ${revision}`
  );
}

function isMorphologyAction(action) {
  return ["split_group", "merge_groups", "set_formation_scale"].includes(action.type);
}

function isLeaseAction(action) {
  return [
    "request_lease",
    "release_lease",
    "offer_lease_handoff",
    "resolve_lease_handoff",
    "set_leased_behavior",
  ].includes(action.type);
}

function morphologySummary(snapshot) {
  const groups = snapshot.groups
    .map((group) => `Group ${group.group_id + 1}: ${group.member_ids.length} members at ${group.formation_scale.toFixed(2)}`)
    .join("; ");
  return `${snapshot.groups.length} ${snapshot.groups.length === 1 ? "group" : "groups"} · ${groups}`;
}

function updateMorphologyTrace(before, trace, action, receipt) {
  const accepted = receipt.accepted ? "accepted" : "rejected";
  document.querySelector("#morphology-trace-before").textContent = before;
  document.querySelector("#morphology-trace-action").textContent = semanticActionLabel(action);
  document.querySelector("#morphology-trace-policy").textContent = trace.policy;
  document.querySelector("#morphology-trace-receipt").textContent =
    `${receipt.code} · ${accepted} · morphology ${receipt.morphology_revision}`;
  document.querySelector("#morphology-trace-after").textContent = morphologySummary(state);
}

function authoritySummary(snapshot) {
  if (snapshot.leases.length === 0) {
    return `No active leases; authority revision ${snapshot.authority_revision}.`;
  }
  const leases = snapshot.leases.map((lease) => {
    const handoff = lease.pending_handoff_to === null
      ? ""
      : `, offered to ${OPERATOR_LABELS[lease.pending_handoff_to]}`;
    return `Member ${lease.member_id + 1}: Operator ${OPERATOR_LABELS[lease.holder_operator_id]}, ${lease.remaining_steps} steps${handoff}`;
  }).join("; ");
  return `${snapshot.leases.length} active leases · ${leases} · authority revision ${snapshot.authority_revision}.`;
}

function updateLeaseTrace(before, trace, action, receipt) {
  const expired = !isLeaseAction(action);
  const accepted = receipt.accepted ? "accepted" : "rejected";
  const actionLabel = isLeaseAction(action) ? semanticActionLabel(action) : "fixed_step_lease_expiry";
  document.querySelector("#lease-trace-before").textContent = before;
  document.querySelector("#lease-trace-action").textContent = actionLabel;
  document.querySelector("#lease-trace-policy").textContent = isLeaseAction(action)
    ? trace.policy
    : "Deterministic expiry at the exclusive fixed-step boundary; no wall clock or hidden arbitration";
  document.querySelector("#lease-trace-receipt").textContent = isLeaseAction(action)
    ? `${receipt.code} · ${accepted} · authority ${receipt.authority_revision}`
    : `${expired ? "fixed_step_expiry" : receipt.code} · authority ${receipt.authority_revision}`;
  document.querySelector("#lease-trace-after").textContent = authoritySummary(state);
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
    .filter(([key]) => ![
      "type",
      "expected_selection_revision",
      "expected_morphology_revision",
      "expected_authority_revision",
    ].includes(key))
    .map(([key, value]) => `${key}=${parameterLabel(value)}`);
  return parameters.length > 0 ? `${action.type}(${parameters.join(", ")})` : action.type;
}

function parameterLabel(value) {
  return value && typeof value === "object" ? JSON.stringify(value) : String(value);
}

function comparisonSpecJson() {
  const scenarioId = comparisonScenario.value;
  const tapeId = COMPARISON_SCENARIOS[scenarioId];
  const seed = comparisonSeed.value.trim();
  if (!tapeId) {
    throw new Error("Choose a supported comparison.");
  }
  if (!/^(0|[1-9][0-9]{0,19})$/.test(seed) || BigInt(seed) > 18446744073709551615n) {
    throw new Error("Comparison seed must be one decimal integer from 0 to 18446744073709551615 without leading zeroes.");
  }
  return JSON.stringify({
    schema: COMPARISON_SPEC_SCHEMA,
    scenario_id: scenarioId,
    seed,
    left_seed: seed,
    right_seed: seed,
    input_tape: {
      schema: COMPARISON_TAPE_SCHEMA,
      tape_id: tapeId,
    },
  });
}

function openComparisonMode() {
  if (comparisonModeActive) {
    return;
  }
  ordinaryStateBeforeComparison = engine.state_json();
  ordinaryRowsBeforeComparison = Array.from(engine.frame_rows());
  comparisonModeActive = true;
  if (animationHandle !== 0) {
    cancelAnimationFrame(animationHandle);
    animationHandle = 0;
  }
  comparisonSeed.value = state.seed;
  demonstration.hidden = true;
  comparisonSection.hidden = false;
  initializeComparison();
  comparisonScenario.focus();
}

function closeComparisonMode() {
  pauseComparison(false);
  const currentState = engine.state_json();
  const currentRows = Array.from(engine.frame_rows());
  const rowsEqual = currentRows.length === ordinaryRowsBeforeComparison.length
    && currentRows.every((value, index) => Object.is(value, ordinaryRowsBeforeComparison[index]));
  if (currentState !== ordinaryStateBeforeComparison || !rowsEqual) {
    comparisonStatus.textContent = "Exit rejected: the ordinary atlas changed while comparison mode was active.";
    comparisonStatus.dataset.error = "true";
    return;
  }
  if (comparisonEngine && typeof comparisonEngine.free === "function") {
    comparisonEngine.free();
  }
  comparisonEngine = null;
  comparisonResult = null;
  comparisonModeActive = false;
  comparisonSection.hidden = true;
  demonstration.hidden = false;
  syncAnimation();
  document.querySelector("#open-comparison-button").focus();
}

function initializeComparison() {
  if (!comparisonModeActive) {
    return;
  }
  pauseComparison(false);
  try {
    const specJson = comparisonSpecJson();
    if (comparisonEngine && typeof comparisonEngine.free === "function") {
      comparisonEngine.free();
    }
    comparisonEngine = new ComparisonEngine(specJson);
    comparisonResult = JSON.parse(comparisonEngine.result_json());
    comparisonStatus.textContent = "Comparison ready. Both lanes are paused at their byte-identical canonical start.";
    comparisonStatus.dataset.error = "false";
    refreshComparison();
  } catch (error) {
    comparisonEngine = null;
    comparisonResult = null;
    comparisonStatus.textContent = error instanceof Error ? error.message : "Comparison specification was rejected.";
    comparisonStatus.dataset.error = "true";
    updateComparisonButtons();
  }
}

function startComparison() {
  if (!comparisonEngine || comparisonResult?.complete) {
    return;
  }
  comparisonRunning = true;
  comparisonPreviousTimestamp = 0;
  comparisonStatus.textContent = "Running both isolated lanes from the same normalized input tape.";
  comparisonStatus.dataset.error = "false";
  updateComparisonButtons();
  if (comparisonAnimationHandle === 0) {
    comparisonAnimationHandle = requestAnimationFrame(animateComparison);
  }
}

function pauseComparison(announcePause = true) {
  comparisonRunning = false;
  if (comparisonAnimationHandle !== 0) {
    cancelAnimationFrame(comparisonAnimationHandle);
    comparisonAnimationHandle = 0;
  }
  comparisonPreviousTimestamp = 0;
  if (announcePause && comparisonResult && !comparisonResult.complete) {
    comparisonStatus.textContent = "Comparison paused. Step, reset, replay, or resume remain available.";
    comparisonStatus.dataset.error = "false";
  }
  updateComparisonButtons();
}

function animateComparison(timestamp) {
  comparisonAnimationHandle = 0;
  if (!comparisonRunning || !comparisonEngine) {
    return;
  }
  if (comparisonPreviousTimestamp === 0) {
    comparisonPreviousTimestamp = timestamp;
  }
  const interval = reducedMotion.matches
    ? COMPARISON_REDUCED_STEP_INTERVAL
    : COMPARISON_STEP_INTERVAL;
  if (timestamp - comparisonPreviousTimestamp >= interval) {
    comparisonPreviousTimestamp = timestamp;
    applyComparisonStep(false);
  }
  if (comparisonRunning) {
    comparisonAnimationHandle = requestAnimationFrame(animateComparison);
  }
}

function stepComparisonEvent() {
  if (comparisonRunning) {
    return;
  }
  applyComparisonStep(true);
}

function applyComparisonStep(announceStep) {
  if (!comparisonEngine || comparisonResult?.complete) {
    return;
  }
  try {
    const receipt = JSON.parse(comparisonEngine.step_event_json());
    comparisonResult = JSON.parse(comparisonEngine.result_json());
    refreshComparison();
    if (comparisonResult.complete) {
      pauseComparison(false);
      comparisonStatus.textContent = "Comparison tape complete. Both lanes remain at the same fixed tick.";
    } else if (announceStep) {
      comparisonStatus.textContent = `Applied input event ${receipt.cursor} of ${receipt.event_count} to both lanes.`;
    }
    comparisonStatus.dataset.error = "false";
  } catch {
    pauseComparison(false);
    comparisonStatus.textContent = "Comparison step failed closed; neither lane result was accepted.";
    comparisonStatus.dataset.error = "true";
  }
}

function resetComparison() {
  if (!comparisonEngine) {
    return;
  }
  pauseComparison(false);
  try {
    comparisonEngine.reset();
    comparisonResult = JSON.parse(comparisonEngine.result_json());
    refreshComparison();
    comparisonStatus.textContent = "Both lanes reset to the byte-identical canonical start.";
    comparisonStatus.dataset.error = "false";
  } catch {
    comparisonStatus.textContent = "Comparison reset failed closed.";
    comparisonStatus.dataset.error = "true";
  }
}

function replayComparison() {
  if (!comparisonEngine) {
    return;
  }
  pauseComparison(false);
  try {
    comparisonResult = JSON.parse(comparisonEngine.replay_all_json());
    refreshComparison();
    comparisonStatus.textContent = "Replayed the complete normalized input tape through both isolated reducers.";
    comparisonStatus.dataset.error = "false";
  } catch {
    comparisonStatus.textContent = "Comparison replay failed closed.";
    comparisonStatus.dataset.error = "true";
  }
}

function refreshComparison() {
  if (!comparisonEngine || !comparisonResult) {
    updateComparisonButtons();
    return;
  }
  const leftRows = comparisonEngine.left_frame_rows();
  const rightRows = comparisonEngine.right_frame_rows();
  renderComparisonInvariants();
  renderComparisonLane("left", comparisonResult.left, leftRows, comparisonLeftCanvas);
  renderComparisonLane("right", comparisonResult.right, rightRows, comparisonRightCanvas);
  renderComparisonMetrics();
  document.querySelector("#comparison-claim-boundary").textContent = comparisonResult.claim_boundary;
  updateComparisonButtons();
}

function updateComparisonButtons() {
  const ready = Boolean(comparisonEngine && comparisonResult);
  const complete = Boolean(comparisonResult?.complete);
  document.querySelector("#comparison-start-button").disabled = !ready || comparisonRunning || complete;
  document.querySelector("#comparison-pause-button").disabled = !comparisonRunning;
  document.querySelector("#comparison-step-button").disabled = !ready || comparisonRunning || complete;
  document.querySelector("#comparison-reset-button").disabled = !ready || comparisonRunning;
  document.querySelector("#comparison-replay-button").disabled = !ready || comparisonRunning;
  comparisonScenario.disabled = comparisonRunning;
  comparisonSeed.disabled = comparisonRunning;
}

function renderComparisonInvariants() {
  const invariants = comparisonResult.invariants;
  document.querySelector("#comparison-progress").textContent =
    `${comparisonResult.cursor} of ${comparisonResult.event_count} input events`;
  document.querySelector("#comparison-ticks").textContent =
    `${comparisonResult.left.metrics.tick} / ${comparisonResult.right.metrics.tick}`;
  document.querySelector("#comparison-start-equality").textContent =
    invariants.canonical_start_equal ? "Byte-identical" : "Rejected mismatch";
  document.querySelector("#comparison-shared-input").textContent =
    invariants.shared_seed && invariants.shared_input_tape ? "Same seed and tape" : "Rejected mismatch";
  document.querySelector("#comparison-isolation").textContent =
    invariants.isolated_mutable_cores && !invariants.ordinary_atlas_state_touched
      ? "Separate lane cores; ordinary atlas untouched"
      : "Isolation failed";
  document.querySelector("#comparison-vector-relation").textContent = {
    equivalent: "Equivalent across all six values",
    intentionally_different: "Intentionally different profiles",
    pending_configuration: "Pending configuration",
    not_applicable: "Not applicable to policy comparison",
  }[comparisonResult.vector_relation] || "Unknown";
}

function renderComparisonLane(side, lane, laneRows, laneCanvas) {
  document.querySelector(`#comparison-${side}-title`).textContent =
    `${side === "left" ? "Lane A" : "Lane B"}: ${lane.descriptor.label}`;
  document.querySelector(`#comparison-${side}-config`).textContent =
    `${lane.descriptor.configuration_id} · catalogue ${lane.descriptor.catalogue_entry_id}`;
  const vector = lane.state.resolved_dynamics;
  document.querySelector(`#comparison-${side}-rates`).textContent =
    `${vector.rates.alignment.toFixed(2)} / ${vector.rates.cohesion.toFixed(2)} / ${vector.rates.separation.toFixed(2)}`;
  document.querySelector(`#comparison-${side}-extras`).textContent =
    `${vector.speed_scale.toFixed(2)} / ${vector.damping.toFixed(2)} / ${vector.jitter.toFixed(2)}`;
  document.querySelector(`#comparison-${side}-provenance`).textContent = lane.final_state_provenance;
  renderComparisonEvidence(side, lane.descriptor.catalogue_entry_id);
  renderComparisonPolicyState(side, lane.state);
  renderComparisonTrace(side, lane.trace);
  drawComparisonLane(laneCanvas, laneRows, lane);
}

function renderComparisonEvidence(side, publicId) {
  const entry = catalogEntries.find((candidate) => candidate.public_id === publicId);
  const source = document.querySelector(`#comparison-${side}-source`);
  if (!entry) {
    source.textContent = `Public catalogue entry ${publicId} is unavailable.`;
    document.querySelector(`#comparison-${side}-evidence`).textContent = "Unavailable";
    document.querySelector(`#comparison-${side}-transfer`).textContent = "Unavailable";
    document.querySelector(`#comparison-${side}-nonclaim`).textContent = "Unavailable";
    return;
  }
  const link = document.createElement("a");
  link.href = entry.source.paper_url;
  link.textContent = entry.title;
  link.rel = "external";
  source.replaceChildren(link, document.createTextNode(` · ${entry.source.system_or_study}`));
  document.querySelector(`#comparison-${side}-evidence`).textContent =
    `${humanize(entry.source.literature_status)} · ${humanize(entry.source.evidence_kind)}`;
  document.querySelector(`#comparison-${side}-transfer`).textContent = entry.reconstruction.transfer_boundary;
  document.querySelector(`#comparison-${side}-nonclaim`).textContent = entry.reconstruction.does_not_claim;
}

function renderComparisonPolicyState(side, laneState) {
  const list = document.querySelector(`#comparison-${side}-policy-state`);
  const fragment = document.createDocumentFragment();
  laneState.fields.forEach((field) => {
    const item = document.createElement("li");
    item.textContent = `Field ${field.field_id + 1}: synthetic Contributor ${CONTRIBUTOR_LABELS[field.contributor_id]}, ${field.polarity}, position ${field.x.toFixed(2)}, ${field.y.toFixed(2)}.`;
    fragment.append(item);
  });
  laneState.leases.forEach((lease) => {
    const item = document.createElement("li");
    item.textContent = `Member ${lease.member_id + 1}: synthetic Operator ${OPERATOR_LABELS[lease.holder_operator_id]} holds the lease for ${lease.remaining_steps} more fixed steps; authority revision ${laneState.authority_revision}.`;
    fragment.append(item);
  });
  const execution = document.createElement("li");
  execution.textContent = `Disc execution: ${laneState.execution_settings.collision_policy === "collision_free" ? "collision-free" : "soft avoidance with overlap permitted"}; minimum surface clearance ${laneState.clearance_metrics.minimum_surface_clearance.toFixed(3)}, ${laneState.clearance_metrics.overlap_pair_count} overlapping pairs, ${laneState.clearance_metrics.total_intervention_count} deterministic corrections.`;
  fragment.append(execution);
  if (laneState.navigation_field !== null) {
    const navigation = document.createElement("li");
    navigation.textContent = `Navigation field: centre ${laneState.navigation_field.x.toFixed(2)}, ${laneState.navigation_field.y.toFixed(2)}; direction ${laneState.navigation_field.direction_x.toFixed(2)}, ${laneState.navigation_field.direction_y.toFixed(2)}; radius ${laneState.navigation_field.radius.toFixed(2)}; strength ${laneState.navigation_field.strength.toFixed(2)}.`;
    fragment.append(navigation);
  }
  if (fragment.childNodes.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No active field or lease provenance in this lane.";
    fragment.append(empty);
  }
  const behavior = document.createElement("li");
  behavior.textContent = `Behavior distribution: ${laneState.behavior_counts.flock} Flock, ${laneState.behavior_counts.cohere} Cohere, ${laneState.behavior_counts.disperse} Disperse.`;
  fragment.append(behavior);
  list.replaceChildren(fragment);
}

function renderComparisonTrace(side, traces) {
  const list = document.querySelector(`#comparison-${side}-trace`);
  if (traces.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No comparison event applied.";
    list.replaceChildren(empty);
    return;
  }
  const fragment = document.createDocumentFragment();
  traces.forEach((trace) => {
    const item = document.createElement("li");
    const heading = document.createElement("strong");
    const input = document.createElement("span");
    const request = document.createElement("span");
    const actions = document.createElement("code");
    const policy = document.createElement("span");
    const receipts = document.createElement("span");
    heading.textContent = `${trace.sequence}. ${trace.normalized_input}`;
    input.textContent = `Input route: ${trace.input_route}`;
    request.textContent = `Semantic request: ${trace.semantic_request}`;
    actions.textContent = `Actions: ${trace.semantic_actions.join(" → ")}`;
    policy.textContent = `Policy: ${trace.policy}`;
    receipts.textContent = `Receipts: ${trace.receipts.map(comparisonReceiptLabel).join(" · ")}; tick ${trace.tick_before}→${trace.tick_after}.`;
    item.append(heading, document.createElement("br"), input, document.createElement("br"), request,
      document.createElement("br"), actions, document.createElement("br"), policy,
      document.createElement("br"), receipts);
    fragment.append(item);
  });
  list.replaceChildren(fragment);
}

function comparisonReceiptLabel(receipt) {
  return `${receipt.accepted ? "accepted" : "rejected"} ${receipt.code} (state ${receipt.state_revision}, authority ${receipt.authority_revision})`;
}

function renderComparisonMetrics() {
  const body = document.querySelector("#comparison-metrics-body");
  const fragment = document.createDocumentFragment();
  comparisonResult.metric_definitions.forEach((definition) => {
    const row = document.createElement("tr");
    const label = document.createElement("th");
    const note = document.createElement("span");
    const left = document.createElement("td");
    const right = document.createElement("td");
    const delta = document.createElement("td");
    label.scope = "row";
    label.textContent = definition.label;
    note.textContent = `${definition.definition} Unit: ${definition.unit}.`;
    label.append(note);
    left.textContent = comparisonMetricValue(definition.metric_id, comparisonResult.left.metrics);
    right.textContent = comparisonMetricValue(definition.metric_id, comparisonResult.right.metrics);
    delta.textContent = comparisonMetricDelta(definition.metric_id, comparisonResult.delta_right_minus_left);
    row.append(label, left, right, delta);
    fragment.append(row);
  });
  body.replaceChildren(fragment);
}

function comparisonMetricValue(metricId, metrics) {
  if (metricId === "group_count") {
    return `${metrics.group_count}; sizes ${metrics.group_sizes.join(" / ")}`;
  }
  if (metricId === "behavior_distribution") {
    const values = metrics.behavior_distribution;
    return `${values.flock} / ${values.cohere} / ${values.disperse}`;
  }
  if (["tick", "active_fields", "active_leases", "overlap_pair_count", "near_miss_pair_count", "total_collision_interventions", "contact_tick_count"].includes(metricId)) {
    return String(metrics[metricId]);
  }
  return Number(metrics[metricId]).toFixed(3);
}

function comparisonMetricDelta(metricId, deltas) {
  if (metricId === "behavior_distribution") {
    const values = deltas.behavior_distribution;
    return `${signedInteger(values.flock)} / ${signedInteger(values.cohere)} / ${signedInteger(values.disperse)}`;
  }
  if (["tick", "group_count", "active_fields", "active_leases", "overlap_pair_count", "near_miss_pair_count", "total_collision_interventions", "contact_tick_count"].includes(metricId)) {
    return signedInteger(deltas[metricId]);
  }
  const value = Number(deltas[metricId]);
  return `${value > 0 ? "+" : ""}${value.toFixed(3)}`;
}

function signedInteger(value) {
  const number = Number(value);
  return `${number > 0 ? "+" : ""}${number}`;
}

function drawComparisonLane(laneCanvas, laneRows, lane) {
  const laneContext = laneCanvas.getContext("2d", { alpha: false });
  const width = laneCanvas.width;
  const height = laneCanvas.height;
  laneContext.fillStyle = "#fbf8f2";
  laneContext.fillRect(0, 0, width, height);
  laneContext.strokeStyle = "#ded5ca";
  laneContext.lineWidth = 1;
  laneContext.setLineDash([4, 10]);
  for (let index = 1; index < 6; index += 1) {
    laneContext.beginPath();
    laneContext.moveTo((width / 6) * index, 0);
    laneContext.lineTo((width / 6) * index, height);
    laneContext.moveTo(0, (height / 6) * index);
    laneContext.lineTo(width, (height / 6) * index);
    laneContext.stroke();
  }
  laneContext.setLineDash([]);
  drawComparisonNavigation(laneContext, lane.state.navigation_field, width, height);
  const projectedRows = [];
  for (let offset = 0; offset + rowWidth <= laneRows.length; offset += rowWidth) {
    projectedRows.push(laneRows.subarray(offset, offset + rowWidth));
  }
  relationEdges(projectedRows).forEach(([firstIndex, secondIndex]) => {
    const first = projectedRows[firstIndex];
    const second = projectedRows[secondIndex];
    laneContext.strokeStyle = "#b9aea2";
    laneContext.lineWidth = 1.2;
    laneContext.beginPath();
    laneContext.moveTo(((first[1] + 1) / 2) * width, ((1 - first[2]) / 2) * height);
    laneContext.lineTo(((second[1] + 1) / 2) * width, ((1 - second[2]) / 2) * height);
    laneContext.stroke();
  });
  lane.state.fields.forEach((field) => drawComparisonField(laneContext, field, width, height));
  drawComparisonClearance(laneContext, projectedRows, width, height);
  projectedRows.forEach((row) => drawComparisonMember(laneContext, row, lane.state, width, height));
  laneCanvas.setAttribute(
    "aria-label",
    `${lane.descriptor.label}, seed ${lane.state.seed}, tick ${lane.state.tick}: ${lane.state.behavior_counts.flock} Flock, ${lane.state.behavior_counts.cohere} Cohere, ${lane.state.behavior_counts.disperse} Disperse; ${lane.state.execution_settings.collision_policy}, minimum surface clearance ${lane.state.clearance_metrics.minimum_surface_clearance.toFixed(3)}, ${lane.state.clearance_metrics.overlap_pair_count} overlaps, ${lane.state.clearance_metrics.near_miss_pair_count} near misses, ${lane.state.clearance_metrics.total_intervention_count} collision interventions; ${lane.state.fields.length} fields and ${lane.state.leases.length} leases.`
  );
}

function drawComparisonNavigation(laneContext, field, width, height) {
  if (field === null) {
    return;
  }
  const x = ((field.x + 1) / 2) * width;
  const y = ((1 - field.y) / 2) * height;
  const radiusX = field.radius * width / 2;
  const radiusY = field.radius * height / 2;
  const length = Math.min(radiusX, radiusY) * 0.7;
  laneContext.save();
  laneContext.fillStyle = "#276d6518";
  laneContext.strokeStyle = "#276d65";
  laneContext.lineWidth = 2.5;
  laneContext.setLineDash([8, 6]);
  laneContext.beginPath();
  laneContext.ellipse(x, y, radiusX, radiusY, 0, 0, Math.PI * 2);
  laneContext.fill();
  laneContext.stroke();
  laneContext.setLineDash([]);
  laneContext.beginPath();
  laneContext.moveTo(x, y);
  laneContext.lineTo(x + field.direction_x * length, y - field.direction_y * length);
  laneContext.stroke();
  laneContext.restore();
}

function drawComparisonClearance(laneContext, rowsForLane, width, height) {
  for (let firstIndex = 0; firstIndex < rowsForLane.length; firstIndex += 1) {
    const first = rowsForLane[firstIndex];
    for (let secondIndex = firstIndex + 1; secondIndex < rowsForLane.length; secondIndex += 1) {
      const second = rowsForLane[secondIndex];
      const clearance = Math.hypot(first[1] - second[1], first[2] - second[2]) - first[3] - second[3];
      if (clearance > 0.03) {
        continue;
      }
      laneContext.save();
      laneContext.strokeStyle = clearance < 0 ? "#8f332d" : "#8a6a24";
      laneContext.lineWidth = clearance < 0 ? 4 : 1.5;
      laneContext.setLineDash(clearance < 0 ? [] : [4, 5]);
      laneContext.beginPath();
      laneContext.moveTo(((first[1] + 1) / 2) * width, ((1 - first[2]) / 2) * height);
      laneContext.lineTo(((second[1] + 1) / 2) * width, ((1 - second[2]) / 2) * height);
      laneContext.stroke();
      laneContext.restore();
    }
  }
}

function drawComparisonField(laneContext, field, width, height) {
  const x = ((field.x + 1) / 2) * width;
  const y = ((1 - field.y) / 2) * height;
  laneContext.save();
  laneContext.translate(x, y);
  laneContext.strokeStyle = "#6f362b";
  laneContext.lineWidth = 3;
  laneContext.beginPath();
  laneContext.arc(0, 0, 22, 0, Math.PI * 2);
  laneContext.stroke();
  laneContext.fillStyle = "#1e1713";
  laneContext.font = "700 14px Aptos, Candara, sans-serif";
  laneContext.textAlign = "center";
  laneContext.textBaseline = "middle";
  laneContext.fillText(`${CONTRIBUTOR_LABELS[field.contributor_id]}${field.polarity === "attract" ? "+" : "−"}`, 0, 1);
  laneContext.restore();
}

function drawComparisonMember(laneContext, row, laneState, width, height) {
  const memberId = Math.round(row[0]);
  const behavior = Math.round(row[10]);
  const groupId = Math.round(row[11]);
  const x = ((row[1] + 1) / 2) * width;
  const y = ((1 - row[2]) / 2) * height;
  const radius = Math.max(6, row[3] * width * 0.5);
  laneContext.save();
  laneContext.translate(x, y);
  laneContext.strokeStyle = GROUP_COLORS[groupId] || "#315d6c";
  laneContext.lineWidth = 2;
  laneContext.beginPath();
  laneContext.arc(0, 0, radius + 7, 0, Math.PI * 2);
  laneContext.stroke();
  laneContext.fillStyle = ["#d9a86c", "#9aae8f", "#d59b87"][behavior];
  laneContext.strokeStyle = "#1e1713";
  laneContext.lineWidth = 2;
  drawComparisonShape(laneContext, radius, behavior);
  laneContext.fill();
  laneContext.stroke();
  laneContext.fillStyle = "#1e1713";
  laneContext.font = "600 12px Aptos, Candara, sans-serif";
  laneContext.textAlign = "center";
  laneContext.textBaseline = "middle";
  laneContext.fillText(String(memberId + 1), 0, 1);
  const lease = laneState.leases.find((candidate) => candidate.member_id === memberId);
  if (lease) {
    laneContext.font = "700 11px Aptos, Candara, sans-serif";
    laneContext.fillText(`L:${OPERATOR_LABELS[lease.holder_operator_id]}`, 0, radius + 19);
  }
  laneContext.restore();
}

function drawComparisonShape(laneContext, radius, behavior) {
  laneContext.beginPath();
  if (behavior === 1) {
    for (let side = 0; side < 6; side += 1) {
      const angle = -Math.PI / 2 + (side * Math.PI) / 3;
      const x = Math.cos(angle) * radius;
      const y = Math.sin(angle) * radius;
      if (side === 0) {
        laneContext.moveTo(x, y);
      } else {
        laneContext.lineTo(x, y);
      }
    }
    laneContext.closePath();
  } else if (behavior === 2) {
    laneContext.moveTo(0, -radius);
    laneContext.lineTo(radius, 0);
    laneContext.lineTo(0, radius);
    laneContext.lineTo(-radius, 0);
    laneContext.closePath();
  } else {
    laneContext.arc(0, 0, radius, 0, Math.PI * 2);
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
  if (comparisonModeActive) {
    if (animationHandle !== 0) {
      cancelAnimationFrame(animationHandle);
      animationHandle = 0;
    }
    return;
  }
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
  if (comparisonModeActive || !state.running) {
    return;
  }
  if (previousTimestamp === 0) {
    previousTimestamp = timestamp;
  }
  const elapsed = Math.min(timestamp - previousTimestamp, 100);
  previousTimestamp = timestamp;

  if (reducedMotion.matches) {
    if (timestamp - reducedTimestamp >= 250) {
      const authorityBefore = state;
      engine.advance(16);
      reducedTimestamp = timestamp;
      state = JSON.parse(engine.state_json());
      traceAnimatedLeaseExpiry(authorityBefore);
      rows = engine.frame_rows();
      updateDomState(false);
      draw();
    }
  } else {
    const authorityBefore = state;
    engine.advance(elapsed);
    state = JSON.parse(engine.state_json());
    traceAnimatedLeaseExpiry(authorityBefore);
    rows = engine.frame_rows();
    updateDomState(false);
    draw();
  }
  animationHandle = requestAnimationFrame(animate);
}

function traceAnimatedLeaseExpiry(before) {
  if (before.authority_revision === state.authority_revision) {
    return;
  }
  document.querySelector("#lease-trace-before").textContent = authoritySummary(before);
  document.querySelector("#lease-trace-action").textContent = "fixed_step_lease_expiry";
  document.querySelector("#lease-trace-policy").textContent =
    "Deterministic expiry at the exclusive fixed-step boundary; no wall clock or hidden arbitration";
  document.querySelector("#lease-trace-receipt").textContent =
    `fixed_step_expiry · authority ${state.authority_revision}`;
  document.querySelector("#lease-trace-after").textContent = authoritySummary(state);
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
  document.querySelector("#state-dynamics-mode").textContent = dynamicsModeLabel();
  document.querySelector("#state-dynamics-rates").textContent =
    `A ${state.dynamics_rates.alignment.toFixed(2)} · C ${state.dynamics_rates.cohesion.toFixed(2)} · S ${state.dynamics_rates.separation.toFixed(2)}`;
  document.querySelector("#state-semantic-qualities").textContent =
    `S ${state.semantic_qualities.space.toFixed(2)} · T ${state.semantic_qualities.time.toFixed(2)} · W ${state.semantic_qualities.weight.toFixed(2)} · F ${state.semantic_qualities.flow.toFixed(2)}`;
  document.querySelector("#state-group-count").textContent = `${state.groups.length} of ${MAX_MORPHOLOGY_GROUPS}`;
  document.querySelector("#state-morphology-revision").textContent = String(state.morphology_revision);
  document.querySelector("#state-lease-count").textContent = `${state.leases.length} of ${MAX_ACTIVE_LEASES}`;
  document.querySelector("#state-authority-revision").textContent = String(state.authority_revision);
  document.querySelector("#state-relations").textContent = String(relationTotal);
  document.querySelector("#state-field-count").textContent = `${state.fields.length} of ${MAX_PERSONAL_FIELDS}`;
  document.querySelector("#state-contributor-count").textContent = `${state.active_contributor_count} of ${CONTRIBUTOR_LABELS.length}`;
  document.querySelector("#state-collision-policy").textContent = state.execution_settings.collision_policy === "collision_free"
    ? "Collision-free rendered discs"
    : "Soft avoidance; overlap permitted";
  document.querySelector("#state-navigation-field").textContent = state.navigation_field === null
    ? "None"
    : `Centre ${state.navigation_field.x.toFixed(2)}, ${state.navigation_field.y.toFixed(2)} · direction ${state.navigation_field.direction_x.toFixed(2)}, ${state.navigation_field.direction_y.toFixed(2)} · radius ${state.navigation_field.radius.toFixed(2)}`;
  document.querySelector("#state-replay-events").textContent =
    `${state.replay_event_count} events / ${state.replay_step_count} steps`;
  document.querySelector("#state-checkpoint-count").textContent = `${savedCheckpoints.size} of ${MAX_CHECKPOINTS}`;
  document.querySelector("#metric-cohesion").textContent = metrics.cohesion.toFixed(3);
  document.querySelector("#metric-polarization").textContent = metrics.polarization.toFixed(3);
  document.querySelector("#metric-spacing").textContent = metrics.nearestSpacing.toFixed(3);
  document.querySelector("#metric-speed").textContent = state.average_speed.toFixed(3);
  document.querySelector("#metric-subgroup").textContent = String(state.subgroup_members.length);
  document.querySelector("#metric-distribution").textContent =
    `${state.behavior_counts.flock} / ${state.behavior_counts.cohere} / ${state.behavior_counts.disperse}`;
  document.querySelector("#flow-alignment-rate").textContent = state.dynamics_rates.alignment.toFixed(2);
  document.querySelector("#flow-cohesion-rate").textContent = state.dynamics_rates.cohesion.toFixed(2);
  document.querySelector("#flow-separation-rate").textContent = state.dynamics_rates.separation.toFixed(2);
  document.querySelector("#flow-distribution").textContent =
    `${state.behavior_counts.flock} / ${state.behavior_counts.cohere} / ${state.behavior_counts.disperse}`;
  document.querySelector("#metric-relations").textContent = String(relationTotal);
  document.querySelector("#metric-fields").textContent = String(state.fields.length);
  document.querySelector("#metric-groups").textContent = String(state.groups.length);
  document.querySelector("#metric-group-sizes").textContent = state.groups
    .map((group) => group.member_ids.length)
    .join(" / ");
  document.querySelector("#metric-formation-extent").textContent =
    (state.groups.reduce((total, group) => total + group.formation_extent, 0) / state.groups.length).toFixed(3);
  document.querySelector("#metric-leases").textContent = String(state.leases.length);
  document.querySelector("#metric-pending-handoffs").textContent = String(
    state.leases.filter((lease) => lease.pending_handoff_to !== null).length
  );
  document.querySelector("#metric-lease-remaining").textContent = state.leases.length === 0
    ? "None"
    : `${Math.min(...state.leases.map((lease) => lease.remaining_steps))} fixed steps`;
  document.querySelector("#metric-min-clearance").textContent =
    state.clearance_metrics.minimum_surface_clearance.toFixed(3);
  document.querySelector("#metric-overlaps").textContent =
    String(state.clearance_metrics.overlap_pair_count);
  document.querySelector("#metric-near-misses").textContent =
    String(state.clearance_metrics.near_miss_pair_count);
  document.querySelector("#metric-latest-interventions").textContent =
    String(state.clearance_metrics.last_step_intervention_count);
  document.querySelector("#metric-total-interventions").textContent =
    String(state.clearance_metrics.total_intervention_count);
  document.querySelector("#metric-contact-ticks").textContent =
    String(state.clearance_metrics.contact_tick_count);
  document.querySelector("#step-button").disabled = state.running;
  document.querySelector("#start-button").disabled = state.running;
  document.querySelector("#pause-button").disabled = !state.running;
  renderResolvedDynamics();
  renderMorphologyRoster();
  renderLeaseRoster();
  updateHistoryControls();
  updateFieldControls();
  canvas.setAttribute(
    "aria-label",
    `${state.running ? "Running" : "Paused"} synthetic swarm. ${scopeLabel(state.scope)} targets ${memberList(state.target_members)}. ${behaviorMixLabel()}. ${dynamicsModeLabel()} resolve alignment ${state.resolved_dynamics.rates.alignment.toFixed(2)}, cohesion ${state.resolved_dynamics.rates.cohesion.toFixed(2)}, Disperse entry ${state.resolved_dynamics.rates.separation.toFixed(2)}, speed scale ${state.resolved_dynamics.speed_scale.toFixed(2)}, damping ${state.resolved_dynamics.damping.toFixed(2)}, and jitter ${state.resolved_dynamics.jitter.toFixed(2)}. ${state.execution_settings.collision_policy === "collision_free" ? "Collision-free disc execution" : "Soft avoidance with overlap permitted"}; minimum surface clearance ${state.clearance_metrics.minimum_surface_clearance.toFixed(3)}, ${state.clearance_metrics.overlap_pair_count} overlapping pairs, ${state.clearance_metrics.near_miss_pair_count} near misses. ${state.navigation_field === null ? "No navigation field" : "One directional navigation field"}. ${state.fields.length} additive personal fields. ${morphologySummary(state)}. ${authoritySummary(state)}`
  );
  updateMotionMode();

  if (updateControls) {
    const activeScope = controls.querySelector(`input[name="scope"][value="${state.scope}"]`);
    if (activeScope) {
      activeScope.checked = true;
    }
    seedInput.value = state.seed;
    updateDynamicsControls();
    updateExecutionControls();
    updateNavigationOutputs();
    updateSemanticControls();
    updateMorphologyControls();
  }
  updateLeaseControls();
}

function renderMorphologyRoster() {
  const fragment = document.createDocumentFragment();
  state.groups.forEach((group) => {
    const item = document.createElement("li");
    item.textContent = `Group ${group.group_id + 1}: ${group.member_ids.length} members (${memberList(group.member_ids)}); scale ${group.formation_scale.toFixed(2)}; observed extent ${group.formation_extent.toFixed(3)}.`;
    fragment.append(item);
  });
  document.querySelector("#morphology-group-roster").replaceChildren(fragment);
}

function renderLeaseRoster() {
  const list = document.querySelector("#lease-roster");
  if (state.leases.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No active leases.";
    list.replaceChildren(empty);
    return;
  }
  const fragment = document.createDocumentFragment();
  state.leases.forEach((lease) => {
    const item = document.createElement("li");
    const handoff = lease.pending_handoff_to === null
      ? "no pending handoff"
      : `handoff offered to Operator ${OPERATOR_LABELS[lease.pending_handoff_to]}`;
    item.textContent = `Member ${lease.member_id + 1}: holder Operator ${OPERATOR_LABELS[lease.holder_operator_id]}, acquired tick ${lease.acquired_at_tick}, expires before tick ${lease.expires_at_tick}, ${lease.remaining_steps} fixed steps remain, ${handoff}.`;
    fragment.append(item);
  });
  list.replaceChildren(fragment);
}

function updateFieldControls() {
  const selectedValue = fieldSelect.value;
  const fragment = document.createDocumentFragment();
  const empty = document.createElement("option");
  empty.value = "";
  empty.textContent = state.fields.length === 0 ? "No active fields" : "Choose an active field";
  fragment.append(empty);
  state.fields.forEach((field) => {
    const option = document.createElement("option");
    option.value = String(field.field_id);
    const lifetime = field.remaining_steps === null
      ? "persistent"
      : `${field.remaining_steps} steps left`;
    option.textContent = `Field ${field.field_id + 1} · Contributor ${CONTRIBUTOR_LABELS[field.contributor_id]} · ${field.polarity} · ${lifetime}`;
    fragment.append(option);
  });
  fieldSelect.replaceChildren(fragment);
  fieldSelect.value = state.fields.some((field) => String(field.field_id) === selectedValue)
    ? selectedValue
    : "";
  const hasSelection = selectedFieldId() !== null;
  document.querySelector("#move-field-button").disabled = !hasSelection;
  document.querySelector("#polarity-field-button").disabled = !hasSelection;
  document.querySelector("#remove-field-button").disabled = !hasSelection;
  document.querySelector("#place-field-button").disabled = state.fields.length >= MAX_PERSONAL_FIELDS;
  renderFieldStateList();
  updateFieldOutputs();
}

function renderFieldStateList() {
  const list = document.querySelector("#field-state-list");
  if (state.fields.length === 0) {
    const empty = document.createElement("li");
    empty.textContent = "No active personal fields.";
    list.replaceChildren(empty);
    return;
  }
  const fragment = document.createDocumentFragment();
  state.fields.forEach((field) => {
    const item = document.createElement("li");
    const lifetime = field.remaining_steps === null
      ? "persistent until removal or reset"
      : `expires in ${field.remaining_steps} fixed steps`;
    item.textContent = `Field ${field.field_id + 1}: synthetic contributor ${CONTRIBUTOR_LABELS[field.contributor_id]}, ${field.polarity}, position ${field.x.toFixed(2)}, ${field.y.toFixed(2)}, ${lifetime}.`;
    fragment.append(item);
  });
  list.replaceChildren(fragment);
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
    const member = members.get(memberId);
    button.setAttribute("aria-pressed", String(state.primary_member === memberId));
    const leaseLabel = (member?.lease_holder_operator_id ?? null) === null
      ? "unleased"
      : `leased by synthetic Operator ${OPERATOR_LABELS[member?.lease_holder_operator_id]}`;
    button.setAttribute("aria-label", `Select member ${memberId + 1} as primary; morphology Group ${(member?.group_id ?? 0) + 1}; ${leaseLabel}`);
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
  drawNavigationField(width, height);
  drawPersonalFields(width, height);
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
  drawClearanceWarnings(projectedRows, width, height);
  projectedRows.forEach((row) => drawMember(row, width, height, !wholeSwarmTargeted));
}

function drawNavigationField(width, height) {
  const field = state.navigation_field;
  if (field === null) {
    return;
  }
  const x = ((field.x + 1) / 2) * width;
  const y = ((1 - field.y) / 2) * height;
  const radiusX = field.radius * width / 2;
  const radiusY = field.radius * height / 2;
  const arrowLength = Math.min(radiusX, radiusY) * 0.72;
  const directionX = field.direction_x;
  const directionY = -field.direction_y;
  const tipX = x + directionX * arrowLength;
  const tipY = y + directionY * arrowLength;
  const angle = Math.atan2(directionY, directionX);
  context.save();
  context.fillStyle = "#276d6518";
  context.strokeStyle = "#276d65";
  context.lineWidth = 3;
  context.setLineDash([10, 7]);
  context.beginPath();
  context.ellipse(x, y, radiusX, radiusY, 0, 0, Math.PI * 2);
  context.fill();
  context.stroke();
  context.setLineDash([]);
  context.beginPath();
  context.moveTo(x, y);
  context.lineTo(tipX, tipY);
  context.lineTo(tipX - Math.cos(angle - Math.PI / 6) * 18, tipY - Math.sin(angle - Math.PI / 6) * 18);
  context.moveTo(tipX, tipY);
  context.lineTo(tipX - Math.cos(angle + Math.PI / 6) * 18, tipY - Math.sin(angle + Math.PI / 6) * 18);
  context.stroke();
  context.fillStyle = "#24444b";
  context.font = "700 16px Aptos, Candara, sans-serif";
  context.fillText("Navigation", x + 8, y - 10);
  context.restore();
}

function drawClearanceWarnings(projectedRows, width, height) {
  for (let firstIndex = 0; firstIndex < projectedRows.length; firstIndex += 1) {
    const first = projectedRows[firstIndex];
    for (let secondIndex = firstIndex + 1; secondIndex < projectedRows.length; secondIndex += 1) {
      const second = projectedRows[secondIndex];
      const centreDistance = Math.hypot(first[1] - second[1], first[2] - second[2]);
      const clearance = centreDistance - first[3] - second[3];
      if (clearance > 0.03) {
        continue;
      }
      context.save();
      context.strokeStyle = clearance < 0 ? "#8f332d" : "#8a6a24";
      context.lineWidth = clearance < 0 ? 5 : 2;
      context.setLineDash(clearance < 0 ? [] : [4, 5]);
      context.beginPath();
      context.moveTo(((first[1] + 1) / 2) * width, ((1 - first[2]) / 2) * height);
      context.lineTo(((second[1] + 1) / 2) * width, ((1 - second[2]) / 2) * height);
      context.stroke();
      context.restore();
    }
  }
}

function drawPersonalFields(width, height) {
  state.fields.forEach((field) => {
    const x = ((field.x + 1) / 2) * width;
    const y = ((1 - field.y) / 2) * height;
    const color = CONTRIBUTOR_COLORS[field.contributor_id] || "#463b69";
    context.save();
    context.translate(x, y);
    context.strokeStyle = color;
    context.fillStyle = `${color}22`;
    context.lineWidth = 4;
    context.setLineDash(field.remaining_steps === null ? [] : [7, 5]);
    context.beginPath();
    context.arc(0, 0, 27, 0, Math.PI * 2);
    context.fill();
    context.stroke();
    context.setLineDash([]);
    context.fillStyle = color;
    context.font = "700 18px Aptos, Candara, sans-serif";
    context.textAlign = "center";
    context.textBaseline = "middle";
    const polarity = field.polarity === "attract" ? "+" : "−";
    context.fillText(`${CONTRIBUTOR_LABELS[field.contributor_id]}${polarity}`, 0, 1);
    context.restore();
  });
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
  const groupId = Math.round(row[11]);

  context.save();
  context.translate(x, y);

  context.strokeStyle = GROUP_COLORS[groupId] || "#315d6c";
  context.lineWidth = 3;
  context.setLineDash([]);
  context.beginPath();
  context.arc(0, 0, radius + 12, 0, Math.PI * 2);
  context.stroke();

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
