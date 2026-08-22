# Runtime Design Lineage and Attribution

IGM is Apache-2.0. Phase 3 reuses mathematical and engineering ideas from related QSOL projects while keeping IGM source files independently implemented for this repository.

## RSH

RSH is the strongest geometry/numerics ancestor for the native runtime design. Its public citation metadata credits **J. Robitaille and Trent Slade** for the Robitaille-Slade Helix geometry/evidence runner.

IGM gratefully acknowledges **J. Robitaille (Dr J) for the RSH geometry contribution** and the wider RSH work that demonstrated useful patterns for:

- bounded f64 geometry;
- explicit frame/vector operations;
- deterministic evidence records;
- SE(3)-style transform composition;
- deterministic shard/prefix reconstruction;
- CPU/reference versus accelerator separation;
- residual-driven CUDA validation.

RSH is MPL-2.0. IGM does not copy or relicense RSH MPL source into Apache-2.0 runtime files. The PR3 Rust implementation was written for IGM from the mathematical descriptions, public contracts, and general engineering lessons.

RSH citation record:

- project: `QSOLKCB/RSH`
- title: *RSH: Robitaille-Slade Helix Geometry and Evidence Runner*
- authors: J. Robitaille; Trent Slade
- DOI: `10.5281/zenodo.21959297`
- licence: MPL-2.0

## GLUBALL

GLUBALL contributes design lineage for:

- integer-first discrete indexing;
- bounded runtime inputs;
- deterministic quotient/remainder partitioning;
- checked count/overflow arithmetic;
- geometry at an explicit floating-point boundary;
- worker-count-independent diagnostic folding;
- evidence/throughput separation for later accelerators.

GLUBALL is MPL-2.0. PR3 does not copy GLUBALL source into IGM; the techniques are independently implemented under IGM's Apache-2.0 licence.

## ETQ-101 / ETQ-303

The QSOL `SONIFICATION` repository's ETQ-303 exact event protocol provides an important state-management idea: **storage identity, exact traversal order, and graph topology are distinct things**.

ETQ-303 uses exact finite product-state indexing and Chinese-remainder traversal/inversion. IGM adapts that mathematical idea to a new execution-only state cell:

```text
Z5 x Z2 x Z3 = 30 states
```

The resulting `IGM-CRT-PENTAFOLD-30-V1` traversal is an original IGM contract. It does not import E8 root geometry, ETQ physical interpretation, audio semantics, or a claim that the traversal is a biological graph walk.

The SONIFICATION implementation is MPL-2.0; no MPL source is copied into IGM.

## QSOL-NEXUS

NEXUS is Apache-2.0 and provides useful bounded-resource engineering precedent, including explicit maximum input sizes and rejecting work before unbounded allocation. IGM adopts that design philosophy for profile bytes, component counts, work-item counts, and worker counts.

PR3 does not import NEXUS application semantics. It reuses only generic bounded-memory/resource-management patterns suitable for a numerical runtime.

## Quake-style inverse square root

No Quake III inverse-square-root source or magic constant is copied into PR3.

A future GPU optimization may evaluate hardware reciprocal-square-root or Newton-refined approximations, but only as an explicitly named accelerator profile with residual validation against the Rust reference. Approximate normalization must never silently replace reference arithmetic.

## Scientific boundary

Design lineage does not create biological evidence.

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

RSH geometry, GLUBALL runtime architecture, ETQ traversal mathematics, and NEXUS resource-management patterns are computational donors only. None is cited as evidence that an IgM protein has the ontology of those donor models.
