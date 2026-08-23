# Phase 4 Replaceable Evidence Adapters

Status: implementation-complete on the Phase 4 PR branch; pending review/merge.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

Phase 4 establishes the boundary between external evidence and model profiles. It exists so a source can be identified, normalized, constrained, and carried forward without silently acquiring a stronger scientific claim.

## Contracts

```text
IGM-SOURCE-ADAPTER-V1
IGM-EVIDENCE-INPUT-V1
IGM-EVIDENCE-CANDIDATE-V1
IGM-PHASE4-EVIDENCE-BUNDLE-V1
IGM-SOURCE-SNAPSHOT-POLICY-V1
IGM-V0-IMPLEMENTATION-CONSTANTS-V1
```

Rust implementation:

```text
runtime/rust/src/phase4.rs
runtime/rust/src/evidence_main.rs
```

Independent registry/policy validation:

```text
tools/validate_sources.py
```

## Source-adapter interface

All adapters implement one replaceable `SourceAdapter` interface. The interface declares:

- stable adapter identity;
- accepted source classes;
- accepted target kind (`parameter` or `constraint`);
- accepted derivation classes.

The common ingestion path then enforces:

1. source exists in `research/sources.json`;
2. source class is allowed by the adapter;
3. source access/reuse metadata exists;
4. the input `support_statement` exactly matches one of the source registry's declared `supports` statements;
5. uncertainty is explicit and finite where numeric;
6. snapshot mode agrees with `research/source-snapshot-policy.json`;
7. output status is derived from the declared transformation, not supplied by the caller;
8. adapter output cannot promote validation level, biological validity, or clinical validity.

The derivation-to-status mapping is intentionally mechanical:

```text
direct       -> observed
transformed  -> source-derived
calibrated   -> calibrated
inferred     -> inferred
assumed      -> assumed
unknown      -> unknown
```

An adapter therefore cannot take a transformed quantity and quietly label it `observed`.

## Cryo-EM parameter adapter

Contract:

```text
IGM-CRYO-EM-PARAMETER-ADAPTER-V1
```

Accepted source classes:

```text
public-structure
peer-reviewed-literature
```

Accepted targets are parameters. Direct, transformed, inferred, and unknown derivations are supported.

The repository includes a conservative reference-only ingestion fixture:

```text
research/evidence/cryo-em-pentamer-count.json
```

It maps the already-registered Chen et al. full-length human IgM pentamer source to an `assembly_sector_count = 5` candidate. The support statement is copied exactly from the registry and the uncertainty record explicitly says that this cardinality observation does not quantify the unresolved flexible-state ensemble or imply exact C5 biological symmetry.

No article or structural payload is copied into Git history by that fixture.

## Molecular-dynamics trajectory adapter

Contract:

```text
IGM-MD-TRAJECTORY-ADAPTER-V1
```

Accepted source class:

```text
molecular-dynamics
```

Accepted targets are parameters. The adapter accepts transformed, inferred, and unknown derivations. It deliberately does not turn a trajectory-derived summary into an `observed` biological parameter merely because the source file contains coordinates.

The adapter is covered by native tests using in-memory synthetic source records. No synthetic MD number is registered as biological evidence.

## Biochemical/calibration constraint adapter

Contract:

```text
IGM-BIOCHEMICAL-CALIBRATION-ADAPTER-V1
```

Accepted source class:

```text
biochemical-measurement
```

Accepted targets are constraints. Direct, transformed, calibrated, inferred, and unknown derivations are supported. A calibrated input remains `calibrated`; the adapter does not relabel it as a direct observation.

The adapter is covered by native tests using in-memory synthetic source records. Those tests are implementation fixtures only.

## Public structural-source registry

`research/sources.json` remains the human-readable public source registry. `schemas/source-registry.schema.json` and `tools/validate_sources.py` now enforce structural-source identity rules.

For structural sources, at least one of the following must be present:

```text
DOI
PDB
EMDB
```

The registry as currently checked in contains all three identifier classes across its structural records. External identifiers are checked for format and duplicate reuse.

Evidence-bearing source classes must preserve:

- `access.status`;
- redistribution guidance;
- explicit `supports` statements;
- explicit `does_not_support` statements.

Adapters copy those boundaries into candidate receipts.

## Per-parameter provenance and uncertainty

Evidence-backed model parameters with status:

```text
observed
source-derived
calibrated
```

must now carry source provenance, derivation, and an explicit uncertainty object under `schemas/model-profile.schema.json`.

Supported evidence-uncertainty envelope kinds are:

```text
unknown
interval
standard-deviation
confidence-interval
source-reported
```

`unknown` and `source-reported` uncertainty must be explained rather than represented by a fabricated zero. Interval-like forms must be finite and ordered.

Phase 5 may later add computational uncertainty/ensemble representations. Phase 4's uncertainty envelope is specifically an evidence-provenance requirement.

## Conflict and unknown preservation

`bundle_candidates` creates an `IGM-EVIDENCE-BUNDLE-V1` without averaging, voting, or silently reconciling sources.

Bundle states are:

```text
single
concordant
conflict
unknown
```

Rules:

- a single candidate may expose its value as the bundle's resolved value;
- multiple concordant candidates remain separate source records and are not merged into one authority;
- conflicting candidates produce `state=conflict` and `resolved_value=null`;
- all-unknown candidates produce `state=unknown`;
- `reconciliation_performed=false` is fixed in Phase 4 adapter output.

A later model profile may make an explicit, reviewable selection. The ingestion layer itself does not conceal disagreement.

## Source snapshots and hashes

Snapshot policy is fail-closed.

```text
reference-only
hash-only
packaged
```

`reference-only` is the default. A source with no explicit policy record cannot use another mode.

`hash-only` requires a lowercase SHA-256 and does not commit the external payload.

`packaged` requires:

```text
redistribution_permission_verified = true
external_payload_committed = true
external_payload_sha256 = <64 lowercase hex characters>
```

The currently registered structural sources remain `reference-only` because the registry metadata expressly says that discoverability is not a blanket redistribution licence. This means Phase 4 implements source snapshots/hashes **only where reuse terms permit** rather than copying data first and asking permission later.

## Externalized V0 implementation constants

`research/v0-implementation-constants.json` names the three legacy V0 drawing constants:

```text
v0_subunit_z_amplitude = 0.08
v0_fab_z_offset        = 0.06
v0_jchain_y_ratio      = 0.35
```

They remain `assumed`, V0-only implementation constants with `biological_meaning_claimed=false`.

Their source-informed inheritance rule is normative:

> A V1+ profile may not inherit these values as biology. It must supply explicit source provenance and uncertainty for any corresponding biological quantity, or leave that quantity unknown.

Existing V0 browser/Rust literals may remain exact backward-compatible projections of the fixture. Phase 4 prevents those numbers from being silently smuggled into a source-informed model.

## CLI

After a release build:

```bash
./target/release/igm-evidence registry
./target/release/igm-evidence adapt research/evidence/cryo-em-pentamer-count.json
```

Multiple inputs targeting the same parameter/constraint can be bundled without forced reconciliation:

```bash
./target/release/igm-evidence bundle input-a.json input-b.json
```

The CLI performs no network fetch. External retrieval remains a separate, replaceable concern so source bytes cannot silently enter an accepted evidence record.

## Phase 4 gate

> **Source ingestion must not silently convert observations into stronger claims than the source supports.**

The executable gate is the combination of source registration, exact support-statement matching, adapter-specific source/derivation admission, explicit uncertainty, snapshot policy, derived status mapping, conflict preservation, and fixed non-promotion fields.

Successful ingestion establishes only that a source observation was represented according to the declared adapter contract. It does not establish that the resulting model is biologically correct, clinically useful, or independently validated.
