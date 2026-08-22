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

## Change control

Changes that weaken these invariants require an explicit major governance review and must not be merged as routine implementation changes.
