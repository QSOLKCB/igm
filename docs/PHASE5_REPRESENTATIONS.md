# Phase 5 Tensor, Graph, and Ensemble Computational Representations

Status: implementation-complete on PR #11, pending review/merge.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**
>
> **INV-MATH-002: A Multidimensional Array Is Not Automatically a Tensor.**
>
> **Phase 5 gate: A representation earns scientific interpretation only from explicit evidence and validation.**

Phase 5 adds computational representation machinery on top of the merged Phase 4 evidence/provenance boundary. It does not create a V1 biological model, biological validation, molecular-dynamics realism, clinical validity, or patient-specific meaning.

## Contracts

```text
IGM-PHASE5-REPRESENTATION-V1
IGM-PHASE5-REPRESENTATION-CONFIG-V1
IGM-NUMERICAL-ARRAY-PROJECTION-V1
IGM-DECLARED-TENSOR-V1
IGM-MODEL-GRAPH-V1
IGM-MODEL-HYPERGRAPH-V1
IGM-PROVENANCE-GRAPH-V1
IGM-TENSOR-FACTOR-GRAPH-V1
IGM-VISUALIZATION-GRAPH-V1
IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1
IGM-ENSEMBLE-STATISTICS-V1
IGM-COMPUTATIONAL-UNCERTAINTY-V1
IGM-TENSOR-NETWORK-ASSESSMENT-V1
IGM-VORTEX-INSPIRED-PROJECTION-V1
IGM-PHASE5-REPRESENTATION-GATE-V1
```

Rust implementation:

```text
runtime/rust/src/phase5.rs
runtime/rust/src/representation_main.rs
runtime/rust/src/lib_v6.rs
```

Reference V0 representation profile:

```text
runtime/profiles/igm-phase5-v0.json
```

## Explicit numerical arrays

Phase 5 exposes plain numerical projections without calling them tensors.

The V0 bundle currently includes:

```text
cartesian-position-array      shape [N, 3]
pair-distance-squared-array   shape [N, N]
```

Both are row-major f64 arrays with stable component labels and:

```text
tensor_claimed = false
scientific_interpretation_claimed = false
```

This preserves `INV-MATH-002` mechanically. A useful matrix is still just a matrix unless transformation semantics justify a tensor claim.

## Declared tensor semantics

The first Phase 5 object allowed to set `tensor_claimed=true` is the centered Cartesian coordinate tensor.

It has shape:

```text
[N components, 3 Cartesian coordinates]
```

and explicitly declares:

- component-axis action by finite-basis permutation under component reindexing;
- Cartesian-axis action as a vector representation under orthogonal frame transformations;
- translation removal by centroid subtraction before projection;
- scalar field `R/f64`.

The tensor claim exists because the transformation semantics are stated and testable, not because the object has two dimensions.

## Typed model graph

`IGM-MODEL-GRAPH-V1` projects only relationships already present in the validated model profile.

The graph contains typed nodes for:

- components;
- declared component domains;
- constraints.

Edges are incidence/classification edges such as:

```text
member-of-declared-domain
participates-in-declared-constraint
```

A multi-participant constraint is not silently exploded into pairwise biological edges.

## Typed model hypergraph

`IGM-MODEL-HYPERGRAPH-V1` preserves n-ary constraint participation directly.

For each constraint with an explicit participant list, the hyperedge retains:

- constraint identifier;
- relationship/constraint kind;
- participant identifiers;
- evidence status;
- source identifiers where present.

The V0 projection fixes:

```text
pairwise_expansion_performed = false
```

so hypergraph convenience cannot manufacture pairwise biology.

## Provenance graph

`IGM-PROVENANCE-GRAPH-V1` is a separate namespace that connects model entities to declared source identifiers.

It never becomes the model graph. For the current V0 fixture, there are no biological source identifiers, so the graph correctly contains no fabricated source authority.

A future source-informed profile may populate provenance edges only through validated profile/source identities from the Phase 4 boundary.

## Graph namespace separation

Phase 5 keeps these objects explicitly distinct:

```text
model graph          IGM-MODEL-GRAPH-V1
execution graph      IGM-EXEC-GRAPH-C5-K2-C3-V1
tensor-factor graph  IGM-TENSOR-FACTOR-GRAPH-V1
visualization graph  IGM-VISUALIZATION-GRAPH-V1
```

The representation gate records every cross-namespace merge flag as `false` and fixes:

```text
cross_namespace_semantic_promotion_claimed = false
```

Execution adjacency therefore remains execution metadata, tensor-factor adjacency remains factorization metadata, and visualization adjacency remains presentation metadata.

## Pair-distance and computational-contact observables

`IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1` computes all unique Euclidean pair distances for the validated geometry.

For the 16-node V0 fixture this produces exactly:

```text
16 choose 2 = 120 unique pair observables
```

Each record contains:

- component IDs and indices;
- squared distance;
- Euclidean distance;
- a computational contact boolean under an explicit assumed cutoff;
- `biological_contact_claimed=false`.

The cutoff lives in `runtime/profiles/igm-phase5-v0.json`. It is an explicit V0 computational assumption, not a measured molecular-contact threshold.

## Geometric accessibility

Phase 5 defines a narrow accessibility observable as nearest-neighbour geometric clearance under another explicit V0 threshold.

For each component it records:

```text
nearest_neighbor_distance
clearance_threshold
geometric_accessibility
biochemical_accessibility_claimed = false
```

This does not claim solvent accessibility, epitope accessibility, binding accessibility, complement accessibility, or any biochemical mechanism.

## Ensemble statistics

The V0 Phase 5 profile uses an explicit deterministic index set from the existing Phase 3B synthetic conformation domain:

```text
indices = [0, 11704, 23408]
sampling = explicit-index-set
population_variance = true
```

Before those summaries are used, Phase 5 executes the fixed Phase 3B residual verification gate. If that gate fails, ensemble construction fails.

For each explicit member, a one-conformation Phase 3B run supplies bounded pair-distance extrema. Phase 5 then reports deterministic statistics for minimum and maximum pair distance:

- count;
- minimum;
- maximum;
- mean;
- median;
- population variance;
- population standard deviation.

The numerical assumption is explicit: the variance is the population variance of the listed deterministic index set only. No random-sampling or population-inference claim is made.

The bundle fixes:

```text
biological_ensemble_claimed = false
scientific_interpretation_claimed = false
```

## Computational uncertainty types

Phase 5 adds a computational uncertainty vocabulary separate from Phase 4 evidence uncertainty:

```text
unknown
interval
distribution
ensemble
```

`unknown` requires a reason.

`interval` requires finite ordered bounds.

`distribution` is metadata-only in this phase. It requires an explicit family and finite parameters; no stochastic sampling is silently performed. A normal distribution requires non-negative standard deviation.

`ensemble` requires a non-zero member count plus finite ordered summary bounds.

These types organize computation. They do not increase evidence strength:

```text
evidence_strength_promoted = false
```

## Tensor-network factorization assessment

Phase 5 does not adopt tensor networks merely because they sound sophisticated.

`IGM-TENSOR-NETWORK-ASSESSMENT-V1` requires both:

1. exact reconstruction; and
2. a material reduction in the declared computational/storage measure.

The current V0 centered-coordinate tensor is assessed using the exact identity factorization:

```text
X = X * I3
```

This is exact, but it stores more elements than the dense tensor. Therefore the current result is deliberately:

```text
exact_reconstruction_verified = true
material_reduction = false
admitted = false
performance_claim = false
```

That negative result is useful. It demonstrates that Phase 5 can reject a mathematically valid factorization when it does not buy anything.

A future tensor-network design must state its tensor semantics, factor graph, exactness/residual contract, computational measure, and evidence boundary before admission.

## Optional vortex-inspired projection

The Phase 5 configuration defaults to:

```text
vortex_inspired_projection_enabled = false
```

When explicitly enabled, the projection emits only a representational radial-squared / Cartesian phase embedding plus axial coordinate.

It fixes:

```text
biological_ontology_claimed = false
scientific_interpretation_claimed = false
```

The projection is optional presentation mathematics. It is not an IgM ontology, biological vortex claim, force law, or mechanistic hypothesis.

## Phase 4 boundary consumption

Phase 5 does not define a second evidence-ingestion system.

The bundle binds:

```text
phase4_source_adapter_contract = IGM-SOURCE-ADAPTER-V1
phase4_boundary_consumed = true
```

Source/provenance authority continues to come from validated model profiles and the Phase 4 registry/adapters. Representation code cannot strengthen a source claim.

## Representation gate

An accepted Phase 5 bundle must establish all of the following:

1. Phase 4 provenance boundary consumed;
2. plain arrays remain non-tensors;
3. any true tensor declares transformation semantics;
4. graph namespaces remain separate;
5. contact/accessibility thresholds are explicit assumptions;
6. ensemble membership/statistical assumptions are explicit;
7. uncertainty kinds are explicit and validated;
8. tensor-network admission requires exactness plus material reduction;
9. vortex-inspired projection remains optional and non-ontological;
10. no scientific, biological, or clinical validity promotion occurs.

The executable gate contract is:

```text
IGM-PHASE5-REPRESENTATION-GATE-V1
```

and the normative rule remains:

> **A representation earns scientific interpretation only from explicit evidence and validation.**

Passing this gate means the representation layer is internally well-formed and non-promoting. It does not mean the representation is biologically correct.

## CLI

After a release build:

```bash
./target/release/igm-represent bundle \
  profiles/igm-schematic-pentamer-v0.json \
  runtime/profiles/igm-phase5-v0.json
```

The output is a deterministic JSON representation bundle with SHA-256 identity and no timing-bearing correctness semantics.
