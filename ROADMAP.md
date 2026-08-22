# IGM Roadmap

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

IGM is non-clinical research software. Each phase must preserve the separation between source evidence, model assumptions, computational correctness, accelerator performance, biological interpretation, and clinical/regulatory use.

## Phase 1 — Documentation and governance foundation

Status: **complete in PR #1**.

- [x] Establish non-clinical research intended purpose.
- [x] Establish Australian ethics/regulatory baseline.
- [x] Define no-patient-data public-repository rule.
- [x] Define replaceable evidence/model/runtime architecture.
- [x] Define V0–V4 validation ladder.
- [x] Define Flinders/SA Health handoff boundary without implying endorsement.
- [x] Add machine-readable governance policy and model-profile schema.
- [x] Add deterministic documentation validation CI.
- [x] Hard-code `INV-BIO-001`.
- [x] Require V0–V2 profiles to keep biological-validity claims false.
- [x] Prevent `unknown` parameters from carrying invented values.
- [x] Require provenance for observed/source-derived/calibrated parameters.
- [x] Record source access/redistribution status.
- [x] Add dependency-free semantic profile validation.

### Phase 1 gate

No implementation may promote computational correctness into biological or clinical validity.

---

## Phase 2 — Deterministic GitHub Pages visual laboratory

Status: **complete in PR #2**.

- [x] Original Apache-2.0 implementation with no BioFabric source/assets copied.
- [x] `IGM-SCHEMATIC-PENTAMER-V0` synthetic profile.
- [x] One canonical state shared by all views.
- [x] Assembly/spatial view.
- [x] Numerical-array view with explicit non-tensor labelling.
- [x] Typed graph layouts and adjacency matrix.
- [x] Original fabric/relation view.
- [x] Vortex-inspired coordinate projection labelled as parameterization only.
- [x] Provenance inspector.
- [x] Deterministic telemetry and fingerprints.
- [x] JSON/CSV/provenance/SVG/WebM exports.
- [x] SHA-256 Pages manifest.
- [x] CI enforcing V0, NOT CLINICAL, and `INV-BIO-001` labelling.
- [x] `INV-MATH-002`, `INV-MATH-003`, `INV-GRAPH-001`, `INV-GRAPH-002`, `INV-VIZ-001`, and `INV-VIZ-002`.

### Phase 2 gate

The browser is a transparent research microscope, not biological authority and not the high-performance runtime.

---

## Phase 3A — Minimal deterministic Rust structural runtime

**Target: PR #3, property-fuzz follow-up in PR #7.**

Status: **complete and merged in PR #3; generated property coverage added in PR #7**.

### Native contract and admission

- [x] Add `IGM-RUST-RUNTIME-V1`.
- [x] Consume the same `IGM-MODEL-PROFILE-V1` used by Pages.
- [x] Keep the initial adapter restricted to `IGM-SCHEMATIC-PENTAMER-V0` / V0.
- [x] Require repository JSON-Schema and semantic gates before CLI execution.
- [x] Repeat runtime-critical profile checks natively in Rust.
- [x] Preserve stable component IDs and provenance metadata.
- [x] Reject unsupported biological/clinical claim promotion.

### Geometry and computational primitives

- [x] Reproduce the current 16-component Pages V0 schematic in Rust f64.
- [x] Cross-check Rust coordinates against the browser reference fixture.
- [x] Generate five sectors using a fixed 72-degree C5 recurrence.
- [x] Evaluate the declared Fab spread trigonometry once at model construction rather than inside the hot loop.
- [x] Add generic bounded articulation primitive without assigning biological meaning.
- [x] Add squared-distance proximity gate so cutoff evaluation requires no square root.
- [x] Keep current Phase-2 depth/asymmetry drawing constants explicitly labelled as schematic implementation constants, not measurements.

### ETQ-inspired exact execution traversal

- [x] Add `IGM-CRT-PENTAFOLD-30-V1`.
- [x] Define execution address `Z5 × Z2 × Z3` = sector × arm × execution lane.
- [x] Add exact traversal `n -> (n mod 5, n mod 2, n mod 3)`.
- [x] Add exact CRT inverse `n=(6s+15a+10l) mod 30`.
- [x] Keep storage index separate from traversal order.
- [x] Hard-code the nonclaim that traversal adjacency is not biological adjacency and not a biological graph walk.
- [x] Record future 30-meaningful-lanes / 32-thread-warp mapping without claiming GPU execution.

### Bounded memory and deterministic parallelism

- [x] Bound profile bytes, component count, worker count, and work-item count.
- [x] Avoid output allocation proportional to the logical ensemble in the structural fixture runner.
- [x] Add deterministic contiguous quotient/remainder partitioning.
- [x] Add checked count arithmetic.
- [x] Add worker-count-independent global result identity.
- [x] Keep worker-specific execution-plan identity separate.
- [x] Test one-worker versus multi-worker result equivalence.

### Receipts and CLI

- [x] Add canonical profile SHA-256 identity.
- [x] Add `result_sha256` independent of worker count.
- [x] Add `manifest_sha256` binding execution partitioning.
- [x] Separate local timing from identity-bearing run data.
- [x] Keep `performance_claim=false` in PR3 reports.
- [x] Add `validate`, `geometry`, `address`, and `run` CLI commands.
- [x] Add native CI.
- [x] Add property-based fuzzing beyond deterministic edge-case tests.

Property-fuzz contract:

```text
IGM-PROPERTY-FUZZ-V1
seeded + reproducible
bounded generated domains
no external fuzz dependency
implementation evidence only
```

The generated properties cover CRT address round trips, partition coverage/balance, squared-distance invariants, cutoff equivalence, bounded articulation, fail-closed non-finite inputs, and worker-independent V0 structural result identity. See `docs/PROPERTY_FUZZING.md`.

### Phase 3A gate

Rust/Pages agreement establishes implementation agreement for the schematic fixture only. PR3 does not create a source-informed biological model, molecular dynamics engine, or clinical result.

---

## Phase 3B — PENTA-CRT CPU optimization profile

**Target: PR #4, timing-benchmark follow-up in PR #8.**

Status: **complete and merged in PR #4; dedicated timing benchmark added in PR #8**.

Turn the PR3 reference machinery into a fast conformational execution engine while keeping every optimization independently testable against the f64 reference.

### Integer state space

- [x] Define explicit discrete degree-of-freedom profile rather than inventing hidden hinge values.
- [x] Add mixed-radix outer conformation indexing.
- [x] Add exhaustive exact encode/decode round-trip tests for the configured radices.
- [x] Keep conformation identity independent of worker/device assignment.
- [x] Add deterministic range slicing for partial campaigns.

Current V0 execution profile:

```text
17 left-arm bins × 17 right-arm bins × 9 J-x bins × 9 J-y bins
= 23,409 explicit synthetic execution states
```

These are computational fixture states, not asserted biological conformations.

### Pentafold reuse

- [x] Initial C5 recurrence exists in PR3 for the static V0 geometry.
- [x] Generalize C5 recurrence to dynamic per-conformation V0 geometry.
- [x] Evaluate one sector seed plus recurrence where the execution profile explicitly admits C5 reuse.
- [x] Apply J-chain/asymmetry terms as explicit sparse defects instead of silently destroying symmetric structure.
- [x] Reject biological-symmetry promotion: the execution profile declares C5 as `assumed` and `biological_symmetry_claimed=false`.

### Hot-loop reductions

- [x] Precompute bounded articulation `sin/cos` lookup tables for the discrete execution profile.
- [x] Use squared distances for cutoff/steric predicates.
- [x] Keep square roots out of the Phase 3B structural hot loop.
- [x] Hoist profile constants, deterministic trig projection, and lookup construction out of the conformation loop.
- [x] Remove per-conformation heap allocation from the optimization hot loop.
- [x] Add fixed-size vectorization-friendly SoA geometry representation.
- [x] Add a dedicated scalar/reference-vs-optimized timing benchmark before making any speedup claim.

Timing-benchmark contract:

```text
IGM-PHASE3B-SCALAR-VS-OPTIMIZED-BENCHMARK-V1
scalar deterministic brute-force reference
actual one-worker PENTA-CRT optimized runtime
warmups + alternating measurement order + median timing
Phase 3B residual gate required before timing
performance_claim = false
speedup_claimed = false
```

See `docs/TIMING_BENCHMARK.md`.

### Structured interaction reuse

- [x] Investigate block-circulant pair reuse for the symmetric V0 execution profile.
- [x] Preserve the narrower result actually supported by the fixture: C5 block reuse is valid for the XY projection, not for all legacy V0 Z drawing terms.
- [x] Reconstruct full non-J 3D squared distances with exact local `dz^2` residual corrections.
- [x] Represent J-chain/asymmetry contributions as sparse direct corrections.
- [x] Test the complete structured result against brute-force full-3D pair evaluation.
- [x] Keep the `1e-12` implementation-equivalence gate rather than weakening it when the initial full-3D C5 hypothesis failed.
- [x] Restrict the optimization admission path to the declared V0 execution profile so future source profiles cannot silently inherit the shortcut.

Current accounting per conformation:

```text
45 planar block distance evaluations
+ 15 direct sparse-J 3D evaluations
+ 105 exact scalar dz² residual corrections
-> complete canonical 120-pair full-3D sequence
```

### Phase 3B gate

Optimization profiles may be faster than the reference but cannot become authority merely because they are faster. Every approximation or structural shortcut must carry an admission test and residual comparison.

---

## Phase 3C — Execution graph, memory layout, and campaign receipts

**Targets: PR #5 and explicit acceptance gate PR #6.**

Status: **complete and merged in PR #5; executable acceptance gate merged in PR #6**.

### Execution graph

- [x] Define `G_exec = C5 □ K2 □ C3` as a scheduling/execution graph only.
- [x] Keep `G_exec` separate from the IGM model graph and provenance graph.
- [x] Add `INV-RUNTIME-001`: **Execution Adjacency Does Not Imply Biological Adjacency.**
- [x] Add exact neighbour tables generated from integer coordinates.
- [x] Add deterministic graph/traversal SHA-256 receipts.

Current scheduling graph contract:

```text
IGM-EXEC-GRAPH-C5-K2-C3-V1
30 vertices
regular degree 5
75 undirected scheduling edges
```

No execution edge is a biological edge.

### GPU-shaped memory without GPU authority

- [x] Define 32-lane padded execution cell with 30 meaningful lanes and two explicitly inactive lanes.
- [x] Add aligned fixed-width SoA/AoSoA-compatible layout suitable for CPU SIMD and future CUDA coalescing experiments.
- [x] Keep padding lanes excluded from scientific/model counts.
- [x] Add compile-time and runtime alignment/size assertions.
- [x] Add fail-closed memory-budget planner before resident allocation/execution planning.
- [x] Add bounded deterministic chunk streaming for campaigns larger than resident buffers.

Current layout contract:

```text
IGM-WARP32-AOSOA-V1
lanes 0..29  = meaningful execution addresses
lanes 30..31 = inactive, non-semantic padding
alignment    = 128 bytes
```

This is a CPU-side future-accelerator layout contract. It is not a claim that CUDA has executed it.

### Campaign identity

- [x] Add run/campaign schema with profile, runtime, algorithm, graph, partition, and artifact identities.
- [x] Add rejected-run preservation with stage/reason receipts.
- [x] Separate correctness receipt from benchmark receipt.
- [x] Add environment/toolchain provenance without hostname, username, raw GPU UUIDs, serials, MAC addresses, or other raw machine identifiers by default.
- [x] Add external `SHA256SUMS` over persisted campaign artifacts.
- [x] Add dependency-free campaign-directory validation.
- [x] Test correctness identity across different worker counts and chunk/memory budgets.
- [x] Add executable `IGM-PHASE3C-ACCEPTANCE-GATE-V1` and gate receipt.

Accepted campaign artifacts:

```text
correctness-receipt.json
benchmark-receipt.json
execution-graph.json
memory-layout.json
memory-plan.json
environment.json
chunks.json
phase3c-gate.json
campaign-manifest.json
SHA256SUMS
```

Correctness identity intentionally excludes timing, worker count, and chunk budget. Those execution-plan observations remain in benchmark/manifest records.

### Phase 3C gate

Execution topology, memory adjacency, warp/SIMD placement, chunk membership, worker assignment, and future device assignment are implementation structures only. They may improve locality or throughput, but they cannot create biological relationships or promote validation level.

An accepted campaign must preserve the profile/algorithm identities, pass the Phase 3B residual gate, remain finite and bounded, produce a worker/chunk-independent correctness identity for the declared slice, and keep benchmark timing outside correctness identity.

---

## Pre-Phase 5 readiness audit

Status: **BLOCKED_ON_PHASE4**.

After PR #8 there are no remaining unchecked Phase 1-3 implementation items. The runtime/performance foundation is complete enough to stop being the reason Phase 5 is delayed.

Phase 4 is still a substantive evidence/provenance architecture phase and must not be skipped. See `docs/PRE_PHASE5_READINESS.md`.

---

## Phase 4 — Replaceable evidence adapters

Status: **next required phase; hard blocker for Phase 5**.

- [ ] Define source-adapter interface.
- [ ] Maintain public structural-source registry with DOI/PDB/EMDB identifiers.
- [ ] Add cryo-EM parameter adapter.
- [ ] Add molecular-dynamics trajectory adapter.
- [ ] Add biochemical/calibration constraint adapter.
- [ ] Preserve source licence/access metadata.
- [ ] Require per-parameter provenance and uncertainty.
- [ ] Add conflict/unknown representation rather than forced reconciliation.
- [ ] Add source snapshots/hashes only where reuse terms permit.
- [ ] Externalize any remaining V0 implementation constants that become biologically meaningful in source-informed profiles.

### Phase 4 gate

Source ingestion must not silently convert observations into stronger claims than the source supports.

---

## Phase 5 — Tensor, graph, and ensemble computational representations

**Entry condition: Phase 4 gate implemented and pre-Phase 5 audit updated from `BLOCKED_ON_PHASE4`.**

- [ ] Define explicit numerical-array projections of model state.
- [ ] Define true tensor types only where transformation semantics are declared.
- [ ] Define typed graph/hypergraph projections of domains, subunits, constraints, provenance, and relationships.
- [ ] Explore graph-structured tensor-network factorization only where it materially reduces computation and is rigorously specified.
- [ ] Keep model graph, execution graph, tensor-factor graph, and visualization graph as separately named objects.
- [ ] Add pair-distance/contact/accessibility observables.
- [ ] Add ensemble statistics with explicit numerical assumptions.
- [ ] Add uncertainty types: unknown, interval, distribution, and ensemble.
- [ ] Keep vortex-inspired coordinates optional and representational only.

### Phase 5 gate

A representation earns scientific interpretation only from explicit evidence and validation.

---

## Phase 6 — GPU acceleration

### Reference and adapter boundary

- [ ] Keep Rust f64 as reference/orchestration authority.
- [ ] Add CUDA f32 adapter with an explicitly named numerical profile.
- [ ] Require independent Rust/CUDA residual comparison.
- [ ] Separate evidence mode from throughput mode.
- [ ] Add complete-readback bounded evidence runs.
- [ ] Add aggregate-only throughput runs that cannot be relabelled as conformance evidence.

### PENTA-CRT CUDA path

- [ ] Map one 30-state execution cell into one 32-thread warp where benchmarking supports it.
- [ ] Preserve two padding lanes as inactive/non-semantic.
- [ ] Add coalesced SoA/AoSoA buffers.
- [ ] Add deterministic device sharding borrowed conceptually from RSH/GLUBALL practice.
- [ ] Evaluate lookup-table geometry and squared-distance kernels.
- [ ] Evaluate sparse J/asymmetry correction kernels.
- [ ] Compare warp-cell design against conventional flat kernels before adopting it.

### Optional fast reciprocal square root

- [ ] Keep reference path on ordinary validated arithmetic.
- [ ] If useful, add a separately named `fast-rsqrt` accelerator profile.
- [ ] Prefer hardware reciprocal-square-root plus explicit Newton refinement over folklore bit hacks unless evidence shows otherwise.
- [ ] Require declared residual tolerance and full evidence-mode comparison.
- [ ] Never silently substitute approximate normalization into reference output.

### Physical validation

- [ ] Run single-GPU physical baseline first.
- [ ] Add Compute Sanitizer memcheck/racecheck campaign support.
- [ ] Record toolchain/GPU class provenance with privacy-safe identifiers.
- [ ] Run repeatability campaigns before scaling.
- [ ] Run 2/4/8 GPU campaigns only after single-GPU correctness gates pass.
- [ ] Measure scaling rather than assuming more GPUs are faster.

### Phase 6 gate

GPU agreement is implementation evidence only. Performance cannot promote biological validation.

---

## Phase 7 — Structure-informed research profiles

- [ ] Create first V1 source-informed IgM pentamer profile.
- [ ] Create separately versioned hexamer profile if supported by evidence.
- [ ] Quantify profile uncertainty and unsupported degrees of freedom.
- [ ] Compare model-derived observables with independent structural observations.
- [ ] Publish accepted and rejected calibration attempts.
- [ ] Keep V3/V4 promotion dependent on external calibration/independent validation.

---

## Phase 8 — Research collaboration package

- [ ] Produce researcher onboarding guide.
- [ ] Produce reproducibility capsule format.
- [ ] Add export formats useful to structural-biology workflows.
- [ ] Add example Jupyter/analysis adapters without making Python authoritative.
- [ ] Prepare neutral research handoff package for external institutions.
- [ ] Invite domain experts to review biological assumptions, provenance, and validation design.
- [ ] Explicitly invite replacement of schematic profiles with better cryo-EM, MD, biochemical, or experimentally calibrated inputs without rewriting runtime infrastructure.

---

## Phase 9 — Optional regulated-research branch

This phase is **not implied by the open-source runtime**. It exists only if qualified downstream collaborators intentionally pursue human-subject, clinical, diagnostic, monitoring, treatment or medical-device work.

- [ ] Determine intended purpose with qualified investigators.
- [ ] Obtain institutional ethics/governance advice before using human participants/data.
- [ ] Conduct privacy/data impact assessment.
- [ ] Determine TGA regulatory status.
- [ ] Assess applicable standards and quality-system requirements.
- [ ] Separate regulated artefacts from exploratory research artefacts.
- [ ] Establish clinical/statistical validation plan.
- [ ] Establish adverse-event/safety reporting where applicable.

No Phase 9 activity may be inferred from completion of Phases 1–8.

---

## Runtime donor / clean-room policy

IGM may reuse mathematical ideas and engineering lessons from related QSOL repositories, but licence boundaries remain explicit.

- **RSH**: geometry/numerics, deterministic shard/prefix and accelerator-validation lineage. MPL-2.0. Credit J. Robitaille and Trent Slade; do not silently copy MPL files into Apache-2.0 IGM.
- **GLUBALL**: bounded Rust runtime, deterministic partitioning and evidence/throughput architecture lineage. MPL-2.0. Reimplement ideas cleanly for IGM.
- **ETQ-101/303 / SONIFICATION**: exact finite state addressing, product-space traversal and CRT inversion lineage. MPL-2.0. Reuse mathematics/contracts conceptually, not source text.
- **QSOL-NEXUS**: bounded resource/memory-management patterns. Apache-2.0.

See `docs/RUNTIME_LINEAGE.md`.
