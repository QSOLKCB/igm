# IGM Roadmap

## Phase 1 — Documentation and governance foundation

- [x] Establish non-clinical research intended purpose.
- [x] Establish Australian ethics/regulatory baseline.
- [x] Define no-patient-data public-repository rule.
- [x] Define replaceable evidence/model/runtime architecture.
- [x] Define V0–V4 validation ladder.
- [x] Define Flinders/SA Health handoff boundary without implying endorsement.
- [x] Add machine-readable governance policy and model-profile schema.
- [x] Add deterministic documentation validation CI.
- [x] Hard-code `INV-BIO-001`: **Perfect Mathematics Does Not Equal Perfect Biological Reality**.
- [x] Require V0–V2 profiles to keep biological-validity claims false.
- [x] Prevent `unknown` parameters from carrying invented values.
- [x] Require provenance for observed/source-derived/calibrated parameters.
- [x] Record source access/redistribution status rather than treating a public URL as reuse permission.
- [x] Add a dependency-free semantic profile validator for cross-field rules such as unique component identifiers.

### Phase 1 gate

No biological simulator implementation may claim more than V0 until its biologically meaningful parameters have traceable source provenance. Mathematical correctness, deterministic execution, CPU/GPU agreement, numerical convergence, and attractive visualizations do not promote biological validity.

---

## Phase 2 — Deterministic GitHub Pages visual laboratory

**Target:** PR #2.

Build a fresh Apache-2.0 browser implementation inspired by useful visualization concepts from VORTEX/VORTEX2, NEXUS, BioFabric, and the registered graph/tensor literature, without copying BioFabric source code or importing LGPL code into this repository.

### Clean-room implementation rule

- [ ] Implement all IGM visualization code from scratch under Apache-2.0.
- [ ] Use BioFabric only as a published visualization-method reference.
- [ ] Do not copy BioFabric Java source, internal algorithms, UI code, icons, assets, or bundled data.
- [ ] Cite Longabaugh 2012 and the official BioFabric site for the conceptual row/column presentation method.
- [ ] Keep the implementation intentionally simple enough for downstream researchers to fork, modify, and replace.
- [ ] Keep external runtime dependencies at zero or near-zero; prefer native browser APIs and repository-owned modules.

### Canonical V0 visualization profile

- [ ] Add `IGM-SCHEMATIC-PENTAMER-V0` as an explicitly synthetic/schematic profile.
- [ ] Represent five schematic subunit sectors, ten articulated Fab-arm placeholders, and one schematic J-chain constraint.
- [ ] Give every component a stable identifier.
- [ ] Label every biologically meaningful parameter as `observed`, `source-derived`, `calibrated`, `inferred`, `assumed`, or `unknown`.
- [ ] Run every profile through JSON Schema and `tools/validate_profile.py` before visualization.
- [ ] Never invent a value for a parameter whose status is `unknown`.

### Shared canonical state

The Pages application must not maintain unrelated per-view models. All views consume the same versioned state:

```text
IGM-MODEL-PROFILE-V1
        |
        v
 canonical IGM state
        |
  +-----+------+--------+---------+
  |            |        |         |
geometry      array    graph   provenance
  |            |        |         |
  +------------+--------+---------+
               |
               v
       derived observables
               |
               v
       deterministic views
```

- [ ] Keep model semantics outside presentation code.
- [ ] Keep view/camera state separate from model state.
- [ ] Make exported model state independent of the currently selected visualization.
- [ ] Ensure changing visualization mode cannot alter model identity or scientific semantics.

### View 1 — Assembly / spatial view

- [ ] Render the schematic assembly as an articulated spatial model.
- [ ] Support bounded rotation, pan, zoom, and component selection.
- [ ] Expose bounded schematic articulation controls without presenting them as measured biology.
- [ ] Display the current validation level and provenance class directly in the viewport.
- [ ] Keep `INV-BIO-001` visibly available in the interface and exported screenshots/recordings.

### View 2 — Numerical array / tensor view

- [ ] Render pairwise-distance matrices, constraint masks, contact-proxy arrays, or other declared numerical arrays.
- [ ] Distinguish a generic multidimensional array from a mathematically declared tensor.
- [ ] Do not call an array a tensor unless its transformation semantics are explicitly defined.
- [ ] Add a machine-readable tensor declaration where a true tensor representation is introduced.
- [ ] Test coordinate-invariant observables under rigid rotations/translations.

Research basis: Lim, *Tensors in computations* emphasizes transformation rules, equivariance, multilinearity, and separability; IGM adopts these as computational design guidance, not biological ontology.

### View 3 — Graph view

- [ ] Support typed relationships rather than one generic edge class.
- [ ] Support undirected, directed, weighted, multi-edge, bipartite, and hyperedge-capable schemas where justified by the model.
- [ ] Do not assign direction, weight, or biological meaning without source/profile justification.
- [ ] Expose graph metrics only as computational observables unless an external profile supports biological interpretation.
- [ ] Treat scale-free, small-world, hub, motif, and centrality interpretations as hypotheses to test, never category defaults.
- [ ] Include deterministic circular, structural, hierarchical, and adjacency-matrix layouts.

Research basis: registered graph-theory sources warn that inappropriate graph types and layouts can create misleading biological interpretation.

### View 4 — Fabric / relation view

Create an original IGM fabric-style renderer based only on the published presentation concept:

```text
node/component      -> one horizontal row
relationship        -> one vertical column
multi-edge relation -> separate visible columns
relationship class  -> deterministic grouped column regions
```

- [ ] Keep nodes and relationships visually orthogonal.
- [ ] Make multiple relationships between the same component pair separately visible.
- [ ] Allow deterministic grouping by relationship class such as structural, constraint, provenance, and derived.
- [ ] Add filters without deleting underlying model state.
- [ ] Provide row/column labels that remain useful in screenshots.
- [ ] Do not infer physical or biological proximity from row adjacency.
- [ ] Do not copy BioFabric code or assets.

### View 5 — Vortex-inspired coordinate projection

- [ ] Add an optional cyclic/vortex-inspired coordinate view for experimentation.
- [ ] Label it permanently as a **coordinate projection / parameterization only**.
- [ ] Never claim that IgM is a vortex or that vortex geometry explains an IgM biological mechanism.
- [ ] Make the same underlying state available in a non-vortex representation.

### Provenance inspector

- [ ] Clicking a component, parameter, constraint, edge, or derived observable shows its provenance record.
- [ ] Display source ID, source class, derivation, access/licence note, validation level, uncertainty, and unsupported claims where applicable.
- [ ] Distinguish `source-derived`, `assumed`, `inferred`, and `unknown` visually and textually.
- [ ] Never let a color alone carry provenance meaning.

### Deterministic telemetry and exports

- [ ] Show model/profile ID and version.
- [ ] Show profile hash/fingerprint.
- [ ] Show logical ensemble size separately from evaluated and displayed sample counts.
- [ ] Show finite/bounds/invariant status.
- [ ] Export canonical state JSON.
- [ ] Export derived-observable CSV where useful.
- [ ] Export provenance JSON.
- [ ] Export SVG or deterministic static snapshot where practical.
- [ ] Support bounded WebM recording using browser APIs without remote services.
- [ ] Include validation level and non-clinical status in exported visual artifacts.

### Browser determinism and Pages CI

- [ ] No `Math.random()` in canonical model generation.
- [ ] Deterministic sampling from logical ensembles.
- [ ] Bound visual complexity independently from logical ensemble size.
- [ ] Keep animation time separate from canonical state identity.
- [ ] Add deterministic model/view smoke tests.
- [ ] Add a generated Pages manifest with SHA-256 identities.
- [ ] Fail CI if `INV-BIO-001`, validation level, or non-clinical labeling disappears from the UI.
- [ ] Fail CI if visualization code mutates canonical model state as a side effect of switching layouts.
- [ ] Deploy only after governance, schema, model, and visualization checks pass.

### New visualization/math invariants to formalize in Phase 2

- [ ] `INV-MATH-002`: **A Multidimensional Array Is Not Automatically a Tensor.**
- [ ] `INV-MATH-003`: **Coordinate Presentation Must Not Alter Coordinate-Invariant Observables.**
- [ ] `INV-GRAPH-001`: **Graph Representation Must Match Declared Relationship Semantics.**
- [ ] `INV-GRAPH-002`: **Topology Is Measured or Sourced, Never Assumed.**
- [ ] `INV-VIZ-001`: **Visualization Layout Must Not Alter Model Semantics.**
- [ ] `INV-VIZ-002`: **Visual Proximity Does Not Imply Biological Proximity.**

### Phase 2 gate

The browser is a transparent research microscope, not the scientific authority and not the high-performance runtime. A perfectly deterministic rendering remains V0 unless independent biological evidence supports promotion. Visual layout, beauty, animation, or apparent symmetry may never become biological evidence.

---

## Phase 3 — Minimal deterministic Rust structural runtime

- [ ] Implement `IGM-RUST-RUNTIME-V1` as the native reference/orchestration layer.
- [ ] Consume the same `IGM-MODEL-PROFILE-V1` schema used by Pages.
- [ ] Implement the schematic pentamer + J-chain profile as V0 only.
- [ ] Represent subunits/domains with exact stable identifiers.
- [ ] Add articulated geometry with bounded angles/translations.
- [ ] Add explicit steric/contact constraint interfaces.
- [ ] Add exact integer indexing for discrete ensemble coordinates.
- [ ] Add bounded finite floating-point geometry at declared mathematical boundaries.
- [ ] Add canonical run manifest and hashes.
- [ ] Prove repeatability across worker counts.
- [ ] Add property-based and edge-case tests.
- [ ] Cross-check selected browser reference states against Rust.

### Phase 3 gate

The schematic model must remain clearly labelled as a computational fixture and must not be described as a validated IgM conformation. Runtime agreement with Pages establishes implementation agreement only.

---

## Phase 4 — Replaceable evidence adapters

- [ ] Define source-adapter interface.
- [ ] Maintain public structural-source registry with DOI/PDB/EMDB identifiers where applicable.
- [ ] Add cryo-EM parameter adapter.
- [ ] Add molecular-dynamics trajectory adapter.
- [ ] Add biochemical/calibration constraint adapter.
- [ ] Preserve source licence/access metadata.
- [ ] Require per-parameter provenance and uncertainty.
- [ ] Add conflict/unknown representation rather than forced reconciliation.
- [ ] Add source snapshots or hashes only where licence/access terms permit.

### Phase 4 gate

Source ingestion must not silently convert source observations into stronger claims than the source supports.

---

## Phase 5 — Tensor, graph and ensemble computational representations

- [ ] Define explicit numerical-array projections of model state.
- [ ] Define mathematically valid tensor types only where transformation semantics are declared.
- [ ] Define typed graph/hypergraph projections of domains, subunits, constraints, provenance, and relationships.
- [ ] Explore graph-structured tensor-network representations where they materially reduce computation and can be specified rigorously.
- [ ] Ensure projections preserve stable model identity where applicable.
- [ ] Add pair-distance/contact/accessibility observables.
- [ ] Add ensemble statistics with explicit numerical assumptions.
- [ ] Add uncertainty representations: unknown, interval, distribution, and ensemble.
- [ ] Keep optional vortex-inspired coordinates as a parameterization only.

### Phase 5 gate

A representation earns a scientific interpretation only from explicit source evidence and validation. Mathematical convenience is never sufficient.

---

## Phase 6 — GPU acceleration

- [ ] Keep Rust CPU execution as reference/orchestration authority.
- [ ] Add CUDA adapter for bounded conformational sweeps and tensor/array kernels.
- [ ] Add deterministic device sharding.
- [ ] Separate evidence mode from throughput mode.
- [ ] Require independent CPU/GPU residual comparison.
- [ ] Add complete-readback evidence profiles.
- [ ] Add Compute Sanitizer campaign support.
- [ ] Record GPU/toolchain provenance without publishing raw hardware identifiers.
- [ ] Run single-GPU baseline before multi-GPU scaling.
- [ ] Run 2/4/8 GPU scaling campaigns only after correctness gates pass.

### Phase 6 gate

GPU agreement is implementation evidence only. Performance observations cannot promote biological validation level.

---

## Phase 7 — Structure-informed research profiles

- [ ] Create first V1 source-informed IgM pentamer profile.
- [ ] Create a separately versioned hexamer profile if supported by sources.
- [ ] Quantify profile uncertainty and unsupported degrees of freedom.
- [ ] Compare model-derived observables with independent structural observations.
- [ ] Publish accepted and rejected calibration attempts.
- [ ] Keep V3/V4 promotion dependent on external calibration/independent research validation.

---

## Phase 8 — Research collaboration package

- [ ] Produce researcher onboarding guide.
- [ ] Produce reproducibility capsule format.
- [ ] Add export formats useful to structural biology workflows.
- [ ] Add example Jupyter/analysis adapters without making Python authoritative.
- [ ] Prepare a neutral research handoff package for external institutions.
- [ ] Invite domain experts to review biological assumptions, provenance and validation design.
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
