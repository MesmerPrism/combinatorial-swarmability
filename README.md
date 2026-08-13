# Combinatorial Swarmability

An accessible, deterministic public interaction atlas for examining how
physical input routes become semantic requests, how scope and authority
resolve those requests, and how one swarm core makes the result measurable.

The first interactive mechanisms are intentionally bounded. Pointer, keyboard,
sliders, and low-rate buttons dispatch app-owned semantic actions into a Rust
core. Members continuously combine local alignment, cohesion, and separation
with an assigned `flock`, `cohere`, or `disperse` rule. Up to eight app-local
personal fields add attract or repel effects with explicit persistent or
expiring lifetimes and synthetic contributor provenance. Three explicit
swarm-wide rates can deterministically move members among the existing Flock,
Cohere, and Disperse modes; they default to zero and are presented as an
app-owned technical analogy rather than a reproduced robot controller. The core
projects simulation state through Rusty Matter's
renderer-neutral `ParticleRenderPayload`; a thin WebAssembly adapter exposes a
bounded frame buffer to a Canvas2D renderer. The canvas is not the only
information or interaction surface.

Atlas-wide history infrastructure records accepted semantic actions and actual
fixed simulation steps in a strict bounded replay tape. Five named checkpoints
can be saved and retrieved within the current browser tab; reset and replay use
the same core rather than a duplicate browser simulation.

This is a reconstruction and research instrument, not an implementation by the
authors of any cited work and not a universal interaction vocabulary. Its
versioned public catalogue keeps source reports, evidence maturity, transfer
limits, and app-owned reconstruction claims separate. The private research
memory remains the scholarly authority.

## Repository shape

- `crates/demo-core`: deterministic fixed-step scene and semantic reducer;
- `crates/demo-wasm`: `wasm-bindgen` browser adapter;
- `web`: semantic HTML, CSS, ES modules, Canvas2D, and the versioned public catalogue;
- `schemas` and `tools/catalog`: the public catalogue schema and locked export profile;
- `tests/fixtures`: action and deterministic-state fixtures;
- `scripts`: build, serve, boundary, dependency, and validation commands;
- `docs`: architecture, accessibility, data-boundary, and deployment decisions.

## Local validation

Rust 1.97.1, `wasm32-unknown-unknown`, PowerShell, Python, and Node/npm are
expected. The build installs the exact `wasm-bindgen-cli` version into
`target/wasm-tools`; it does not install a global CLI.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Test-All.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Build-Web.ps1 -Release
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\Serve-Web.ps1
```

Open `http://127.0.0.1:4173/combinatorial-swarmability/` after starting the
server.

## License

Project-owned source and documentation are licensed under
`AGPL-3.0-or-later`. Dependencies and external references retain their own
licenses and authority.
