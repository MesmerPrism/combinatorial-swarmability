# Public catalogue export

## Decision

The first public atlas catalogue is a locked, versioned projection of a
private research-memory authority. The research memory remains authoritative;
this repository owns only the reviewed public runtime copy.

The transform is deliberately one-way. It accepts a caller-supplied tabular
authority input, verifies its exact hash and row count, selects seven rows by
public source ID plus control family, and emits only the fields declared in
`schemas/public-catalog.v1.schema.json`. It never records the input path.

## Exported distinctions

Each item keeps these layers separate:

- `source`: public source identity, public URLs, source locus, evidence kind,
  literature status, projection status, and checked date;
- `reported`: the input, semantic action, control locus, scope, timing,
  participant configuration, combination policy, and substrate represented by
  the checked source projection;
- `reconstruction`: what this app implements or plans, the inherited transfer
  boundary, and an explicit non-reproduction claim;
- `facets`: a small public vocabulary used only for atlas filtering.

An implemented reconstruction is not relabelled as an authors' implementation.
An elicited mapping is not relabelled as implemented. A technical
demonstration is not relabelled as an evaluated participant outcome.

## Rejected material

The public transform does not emit local paths, source-note paths or filenames,
private catalogue identifiers, internal commentary, credentials, participant
data, or fields outside the versioned schema. Rows outside the seven-entry
profile are counted as rejected but are not copied into the repository.

## Reproduction and review

The locked review command supplies the private authority path at runtime:

```powershell
./scripts/Export-PublicCatalog.ps1 -InputPath <reviewed-authority-input>
./scripts/Test-PublicCatalogExport.ps1 -InputPath <reviewed-authority-input>
./scripts/Test-PublicCatalog.ps1
./scripts/Test-PublicBoundary.ps1
```

The generated manifest binds the source byte hash, row count, encoding, line
endings, schema hash, profile hash, output hash, selected/rejected counts, and
boundary checks. The first manifest remains `pending_locked_review` until its
exact output hash is accepted. Later conforming additions still require
source-level evidence review, but may reuse the accepted transform as ordinary
fast product work.
