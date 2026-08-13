# Public Data Boundary

## Current state

The repository contains one versioned public catalogue projection at
`web/data/catalog.v1.json`. Its manifest binds the exact private authority
input hash without recording its path or copying rejected rows. The private
research memory remains authoritative.

## Versioned allowlist

The public snapshot may contain only these top-level fields:

- `schema`;
- `export_version`;
- `source_binding`;
- `items`.

Each item separates these allowlisted objects:

- public identity and display order;
- public source identity, public URLs, checked source locus, evidence kind,
  literature status, projection status, and checked date;
- reported input, semantic action, control family, quantity, scope, timing,
  participant configuration, combination policy, and substrate;
- app-owned reconstruction status, interaction, effect, transfer boundary,
  semantic actions, and non-reproduction claim;
- canonical filter facets.

The exact field contract is `schemas/public-catalog.v1.schema.json`. Missing
public URLs remain `null`; they are never inferred from private process
metadata. The first manifest remains `pending_locked_review` until the exact
output hash is accepted.

## Rejected content

The scan fails on machine paths, source-note paths or filenames, private
catalogue identifiers, secrets, non-public URL schemes, analytics, behavioral
logging, remote scripts, and fields not on the allowlist. Participant,
consent, study, and confidential partner data are never eligible for this
repository.

## Publication gate

Before publishing the first public projection:

1. bind the exact private source revision without copying private bytes;
2. review each exported claim, URL, evidence class, and limitation;
3. generate the allowlisted snapshot and byte-level manifest;
4. scan the complete public tree and built artifact;
5. approve the snapshot separately from application behavior;
6. change only the manifest review state after acceptance;
7. publish only the reviewed catalogue bytes and preserve their hash.
