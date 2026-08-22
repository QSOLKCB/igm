# Phase 3B Scalar/Reference vs Optimized Timing Benchmark

Status: bounded performance-observation infrastructure for the schematic V0 runtime.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

The benchmark exists to answer a narrow engineering question: does the admitted Phase 3B PENTA-CRT optimization execute the same declared V0 conformation slice faster than the scalar deterministic reference workload on a particular machine and toolchain?

It does **not** establish biological validity, clinical validity, molecular realism, or a universal performance claim.

## Contract

```text
IGM-PHASE3B-SCALAR-VS-OPTIMIZED-BENCHMARK-V1
schema: IGM-PENTA-CRT-TIMING-BENCHMARK-V1
reference: IGM-PENTA-CRT-F64-REFERENCE-BRUTE-V1
optimized: IGM-PENTA-CRT-CPU-V1
```

The optimized side is the actual one-worker `run_penta_crt` implementation. The scalar reference side reconstructs the same synthetic Phase 3B conformation address, evaluates deterministic per-conformation Fab trigonometry, builds the V0 geometry, evaluates all 120 pairwise squared distances directly, and performs the same style of deterministic pair/result consumption so the compiler cannot erase the workload.

This comparison deliberately uses one worker on the optimized side. It is intended to isolate algorithmic improvement from parallel scheduling.

## Mandatory precondition

The benchmark refuses to run unless the existing Phase 3B residual gate passes with the fixed tolerance:

```text
1e-12
```

Timing can never replace that residual gate. A faster result that fails reference equivalence is rejected before timing evidence is accepted.

## Measurement protocol

The benchmark:

1. loads the validated V0 model and PENTA-CRT execution profile;
2. runs the Phase 3B residual verification gate;
3. validates a bounded conformation slice;
4. warms both scalar/reference and optimized paths;
5. runs an alternating reference/optimized order across repeated measurements to reduce simple order bias;
6. records every elapsed time in nanoseconds;
7. reports median elapsed time and median-derived conformations/second;
8. reports the local observed ratio `reference_median_ns / optimized_median_ns`;
9. verifies each path remains deterministic across repetitions;
10. keeps all timing outside correctness identity.

The report always carries:

```text
benchmark_timing_identity_bearing = false
correctness_identity_includes_timing = false
speedup_claimed = false
performance_claim = false
biological_validity_claimed = false
clinical_validity_claimed = false
validation_level = V0
```

The numeric `observed_speedup_ratio` is therefore a local measurement, not a project-wide claim.

## Running it

Use a release build for meaningful observations:

```bash
cargo run --locked --release --bin igm-benchmark -- \
  --start 0 \
  --count 4096 \
  --repetitions 9 \
  --warmups 2 \
  --verify-samples 1024
```

For a full synthetic execution-domain observation:

```bash
cargo run --locked --release --bin igm-benchmark -- \
  --start 0 \
  --count 23409 \
  --repetitions 9 \
  --warmups 2 \
  --verify-samples 4096
```

Redirect stdout if a machine-readable receipt is wanted:

```bash
./target/release/igm-benchmark --count 23409 --repetitions 9 > benchmark.json
```

## Bounds

The harness fails closed when:

- fewer than 64 conformations are requested;
- the slice exceeds the admitted 23,409-state V0 execution domain;
- repetitions fall outside `[3,31]`;
- warmups fall outside `[1,8]`;
- verification samples exceed the Phase 3B bounded verification domain;
- timing or derived throughput becomes non-finite;
- a timing duration is zero;
- either path becomes non-deterministic across repetitions;
- the Phase 3B residual gate fails.

## What would be required for an actual speedup claim?

This PR intentionally does **not** make one.

Before writing a repository-level statement such as “PENTA-CRT is X times faster,” retain benchmark receipts from release builds, name the exact model/optimization identities and conformation slice, disclose the hardware/toolchain class, repeat the measurement enough to characterize run-to-run spread, preserve the raw per-repetition timings, and state the scope of the comparison.

CI is a regression gate, not a performance laboratory. Hosted-runner timing can vary for reasons unrelated to the algorithm, so CI checks benchmark structure and safety properties but does not require `observed_speedup_ratio > 1`.

## Scientific boundary

A timing result is engineering evidence only.

Faster execution cannot create a biological relationship, promote V0 to a higher validation level, turn an assumed execution state into an observed conformation, or establish clinical utility.
