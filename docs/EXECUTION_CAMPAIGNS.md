# Phase 3C Execution Graph, Memory Layout, and Campaign Receipts

Status: deterministic non-clinical research infrastructure.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

> **INV-RUNTIME-001: Execution Adjacency Does Not Imply Biological Adjacency.**

Phase 3C turns the Phase 3B PENTA-CRT engine into a bounded campaign runtime with an explicit scheduling graph, GPU-shaped memory contract, chunk planner, and reproducibility artifacts. None of these runtime structures is a biological model of IgM.

## Contracts

- campaign runtime: `IGM-EXEC-CAMPAIGN-V1`
- execution graph: `IGM-EXEC-GRAPH-C5-K2-C3-V1`
- traversal receipt: `IGM-EXEC-TRAVERSAL-RECEIPT-V1`
- memory layout: `IGM-WARP32-AOSOA-V1`
- memory plan: `IGM-MEMORY-PLAN-V1`
- correctness receipt: `IGM-CAMPAIGN-CORRECTNESS-RECEIPT-V1`
- benchmark receipt: `IGM-CAMPAIGN-BENCHMARK-RECEIPT-V1`
- environment receipt: `IGM-CAMPAIGN-ENVIRONMENT-V1`
- accepted manifest: `IGM-CAMPAIGN-MANIFEST-V1`
- rejected receipt: `IGM-CAMPAIGN-REJECTION-V1`

The campaign runtime consumes the existing validated `IGM-PENTA-CRT-CPU-V1` engine. It does not replace the Phase 3A/3B numerical authority hierarchy.

## Execution graph

The scheduling graph is the Cartesian product

```text
G_exec = C5 □ K2 □ C3
```

with coordinates:

```text
sector × arm × execution-lane
 Z5       Z2          Z3
```

There are exactly 30 execution vertices. Every vertex has five neighbours:

1. previous sector;
2. next sector;
3. opposite arm;
4. previous execution lane;
5. next execution lane.

Therefore the undirected execution graph has:

```text
30 vertices
regular degree 5
75 undirected edges
```

The neighbour table is generated from exact integer coordinates and the existing CRT execution address. A traversal receipt records both the graph hash and sequence/address/storage hash.

### Critical semantic boundary

`G_exec` is a scheduler graph. It is deliberately separate from:

- the IGM component/model graph;
- the provenance graph;
- future tensor-factor graphs;
- visualization layout graphs.

A runtime neighbour relation says only that two execution addresses differ by one declared product coordinate. It does **not** assert physical contact, biochemical interaction, structural proximity, causal influence, or any other biological relationship.

## 30 meaningful lanes inside a 32-lane cell

Phase 3C defines an aligned, fixed-width structure-of-arrays execution cell:

```text
active[32]
sector[32]
arm[32]
lane[32]
storage_index[32]
reserved[32]
value[32]
```

Lanes `0..29` correspond to the 30 exact PENTA-CRT execution addresses. Lanes `30` and `31` are explicit padding lanes.

The padding lanes are:

- inactive;
- initialized with sentinel metadata;
- excluded from scientific/model counts;
- excluded from the execution graph;
- non-semantic;
- present only to provide a future warp/SIMD-friendly width.

The Rust type uses `repr(C, align(128))`, compile-time alignment/size assertions, and runtime tests. The current layout is a CPU-side contract for future accelerator work; it is not evidence of CUDA execution.

## Memory budget planner

A campaign declares an explicit resident-memory budget before execution.

The planner computes:

```text
bytes_per_execution_cell
resident_capacity_cells
chunk_count
last_chunk_cells
```

It fails closed if:

- the budget is zero;
- the budget exceeds the declared Phase 3C upper bound;
- the budget cannot hold one execution cell;
- chunk-count arithmetic overflows;
- the required chunk count exceeds the bounded campaign limit.

This is derived from QSOL-NEXUS-style bounded-resource discipline: allocation shape is decided before execution rather than discovered after an out-of-memory failure.

## Bounded chunk streaming

Campaigns larger than the resident cell budget are deterministically divided into contiguous `[start,end)` chunks.

Chunking is execution planning only. The correctness result identity is designed to remain invariant under changes in:

- worker count;
- memory budget;
- chunk size;
- number of chunks.

This is possible because the Phase 3B per-conformation diagnostic fold is associative/commutative, while global minimum/maximum squared distances are also associative reductions.

The campaign manifest records chunking and workers separately, so changing the execution plan changes the campaign manifest but not the correctness result for the same conformation slice.

## Correctness receipt versus benchmark receipt

Phase 3C deliberately produces two different records.

### Correctness receipt

The correctness receipt binds:

- model-profile SHA-256;
- PENTA-CRT optimization-profile SHA-256;
- numerical profile;
- execution-graph and traversal identities;
- memory-layout contract;
- exact conformation slice;
- deterministic diagnostic fold;
- pair-distance extrema;
- reference-verification residuals and tolerance;
- worker-independent and chunk-independent result SHA-256.

It does **not** contain elapsed wall-clock time.

### Benchmark receipt

The benchmark receipt contains local observations such as:

- elapsed seconds;
- conformations/second;
- requested worker count;
- memory budget;
- resident capacity;
- chunk count.

It carries:

```text
identity_bearing_correctness = false
performance_claim = false
```

A fast run is not a more correct run, and neither is biological evidence.

## Privacy-safe environment receipt

The environment record intentionally includes only coarse reproducibility information:

- operating-system family;
- architecture;
- optional `rustc --version`;
- optional `cargo --version`;
- available parallelism.

It explicitly does not include:

- hostname;
- username;
- GPU UUID;
- MAC address;
- serial number;
- other raw machine identifiers.

Future GPU campaigns may record a privacy-safe GPU class/toolchain profile while retaining the same rule.

## Rejected-run preservation

`igm-campaign run` refuses to overwrite an existing output directory.

If engine admission, verification, memory planning, or campaign execution fails after an output path has been selected, the CLI writes a new rejected-run directory containing:

```text
rejected.json
SHA256SUMS
```

The rejection record preserves the failure stage and reason. It cannot later be relabelled as an accepted correctness receipt without actually rerunning and satisfying the acceptance gates.

## Accepted campaign directory

An accepted run writes:

```text
correctness-receipt.json
benchmark-receipt.json
execution-graph.json
memory-layout.json
memory-plan.json
environment.json
chunks.json
campaign-manifest.json
SHA256SUMS
```

`campaign-manifest.json` binds the profile, algorithm, partition, graph/traversal, and artifact identities. `SHA256SUMS` covers every JSON artifact in the directory but not itself.

The dependency-free validator checks the directory:

```bash
python3 tools/validate_campaign.py CAMPAIGN_DIR
```

## CLI

Inspect the exact scheduler graph:

```bash
cargo run --locked --release --bin igm-campaign -- graph
```

Inspect the padded memory contract:

```bash
cargo run --locked --release --bin igm-campaign -- layout
```

Preview bounded chunking:

```bash
cargo run --locked --release --bin igm-campaign -- \
  plan --start 100 --count 4096 --budget-bytes 1048576
```

Run and persist a campaign:

```bash
cargo run --locked --release --bin igm-campaign -- \
  run --start 100 --count 4096 --workers 16 \
  --budget-bytes 1048576 --verify-samples 257 \
  --out artifacts/campaign-example
```

## Nonclaims

Phase 3C does not establish:

- IgM biological adjacency from `G_exec`;
- an IgM biological 30-state mechanism;
- a ternary biological process;
- CUDA execution;
- GPU correctness;
- GPU speedup;
- a measured molecular memory layout;
- a biological interpretation of padding lanes;
- patient-specific output;
- clinical utility;
- diagnosis, prognosis, monitoring, treatment, or medical-device status.

The runtime is allowed to become extremely efficient. Biology remains free to disagree with it.
