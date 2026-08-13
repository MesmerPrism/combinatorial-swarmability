# Deployment

## Intended route

The independent public repository is intended to be
`MesmerPrism/combinatorial-swarmability`, served as a GitHub Pages project at:

```text
https://mesmerprism.com/combinatorial-swarmability/
```

The repository must not add a `CNAME`; it inherits the Mesmer Prism user-site
domain and project path. All runtime asset URLs are relative so the same build
works under the project subpath.

## Build

`scripts/Build-Web.ps1 -Release` performs a locked Wasm build, invokes the exact
repository-local `wasm-bindgen` CLI with `--target web`, and copies only the
public web tree into `dist/`. `scripts/Serve-Web.ps1` stages that artifact under
the intended subpath for local HTTP checks.

The Pages workflow is manual (`workflow_dispatch`) until publication is
explicitly approved. Enabling Pages, creating the GitHub repository, pushing,
dispatching deployment, or changing the Mesmer Prism website are not local
implementation steps.

## Activation checkpoint

Before any remote action, review:

- repository name `MesmerPrism/combinatorial-swarmability`;
- public visibility;
- exact local branch, head, tree, and complete tracked-file inventory;
- Matter revision and `Cargo.lock`;
- validation and artifact hashes;
- proposed Pages URL;
- a clean public-boundary report;
- final host/core toolchain closure and continued Rust 1.97.1 baseline.

The later Mesmer Prism project page belongs in an isolated website worktree and
must remain a separate commit or pull request.

