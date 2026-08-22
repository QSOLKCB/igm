# Flinders Research Handoff

## Purpose

This document describes how the open-source IGM project could be evaluated by researchers at an institution such as the **Flinders Centre for Innovation in Cancer** without implying affiliation, endorsement, approval or sponsorship.

IGM is not currently represented as a Flinders University, SA Health or Flinders Centre for Innovation in Cancer project.

## What the upstream project can provide

A future handoff package should contain:

- exact repository release/commit identity;
- Apache-2.0 licence;
- model-profile schema and validation level;
- public source registry;
- parameter provenance and assumptions;
- deterministic CPU reference implementation;
- accelerator implementation and residual evidence where available;
- reproducibility instructions;
- accepted and rejected validation records;
- explicit medical/research boundary;
- Australian ethics/regulatory foundation;
- a clear list of questions requiring domain-expert review.

## What the upstream project cannot provide

The generic repository cannot provide:

- institutional ethics approval;
- site governance approval;
- permission to access SA Health data/patients/facilities;
- patient consent or waiver of consent;
- a TGA medical-device classification decision;
- clinical validation;
- biological validation by virtue of simulation alone;
- institutional endorsement;
- authority to recruit participants or use clinical data.

## Suggested researcher entry points

A structural-biologist or immunology group could evaluate the project by replacing V0/V1 schematic parameters with better evidence while preserving the execution machinery.

Potential paths include:

### Cryo-EM adapter

Use public or appropriately governed cryo-EM/PDB/EMDB structures to define:

- core coordinates;
- domain orientation constraints;
- hinge ranges or conformational classes;
- unresolved/flexible regions;
- uncertainty and source limitations.

### Molecular-dynamics adapter

Map validated trajectory data into a versioned ensemble profile while preserving:

- force-field/protocol identity;
- simulation conditions;
- trajectory provenance;
- selection/downsampling rules;
- uncertainty and convergence caveats.

### Biochemical/calibration adapter

Introduce independently measured constraints or observables without hard-coding them into the generic runtime.

### Experimental comparison

Compute model-derived observables against independent data and record both successes and failures.

## Human research at Flinders / SA Health

Flinders University states that research involving human participants and/or their data requires human-research ethics approval before commencing. Research involving SA Health sites, or participants/data accessed through SA Health sites, follows an SA Health HREC pathway.

Accordingly, if a future IGM collaboration moves from public structural data into human/clinical data, the investigators should work through the appropriate institutional research-ethics and governance processes before introducing that data into the workflow.

See `docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md`.

## Recommended first collaboration scope

The safest and most useful first external evaluation would be **non-clinical structural validation using public data**.

Example:

1. freeze a V0 schematic implementation;
2. build a V1 profile from public full-length human IgM cryo-EM records;
3. reproduce deterministic observables;
4. compare model geometry with independent public structural observations;
5. identify where simplified constraints are wrong or incomplete;
6. let domain researchers define the next scientifically meaningful model revision.

This produces useful falsification opportunities without requiring patient data.

## Core invariant for handoff

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

Researchers are invited to challenge, replace and invalidate biological assumptions. The runtime should make doing so easier, not defend its initial model.

## Suggested handoff language

A neutral description is:

> IGM is an open-source deterministic simulation framework for replaceable IgM structural models. Its initial models are explicitly schematic or source-informed research artifacts rather than clinical or biological truth claims. The project is designed so domain researchers can replace assumptions with cryo-EM, molecular-dynamics, biochemical or calibrated experimental inputs while retaining reproducible CPU/GPU execution and provenance machinery.

## Institutional naming rule

Do not add Flinders, SA Health, a laboratory, investigator or centre logo/name as a partner, sponsor, validator or collaborator unless that relationship actually exists and permission has been obtained where required.
