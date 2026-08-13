# Architecture

## Decision

Use one deterministic Rust core with semantic actions, a thin WebAssembly
adapter, semantic HTML controls, and Canvas2D. Build the public atlas by adding
input adapters, app-owned policy, and literature-bound configurations around
that core rather than duplicating simulations. Keep the product a direct
`fast` package. Do not add a `morphospace/` workspace until a real cross-owner
composition or activation decision exists.

## Scope

The atlas shell exposes seven allowlisted interaction mappings and their exact
evidence and transfer boundaries. Implemented mechanisms demonstrate
target-independent collective steering and speed actions over three scopes:
one selected member, a selected subgroup, or the whole swarm. `Flock`
maintains local alignment and spacing, `Cohere` steers toward peers sharing
that rule, and `Disperse` steers away from matching peers or the swarm centre.
Additive personal fields separately demonstrate app-local synthetic
contributor provenance, attract/repel polarity, persistent/expiring lifetime,
and order-independent superposition. Synthetic contributors are not accounts,
people, durable identity, or live multi-user authority.
Raw dynamics controls expose three swarm-wide rates for entering the existing
Flock, Cohere, and Disperse modes. Each rate is bounded to 0.00–1.00 transition
per member-second and defaults to 0.00, so this mechanism does not alter the
accepted scene until explicitly activated. It is an app-owned technical
analogy to endogenous transition controls, not a reproduction of the source's
exploration/task/other robot-state model.
Semantic dynamics controls expose bounded 0–1 Space, Time, Weight, and Flow
qualities. The source projection supplies qualitative directions and
couplings, but no portable coefficients, so the app uses documented linear
interpolation between app-owned endpoints. Space couples alignment and
separation, Time scales speed, Weight couples cohesion, and Flow couples
damping and deterministic jitter. Semantic and raw controls select the owner
of one explicit `ResolvedDynamics` vector consumed by the established core;
they never select different simulation implementations.
Morphology actions explicitly split, merge, and rescale canonical app-owned
groups. IDs are bounded to 0–7, member rosters remain sorted and exhaustive,
split retains its source ID while assigning the smallest unused ID, and merge
retains the lower participating ID and its scale. Formation scale is a separate
0.50–2.00 radial target with neutral default 1.00; it does not rewrite the
resolved dynamics vector. Morphology groups are independent from the existing
target subgroup, fields, contributor provenance, behavior assignment, and
dynamics authority.
Lease authority is a separate app-local state machine over canonical member
IDs and four fixed synthetic operator channels. At most eight leases can be
active. Request acquires only an unheld member for 1–600 fixed steps, exact
holder release is explicit, and fixed-step expiry is deterministic. Handoff is
a two-action offer plus named-receiver accept/decline; acceptance changes only
the holder and preserves the original expiry. One holder-gated behavior action
makes authority consequences visible without reusing target-scope policy.
Comparison mode is atlas-wide infrastructure rather than an eighth literature
entry. A strict immutable comparison specification binds one canonical decimal
seed, equal left/right seed copies, one scenario ID, and the exact versioned
normalized-input tape ID. The runner clones one canonical `DemoCore` into two
owned lane values, applies each input event transactionally to temporary lane
clones, verifies seed and tick lockstep, then commits both or neither. The lanes
share no mutable core, RNG progress, replay recorder, fields, leases, morphology,
logs, metrics, or browser checkpoint registry.

The equivalent raw/semantic profile uses one fixed comparison-only raw-vector
mirror of the semantic midpoint. It has no caller-supplied coefficients and is
not arbitrary parameter plumbing. The contrasting profile uses the established
three raw rate actions and four semantic-quality actions. The authority profile
maps the same two normalized synthetic influence requests either into two
order-stable additive fields or an exclusive lease where the second operator's
request and use fail closed. All profiles continue through the same reducer and
simulation path.

## Non-scope

This repository does not own scholarly evidence, a universal gesture or input
vocabulary, multi-user authority, sessions, networking, participant data,
robot deployment, XR relations, or generic ecosystem contracts.

## Authority

| Surface | Owner |
| --- | --- |
| Sources, evidence, and private mapping catalog | Private research memory |
| Public synthesis and site navigation | MesmerPrism.github.io |
| Generic particle payload contract | Rusty Matter |
| Scene semantics, reducer, deterministic replay, local checkpoints, palette, and browser behavior | This app |
| DOM accessibility and Canvas2D projection | Browser adapter in this app |
| Future tracked-space relation | Rusty Lattice, only when required |
| Future accepted multi-user sessions, peers, or leases | Rusty Manifold, only when required |
| Future Quest/OpenXR hosting | A separate Quest adapter over `demo-core` |

## Interfaces

```text
pointer / keyboard / switch-style button / replay / optional live adapters
        |
        v
NormalizedInput (route-specific, bounded)
        |
        v
SemanticAction (route-free intent)
        |
        v
scope and authority policy -- selection revision / resolved targets
field combination policy -- bounded additive superposition / expiry
raw dynamics policy -- bounded global rates / deterministic weighted selection
semantic dynamics policy -- bounded qualities / inspectable app-owned interpolation
morphology policy -- canonical group IDs / conservation / formation scale
lease policy -- exact holder / named receiver consent / fixed-step expiry
comparison policy -- immutable scenario / isolated lanes / lockstep tape
        |
        v
fixed-step deterministic flock state
        |
        v
Matter ParticleRenderPayload
        |
        v
bounded Float32 frame rows + semantic DOM summary
        |
        v
Canvas2D
```

`demo-core` uses fixed-step updates and seeded SplitMix64 initialization. It
keeps selection revision separate from state revision so actions built against
stale target selections fail closed. The browser never sends input-modality
names into the reducer.

Personal-field actions are likewise route-free: place, move, set polarity, and
remove. The reducer validates field ID, synthetic contributor channel,
normalized position, lifetime, and the eight-field bound. Active fields remain
sorted by stable ID before acceleration is summed, so placement order does not
change the outcome. Fixed-step expiry, reset, and replay use the same core path.

Raw dynamics actions are three explicit route-free setters: alignment,
cohesion, and separation. There is no generic parameter map or exposed
randomness control. At each fixed step, a pure seed/tick/member draw applies
the combined per-second rate and chooses one existing collective mode by the
three weights. This makes parameter order irrelevant while keeping same-seed
state transitions, checkpoints, and replay exact.

Semantic dynamics actions are four explicit route-free setters: Space, Time,
Weight, and Flow. Updating any one quality resolves the complete current
semantic profile into the same raw transition rates and the minimal effective
speed-scale, damping, and deterministic-jitter additions. Switching a raw rate
returns those additions to neutral and gives raw controls authority. Snapshot
validation recomputes the vector from its owner and rejects mismatches, so no
hidden translated state can drift across inspection, checkpoint, or replay.
Jitter is a pure seed/tick/member stream, not runtime entropy. The browser's
expandable inspector dispatches no action and therefore cannot mutate state.

The core also owns a strict `combinatorial.swarmability.replay.v1` tape. It
records accepted semantic actions plus the fixed-step counts actually executed
by browser elapsed-time updates. Replay starts from the tape's seed and runs
those same reducer and simulation paths. The tape rejects unknown fields,
unsupported schemas, zero or excessive step counts, rejected actions, and
oversized event histories. Restored snapshots do not silently acquire replay
provenance.

The browser owns only session-local checkpoint names, up to five replay-tape
copies, and a 50-entry readable operation log. Those controls add no durable
storage, account, network, or multi-user authority.

Morphology actions carry their own expected revision so stale group operations
fail closed independently of target selection. `SplitGroup` uses one explicit
alternating ascending-member-ID rule. `MergeGroups` names both participants and
the canonical survivor; operand order is intentionally commutative only for the
same exact pair. `SetFormationScale` names one existing group. Unknown,
duplicate, noncanonical, singleton, over-limit, stale, nonfinite, and
out-of-range requests are rejected without changing state. Split and merge do
not clear unrelated mechanisms, and all three actions use ordinary snapshot,
checkpoint, reset, and replay paths.

Every lease mutation and holder-gated use carries a monotonically increasing
authority revision. Stale, unknown-member/operator, duplicate acquisition,
non-holder release/use/offer, self-handoff, mismatched receiver, excessive
lifetime, over-cap, expired-lease use, and replayed mutation fail closed.
Expiry increments the same authority revision exactly when the fixed tick
reaches the exclusive expiry boundary. Leases remain attached to canonical
members through group split/merge/rescale; those actions neither transfer nor
clear authority. Reset clears leases and increments the revision. Snapshot and
replay preserve acquisition tick, exclusive expiry, remaining lifetime,
pending consent, holder, and revision exactly.

The comparison contract uses
`combinatorial.swarmability.comparison-spec.v1`,
`combinatorial.swarmability.normalized-input-tape.v1`, versioned step receipts,
and `combinatorial.swarmability.comparison-result.v1`. Specifications reject
unknown fields, unsupported schemas, noncanonical or mismatched seeds, and tape
IDs that do not belong to the selected profile. Input events and lane receipts
are bounded. Reset reconstructs the canonical start; replay applies the complete
immutable tape again. The browser owns only pause/run pacing and never supplies
elapsed time to comparison cores.

The Wasm adapter exposes one fixed-size row per member:

```text
[member_id, x, y, radius, vx, vy, speed,
 primary_selected, subgroup_selected, currently_targeted, behavior_code,
 morphology_group_id]
```

Rows are reconstructed from Matter's `ParticleRenderPayload` plus app-owned
selection and collective-rule projection. No DOM handles, renderer resources, colors, JavaScript
objects, endpoints, or private metadata enter the core contract.

## Observability

Every semantic action returns a bounded receipt with acceptance, a stable code,
changed member IDs, and current state/selection revisions. The DOM shows the
current scope, target count, selected members, collective-rule distribution,
visible neighbour-link count, tick, seed, run state, and most recent action.
It also shows the latest input route, normalized input, semantic action, policy
resolution, core receipt, cohesion, polarization, nearest spacing, speed, and
subgroup membership. Active field count, contributor count, position,
polarity, and lifetime also have non-canvas projections. The three raw rates
and current Flock/Cohere/Disperse distribution are text. Semantic quality
values, active control authority, all six resolved quantities, and the
source-supported/app-owned coupling distinction are also ordinary DOM text,
while canonical group rosters, member counts, formation scales, morphology
revision, before/after receipts, and observed formation extent expose morphology
without relying on the canvas. Active lease holder, acquisition/expiry tick,
remaining lifetime, pending receiver consent, authority revision, disabled-
command reasons, and before/after receipts expose simulated authority as text.
Cohesion,
polarization, spacing, speed, and relation counts expose consequences. Replay
event/step totals, checkpoint count, and the
bounded session operation log also have semantic DOM projections. Animation
frames do not enter the live region or operation log.
Comparison mode adds two labelled non-authoritative canvases plus a complete
table and textual lane projections. Each lane names its public catalogue entry,
configuration, source status, transfer boundary, non-claim, normalized input,
semantic actions, policy, action receipts, final revision provenance, fields,
leases, and behavior distribution. Same-tick cohesion, polarization, mean pair
spacing, average speed, group count/sizes/extent, behavior counts, field/lease
counts, and right-minus-left deltas include definitions and units.

## Validation

The validation boundary covers the public catalogue schema, filter facets,
source links, evidence labels, transfer boundaries, deterministic state hashes, serde round trips,
all three scopes, convergent cohere and divergent disperse structure,
invalid/empty/stale selection, pause/step/reset/restart, strict replay round
trips and damaged replay rejection, additive-field order independence,
bounded/damaged fields, expiry/removal, same-seed metrics, and cross-mechanism
reset/replay, raw-rate bounds and damaged input rejection, parameter-order
independence, deterministic state distributions, and same-seed metric effects,
semantic bounds/defaults and damaged-vector rejection, deterministic
translation and qualitative monotonicity, action-order independence,
raw-vector equivalence, inspection stability, same-seed outcome differences,
and coexistence with scope, fields, checkpoint, reset, and replay,
canonical morphology identity and conservation, deterministic partition and
merge rules, group/scale bounds, stale and damaged operation rejection,
formation-scale effects, and coexistence with scope, fields, raw/semantic
dynamics, checkpoints, reset, and replay,
lease state-machine/damage matrices, fixed-step expiry boundaries, holder-only
use, named-receiver consent, stale/replay resistance, caps, exact checkpoint/
replay/reset, morphology lifecycle interactions, and cross-mechanism preservation,
canonical comparison-start equality, same-spec determinism, equivalent and
intentionally different vector profiles, immutable intended differences,
lane/tick/seed/tape damage rejection, transactional isolation, bounded traces,
lockstep step/reset/replay, superposition-versus-lease receipts and provenance,
metric values and delta signs, and ordinary-atlas entry/exit preservation,
Matter payload conversion, exact Git dependency pins, frozen builds, the Wasm
bundle, public-boundary scans, and browser interaction/accessibility checks.

## Reference lessons

Rusty Matter's existing particle contract is reused rather than copied. The
browser is an adapter over a reusable core; later XR work reuses semantic
actions and payloads, not DOM or Canvas code.

## Mitigation map

| Risk | Mitigation |
| --- | --- |
| Input device silently defines authority | Modality-free semantic actions and independent target scope |
| Canvas becomes inaccessible authority | Equivalent DOM controls and textual state |
| Generated summary becomes scholarly evidence | Hash-bound allowlist transform keeps source reports and reconstruction fields separate |
| One input device becomes a hidden policy | Route-specific normalization ends before semantic actions and target resolution |
| Browser policy leaks into reusable core | Palette, projection, timing loop, and DOM remain in `web/` |
| Moving upstream changes behavior | Full 40-character Matter revision and committed lockfile |
| Stale target mutates the wrong members | Selection-revision check rejects stale speed and collective-rule actions |
| Moving dots imply collectivity without showing it | Deterministic neighbour rules, relational lines, distinct rule shapes, and pair-distance tests |
| Browser history becomes a second simulation | Checkpoints contain versioned core replay tapes and reconstruct through the same reducer |
| A loaded snapshot gains false provenance | Snapshot restoration disables replay export; only validated tapes remain replayable |
| Synthetic provenance becomes identity or authority | Four app-local labels are ephemeral reducer values with no account, storage, or network contract |
| Field combination depends on action arrival order | Fields are bounded and sorted by stable ID before additive acceleration is summed |
| A technical analogy is presented as source reproduction | UI and catalogue non-claims distinguish app-owned Flock/Cohere/Disperse rates from the source robot-state model |
| Arbitrary parameter plumbing hides unstable behavior | Only three named rate actions exist, with fixed 0.00–1.00 bounds and zero defaults |
| Runtime randomness breaks comparison or replay | Transition draws are pure functions of seed, fixed tick, member ID, and stream ID |
| Semantic labels hide a second simulation path | Raw and semantic actions resolve one explicit vector consumed by the established core |
| Invented coefficients look source-authored | The UI labels interpolation endpoints as app-owned and preserves the source's qualitative directions separately |
| Inspecting translation changes the experiment | The resolved-vector panel is a read-only DOM projection with no reducer action |
| A rendered-clip perception study is presented as live control evidence | The UI and catalogue state that live control, accessibility, agency, authorship, and a universal interpreter were not evaluated |
| Visual clusters become hidden or inconsistent group authority | Canonical bounded groups and complete member rosters live in the reducer and semantic DOM |
| Split or merge silently destroys unrelated state | Lifecycle tests preserve scope selection, fields, provenance, behaviors, and dynamics authority |
| Rescale becomes an undocumented dynamics rewrite | Formation scale is an explicit per-group target with independent bounds and a neutral default |
| Synthetic operators are mistaken for people or sessions | Four fixed labels have no account, storage, networking, or persistent identity contract |
| Handoff silently reallocates authority | Holder offer and exact named-receiver accept/decline are separate revision-fenced actions |
| Wall time or animation rate changes ownership | Lease lifetime and expiry use only deterministic fixed steps recorded in replay |
| Comparison lanes contaminate one another | Two owned cores, transactional temporary clones, invariant checks, and separate traces/results |
| Different starts masquerade as mapping effects | One canonical snapshot is cloned and byte equality plus seed/tick lockstep fail closed |
| A metric delta looks like a research finding | Every table and result receipt labels it as an app-owned deterministic reconstruction outcome |
| Comparison rewrites literature evidence | Each lane cites an unchanged accepted catalogue row with its own status, transfer boundary, and non-claim |
| Comparison corrupts the ordinary atlas | A separate Wasm owner freezes ordinary animation and byte-checks ordinary state and frame rows on exit |

## Next slice

The planned seven-entry atlas and comparison tranche is complete. Input-device
adapters, Pages deployment, LSL, Optics integration, live multi-user authority,
and Quest adaptation remain separate later decisions and are not activated by
this architecture.
