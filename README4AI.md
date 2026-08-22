# IGM — AI/agent project contract

This file is normative for automated contributors together with `AGENTS.md`, `docs/CORE_INVARIANTS.md`, and `governance/policy.json`.

## Project purpose

IGM is research software for deterministic simulation of **hypothetical or source-informed IgM structural models**. The runtime may support articulated geometry, tensors, graphs, optional vortex-inspired coordinates, deterministic execution graphs, CPU optimization profiles, and future accelerator adapters.

The project does **not** define biological truth. It provides reproducible computational machinery that qualified researchers can populate with stronger structural or experimental evidence.

## Hard biological invariant

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

## Hard runtime invariant

**INV-RUNTIME-001: Execution Adjacency Does Not Imply Biological Adjacency.**

Execution/scheduling graph edges, CRT traversal order, memory adjacency, SIMD/warp lanes, chunk membership, worker assignment, and device assignment are computational metadata only.

Do not convert any of them into:

- molecular contact;
- biological proximity;
- biochemical interaction;
- causal influence;
- structural evidence;
- a biological graph edge.

Keep model/biological, execution, provenance, visualization, and tensor-factor graph namespaces separate unless an explicit adapter defines a mapping.

## Authority order

Use this authority order when resolving ambiguity:

1. published/approved external evidence explicitly registered with provenance;
2. versioned IGM model profile and its declared assumptions;
3. deterministic Rust reference implementation;
4. admitted optimization profiles validated against the reference;
5. campaign/orchestration records;
6. accelerator implementations validated against the reference;
7. derived visualisations and performance observations.

Lower layers may not silently redefine higher layers.

## Current runtime contracts

Current V0 runtime infrastructure includes:

```text
IGM-RUST-RUNTIME-V1
IGM-CRT-PENTAFOLD-30-V1
IGM-PENTA-CRT-CPU-V1
IGM-EXEC-GRAPH-C5-K2-C3-V1
IGM-WARP32-AOSOA-V1
IGM-EXEC-CAMPAIGN-V1
```

The PENTA-CRT CPU profile contains 23,409 explicit **synthetic execution states**. That cardinality is not a claim about the number of real IgM conformations.

The execution graph contains 30 scheduler vertices. That is not a claim about IgM biological state count or biological adjacency.

The padded memory cell contains 32 runtime lanes, of which 30 are meaningful execution addresses and two are inactive non-semantic padding. Padding lanes never enter scientific/model counts.

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
- `this execution graph contains 30 scheduling vertices`;
- `this campaign preserved correctness identity across worker/chunk plans`;
- `this accelerator agrees with the reference within tolerance Z`.

Not allowed without independent evidence:

- `IgM behaves as a vortex`;
- `the execution graph describes IgM biological interactions`;
- `32 runtime lanes imply 32 biological states`;
- `this geometry explains a disease mechanism`;
- `this simulation predicts pathogenic accumulation`;
- `GPU agreement validates the biology`.

A mathematically useful representation is not automatically a biological ontology.

## Replaceable-source requirement

Do not hard-code biological assumptions into scheduler, sharding, evidence, campaign, memory-layout, or accelerator infrastructure when they can be isolated in a model profile or adapter.

Preferred architecture:

```text
source
  -> adapter
  -> model profile
  -> reference runtime
  -> admitted optimization
  -> bounded campaign/orchestration
  -> accelerator
  -> observables
  -> evidence
```

A future researcher should be able to replace schematic inputs with cryo-EM, molecular-dynamics, biochemical or calibrated experimental inputs without rewriting generic execution infrastructure. If a future profile violates an optimization assumption, the optimization must fail admission instead of silently surviving source replacement.

## Campaign rules

For `IGM-EXEC-CAMPAIGN-V1`:

- memory budgets are checked before execution;
- campaigns are split into deterministic contiguous chunks when needed;
- correctness identity is separate from worker/chunk/memory-plan identity;
- timing and throughput belong only in benchmark receipts;
- `performance_claim=false` remains mandatory for local observations;
- rejected campaigns are preserved with failure stage/reason;
- accepted artifacts receive external SHA-256 checksums;
- environment records omit hostname, username, GPU UUID, MAC, serial number, and similar raw machine identifiers by default;
- benchmark success cannot promote biological validity.

Validate persisted campaigns with:

```bash
python3 tools/validate_campaign.py CAMPAIGN_DIR
```

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
- fail closed on invalid optimization assumptions;
- preserve rejected/failed validation reports where useful;
- distinguish correctness evidence from throughput benchmarks;
- keep execution graph semantics separate from biological graph semantics;
- exclude padding lanes from scientific counts;
- accelerators are implementations, not scientific authorities;
- do not weaken tests or tolerances to make a model pass.

## Contribution rule

When uncertain whether a change is a computational claim or a biological/clinical claim, classify it as the latter and require stronger review.
