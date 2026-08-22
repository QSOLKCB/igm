# Pre-Phase 5 Readiness Audit

Status: **READY_ON_PHASE4_MERGE**.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**
>
> **INV-RUNTIME-001: Execution Adjacency Does Not Imply Biological Adjacency.**

This audit exists to stop the project from treating runtime correctness, source ingestion, or representational convenience as scientific authority.

## Completed foundation

The repository now has:

- Phase 1 governance, evidence boundaries, V0-V4 ladder, source provenance rules, and no-patient-data default;
- Phase 2 deterministic Pages visual laboratory and cross-view invariants;
- Phase 3A Rust structural reference, browser parity, deterministic CRT traversal, bounded parallelism, worker-independent identity, and seeded property-based fuzzing;
- Phase 3B explicit synthetic execution state space, PENTA-CRT optimization, LUT geometry, squared-distance hot loop, structured XY reuse with exact Z residual corrections, sparse J correction, fixed `1e-12` residual gate, and dedicated scalar/reference-vs-optimized timing benchmark;
- Phase 3C execution graph, 30+2 memory layout, bounded chunk campaigns, accepted/rejected receipts, immutable accepted-campaign handles, explicit acceptance gate, and benchmark/correctness identity separation;
- Phase 4 replaceable source adapters, structural-source registry validation, explicit uncertainty/provenance requirements, conflict/unknown preservation, source snapshot policy, and V0 constant non-inheritance rules.

## Phase 4 gate now implemented

Phase 4 adds an executable evidence-ingestion boundary rather than a documentation promise.

The gate requires:

1. a registered source with stable identity;
2. adapter/source-class compatibility;
3. preserved source access and redistribution metadata;
4. an input support statement that exactly matches a registered `supports` statement;
5. explicit evidence uncertainty;
6. adapter-derived evidence status from the declared transformation;
7. source snapshot mode that satisfies the repository snapshot policy;
8. no validation-level, biological-validity, or clinical-validity promotion;
9. conflict/unknown preservation instead of automatic reconciliation.

The normative Phase 4 rule remains:

> **Source ingestion must not silently convert observations into stronger claims than the source supports.**

See `docs/EVIDENCE_ADAPTERS.md`.

## Phase 5 readiness state

On this PR branch, Phase 5 status is:

```text
READY_ON_PHASE4_MERGE
```

This is intentionally not `READY_ON_MAIN` until the Phase 4 PR is reviewed and merged.

After merge, Phase 5 may begin because the evidence/provenance boundary it depends on will exist in code, schemas, tests, and CI. Phase 5 must continue to distinguish:

```text
source observation
model parameter
assumption
unknown/conflict
runtime representation
validation level
```

without collapsing those categories.

## What readiness does not mean

Phase 4 readiness does **not** mean that IGM now has a validated source-informed IgM model. That remains later work.

In particular, Phase 4 does not:

- create a V1 biological profile;
- claim that the cryo-EM ingestion fixture is a complete IgM model;
- turn MD or biochemical adapter unit tests into biological evidence;
- calibrate the legacy V0 drawing constants;
- establish molecular-dynamics realism;
- promote V0 to V1/V2/V3/V4;
- create clinical meaning.

The first actual structure-informed model remains a separate source/model-validation task.

## Performance readiness

The Phase 3B timing benchmark remains observation-only:

```text
speedup_claimed = false
performance_claim = false
benchmark_timing_identity_bearing = false
correctness_identity_includes_timing = false
```

A faster ingestion or runtime path cannot increase evidence strength.

## Decision

If this Phase 4 PR merges with its source/adaptation CI green, the project may proceed to Phase 5 representation work.

Phase 5 must consume the new evidence contracts rather than bypass them.
