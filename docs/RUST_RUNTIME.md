# IGM Rust Runtime

Status: Phase 3 reference/orchestration implementation.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

`IGM-RUST-RUNTIME-V1` is a deterministic native runtime for the repository's schematic V0 model profile. It exists to establish computational behavior, bounded execution, reproducibility, and a future CPU reference for accelerator comparison. It does not establish biological validity and is not a clinical tool.

## Contracts

- runtime: `IGM-RUST-RUNTIME-V1`
- profile: `IGM-MODEL-PROFILE-V1`
- current model adapter: `IGM-SCHEMATIC-PENTAMER-V0`
- execution traversal: `IGM-CRT-PENTAFOLD-30-V1`
- run report: `IGM-RUST-STRUCTURAL-RUN-V1`

The PR3 adapter intentionally accepts only the V0 schematic model. Later source-informed profiles require separately versioned adapters and validation work.

## Fail-closed profile admission

The CLI does not bypass the repository's existing profile gates. Before `validate`, `geometry`, or `run`, it locates the IGM checkout and runs:

```bash
python3 tools/validate_json_schema.py profiles/igm-schematic-pentamer-v0.json
python3 tools/validate_profile.py profiles/igm-schematic-pentamer-v0.json
```

The Rust loader then repeats runtime-critical checks, including:

- exact profile/schema/model identity for the PR3 adapter;
- V0/non-clinical claim boundary;
- bounded profile file size;
- unique/exact component identifiers;
- evidence-backed parameter provenance requirements;
- no value on `unknown` parameters;
- finite numeric values and declared bounds for consumed parameters;
- exact five-sector, two-arm, and J-chain-marker schematic constraints.

This duplication is deliberate defense in depth. A future standalone packaging format should carry equivalent schema and semantic validation without requiring Python.

## Pentafold geometry

The five schematic sectors are generated from one starting radial direction with a fixed 72-degree recurrence:

```text
x' = c72*x - s72*y
y' = s72*x + c72*y
```

where the declared f64 projections correspond to:

```text
c72 = cos(2*pi/5) = (sqrt(5)-1)/4
s72 = sin(2*pi/5) = sqrt(10+2*sqrt(5))/4
```

The runtime evaluates the declared Fab spread angle once during geometry construction and derives both arm directions algebraically. The structural hot loop performs no trigonometric calls.

Three small depth/asymmetry constants remain inherited from the Phase-2 V0 schematic drawing (`0.08`, `0.06`, `0.35`). They are explicitly implementation-level schematic constants used only for current cross-runtime parity. They are not biological measurements, calibration values, or candidates for V1 evidence promotion.

## ETQ-inspired exact traversal

The runtime defines a computational execution cell:

```text
Z5 x Z2 x Z3
sector x arm x lane
```

with exactly 30 addresses. Sequence index `n` maps to:

```text
sector = n mod 5
arm    = n mod 2
lane   = n mod 3
```

and the exact Chinese-remainder inverse is:

```text
n = (6*sector + 15*arm + 10*lane) mod 30
```

Storage order is deliberately different:

```text
storage_index = 6*sector + 3*arm + lane
```

This follows the useful ETQ-303 discipline of separating exact traversal order, storage address, and graph semantics.

**The 30-state traversal is scheduling metadata only. It is not a biological graph walk and does not imply IgM adjacency, dynamics, or a ternary biological mechanism.**

The 30 meaningful states also leave two spare lanes in a future 32-thread CUDA warp. PR3 records that mapping as a future accelerator target but does not claim physical GPU execution.

## Squared-distance hot loop

Pairwise structural checks use squared Euclidean distance:

```text
d2 = dx*dx + dy*dy + dz*dz
```

No square root is required for ordering or cutoff tests. The generic `SquaredDistanceGate` deliberately has no biological meaning until a later source/model adapter assigns semantics.

Each structural work item evaluates all unordered component pairs and all 30 exact execution addresses without allocating per work item.

## Bounded memory and work

Following the project's bounded-execution discipline, PR3 declares hard runtime ceilings:

- profile file: 4 MiB;
- components: 4,096;
- workers: 256;
- work items: 100,000,000, additionally capped by the profile's logical domain.

The current run path stores only profile/geometry state plus one summary per worker. It does not allocate output proportional to logical ensemble size.

## Deterministic partitioning

Work items use contiguous quotient/remainder partitioning. Coverage is exact, gap-free, and non-overlapping.

Each work item receives a position-bound deterministic diagnostic. Worker results are combined with a commutative XOR fold so the global result identity is independent of worker count.

A run therefore records two identities:

- `result_sha256`: worker-count-independent computational result identity;
- `manifest_sha256`: execution-plan identity, which intentionally includes worker partitioning.

The test suite compares one-worker and seven-worker runs and requires identical result identities.

## Browser parity

The runtime includes the 16-component Phase-2 browser V0 coordinate fixture and checks the Rust geometry against it with a tight f64 residual. This establishes implementation agreement with the current schematic browser projection only.

It does **not** establish that the coordinates are biologically correct.

## CLI

```bash
cargo run -- validate
cargo run -- geometry
cargo run -- address 17
cargo run -- run
cargo run -- run --work-items 100000 --workers 16
```

For throughput observations:

```bash
cargo run --release -- run --work-items 1000000 --workers 16
```

The CLI reports local elapsed time and work-items/second separately from the deterministic run summary. Timing is excluded from result identity and carries `performance_claim=false`.

## PR3 nonclaims

PR3 does not provide:

- a source-informed V1 biological model;
- molecular dynamics;
- atomistic simulation;
- a biological hinge distribution;
- patient-specific input or output;
- clinical interpretation;
- CUDA execution;
- multi-GPU execution;
- fast reciprocal-square-root approximation;
- a claim that the CRT execution graph is an IgM biological graph.

Those boundaries are intentional.
