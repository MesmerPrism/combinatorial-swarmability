# Public Data Boundary

## Current state

The repository contains only `web/data/catalog.synthetic.json`, a deliberately
synthetic fixture. It does not claim to reproduce a private catalog row or an
original authors' implementation.

## Proposed allowlist

A future public snapshot may contain only these top-level fields:

- `schema`;
- `export_version`;
- `export_status`;
- `items`.

Each item may contain only:

- `public_id`;
- `title`;
- `summary`;
- `semantic_action`;
- `target_scopes`;
- `evidence_class`;
- `reconstruction`;
- `limitation`;
- `paper_url`;
- `artifact_url`;
- `checked_date`.

The first real export is a separate `locked` public/private-boundary review.
Missing values remain `null`; they are never inferred from private process
metadata.

## Rejected content

The scan fails on machine paths, local repository names, source-note paths,
private note IDs, private catalog identifiers, secrets, absolute endpoints in
runtime code, analytics, behavioral logging, remote scripts, and fields not on
the allowlist. Participant, consent, study, and confidential partner data are
never eligible for this repository.

## Publication gate

Before replacing the synthetic fixture:

1. bind the exact private source revision without copying private bytes;
2. review each exported claim, URL, evidence class, and limitation;
3. generate the allowlisted snapshot;
4. scan the complete public tree and built artifact;
5. approve the snapshot separately from application behavior;
6. publish only the reviewed bytes and preserve their hash.

