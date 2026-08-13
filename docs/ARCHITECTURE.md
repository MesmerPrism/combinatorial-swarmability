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

The Wasm adapter exposes one fixed-size row per member:

```text
[member_id, x, y, radius, vx, vy, speed,
 primary_selected, subgroup_selected, currently_targeted, behavior_code]
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
polarity, and lifetime also have non-canvas projections. Replay event/step totals, checkpoint count, and the
bounded session operation log also have semantic DOM projections. Animation
frames do not enter the live region or operation log.

## Validation

The validation boundary covers the public catalogue schema, filter facets,
source links, evidence labels, transfer boundaries, deterministic state hashes, serde round trips,
all three scopes, convergent cohere and divergent disperse structure,
invalid/empty/stale selection, pause/step/reset/restart, strict replay round
trips and damaged replay rejection, additive-field order independence,
bounded/damaged fields, expiry/removal, same-seed metrics, and cross-mechanism
reset/replay,
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

## Next slice

Bind the additive personal-field catalogue entry to this shared mechanism,
then continue with raw dynamics parameters. Optics integration,
multi-user authority, and Quest adaptation remain separate later decisions.
