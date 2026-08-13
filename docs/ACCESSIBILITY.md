# Accessibility

The canvas is a visual projection, not the sole interface or state authority.

## Interaction

- Motion begins only after an intentional Start action.
- Pause, Step, Reset, and seeded restart remain available at all times.
- Pointer selection on the canvas and keyboard-operable member buttons dispatch
  the same selection actions.
- Slower and Faster buttons provide a low-rate, non-drag action route.
- Flock, Cohere, and Disperse buttons assign collective steering without a
  timed gesture or continuous pointer movement.
- Scope controls are ordinary radio inputs and remain usable while paused.
- Native range inputs, selects, and buttons provide a synthetic equivalent for
  placing, moving, changing polarity, and removing personal fields without
  device motion or gesture hardware.
- Native range inputs expose the three labelled raw dynamics rates. Arrow keys
  provide the same bounded semantic actions as pointer adjustment, and the
  deterministic replay tape is the synthetic route for exact repetition.
- Four native range inputs expose Space, Time, Weight, and Flow without a
  camera, pose tracker, timed gesture, or continuous pointing requirement.
  Arrow keys and deterministic replay are equivalent synthetic routes.
- Split and merge use labelled native group selects and buttons. Formation
  scale uses a native bounded range input plus an explicit apply button. The
  deterministic replay tape is the equivalent route for exact repetition.
- Lease authority uses labelled native operator/member/receiver selects,
  request/release/offer/accept/decline buttons, a fixed-step lifetime range,
  and one holder-gated behavior select. Unavailable commands are disabled and
  accompanied by an ordinary-text reason.
- The member grid exposes primary and subgroup selection without requiring
  precise canvas pointing.
- Catalogue facets are labelled native selects with one clear-filter action.
- Atlas entries are ordinary buttons with pressed state; planned mechanisms
  never masquerade as enabled controls.
- Save, retrieve, reset, and replay are ordinary buttons with a labelled native
  select. Five session-local checkpoint names avoid precision, drag, or timing
  requirements.
- Comparison mode uses a labelled native scenario select and decimal seed input.
  Start, pause, one-event step, reset-both, full replay, and return controls are
  ordinary keyboard-operable buttons. Reduced motion slows automatic event
  pacing while both lanes still update in lockstep.

## Perception

- Primary, subgroup, and current-target states use labels and geometry as well
  as color.
- Flock, Cohere, and Disperse use circle, hexagon, and diamond member shapes,
  reinforced by textual per-member abbreviations and the collective-rule
  summary. Rule identity never depends on color alone.
- Neighbour lines expose the local relation graph visually; the current link
  count is also presented as text.
- Current scope, target members, collective-rule distribution, tick, seed, and
  run state are present in the DOM.
- The input, semantic-action, policy, receipt, and quantitative traces have
  complete non-canvas representations.
- Replay event/step counts, saved-checkpoint count, and the latest 50 semantic
  or history operations remain visible as structured text.
- Every active field exposes its synthetic contributor label, normalized
  position, polarity, and remaining or persistent lifetime as structured text;
  field identity never depends on canvas color.
- Alignment, cohesion, and separation rates plus the live Flock/Cohere/Disperse
  distribution are ordinary text; parameter consequences also remain visible
  through cohesion, polarization, spacing, and relation metrics.
- Semantic values, active raw-or-semantic control authority, and the resolved
  alignment, cohesion, separation, speed-scale, damping, and jitter vector are
  structured text. The native expandable inspector can be opened or closed
  without dispatching an action or changing simulation state.
- Canonical group IDs, complete member rosters, counts, formation scales,
  observed extents, before/after action traces, and morphology revisions are
  structured text. Canvas group rings are supplemental rather than the only
  membership signal.
- Every active lease exposes its canonical member, synthetic holder,
  acquisition/expiry ticks, remaining fixed steps, pending receiver, and
  authority revision as text. Lease actions and automatic expiry have
  before/action/policy/receipt/after traces outside the canvas.
- The canvas has a concise text description and is never focus-required.
- Focus indicators are explicit; forced-colors and 200% zoom layouts are
  supported.
- Each comparison lane has a heading, configuration ID, catalogue/source card,
  evidence status, transfer boundary, independent non-claim, six-value resolved
  vector, final-state revisions, field/lease provenance, and complete input →
  semantic action → policy → receipt trace. A semantic table gives both values,
  the Lane B minus Lane A delta, definition, and unit; color and canvas are never
  required to distinguish lanes or policy outcomes.

## Motion and announcements

- `prefers-reduced-motion` limits the running simulation to a low update rate
  and disables decorative transitions.
- Simulation frames do not update an ARIA live region or append history rows.
- The live status changes only after semantic actions, validation errors, or
  major run-state transitions.

## Browser verification

Desktop and mobile checks cover keyboard selection/action, pointer selection,
scope-aware collective rules, field placement/motion/polarity/lifetime,
raw and semantic dynamics sliders, resolved-vector and mode-distribution
consequences, split/merge/rescale controls, group rosters and metrics,
lease acquisition/release/handoff/expiry, consent states and disabled reasons,
inspection without state drift, focus visibility,
reduced-motion state, intentional start, pause, single-step, reset/replay, and
horizontal overflow. Comparison checks cover equivalent and intentionally
different raw/semantic vectors, superposition/lease rejection provenance,
lockstep start/pause/step/reset/replay, ordinary-atlas preservation, evidence
cards, trace disclosure, the metrics table, mobile single-column lanes,
forced-colors borders, and zero diagnostics.
