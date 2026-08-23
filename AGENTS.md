# AGENTS.md

## Scope

These instructions apply to the entire repository unless a deeper `AGENTS.md` explicitly narrows them.

## Mission

Build reproducible research infrastructure for IgM structural simulation while maintaining a hard separation between:

1. source evidence;
2. model assumptions;
3. computational correctness;
4. accelerator performance;
5. biological interpretation;
6. clinical/regulatory use.

Never collapse these layers.

## Core invariant

**INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

This is a hard project invariant, not explanatory prose.

Preserve:

```text
mathematical correctness
    != computational correctness
    != biological validity
    != clinical validity
```

If code, documentation, a model profile, an automated report or a proposed claim violates this separation, reject the promotion and require external evidence appropriate to the biological or clinical claim.

See `docs/CORE_INVARIANTS.md` and `governance/policy.json`.

## Runtime adjacency invariant

**INV-RUNTIME-001: Execution Adjacency Does Not Imply Biological Adjacency.**

Scheduler graphs, CRT traversal order, memory adjacency, SIMD/warp lanes, chunks, workers, and device assignments are computational metadata only. They must never be promoted into molecular contact, physical proximity, biological interaction, causal influence, or another model relationship without an explicit separately sourced model/profile mapping.

Keep model/biological graphs, execution graphs, provenance graphs, visualization graphs, and tensor-factor graphs separately named.

## Mandatory reading before changes

Read:

- `README.md`
- `README4AI.md`
- `docs/CORE_INVARIANTS.md`
- `docs/MEDICAL_RESEARCH_BOUNDARY.md`
- `docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md`
- `docs/ARCHITECTURE.md`
- `governance/policy.json`

before changing model semantics, data handling, medical language, or validation status.

For runtime scheduling, memory-layout, campaign, accelerator, property-fuzz, performance-benchmark, or evidence-adapter changes also read:

- `docs/RUST_RUNTIME.md`
- `docs/PENTA_CRT_CPU.md`
- `docs/EXECUTION_CAMPAIGNS.md`
- `docs/PROPERTY_FUZZING.md`
- `docs/TIMING_BENCHMARK.md`
- `docs/EVIDENCE_ADAPTERS.md`
- `docs/PRE_PHASE5_READINESS.md`
- `docs/RUNTIME_LINEAGE.md`
- `docs/RESEARCH_DATA_AND_PROVENANCE.md`

## Prohibited claims

Do not state or imply that this repository diagnoses, predicts, monitors, prevents, treats or cures disease; recommends treatment; models an individual patient; or is clinically validated or approved.

Do not convert personal experience, anecdote, visual resemblance, numerical stability, accelerator agreement, benchmark speed, source ingestion success, or an attractive plot into biological evidence.

## Human data

The public repository is a no-patient-data workspace.

Do not commit or request identifiable, re-identifiable, coded clinical, genomic, pathology, imaging, treatment or other participant data. Synthetic fixtures and public/open structural data are the development defaults.

If a proposed task requires human participants or their data, stop and identify the required institutional ethics/governance path before implementation.

## Model semantics

- Biological parameters belong in versioned model profiles/adapters.
- Runtime/scheduler code must remain as biology-agnostic as practical.
- A vortex-like coordinate system is a parameterization only unless independent evidence establishes more.
- Tensor and graph representations are computational structures, not biological claims.
- Each biologically meaningful parameter must support explicit provenance.
- Unknown or unsupported values must remain unknown/assumed, not silently invented.
- `unknown` parameters must not carry a runtime value; use `assumed` for explicit placeholders.
- `observed`, `source-derived`, and `calibrated` parameters require source provenance and derivation metadata.
- Evidence-backed parameters must carry explicit uncertainty rather than an omitted or fabricated zero uncertainty.
- V0–V2 profiles must keep `biological_validity_claimed=false`.
- Component identifiers must be unique within a profile.

## Profile validation

`schemas/model-profile.schema.json` defines the portable structural contract.

`tools/validate_profile.py` is the dependency-free semantic pre-execution gate for cross-field requirements that portable JSON Schema cannot express directly, including uniqueness by component `id`.

Future runtimes, Pages tools, and accelerator paths must reject a profile that fails either structural/schema validation or the semantic pre-execution gate. Do not bypass the validator because a renderer or kernel could otherwise consume the data.

## Phase 4 evidence ingestion

**Phase 4 gate: Source ingestion must not silently convert observations into stronger claims than the source supports.**

`IGM-SOURCE-ADAPTER-V1` is the replaceable source-adapter boundary.

All evidence-adapter work must preserve these rules:

- The source must resolve in `research/sources.json`.
- Structural source records must retain DOI/PDB/EMDB identifiers where available and must retain access/redistribution guidance.
- Adapter input must cite a `support_statement` that exactly matches a registered source `supports` statement.
- Adapters derive output evidence status from the declared transformation. Callers do not get to relabel transformed evidence as direct observation.
- Evidence-backed parameters require explicit uncertainty. `unknown` or `source-reported` uncertainty requires explanatory notes; do not invent zero uncertainty.
- Cryo-EM, molecular-dynamics, and biochemical/calibration adapters remain separately named contracts.
- Conflict and unknown states must be preserved. Do not average, vote, or force reconciliation in the ingestion layer.
- Snapshot default is `reference-only`. Hash-only or packaged source material requires the explicit source snapshot policy to permit that mode.
- A packaged external payload requires verified redistribution permission and a SHA-256 identity.
- Preserve `does_not_support` limitations in adapted evidence records.
- Source ingestion may not promote validation level, biological validity, or clinical validity.
- `research/v0-implementation-constants.json` is V0-only. Source-informed profiles may not silently inherit those assumed drawing constants as biology.
- Adapter success is provenance/normalization evidence only. It is not independent biological validation.

## Execution and campaign semantics

- `IGM-EXEC-GRAPH-C5-K2-C3-V1` is a scheduler graph only.
- The 30 meaningful execution lanes and two padding lanes are runtime layout metadata, not biological state counts.
- Padding lanes must remain inactive, non-semantic, and excluded from scientific/model counts.
- Memory budgets must be validated before allocation and execution.
- Large campaigns must use deterministic bounded chunking rather than unbounded resident allocation.
- Correctness identity must remain separate from benchmark/timing identity.
- Worker count, chunk count, and memory budget may affect the manifest/benchmark but must not silently alter a worker/chunk-independent correctness result for the same admitted numerical profile and conformation slice.
- Rejected campaigns should be preserved with their failure reason and must not be relabelled as accepted evidence.
- Environment receipts must omit hostname, username, raw GPU UUIDs, serial numbers, MAC addresses, and similar machine identifiers by default.

## Property-based fuzzing

`IGM-PROPERTY-FUZZ-V1` is implementation testing for Phase 3A. It must remain reproducible, bounded, and scientifically non-promoting.

- Generated cases must come from an explicit replayable seed.
- CI must pin the seed and case count.
- Local seed overrides are allowed for exploration, but a discovered failing seed must be preserved when converted into a regression test.
- Generated domains must remain bounded so fuzzing cannot become an accidental unbounded campaign.
- Property failures should report the seed and case index needed for reproduction.
- Fuzzing may test address bijections, partition coverage, numerical invariants, bounded transforms, fail-closed inputs, and worker-independent correctness.
- Random generators, fuzz seeds, and case order must never enter scientific/model identity unless a future stochastic model explicitly declares that contract.
- Property-fuzz success is computational evidence only. It cannot create a source-informed model, molecular-dynamics result, biological validity, clinical validity, or validation-level promotion.

## Performance benchmarking

`IGM-PHASE3B-SCALAR-VS-OPTIMIZED-BENCHMARK-V1` is the dedicated Phase 3B algorithmic timing comparison.

- The benchmark must compare the same admitted V0 conformation slice.
- The Phase 3B fixed residual gate must pass before timing results are accepted.
- The optimized side must use one worker for the scalar/reference-vs-optimized algorithmic comparison.
- Warmups and repeated measurements are mandatory; the harness alternates measurement order and reports medians.
- Timing, throughput, worker choice, and observed speedup ratio are non-identity metadata.
- CI must exercise the benchmark contract but must not require a speedup on a shared hosted runner.
- Benchmark output must keep `speedup_claimed=false` and `performance_claim=false`.
- A repository-level speedup statement requires retained release-build benchmark receipts, exact profile/algorithm/slice identities, hardware/toolchain scope, repeated-run evidence, and raw timings. One local or CI run is insufficient.
- A performance observation cannot promote biological or clinical validation.

## Pre-Phase 5 readiness

`docs/PRE_PHASE5_READINESS.md` is normative project sequencing guidance.

On the Phase 4 PR branch the state is `READY_ON_PHASE4_MERGE`. Do not claim `READY_ON_MAIN` until the Phase 4 PR has actually merged with its evidence-adapter CI green.

Phase 5 representation work must consume the Phase 4 source/provenance contracts rather than bypass them.

## Validation

Use the V0–V4 ladder in `docs/VALIDATION_LADDER.md`.

A test passing establishes only what that test measures. CPU/GPU agreement establishes implementation agreement within a declared tolerance, not biological validity.

Never weaken a validation threshold solely to make current output pass.

## Reproducibility

Prefer:

- integer indices for discrete domains;
- bounded inputs;
- deterministic partitioning;
- canonical serialization for evidence records;
- strict standards-compliant JSON without NaN/Infinity extensions;
- explicit version/commit/source identities;
- separate correctness and performance reports;
- rejected-run preservation where useful.

## Documentation

Any new biological source or calibrated parameter must document:

- source identifier and URL/DOI/PDB/EMDB/etc.;
- licence/access status where relevant;
- what the source actually supports;
- what is inferred;
- what remains unsupported;
- model/profile version using it.

A publicly reachable URL does not itself grant redistribution permission. Record access and reuse status explicitly and verify the source's current terms before packaging source artifacts.

## Medical-device boundary

Intended purpose controls regulatory status. If a change introduces patient-specific diagnosis, prognosis, monitoring, treatment, clinical decision support, or another medical-device purpose, do not casually merge it into the research profile. Open a dedicated regulatory/ethics design task and require qualified review.

## Institutional names

Do not imply affiliation, endorsement, approval or sponsorship by Flinders University, SA Health, NHMRC, TGA or any other organisation without explicit documented permission.

## CI gate

PRs must pass:

```bash
python3 tools/validate_docs.py
python3 tools/validate_profile.py --self-test
cargo test --locked --all-targets
```

Phase 3A runtime/property-fuzz changes must additionally run the replayable generated-property harness:

```bash
IGM_PROPERTY_FUZZ_SEED=0x49474d50524f5037 \
IGM_PROPERTY_FUZZ_CASES=2048 \
cargo test --locked --test property_fuzz -- --nocapture
```

Phase 3B performance changes must additionally run a bounded release benchmark and validate only its contract/nonclaim fields, never a minimum speedup:

```bash
./target/release/igm-benchmark --count 257 --repetitions 3 --warmups 1 --verify-samples 257
```

Phase 3C campaign changes must additionally exercise and validate an accepted campaign directory with:

```bash
python3 tools/validate_campaign.py CAMPAIGN_DIR
```

Phase 4 evidence/source changes must additionally run:

```bash
python3 tools/validate_sources.py --self-test
python3 tools/validate_sources.py
python3 tools/validate_phase4.py
./target/release/igm-evidence registry
./target/release/igm-evidence adapt research/evidence/cryo-em-pentamer-count.json
```

until broader implementation CI replaces or extends these gates.
