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
- The member grid exposes primary and subgroup selection without requiring
  precise canvas pointing.
- Catalogue facets are labelled native selects with one clear-filter action.
- Atlas entries are ordinary buttons with pressed state; planned mechanisms
  never masquerade as enabled controls.
- Save, retrieve, reset, and replay are ordinary buttons with a labelled native
  select. Five session-local checkpoint names avoid precision, drag, or timing
  requirements.

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
- The canvas has a concise text description and is never focus-required.
- Focus indicators are explicit; forced-colors and 200% zoom layouts are
  supported.

## Motion and announcements

- `prefers-reduced-motion` limits the running simulation to a low update rate
  and disables decorative transitions.
- Simulation frames do not update an ARIA live region or append history rows.
- The live status changes only after semantic actions, validation errors, or
  major run-state transitions.

## Browser verification

Desktop and mobile checks cover keyboard selection/action, pointer selection,
scope-aware collective rules, focus visibility, reduced-motion state,
intentional start, pause, single-step, reset, and horizontal overflow.
