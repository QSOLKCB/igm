# IGM

**Research-grade simulation infrastructure for immunoglobulin M (IgM) structural hypotheses, with strict separation between computation and biological/clinical claims.**

IGM is an open-source research software project for building deterministic, replaceable models of IgM assemblies. The project is intentionally designed so that simplified geometric constraints can later be replaced by better cryo-EM, molecular-dynamics, biochemical, or experimentally calibrated inputs without rewriting the execution runtime.

> **Important:** this repository is research software, not a medical device, diagnostic system, treatment tool, clinical decision-support system, or patient-specific predictor. It does not establish biological truth. Model outputs are hypotheses or computational observations whose biological interpretation belongs to appropriately qualified researchers and, where applicable, regulated research processes.

## Core invariant

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

This is hard-coded as `INV-BIO-001` across the project governance and agent contracts. Mathematical exactness, deterministic execution, numerical convergence, reproducibility, CPU/GPU agreement, and accelerator performance cannot by themselves establish biological or clinical validity.

The project preserves:

```text
mathematical correctness
    != computational correctness
    != biological validity
    != clinical validity
```

See [Core Invariants](docs/CORE_INVARIANTS.md).

## Runtime invariant

> **Execution Adjacency Does Not Imply Biological Adjacency.**

`INV-RUNTIME-001` prevents scheduler graphs, CRT traversal order, memory adjacency, warp/SIMD lanes, chunks, workers, or future device assignments from being mistaken for molecular contacts or biological relationships.

The runtime is allowed to arrange work in mathematically convenient ways. Biology is under no obligation to share that arrangement.

## Why this repository exists

IgM is structurally interesting because experimentally observed assemblies can involve multimeric organization, flexible antibody arms, asymmetric constraints, and large conformational state spaces. Those properties are computationally suitable for deterministic CPU/GPU exploration, provided the software does not confuse a convenient mathematical representation with biology.

The project therefore separates the main layers:

```text
published / approved structural evidence
                |
                v
        source adapters
                |
                v
      IGM model profiles
   geometry | tensor | graph
                |
                v
 deterministic Rust reference
                |
                v
 PENTA-CRT optimization profile
                |
                v
 campaign / memory / receipt layer
                |
        +-------+-------+
        |               |
        v               v
   CPU reference    future GPU adapters
        |               |
        +-------+-------+
                v
       derived observables
                |
                v
 provenance + validation reports
```

A vortex-like coordinate system, tensor representation, graph representation, execution graph, or articulated geometry may be useful computationally. None of those representations is, by itself, a claim that an IgM molecule *is* a vortex, tensor, graph, scheduler topology, or any other mathematical object.

## Research boundary

This repository MUST NOT be used to claim that it:

- diagnoses, predicts, prevents, monitors, treats, or cures disease;
- recommends therapy, dosage, clinical management, or patient action;
- represents an individual patient's IgM or disease state without an independently approved research protocol;
- validates a biological mechanism merely because a simulation is numerically stable;
- substitutes for wet-lab, structural-biology, clinical, ethics, regulatory, or statistical validation;
- is approved, endorsed, sponsored, or clinically validated by Flinders University, SA Health, the TGA, NHMRC, or any other institution unless that body explicitly says so.

See [Medical and Research Boundary](docs/MEDICAL_RESEARCH_BOUNDARY.md) and [Disclaimer](DISCLAIMER.md).

## Australian research-ethics foundation

The documentation is designed to align with the principles and responsibilities in the current Australian research environment, including:

- NHMRC **National Statement on Ethical Conduct in Human Research 2025**;
- **Australian Code for the Responsible Conduct of Research 2018**;
- the **Privacy Act 1988** and Australian Privacy Principles where personal information is involved;
- TGA guidance on software-based medical devices, with an explicit non-clinical intended purpose for this repository.

These references are a governance baseline, not legal advice and not a declaration that every downstream use is compliant. Downstream researchers remain responsible for ethics approval, research governance, privacy, data agreements, institutional policies, regulatory classification, and professional obligations applicable to their work.

See [Australian Ethics and Regulatory Foundation](docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md).

## Replaceable model architecture

A core design requirement is **replaceability without runtime replacement**.

A model profile should be able to declare, for example:

- assembly cardinality;
- subunit/domain identifiers;
- articulated joints and bounded ranges;
- spatial constraints;
- tensor fields or feature arrays;
- graph edges and interaction hypotheses;
- provenance for every biologically meaningful parameter;
- evidence level and validation status.

The runtime should consume the profile through a stable interface. Better evidence should therefore replace a profile or adapter, not force a rewrite of deterministic scheduling, GPU sharding, evidence manifests, or reproducibility tooling.

## Current deterministic runtime

### Phase 3A reference

`IGM-RUST-RUNTIME-V1` provides the bounded Rust reference/orchestration layer with exact integer indexing, deterministic worker partitioning, checked f64 geometry, reproducible identities, and browser-fixture parity.

### Phase 3B PENTA-CRT CPU profile

`IGM-PENTA-CRT-CPU-V1` adds an explicit synthetic 23,409-state execution profile:

```text
17 × 17 × 9 × 9 = 23,409
```

It uses mixed-radix indexing, deterministic articulation lookup tables, PENTAFOLD/C5 reuse where mathematically valid, sparse J-marker corrections, and squared-distance hot loops. The accepted optimization reuses the C5-structured XY projection and restores exact local Z residuals rather than pretending the entire V0 3D fixture is block-circulant.

### Phase 3C campaign layer

`IGM-EXEC-CAMPAIGN-V1` adds:

- exact `C5 □ K2 □ C3` execution graph receipts;
- `INV-RUNTIME-001`;
- a 32-lane aligned memory contract with 30 meaningful and two inactive padding lanes;
- bounded memory planning and deterministic chunk streaming;
- worker/chunk-independent correctness receipts;
- separate benchmark observations;
- privacy-safe environment provenance;
- accepted/rejected campaign preservation and `SHA256SUMS`.

The execution graph and padded lanes are scheduler/memory structures only. They are not biological state claims and do not establish GPU execution.

See [Phase 3C Execution Campaigns](docs/EXECUTION_CAMPAIGNS.md).

## Planned computational representations

### Articulated geometry

Useful for bounded conformational sweeps, reach/accessibility calculations, steric checks, and geometric observables.

### Tensor representation

Useful for highly parallel numeric workloads, pairwise distances, masks, contact fields, constraint evaluation, ensemble statistics, and GPU acceleration.

### Graph representation

Useful for structural connectivity, domain/subunit relationships, interaction hypotheses, and data provenance.

### Vortex-inspired coordinates

Permitted as an optional **coordinate/parameterization strategy only**. A vortex parameterization must never be promoted to a biological mechanism without independent evidence.

See [Architecture](docs/ARCHITECTURE.md).

## Validation ladder

The project distinguishes computational correctness from biological validation:

| Level | Meaning |
|---|---|
| V0 | synthetic/schematic model only |
| V1 | source-informed parameterization with traceable provenance |
| V2 | independently reproduced computational outputs |
| V3 | calibrated against external structural/experimental observations |
| V4 | independently validated by qualified researchers under an appropriate protocol |

Only qualified downstream research should promote a model into V3/V4, and no validation level turns this repository into a clinical tool automatically.

See [Validation Ladder](docs/VALIDATION_LADDER.md).

## Human and clinical data

The public repository defaults to **no patient data**.

Do not commit identifiable, re-identifiable, coded clinical, genomic, pathology, treatment, imaging, or other human participant data to this repository. Research involving human participants or their data must follow the relevant ethics, governance, privacy and institutional processes before that data is introduced into a research workflow.

## Flinders research handoff

The project is intentionally documented so that researchers at institutions such as the **Flinders Centre for Innovation in Cancer** can evaluate it as a research artifact without inheriting unsupported biological claims. Any actual Flinders or SA Health study would remain subject to the institution's own ethics, governance, data, regulatory and scientific review requirements.

See [Flinders Research Handoff](docs/FLINDERS_RESEARCH_HANDOFF.md).

## Quick runtime examples

```bash
cargo test --locked --all-targets
cargo build --locked --release

./target/release/igm-runtime validate
./target/release/igm-penta-crt verify --samples 257
./target/release/igm-campaign graph
./target/release/igm-campaign layout
./target/release/igm-campaign run \
  --start 100 --count 4096 --workers 16 \
  --budget-bytes 1048576 --verify-samples 257 \
  --out artifacts/campaign-example

python3 tools/validate_campaign.py artifacts/campaign-example
```

## Repository map

```text
README.md                         human overview
README4AI.md                      machine-oriented project contract
AGENTS.md                         mandatory agent rules
DISCLAIMER.md                     research/medical/legal-context disclaimer
CONTRIBUTING.md                   contribution and provenance rules
ROADMAP.md                        staged implementation plan
docs/CORE_INVARIANTS.md           non-promotable scientific/runtime invariants
docs/ARCHITECTURE.md              replaceable model/runtime architecture
docs/RUST_RUNTIME.md              Phase 3A Rust reference contract
docs/PENTA_CRT_CPU.md             Phase 3B CPU optimization profile
docs/EXECUTION_CAMPAIGNS.md       Phase 3C graph/memory/campaign receipts
docs/MEDICAL_RESEARCH_BOUNDARY.md non-clinical intended-use boundary
docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md Australian governance baseline
docs/RESEARCH_DATA_AND_PROVENANCE.md provenance and data rules
docs/VALIDATION_LADDER.md         computational vs biological validation
docs/FLINDERS_RESEARCH_HANDOFF.md institutional handoff notes
governance/policy.json            machine-readable safety/governance contract
schemas/model-profile.schema.json model-profile schema
schemas/campaign-manifest.schema.json campaign manifest schema
schemas/correctness-receipt.schema.json correctness receipt schema
research/sources.json             structural/governance source registry
tools/validate_docs.py            deterministic documentation checks
tools/validate_campaign.py        persisted campaign validator
```

## Licence

Apache License 2.0. See [LICENSE](LICENSE).

Apache-2.0 enables broad research reuse, modification, distribution and collaboration. The licence's warranty and liability terms do **not** replace ethics approval, regulatory obligations, professional responsibilities or legal advice for a downstream research program.

## Current status

Phases 1, 2, 3A, and 3B are merged. Phase 3C campaign/execution infrastructure is implemented in PR #5 pending review. The current biological model remains **V0 schematic** and no clinical validity is claimed.
