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
  "derivation": "direct|transformed|calibrated|inferred",
  "uncertainty": null,
  "notes": "What the source actually supports"
}
```

Never convert `unknown` into a guessed numeric value merely because the runtime requires a number. If a simulation requires a placeholder, label it `assumed` and isolate it in the model profile.

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

## Research-data identity

Every imported data object used to produce an accepted model or report should eventually have:

- source identity;
- retrieval date/version;
- original file hash where lawful and useful;
- normalized representation hash;
- adapter version;
- transformation manifest;
- model-profile version consuming it.

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

## Conflict preservation

If credible sources disagree:

- retain both source records;
- record the conflict;
- avoid averaging them into an apparently precise value without justification;
- allow separate model profiles where appropriate.

The runtime should support disagreement rather than conceal it.

## Corrections

When a source interpretation or derived parameter is found to be wrong:

1. record the correction;
2. identify affected model versions/runs;
3. do not silently rewrite archived evidence;
4. publish a superseding profile or result;
5. preserve enough lineage for a researcher to understand the change.
