# Research Data and Provenance

## Goal

IGM should make it difficult to forget where a biologically meaningful number came from.

Every source-informed parameter should be traceable to evidence, and every unsupported value should be labelled as an assumption rather than quietly dressed up as biology.

This document is subordinate to the core invariant:

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

## Source classes

Use explicit classes such as:

- `public-structure` — public PDB/EMDB or comparable structural record;
- `peer-reviewed-literature` — journal/conference paper;
- `preprint` — non-peer-reviewed research manuscript;
- `molecular-dynamics` — simulation trajectory or derived dataset;
- `biochemical-measurement` — experimental measurement;
- `institutional-private` — governed non-public research data;
- `synthetic-fixture` — intentionally invented software-test data;
- `assumption` — model choice without direct evidence;
- `unknown` — no supported value available.

## Parameter provenance

A source-informed parameter should carry at least:

```json
{
  "name": "example_parameter",
  "value": 0.0,
  "unit": "example-unit",
  "status": "source-derived",
  "source_id": "source.example",
  "derivation": "transformed",
  "uncertainty": {
    "kind": "source-reported",
    "notes": "What uncertainty the source reports, or why it is not quantified"
  },
  "notes": "What the source actually supports"
}
```

For `observed`, `source-derived`, and `calibrated` parameters, provenance **and** an explicit uncertainty object are mandatory. Absence of a reported uncertainty is represented explicitly, for example with `kind=unknown` plus an explanation. Do not convert missing uncertainty into numeric zero.

Never convert `unknown` into a guessed numeric value merely because the runtime requires a number. If a simulation requires a placeholder, label it `assumed` and isolate it in the model profile.

## Phase 4 source-adapter boundary

The executable Phase 4 source ingestion contract is documented in `docs/EVIDENCE_ADAPTERS.md` and implemented in `runtime/rust/src/phase4.rs`.

The normative gate is:

> **Source ingestion must not silently convert observations into stronger claims than the source supports.**

Every adapted candidate must preserve:

- source identity;
- source class and authority;
- DOI/PDB/EMDB/PubMed/URL locator metadata where present;
- access and redistribution guidance;
- an exact registry `supports` statement;
- registry `does_not_support` limitations;
- derivation class;
- explicit uncertainty;
- source snapshot mode;
- non-promotion of validation and clinical/biological validity.

Output evidence status is derived from the declared transformation rather than supplied as an arbitrary caller label:

```text
direct       -> observed
transformed  -> source-derived
calibrated   -> calibrated
inferred     -> inferred
assumed      -> assumed
unknown      -> unknown
```

## Claim-preserving ingestion

A source adapter may normalize representation, units or indexing. It must not increase the strength of the source claim.

Examples:

```text
source reports a resolved Fc/J-chain core
    -> adapter may encode those coordinates
    -> adapter may NOT declare every unresolved Fab conformation fixed

source reports a flexible hinge range
    -> profile may represent a bounded range
    -> profile may NOT claim the range is exhaustive in vivo
```

Phase 4 additionally requires the adapter input's `support_statement` to exactly match a statement registered in that source's `supports` list. A stronger sentence must be added and reviewed at the source-registry layer before an adapter can rely on it.

## Public structural sources

Publicly available structural records are preferred for early V1 profiles because they permit transparent provenance and independent inspection.

Each registry entry should capture, where known:

- DOI/PubMed identifier;
- PDB and/or EMDB identifiers;
- authors/title/year;
- organism/sample context;
- measurement method;
- resolution where relevant;
- licence/access conditions;
- exact model parameters derived from it;
- limitations.

`schemas/source-registry.schema.json` and `tools/validate_sources.py` now require structural sources to retain at least one DOI/PDB/EMDB identifier, and the checked-in structural registry collectively contains all three identifier classes.

## Research-data identity

Every imported data object used to produce an accepted model or report should eventually have:

- source identity;
- retrieval date/version;
- original file hash where lawful and useful;
- normalized representation hash;
- adapter version;
- transformation manifest;
- model-profile version consuming it.

Phase 4 implements the source-byte side of this through `research/source-snapshot-policy.json`:

```text
reference-only
hash-only
packaged
```

`reference-only` is the fail-closed default. `hash-only` requires a SHA-256 without committing the source payload. `packaged` requires explicit verified redistribution permission plus a SHA-256. Public availability alone is not packaging permission.

## Human and private data

The public repository is not a storage location for human participant or clinical data.

A future governed research program may use private data only in an approved environment with appropriate ethics, governance, privacy, storage and access controls. Such data should be referenced by opaque approved identifiers and hashes where appropriate, not copied into public Git history.

## Synthetic fixtures

Synthetic fixtures are welcome for testing. They must be unmistakably marked:

```text
source_class: synthetic-fixture
biological_evidence: false
```

Synthetic test success cannot promote a model beyond V0.

The MD and biochemical Phase 4 adapters use in-memory synthetic records for positive implementation tests rather than registering invented scientific values as biological evidence.

## Conflict preservation

If credible sources disagree:

- retain both source records;
- record the conflict;
- avoid averaging them into an apparently precise value without justification;
- allow separate model profiles where appropriate.

`IGM-PHASE4-EVIDENCE-BUNDLE-V1` represents `single`, `concordant`, `conflict`, and `unknown`. Multiple candidates are never automatically reconciled, even when concordant. `reconciliation_performed=false` is fixed at the ingestion layer.

The runtime should support disagreement rather than conceal it.

## Legacy V0 constants

`research/v0-implementation-constants.json` externalizes the known legacy V0 drawing constants as assumed, non-biological implementation metadata.

A source-informed profile may not silently inherit those values. If a corresponding quantity becomes biologically meaningful, it needs source provenance and uncertainty or must remain unknown.

## Corrections

When a source interpretation or derived parameter is found to be wrong:

1. record the correction;
2. identify affected model versions/runs;
3. do not silently rewrite archived evidence;
4. publish a superseding profile or result;
5. preserve enough lineage for a researcher to understand the change.
