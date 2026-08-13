# Combinatorial Swarmability Demo Agent Guide

This public repository owns runtime behavior for the Combinatorial Swarmability
browser demo. It is an ordinary direct `fast` product package, not a
Morphospace composition authority.

Read in this order:

1. `README.md`
2. `docs/ARCHITECTURE.md`
3. `docs/PUBLIC_DATA_BOUNDARY.md`
4. `docs/ACCESSIBILITY.md`
5. `docs/DEPLOYMENT.md`

## Boundaries

- `demo-core` owns deterministic scene behavior, semantic actions, selection,
  target-scope policy, and app-local reduction.
- Rusty Matter owns the renderer-neutral `ParticleRenderPayload` consumed by
  adapters. Its Git dependency must stay pinned to one full commit revision.
- `demo-wasm` is a thin bounded browser adapter. DOM, Canvas2D, palette, and
  projection remain app-owned.
- Do not add Lattice, Manifold, GUI, Optics, Quest, LSL, Bevy, egui, Makepad,
  wgpu, a backend, accounts, analytics, cookies, or behavioral logging unless
  a later accepted requirement creates that boundary.
- Never copy private research notes, paths, internal IDs, catalog rows, or
  operational metadata into this repository. Public catalog data enters only
  through the separately reviewed allowlist described in
  `docs/PUBLIC_DATA_BOUNDARY.md`.
- Keep optional integrations inert and documented until explicitly accepted.

## Validation

Run `powershell -File scripts/Test-All.ps1`. Build output belongs in `dist/`
and validation artifacts in `output/`; both are generated and ignored.

