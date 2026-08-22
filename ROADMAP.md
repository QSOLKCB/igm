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

### Phase 1 gate

No biological simulator implementation may claim more than V0 until its biologically meaningful parameters have traceable source provenance.

## Phase 2 — Minimal deterministic structural kernel

- [ ] Define `IGM-MODEL-PROFILE-V1`.
- [ ] Implement a schematic pentamer + J-chain profile as V0 only.
- [ ] Represent subunits/domains with exact stable identifiers.
- [ ] Add articulated geometry with bounded angles/translations.
- [ ] Add explicit steric/contact constraint interfaces.
- [ ] Add deterministic CPU reference implementation in Rust.
- [ ] Add canonical run manifest and hashes.
- [ ] Prove repeatability across worker counts.
- [ ] Add property-based and edge-case tests.

### Phase 2 gate

The schematic model must remain clearly labelled as a computational fixture and must not be described as a validated IgM conformation.

## Phase 3 — Replaceable evidence adapters

- [ ] Define source-adapter interface.
- [ ] Add public structural-source registry with DOI/PDB/EMDB identifiers where applicable.
- [ ] Add cryo-EM parameter adapter.
- [ ] Add molecular-dynamics trajectory adapter.
- [ ] Add biochemical/calibration constraint adapter.
- [ ] Preserve source licence/access metadata.
- [ ] Require per-parameter provenance and uncertainty.
- [ ] Add conflict/unknown representation rather than forced reconciliation.

### Phase 3 gate

Source ingestion must not silently convert source observations into stronger claims than the source supports.

## Phase 4 — Tensor and graph representations

- [ ] Define tensor projection of model state.
- [ ] Define graph projection of domains/subunits/constraints.
- [ ] Ensure projections round-trip to a stable model identity where applicable.
- [ ] Add pair-distance/contact/accessibility observables.
- [ ] Add ensemble statistics with explicit numerical assumptions.
- [ ] Add optional vortex-inspired coordinate adapter as a parameterization only.

## Phase 5 — GPU acceleration

- [ ] Keep Rust CPU execution as reference/orchestration authority.
- [ ] Add CUDA adapter for bounded conformational sweeps.
- [ ] Add deterministic device sharding.
- [ ] Separate evidence mode from throughput mode.
- [ ] Require independent CPU/GPU residual comparison.
- [ ] Add complete-readback evidence profiles.
- [ ] Add Compute Sanitizer campaign support.
- [ ] Record GPU/toolchain provenance without publishing raw hardware identifiers.
- [ ] Run single-GPU baseline before multi-GPU scaling.
- [ ] Run 2/4/8 GPU scaling campaigns only after correctness gates pass.

### Phase 5 gate

GPU agreement is implementation evidence only. Performance observations cannot promote biological validation level.

## Phase 6 — Structure-informed research profiles

- [ ] Create first V1 source-informed IgM pentamer profile.
- [ ] Create a separately versioned hexamer profile if supported by sources.
- [ ] Quantify profile uncertainty and unsupported degrees of freedom.
- [ ] Compare model-derived observables with independent structural observations.
- [ ] Publish accepted and rejected calibration attempts.

## Phase 7 — Research collaboration package

- [ ] Produce researcher onboarding guide.
- [ ] Produce reproducibility capsule format.
- [ ] Add export formats useful to structural biology workflows.
- [ ] Add example Jupyter/analysis adapters without making Python authoritative.
- [ ] Prepare a neutral research handoff package for external institutions.
- [ ] Invite domain experts to review biological assumptions, provenance and validation design.

## Phase 8 — Optional regulated-research branch

This phase is **not implied by the open-source runtime**. It exists only if qualified downstream collaborators intentionally pursue human-subject, clinical, diagnostic, monitoring, treatment or medical-device work.

- [ ] Determine intended purpose with qualified investigators.
- [ ] Obtain institutional ethics/governance advice before using human participants/data.
- [ ] Conduct privacy/data impact assessment.
- [ ] Determine TGA regulatory status.
- [ ] Assess applicable standards and quality-system requirements.
- [ ] Separate regulated artefacts from exploratory research artefacts.
- [ ] Establish clinical/statistical validation plan.
- [ ] Establish adverse-event/safety reporting where applicable.

No Phase 8 activity may be inferred from completion of Phases 1–7.
