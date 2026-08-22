# Phase 3A Property-Based Fuzzing

Status: deterministic implementation testing for the schematic V0 runtime.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

The Phase 3A property-fuzz harness expands test coverage beyond hand-picked deterministic edge cases. The property-based fuzzing contract generates many bounded inputs from an explicit reproducible seed and checks invariants that should hold across whole input classes.

This is **not** biological validation. Rust/Pages agreement, randomized property coverage, deterministic replay, and failure shrinking-by-seed all remain implementation evidence for the schematic fixture only.

## Contract

Harness name:

```text
IGM-PROPERTY-FUZZ-V1
```

Implementation:

```text
tests/property_fuzz.rs
```

The harness is dependency-free and uses a small local SplitMix64 generator. The generator is test infrastructure only. It does not enter runtime output, model identity, correctness receipts, biological interpretation, or campaign identity.

Default reproducible seed:

```text
0x49474d50524f5037
```

Override locally with:

```bash
IGM_PROPERTY_FUZZ_SEED=0x123456789abcdef0 \
IGM_PROPERTY_FUZZ_CASES=4096 \
cargo test --locked --test property_fuzz -- --nocapture
```

`IGM_PROPERTY_FUZZ_CASES` is clamped to a bounded maximum so a malformed environment cannot turn CI into an unbounded campaign.

## Properties exercised

### CRT execution addressing

Generated valid and invalid sequence/address values test:

- `sequence -> address -> sequence` round trips over the declared `Z5 x Z2 x Z3` domain;
- valid storage indices remain inside `[0,29]`;
- invalid sector/arm/lane coordinates fail closed;
- invalid traversal sequence values fail closed.

This is scheduler/addressing verification only. It does not establish a biological graph walk.

### Deterministic work partitioning

Generated item counts and worker counts test:

- exact `[0,N)` coverage;
- no gaps or overlaps;
- stable worker ordinals;
- positive ranges;
- exact length arithmetic;
- quotient/remainder balancing with range lengths differing by at most one;
- effective workers bounded by both requested workers and item count.

Worker placement remains execution structure only.

### Squared-distance arithmetic

Bounded exactly representable binary coordinates test:

- non-negativity;
- symmetry;
- translation invariance;
- agreement between `SquaredDistanceGate` and the direct squared-distance predicate;
- fail-closed behavior for NaN and infinities.

These are numerical properties of the implementation, not molecular-distance observations.

### Bounded articulation

Generated bounded rotations test:

- exact preservation of `z` under the declared Z-axis primitive;
- preservation of planar radius about the pivot within the implementation tolerance;
- rejection of angles outside the declared bounds.

The primitive remains generic. Passing these properties does not assign a biological hinge or motion model.

### Worker-independent structural identity

Generated small V0 structural runs execute the same declared slice with different worker counts and require:

- identical correctness `result_sha256`;
- identical diagnostic fold;
- identical pair-distance extrema;
- `result_identity_worker_independent=true`;
- `validation_level=V0`;
- `non_clinical=true`;
- `biological_validity_claimed=false`;
- `clinical_validity_claimed=false`;
- `performance_claim=false`.

This directly preserves the Phase 3A gate while exercising more execution plans than fixed examples alone.

## Reproduction rule

A failure report must preserve at least:

```text
seed
property/test name
case index
```

The same seed and case count must reproduce the same generated sequence for the same harness version. A discovered failing seed should be retained as a deterministic regression case if the defect is fixed.

## Phase 3A gate

Rust/Pages agreement establishes implementation agreement for the schematic fixture only. Property-based fuzzing strengthens confidence that the implementation respects its declared contracts over many generated inputs, but it does not create a source-informed biological model, molecular-dynamics engine, clinical result, or validation-level promotion.
