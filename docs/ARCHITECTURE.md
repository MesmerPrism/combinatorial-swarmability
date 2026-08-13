# Architecture

## Decision

Use one deterministic Rust core with semantic actions, a thin WebAssembly
adapter, semantic HTML controls, and Canvas2D. Keep the first product slice a
direct `fast` package. Do not add a `morphospace/` workspace until a real
cross-owner composition or activation decision exists.

## Scope

The slice demonstrates target-independent collective steering and speed
actions over three scopes: one selected member, a selected subgroup, or the
whole swarm. `Flock` maintains local alignment and spacing, `Cohere` steers
toward peers sharing that rule, and `Disperse` steers away from matching peers
or the swarm centre. Pointer, keyboard, and button routes adapt into the same
`SemanticAction` values.

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
| Scene semantics, reducer, deterministic flock, palette, and browser behavior | This app |
| DOM accessibility and Canvas2D projection | Browser adapter in this app |
| Future tracked-space relation | Rusty Lattice, only when required |
| Future accepted sessions, peers, leases, or replay | Rusty Manifold, only when required |
| Future Quest/OpenXR hosting | A separate Quest adapter over `demo-core` |

## Interfaces

```text
pointer / keyboard / buttons
        |
        v
SemanticAction (input-modality-free)
        |
        v
Demo reducer -- selection revision / target scope / receipts
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
Animation frames do not enter the live region.

## Validation

The validation boundary covers deterministic state hashes, serde round trips,
all three scopes, convergent cohere and divergent disperse structure,
invalid/empty/stale selection, pause/step/reset/restart,
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
| Generated summary becomes scholarly evidence | Synthetic fixture until a locked allowlisted export is reviewed |
| Browser policy leaks into reusable core | Palette, projection, timing loop, and DOM remain in `web/` |
| Moving upstream changes behavior | Full 40-character Matter revision and committed lockfile |
| Stale target mutates the wrong members | Selection-revision check rejects stale speed and collective-rule actions |
| Moving dots imply collectivity without showing it | Deterministic neighbour rules, relational lines, distinct rule shapes, and pair-distance tests |

## Next slice

After the public boundary is accepted, replace the synthetic reference card
with one allowlisted, source-linked reconstruction record. Optics integration,
multi-user authority, and Quest adaptation remain separate later decisions.
