# Contributing to IGM

Thank you for helping build reproducible IgM research infrastructure.

## Before contributing

Read:

- `AGENTS.md`
- `README4AI.md`
- `docs/CORE_INVARIANTS.md`
- `docs/MEDICAL_RESEARCH_BOUNDARY.md`
- `docs/RESEARCH_DATA_AND_PROVENANCE.md`
- `docs/VALIDATION_LADDER.md`

## Hard invariant

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

A contribution must not promote mathematical/computational correctness into biological or clinical validity without independent evidence appropriate to that claim.

## Biological parameters

For every biologically meaningful new parameter, include:

- source ID;
- source class;
- exact source support;
- derivation method;
- units;
- uncertainty if known;
- whether the value is observed, source-derived, calibrated, inferred, assumed or unknown.

If no support exists, use `assumed` or `unknown`.

## Human data

Do not submit patient or participant data to this public repository.

This includes identifiable, re-identifiable, coded, clinical, genomic, pathology, imaging, treatment and other protected/private records.

## Medical claims

Do not submit ordinary PRs that add patient-specific diagnosis, prognosis, monitoring, treatment recommendation, treatment selection or clinical decision support. Such a change alters intended purpose and requires a dedicated regulatory/ethics design process.

## Software contributions

Prefer:

- deterministic behavior;
- bounded inputs;
- explicit overflow/error handling;
- stable schemas;
- independent reference validation for accelerators;
- correctness tests separate from benchmarks;
- minimal dependencies;
- source provenance preserved through transformations.

## Pull requests

A PR should explain:

1. what changes;
2. which contract/profile it affects;
3. what evidence supports biological parameters;
4. validation level before/after;
5. tests executed;
6. known limitations;
7. whether any medical/clinical intended purpose changes (normally `no`).

## Apache-2.0 contributions

By intentionally submitting a contribution for inclusion, contributions are handled under the repository's Apache License 2.0 terms unless explicitly stated otherwise as described by the licence.
