# IGM — AI/agent project contract

This file is normative for automated contributors together with `AGENTS.md`, `docs/CORE_INVARIANTS.md`, and `governance/policy.json`.

## Project purpose

IGM is research software for deterministic simulation of **hypothetical or source-informed IgM structural models**. The runtime may support articulated geometry, tensors, graphs, and optional vortex-inspired coordinates.

The project does **not** define biological truth. It provides reproducible computational machinery that qualified researchers can populate with stronger structural or experimental evidence.

## Hard invariant

**INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

This sentence is normative. Mathematical exactness, deterministic execution, numerical convergence, reproducibility, CPU/GPU agreement, or accelerator performance do not by themselves establish biological validity or clinical validity.

Agents must preserve:

```text
mathematical correctness
    != computational correctness
    != biological validity
    != clinical validity
```

See `docs/CORE_INVARIANTS.md` and `governance/policy.json`.

## Authority order

Use this authority order when resolving ambiguity:

1. published/approved external evidence explicitly registered with provenance;
2. versioned IGM model profile and its declared assumptions;
3. deterministic reference implementation;
4. accelerator implementations validated against the reference;
5. derived visualisations and performance observations.

Lower layers may not silently redefine higher layers.

## Hard medical boundary

Automated contributors MUST NOT introduce claims that the repository:

- diagnoses or screens for disease;
- predicts patient outcomes;
- recommends or selects treatment;
- estimates clinical response for an individual;
- monitors a patient's condition;
- prevents, treats or cures disease;
- constitutes a clinically validated digital biomarker;
- is a medical device or an approved clinical decision-support system;
- is endorsed by any university, hospital, regulator or research institute without explicit evidence.

If a requested feature would change the intended purpose toward a medical-device function, stop and require an explicit regulatory/ethics review task before implementation.

## Human-data default

`human_data_default = deny`.

Do not add patient records, case notes, pathology values, treatment histories, genomic data, imaging, identifiable information, coded participant data, or private clinical datasets to the public repository.

Synthetic fixtures and openly licensed/public structural data are preferred for software development.

## Biological claim discipline

Every biologically meaningful parameter must eventually carry provenance.

Allowed:

- `this model uses a five-sector schematic assembly`;
- `this parameter is derived from source X`;
- `this run satisfies computational invariant Y`;
- `this accelerator agrees with the reference within tolerance Z`.

Not allowed without independent evidence:

- `IgM behaves as a vortex`;
- `this geometry explains a disease mechanism`;
- `this simulation predicts pathogenic accumulation`;
- `GPU agreement validates the biology`.

A mathematically useful representation is not automatically a biological ontology.

## Replaceable-source requirement

Do not hard-code biological assumptions into scheduler, sharding, evidence, or accelerator infrastructure when they can be isolated in a model profile or adapter.

Preferred architecture:

```text
source -> adapter -> model profile -> reference runtime -> accelerator -> observables -> evidence
```

A future researcher should be able to replace schematic inputs with cryo-EM, molecular-dynamics, biochemical or calibrated experimental inputs without rewriting the execution engine.

## Validation labels

Use the repository validation ladder:

- `V0`: synthetic/schematic;
- `V1`: source-informed;
- `V2`: computationally reproduced;
- `V3`: externally calibrated;
- `V4`: independently validated research model.

Agents may automatically establish V0–V2 only when the corresponding requirements are genuinely met. V3/V4 require explicit external evidence and qualified research judgement.

Never infer a clinical status from these labels.

## Australian governance baseline

Consult `docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md` before adding anything involving humans, health information, clinical use, clinical investigation, or patient-facing claims.

Current project framing is intentionally non-clinical research software.

## Engineering principles

- deterministic inputs and outputs where feasible;
- exact integer indexing for discrete domains;
- bounded numeric inputs;
- stable schemas;
- explicit versioning;
- fail closed on malformed provenance;
- preserve rejected/failed validation reports where useful;
- distinguish correctness evidence from throughput benchmarks;
- accelerators are implementations, not scientific authorities;
- do not weaken tests to make a model pass.

## Contribution rule

When uncertain whether a change is a computational claim or a biological/clinical claim, classify it as the latter and require stronger review.
