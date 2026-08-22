# Pre-Phase 5 Readiness Audit

Status: **Phase 1 through Phase 3 complete after the Phase 3B timing benchmark; Phase 5 remains blocked on Phase 4.**

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**
>
> **INV-RUNTIME-001: Execution Adjacency Does Not Imply Biological Adjacency.**

This audit exists to stop the project from accidentally treating an old unchecked box, stale status line, or runtime optimization as permission to skip the evidence-adapter layer.

## Completed foundation before Phase 4

The repository now has:

- Phase 1 governance, evidence boundaries, V0-V4 ladder, source provenance rules, and no-patient-data default;
- Phase 2 deterministic Pages visual laboratory and cross-view invariants;
- Phase 3A Rust structural reference, browser parity, deterministic CRT traversal, bounded parallelism, worker-independent identity, and seeded property-based fuzzing;
- Phase 3B explicit synthetic execution state space, PENTA-CRT optimization, LUT geometry, squared-distance hot loop, structured XY reuse with exact Z residual corrections, sparse J correction, fixed `1e-12` residual gate, and dedicated scalar/reference-vs-optimized timing benchmark;
- Phase 3C execution graph, 30+2 memory layout, bounded chunk campaigns, accepted/rejected receipts, immutable accepted-campaign handles, explicit acceptance gate, and benchmark/correctness identity separation.

The pre-Phase 5 audit found no additional unchecked Phase 1-3 implementation item after the timing benchmark was added.

## Phase 4 is not optional

Phase 5 must **not** begin by pretending Phase 4 has already happened.

The following Phase 4 work remains intentionally open:

1. define the replaceable source-adapter interface;
2. maintain structural-source identifiers such as DOI/PDB/EMDB in the public source registry;
3. add a cryo-EM parameter adapter;
4. add a molecular-dynamics trajectory adapter;
5. add a biochemical/calibration constraint adapter;
6. preserve source licence/access metadata through adapters;
7. require per-parameter provenance and uncertainty;
8. preserve conflict and unknown states rather than forcing reconciliation;
9. snapshot/hash source material only where reuse terms permit;
10. externalize V0 implementation constants if they become biologically meaningful in a source-informed profile.

These are scientific-evidence plumbing tasks, not runtime cleanup. They deserve their own reviewable implementation rather than being smuggled into a tensor/graph PR.

## Phase 5 readiness rule

Phase 5 status is therefore:

```text
BLOCKED_ON_PHASE4
```

Phase 5 may begin only after the Phase 4 gate is executable enough to prevent source ingestion from silently strengthening claims beyond what the source supports.

At minimum, a Phase 5 representation must be able to distinguish:

```text
source observation
model parameter
assumption
unknown/conflict
runtime representation
validation level
```

without collapsing those categories.

## Performance readiness

The Phase 3B timing benchmark closes the final pre-Phase 4 runtime-performance checklist item, but it does not create a performance claim.

A benchmark receipt records local timing observations with:

```text
speedup_claimed = false
performance_claim = false
benchmark_timing_identity_bearing = false
correctness_identity_includes_timing = false
```

Hosted CI checks the benchmark contract and bounded execution. It does not require the optimized path to be faster on a shared runner.

## Decision

The next substantive scientific architecture PR should be **Phase 4**, not Phase 5.

Once Phase 4 is complete, this audit should be updated from `BLOCKED_ON_PHASE4` to an explicit Phase 5-ready state only if its source/provenance gate is actually enforced in code and tests.
