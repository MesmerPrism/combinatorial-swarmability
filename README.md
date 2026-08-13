# Combinatorial Swarmability

An accessible, deterministic browser reconstruction for examining how
collective steering rules can address one swarm member, a selected subgroup,
or the whole swarm. Changing input route does not change target scope or
authority.

The first slice is intentionally narrow. Pointer, keyboard, and low-rate
button controls all dispatch the same app-owned semantic actions into a Rust
core. Members continuously combine local alignment, cohesion, and separation
with an assigned `flock`, `cohere`, or `disperse` rule. The core projects
simulation state through Rusty Matter's
renderer-neutral `ParticleRenderPayload`; a thin WebAssembly adapter exposes a
bounded frame buffer to a Canvas2D renderer. The canvas is not the only
information or interaction surface.

This is a reconstruction and research instrument, not an implementation by the
authors of any cited work and not a universal interaction vocabulary. The
included public catalog record is synthetic placeholder data pending a
separate reviewed research-to-public export.

## Repository shape

- `crates/demo-core`: deterministic fixed-step scene and semantic reducer;
- `crates/demo-wasm`: `wasm-bindgen` browser adapter;
- `web`: semantic HTML, CSS, ES modules, Canvas2D, and synthetic public data;
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
