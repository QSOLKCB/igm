# IGM Core Invariants

These invariants are normative project constraints. Implementations, model profiles, accelerators, visualisations, reports and automated agents must preserve them.

## INV-BIO-001 — Perfect Mathematics Does Not Equal Perfect Biological Reality

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

A mathematically exact, internally consistent, deterministic, reproducible or numerically converged model is not thereby a biologically correct model.

This invariant forbids promotion by computation alone.

The following observations are insufficient, individually or together, to establish biological truth:

- exact algebraic identities;
- deterministic execution;
- zero numerical residual inside a model;
- CPU/GPU agreement;
- cross-machine reproducibility;
- stable tensor or graph structure;
- attractive geometric symmetry;
- high-resolution rendering;
- large-scale parameter sweeps;
- benchmark performance;
- statistical regularity in synthetic or assumed inputs.

Biological interpretation requires evidence outside the mathematics/runtime, such as appropriately sourced structural, biochemical, experimental or independently validated research evidence.

### Required consequence

Every report produced by IGM must preserve the distinction:

```text
mathematical correctness
    != computational correctness
    != biological validity
    != clinical validity
```

A downstream research team may establish stronger biological support through appropriate evidence and validation, but that promotion must be explicit, traceable and external to mere runtime success.

### Agent rule

If an automated agent encounters language that implies "the model is mathematically perfect, therefore the biology is correct", it must reject or rewrite that claim.

### Accelerator rule

GPU agreement or performance can validate an implementation against a reference. It cannot validate a biological mechanism.

### Model-profile rule

A model profile must identify biologically meaningful parameters as one of:

- observed;
- source-derived;
- calibrated;
- inferred;
- assumed;
- unknown.

No parameter may be silently promoted to `observed` or `source-derived` because the model becomes numerically stable.

## INV-BIO-002 — Representation Is Not Ontology

Geometry, tensors, graphs, vortex-inspired coordinates and other mathematical representations are computational tools. Their usefulness does not establish that the biological system literally has the same ontology as the representation.

## INV-BIO-003 — Unknown Beats Plausible Invention

When biological evidence is absent or ambiguous, preserve `unknown` or an explicitly labelled assumption rather than inventing a plausible value and presenting it as evidence.

## INV-BIO-004 — Runtime Success Is Not Clinical Evidence

No passing test, accepted run, validation receipt, residual report, benchmark or reproducibility record may be described as diagnosis, prognosis, treatment evidence, patient monitoring or clinical validation unless a separate appropriately governed research program has actually established that claim.

## INV-MATH-002 — A Multidimensional Array Is Not Automatically a Tensor

A convenient multi-indexed array used for computation or rendering is not automatically a mathematical tensor. IGM may call an object a tensor only when its transformation semantics or another mathematically valid tensor definition is explicitly declared.

The Phase-2 pairwise-distance heatmap is therefore labelled a **numerical array, not a declared tensor**.

## INV-MATH-003 — Coordinate Presentation Must Not Alter Coordinate-Invariant Observables

Changes of presentation coordinates may alter displayed coordinates while leaving declared coordinate-invariant observables unchanged. For the Phase-2 schematic, rigid rotation and translation must preserve pairwise Euclidean distances within the declared numerical tolerance.

A failure of this invariant is a computational defect, not biological evidence.

## INV-GRAPH-001 — Graph Representation Must Match Declared Relationship Semantics

Graph direction, weights, multiplicity, bipartite partitions, hyperedges and relationship classes must be justified by the model/profile semantics. A renderer must not invent relationship meaning because a graph type is visually convenient.

## INV-GRAPH-002 — Topology Is Measured or Sourced, Never Assumed

Topological interpretations such as scale-free, small-world, hub, motif, centrality or community structure are hypotheses or measured computational observables. They must not be assumed simply because the subject is biological.

## INV-VIZ-001 — Visualization Layout Must Not Alter Model Semantics

Assembly, array, graph, fabric, vortex-inspired, camera, filtering and layout controls are presentation state. Switching them must not mutate canonical model identity, profile claims or scientific semantics.

## INV-VIZ-002 — Visual Proximity Does Not Imply Biological Proximity

Objects placed near each other on screen, adjacent fabric rows, nearby graph nodes, or visually clustered marks are not thereby biologically close or interacting. Such interpretation requires explicit model semantics and appropriate evidence.

## INV-RUNTIME-001 — Execution Adjacency Does Not Imply Biological Adjacency

Runtime scheduling structures are not biological structures.

In particular, adjacency in `IGM-EXEC-GRAPH-C5-K2-C3-V1`, CRT traversal order, memory-neighbour placement, warp-lane position, SIMD grouping, chunk membership, worker assignment, or device assignment must not be interpreted as physical proximity, molecular contact, biochemical interaction, causal influence, or any other biological relationship.

The execution graph is permitted to be chosen for deterministic traversal, locality, memory coalescing, parallel scheduling, or other computational reasons. A separate model/profile graph must carry any biological or structural relationship semantics, with appropriate provenance.

### Required consequence

The project must keep these namespaces conceptually separate:

```text
model / biological graph
execution / scheduling graph
provenance graph
visualization graph
tensor-factor graph
```

An implementation may map between them only through an explicit declared adapter. A convenient execution mapping cannot silently create biological edges.

## Change control

Changes that weaken these invariants require an explicit major governance review and must not be merged as routine implementation changes.
