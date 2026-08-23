# Phase 4 Replaceable Evidence Adapters

Status: **complete and merged in PR #9**.

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
runtime/rust/src/phase4_v2.rs
runtime/rust/src/evidence_main.rs
```

Independent registry/policy validation:

```text
tools/validate_sources.py
tools/validate_phase4.py
tools/validate_json_schema.py
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
4. the input support claim resolves through a registered source claim binding for the exact target/value/unit/derivation contract;
5. uncertainty is explicit, kind-valid, and finite where numeric;
6. snapshot mode agrees with `research/source-snapshot-policy.json`;
7. output status is derived from the declared transformation, not supplied by the caller;
8. duplicate candidate identities cannot manufacture corroboration;
9. adapter output cannot promote validation level, biological validity, or clinical validity.

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

It maps the already-registered Chen et al. full-length human IgM pentamer source to an `assembly_sector_count = 5` candidate through an explicit registered claim binding. The uncertainty record says that this cardinality observation does not quantify the unresolved flexible-state ensemble or imply exact C5 biological symmetry.

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

Accepted targets are parameters. The adapter accepts transformed, inferred, and unknown derivations. It deliberately does not turn a trajectory-derived summary into an `observed` biological parameter merely because a source file contains coordinates.

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

`research/sources.json` remains the human-readable public source registry. `schemas/source-registry.schema.json` and `tools/validate_sources.py` enforce structural-source identity rules.

For structural sources, at least one of the following must be present:

```text
DOI
PDB
EMDB
```

The registry contains all three identifier classes across its structural records. External identifiers are checked for format and duplicate reuse.

Evidence-bearing source classes preserve:

- `access.status`;
- redistribution guidance;
- explicit `supports` statements;
- explicit `does_not_support` statements;
- structured claim bindings used for target/value/derivation admission where a direct machine-readable mapping is required.

Adapters carry those boundaries into candidate receipts.

## Per-parameter provenance and uncertainty

Evidence-backed model parameters with status:

```text
observed
source-derived
calibrated
```

must carry source provenance, derivation, and an explicit uncertainty object under `schemas/model-profile.schema.json`.

Supported evidence-uncertainty envelope kinds are:

```text
unknown
interval
standard-deviation
confidence-interval
source-reported
```

Kind-specific rules are enforced. Interval-like forms require ordered bounds, standard deviation must be non-negative, confidence levels remain bounded, and `unknown`/`source-reported` uncertainty requires explanatory notes.

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
- duplicate candidate identities are rejected before classification, so one observation cannot be counted twice as corroboration;
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

`packaged` requires verified redistribution permission, a repository-relative payload path, a committed payload, and a SHA-256 computed from the actual bytes. Admission fails if the file is absent, escapes the allowed repository scope, or its bytes do not match the declared digest.

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

## Structural schema admission

Phase 4 instances are validated against their declared schemas, not merely parsed as JSON. Runtime structs also deny unknown fields at the hardened public boundary so misspelled or extra fields cannot bypass `additionalProperties=false` semantics.

The dedicated Phase 4 CI validates:

- `research/sources.json` against the source-registry schema;
- `research/source-snapshot-policy.json` against the snapshot-policy schema;
- evidence inputs against the evidence-input schema;
- emitted bundles against the evidence-bundle schema;
- deliberate rejection of extra properties, duplicate evidence, unsupported target/value claims, and bad snapshot admission.

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

The executable gate is the combination of source registration, structured source claim binding, adapter-specific source/derivation admission, strict structural schema validation, explicit uncertainty, snapshot policy, actual packaged-byte digest verification, duplicate-evidence rejection, derived status mapping, conflict preservation, and fixed non-promotion fields.

Successful ingestion establishes only that a source observation was represented according to the declared adapter contract. It does not establish that the resulting model is biologically correct, clinically useful, or independently validated.
