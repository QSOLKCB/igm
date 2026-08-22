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

## Mandatory reading before changes

Read:

- `README.md`
- `README4AI.md`
- `docs/MEDICAL_RESEARCH_BOUNDARY.md`
- `docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md`
- `docs/ARCHITECTURE.md`
- `governance/policy.json`

before changing model semantics, data handling, medical language, or validation status.

## Prohibited claims

Do not state or imply that this repository diagnoses, predicts, monitors, prevents, treats or cures disease; recommends treatment; models an individual patient; or is clinically validated or approved.

Do not convert personal experience, anecdote, visual resemblance, numerical stability, accelerator agreement or an attractive plot into biological evidence.

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

## Medical-device boundary

Intended purpose controls regulatory status. If a change introduces patient-specific diagnosis, prognosis, monitoring, treatment, clinical decision support, or another medical-device purpose, do not casually merge it into the research profile. Open a dedicated regulatory/ethics design task and require qualified review.

## Institutional names

Do not imply affiliation, endorsement, approval or sponsorship by Flinders University, SA Health, NHMRC, TGA or any other organisation without explicit documented permission.

## CI gate

PRs must pass `python3 tools/validate_docs.py` until broader implementation CI replaces or extends it.
