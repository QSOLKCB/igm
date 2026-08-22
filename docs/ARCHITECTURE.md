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
| Source adapters                                             |
| parse -> identify -> normalize -> preserve uncertainty      |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Versioned model profile                                    |
| components | coordinates | bounds | constraints | sources  |
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
| Accelerator adapters                                       |
| CUDA / future GPU backends                                 |
| validated against reference, never scientific authority    |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Observables and ensembles                                  |
| distances | accessibility | contacts | steric rejects      |
+-----------------------------+-----------------------------+
                              |
                              v
+-----------------------------------------------------------+
| Evidence package                                           |
| inputs | versions | provenance | tolerances | results       |
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

A graph can encode:

- components/domains as nodes;
- structural connections as edges;
- hypothesised interactions as separately typed edges;
- provenance links;
- constraint dependencies.

Observed and hypothesised edges must be distinguishable.

### Vortex-inspired coordinate adapter

A vortex-like or cyclic coordinate scheme may be useful for indexing, phase-like angles, cyclic assemblies, or visualisation.

The rule is strict:

> A vortex coordinate system describes how the simulator parameterizes state. It does not establish that IgM is a physical vortex or that vortex dynamics explain IgM biology.

This adapter should remain optional and replaceable.

## Runtime principles

The planned Rust reference runtime should prefer:

- integer identities and indices;
- bounded numeric domains;
- deterministic work partitioning;
- canonical serialization;
- explicit overflow checks;
- reproducible random/sampling seeds if stochastic methods are added;
- no hidden dependence on GPU scheduling order;
- stable error taxonomy;
- fail-closed provenance validation.

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

## Source replacement contract

A researcher replacing schematic inputs with cryo-EM, MD or biochemical evidence should ideally need to change only:

1. source registry entry;
2. source adapter;
3. model profile;
4. validation/calibration records.

They should not need to rewrite:

- worker scheduling;
- device sharding;
- evidence manifests;
- run hashing;
- accelerator orchestration;
- generic observables that remain semantically compatible.

## Scientific non-authority of the runtime

The runtime can establish facts such as:

- input profile X produced output Y;
- all configured constraints were evaluated;
- a run was complete;
- CPU and GPU implementations agreed within tolerance;
- results were reproducible under a declared environment.

It cannot, by itself, establish:

- that the profile is biologically correct;
- that a simulated conformation occurs in vivo;
- that an observable is clinically meaningful;
- that a mechanism explains disease;
- that an intervention would work.

Those are downstream scientific questions.
