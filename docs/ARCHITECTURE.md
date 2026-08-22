# IGM Architecture

## Design objective

The runtime must outlive any single biological hypothesis.

IGM therefore separates **evidence**, **model assumptions**, **execution**, and **interpretation**. Better structural information should replace model inputs and adapters without forcing researchers to rewrite deterministic scheduling, GPU kernels, evidence packaging, or analysis plumbing.

## Layered architecture

```text
+-----------------------------------------------------------+
| External evidence                                         |
| cryo-EM | PDB/EMDB | MD | biochemical | calibrated data   |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Source adapters                                            |
| parse -> identify -> normalize -> preserve uncertainty    |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Versioned model profile                                   |
| components | coordinates | bounds | constraints | sources |
+-----------------------------+-----------------------------+
                              |
                 +------------+-------------+
                 |                          |
                 v                          v
+-----------------------------+   +--------------------------+
| Deterministic CPU reference |   | Projection layers        |
| geometry + constraints      |   | tensor | graph | visual  |
+-----------------------------+   +--------------------------+
                 |
                 v
+-----------------------------------------------------------+
| Deterministic optimization/runtime layer                  |
| PENTA-CRT | exact indexing | bounded scheduling | receipts |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Campaign/orchestration layer                              |
| execution graph | memory plan | chunks | accepted/rejected |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Accelerator adapters                                      |
| CUDA / future GPU backends                                |
| validated against reference, never scientific authority   |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Observables and ensembles                                 |
| distances | accessibility | contacts | steric rejects     |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Evidence package                                          |
| inputs | versions | provenance | tolerances | results      |
+-----------------------------------------------------------+
```

## Model profile

A model profile is the boundary between biology and runtime engineering.

A profile should eventually declare:

- stable model/profile identifier and semantic version;
- validation level;
- assembly type/cardinality;
- components and stable component IDs;
- coordinate representation;
- bounded degrees of freedom;
- geometric or graph constraints;
- parameter value, units, uncertainty and provenance;
- unsupported/assumed parameters explicitly;
- source adapter identities;
- derived-observable definitions;
- compatibility requirements.

The initial schema is `schemas/model-profile.schema.json`.

## Representations

### Articulated geometry

The geometry representation is intended for conformational sweeps and spatial observables. A schematic pentamer can be useful as a V0 software fixture, but geometry must not silently acquire biological authority.

Typical operations:

- rigid transforms;
- bounded hinge rotations;
- inter-domain distances;
- steric intersection tests;
- reachable-volume estimates;
- surface/accessibility approximations.

### Tensor projection

The tensor layer is a projection of model state into GPU-friendly arrays. Possible fields include:

- coordinates;
- orientations;
- masks;
- pairwise distances;
- constraint residuals;
- contact probabilities or scores where a source-backed model defines them;
- per-state acceptance flags.

A tensor is an execution representation, not a claim about molecular ontology.

### Graph projection

A model graph can encode:

- components/domains as nodes;
- structural connections as edges;
- hypothesised interactions as separately typed edges;
- provenance links;
- constraint dependencies.

Observed and hypothesised edges must be distinguishable.

### Execution graph

The Phase 3C scheduling graph is a different object:

```text
G_exec = C5 □ K2 □ C3
```

It maps the exact 30-state CRT execution cell and exists for traversal, locality, memory layout, and future accelerator scheduling.

`INV-RUNTIME-001` is normative:

> **Execution Adjacency Does Not Imply Biological Adjacency.**

The architecture therefore keeps model graphs, execution graphs, provenance graphs, visualization graphs, and future tensor-factor graphs separately named and separately versioned.

### Vortex-inspired coordinate adapter

A vortex-like or cyclic coordinate scheme may be useful for indexing, phase-like angles, cyclic assemblies, or visualisation.

The rule is strict:

> A vortex coordinate system describes how the simulator parameterizes state. It does not establish that IgM is a physical vortex or that vortex dynamics explain IgM biology.

This adapter should remain optional and replaceable.

## Runtime principles

The Rust reference/runtime layer prefers:

- integer identities and indices;
- bounded numeric domains;
- deterministic work partitioning;
- canonical serialization;
- explicit overflow checks;
- reproducible random/sampling seeds if stochastic methods are added;
- no hidden dependence on GPU scheduling order;
- stable error taxonomy;
- fail-closed provenance validation.

Current native contracts include:

```text
IGM-RUST-RUNTIME-V1
IGM-CRT-PENTAFOLD-30-V1
IGM-PENTA-CRT-CPU-V1
IGM-EXEC-CAMPAIGN-V1
```

## Campaign layer

Phase 3C adds a runtime/orchestration boundary between the optimized CPU engine and future accelerators.

It provides:

- an exact execution graph and traversal receipt;
- a 32-lane aligned memory-layout contract with 30 meaningful and two non-semantic padding lanes;
- pre-execution memory-budget planning;
- deterministic contiguous chunk streaming;
- worker/chunk-independent correctness identity;
- worker/chunk/memory-plan manifest identity;
- separate correctness and benchmark receipts;
- privacy-safe environment provenance;
- accepted and rejected campaign preservation;
- external artifact checksums.

Changing memory budget or worker count may change a manifest and benchmark observation but must not change the declared correctness result for the same admitted model, algorithm, numerical profile, and conformation slice.

## GPU-shaped memory is not GPU authority

`IGM-WARP32-AOSOA-V1` deliberately shapes one execution cell around 32 lanes:

```text
30 meaningful execution addresses
+ 2 inactive padding lanes
= 32 runtime lanes
```

The two padding lanes are excluded from scientific/model counts. Their existence is a memory/scheduling choice only.

This contract is useful preparation for CUDA or SIMD, but its presence does not establish that a GPU has executed the model or that a 32-thread warp is the optimal physical implementation.

## GPU model

GPU acceleration should follow an evidence-first pattern:

```text
reference inputs
      |
      +--> Rust CPU reference
      |
      +--> CUDA adapter
                 |
                 v
        complete bounded readback
                 |
                 v
        independent residual check
```

Two modes are expected:

### Evidence mode

- bounded state count;
- complete required readback;
- exact coverage accounting;
- finite-value checks;
- declared residual tolerances;
- repeatability checks;
- no biological promotion based on GPU success.

### Throughput mode

- very large sweeps permitted;
- aggregate diagnostics may replace full readback;
- performance observation only;
- cannot later be relabelled as conformance evidence unless evidence-mode requirements were actually satisfied.

## Receipt separation

The correctness record and performance record have different jobs.

Correctness receipts bind deterministic inputs, algorithms, numerical profiles, graph/traversal identities, conformation ranges, diagnostics, and residual evidence.

Benchmark receipts may record elapsed time, throughput, worker counts, chunk counts, and memory budgets, but they carry `performance_claim=false` and are excluded from correctness identity.

A fast result is not a more biologically valid result.

## Source replacement contract

A researcher replacing schematic inputs with cryo-EM, MD or biochemical evidence should ideally need to change only:

1. source registry entry;
2. source adapter;
3. model profile;
4. validation/calibration records.

They should not need to rewrite:

- worker scheduling;
- execution graph mechanics where still compatible;
- bounded chunk planning;
- device sharding;
- evidence manifests;
- run hashing;
- accelerator orchestration;
- generic observables that remain semantically compatible.

An optimization that depends on assumptions absent from the new profile must fail admission rather than silently survive source replacement.

## Scientific non-authority of the runtime

The runtime can establish facts such as:

- input profile X produced output Y;
- all configured constraints were evaluated;
- a run was complete;
- a campaign passed its declared computational residual gates;
- correctness identity remained stable across worker/chunk plans;
- CPU and GPU implementations agreed within tolerance;
- results were reproducible under a declared environment.

It cannot, by itself, establish:

- that the profile is biologically correct;
- that a simulated conformation occurs in vivo;
- that an execution-graph edge is a biological edge;
- that an observable is clinically meaningful;
- that a mechanism explains disease;
- that an intervention would work.

Those are downstream scientific questions.
