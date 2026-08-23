#!/usr/bin/env python3
"""Deterministic governance/documentation validation for IGM."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVARIANT = "Perfect Mathematics Does Not Equal Perfect Biological Reality"
RUNTIME_INVARIANT = "Execution Adjacency Does Not Imply Biological Adjacency"
PHASE3A_GATE = (
    "Rust/Pages agreement establishes implementation agreement for the schematic fixture only. "
    "PR3 does not create a source-informed biological model, molecular dynamics engine, or clinical result."
)
PHASE4_GATE = "Source ingestion must not silently convert observations into stronger claims than the source supports."
BENCHMARK_CONTRACT = "IGM-PHASE3B-SCALAR-VS-OPTIMIZED-BENCHMARK-V1"
PRE_PHASE5_STATUS = "READY_ON_PHASE4_MERGE"

REQUIRED_FILES = [
    "LICENSE",
    "README.md",
    "README4AI.md",
    "AGENTS.md",
    "DISCLAIMER.md",
    "CONTRIBUTING.md",
    "ROADMAP.md",
    "docs/ARCHITECTURE.md",
    "docs/CORE_INVARIANTS.md",
    "docs/MEDICAL_RESEARCH_BOUNDARY.md",
    "docs/AUSTRALIAN_ETHICS_AND_REGULATORY.md",
    "docs/RESEARCH_DATA_AND_PROVENANCE.md",
    "docs/VALIDATION_LADDER.md",
    "docs/FLINDERS_RESEARCH_HANDOFF.md",
    "docs/EXECUTION_CAMPAIGNS.md",
    "docs/PROPERTY_FUZZING.md",
    "docs/TIMING_BENCHMARK.md",
    "docs/PRE_PHASE5_READINESS.md",
    "docs/EVIDENCE_ADAPTERS.md",
    "governance/policy.json",
    "research/sources.json",
    "research/source-snapshot-policy.json",
    "research/v0-implementation-constants.json",
    "research/evidence/cryo-em-pentamer-count.json",
    "schemas/model-profile.schema.json",
    "schemas/campaign-manifest.schema.json",
    "schemas/correctness-receipt.schema.json",
    "schemas/phase3c-gate-receipt.schema.json",
    "schemas/source-registry.schema.json",
    "schemas/evidence-input.schema.json",
    "schemas/evidence-bundle.schema.json",
    "schemas/source-snapshot-policy.schema.json",
    "tools/validate_profile.py",
    "tools/validate_campaign.py",
    "tools/validate_campaign_v2.py",
    "tools/validate_sources.py",
    "tools/validate_phase4.py",
]

INVARIANT_FILES = [
    "README4AI.md",
    "AGENTS.md",
    "DISCLAIMER.md",
    "CONTRIBUTING.md",
    "docs/CORE_INVARIANTS.md",
    "docs/VALIDATION_LADDER.md",
    "governance/policy.json",
]

RUNTIME_INVARIANT_FILES = [
    "AGENTS.md",
    "ROADMAP.md",
    "docs/CORE_INVARIANTS.md",
    "docs/EXECUTION_CAMPAIGNS.md",
    "governance/policy.json",
]

BIOLOGICAL_SOURCE_CLASSES = {
    "peer-reviewed-literature",
    "public-structure",
    "background-only",
}


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def reject_json_constant(value: str) -> None:
    raise ValueError(f"non-standard JSON constant is forbidden: {value}")


def load_json(path: str):
    try:
        return json.loads(
            (ROOT / path).read_text(encoding="utf-8"),
            parse_constant=reject_json_constant,
        )
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        fail(f"{path}: {exc}")


def main() -> int:
    for relative in REQUIRED_FILES:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size == 0:
            fail(f"required file missing or empty: {relative}")

    for relative in INVARIANT_FILES:
        text = (ROOT / relative).read_text(encoding="utf-8")
        if INVARIANT not in text:
            fail(f"hard invariant missing from {relative}")

    for relative in RUNTIME_INVARIANT_FILES:
        text = (ROOT / relative).read_text(encoding="utf-8")
        if RUNTIME_INVARIANT not in text:
            fail(f"runtime invariant missing from {relative}")

    property_doc = (ROOT / "docs/PROPERTY_FUZZING.md").read_text(encoding="utf-8")
    for required in (
        "IGM-PROPERTY-FUZZ-V1",
        "property-based fuzzing",
        "implementation evidence for the schematic fixture only",
        "does not create a source-informed biological model",
    ):
        if required not in property_doc:
            fail(f"property-fuzz documentation missing required boundary/contract text: {required!r}")

    benchmark_doc = (ROOT / "docs/TIMING_BENCHMARK.md").read_text(encoding="utf-8")
    for required in (
        BENCHMARK_CONTRACT,
        "scalar deterministic reference",
        "1e-12",
        "benchmark_timing_identity_bearing = false",
        "correctness_identity_includes_timing = false",
        "speedup_claimed = false",
        "performance_claim = false",
        "does not itself authorize a speedup claim",
    ):
        if required not in benchmark_doc:
            fail(f"timing benchmark documentation missing contract/boundary text: {required!r}")

    evidence_doc = (ROOT / "docs/EVIDENCE_ADAPTERS.md").read_text(encoding="utf-8")
    for required in (
        "IGM-SOURCE-ADAPTER-V1",
        "IGM-CRYO-EM-PARAMETER-ADAPTER-V1",
        "IGM-MD-TRAJECTORY-ADAPTER-V1",
        "IGM-BIOCHEMICAL-CALIBRATION-ADAPTER-V1",
        "IGM-PHASE4-EVIDENCE-BUNDLE-V1",
        "reference-only",
        "conflict",
        PHASE4_GATE,
    ):
        if required not in evidence_doc:
            fail(f"Phase 4 evidence-adapter documentation missing required text: {required!r}")

    readiness_doc = (ROOT / "docs/PRE_PHASE5_READINESS.md").read_text(encoding="utf-8")
    for required in (
        "Status: **READY_ON_PHASE4_MERGE**.",
        "Phase 4 gate now implemented",
        PHASE4_GATE,
        "Phase 5 representation work",
        "does **not** mean that IGM now has a validated source-informed IgM model",
    ):
        if required not in readiness_doc:
            fail(f"pre-Phase 5 readiness audit missing required text: {required!r}")
    if "Status: **READY_ON_MAIN**." in readiness_doc:
        fail("pre-Phase 5 audit must not claim READY_ON_MAIN before PR #9 merges")

    roadmap = (ROOT / "ROADMAP.md").read_text(encoding="utf-8")
    if "- [x] Add property-based fuzzing beyond deterministic edge-case tests." not in roadmap:
        fail("Phase 3A property-fuzz roadmap item must remain complete")
    if PHASE3A_GATE not in roadmap:
        fail("Phase 3A gate text changed or is missing from ROADMAP.md")
    if "- [x] Add a dedicated scalar/reference-vs-optimized timing benchmark before making any speedup claim." not in roadmap:
        fail("Phase 3B timing-benchmark roadmap item must be complete")
    if BENCHMARK_CONTRACT not in roadmap:
        fail("ROADMAP.md must name the Phase 3B timing-benchmark contract")
    if PRE_PHASE5_STATUS not in roadmap:
        fail("ROADMAP.md must record READY_ON_PHASE4_MERGE")
    if "Status: **implemented in PR #9, pending review/merge**." not in roadmap:
        fail("ROADMAP.md must keep Phase 4 pending review/merge until PR #9 merges")
    for item in (
        "Define source-adapter interface.",
        "Maintain public structural-source registry with DOI/PDB/EMDB identifiers.",
        "Add cryo-EM parameter adapter.",
        "Add molecular-dynamics trajectory adapter.",
        "Add biochemical/calibration constraint adapter.",
        "Preserve source licence/access metadata.",
        "Require per-parameter provenance and uncertainty.",
        "Add conflict/unknown representation rather than forced reconciliation.",
        "Add source snapshots/hashes only where reuse terms permit.",
        "Externalize any remaining V0 implementation constants that become biologically meaningful in source-informed profiles.",
    ):
        if f"- [x] {item}" not in roadmap:
            fail(f"ROADMAP.md must mark Phase 4 item implemented: {item}")
    if PHASE4_GATE not in roadmap:
        fail("ROADMAP.md must preserve the Phase 4 gate")
    if "**Entry condition: PR #9 merged with the Phase 4 gate and source-adapter CI green.**" not in roadmap:
        fail("Phase 5 entry condition must require merged Phase 4 gate and green source-adapter CI")

    license_text = (ROOT / "LICENSE").read_text(encoding="utf-8")
    if "Apache License" not in license_text or "Version 2.0" not in license_text:
        fail("LICENSE is not recognizably Apache-2.0")

    policy = load_json("governance/policy.json")
    if policy.get("schema") != "igm-governance-policy/1":
        fail("unexpected governance policy schema")
    if policy.get("human_data_default") != "deny":
        fail("human_data_default must remain deny")
    if policy.get("clinical_use_default") != "deny":
        fail("clinical_use_default must remain deny")
    if policy.get("institutional_endorsement_default") != "deny":
        fail("institutional_endorsement_default must remain deny")
    invariants = {item.get("id"): item for item in policy.get("core_invariants", [])}
    inv = invariants.get("INV-BIO-001")
    if not inv or inv.get("name") != INVARIANT or inv.get("normative") is not True:
        fail("INV-BIO-001 missing, renamed, or non-normative")
    runtime_inv = invariants.get("INV-RUNTIME-001")
    if (
        not runtime_inv
        or runtime_inv.get("name") != RUNTIME_INVARIANT
        or runtime_inv.get("normative") is not True
    ):
        fail("INV-RUNTIME-001 missing, renamed, or non-normative")

    schema = load_json("schemas/model-profile.schema.json")
    if schema.get("properties", {}).get("schema", {}).get("const") != "IGM-MODEL-PROFILE-V1":
        fail("model profile schema contract changed unexpectedly")
    components = schema.get("properties", {}).get("components", {})
    if components.get("x-igm-unique-by") != "id":
        fail("component-id semantic uniqueness contract missing")
    claims = schema.get("properties", {}).get("claims", {}).get("properties", {})
    for key in (
        "clinical_validity_claimed",
        "medical_device_claimed",
        "diagnostic_use_claimed",
        "treatment_use_claimed",
    ):
        if claims.get(key, {}).get("const") is not False:
            fail(f"upstream model schema must hard-code {key}=false")

    schema_text = (ROOT / "schemas/model-profile.schema.json").read_text(encoding="utf-8")
    for required_fragment in (
        '"enum": ["V0", "V1", "V2"]',
        '"biological_validity_claimed"',
        '"const": false',
        '"status": {"const": "unknown"}',
        '"not": {"required": ["value"]}',
        '"enum": ["observed", "source-derived", "calibrated"]',
        '"required": ["source_id", "derivation"]',
        '"required": ["uncertainty"]',
        '"evidenceUncertainty"',
    ):
        if required_fragment not in schema_text:
            fail(f"model schema missing hardening fragment: {required_fragment}")

    campaign_schema = load_json("schemas/campaign-manifest.schema.json")
    campaign_props = campaign_schema.get("properties", {})
    if campaign_props.get("schema", {}).get("const") != "IGM-CAMPAIGN-MANIFEST-V2":
        fail("unexpected campaign manifest schema contract")
    if campaign_props.get("gate_contract", {}).get("const") != "IGM-PHASE3C-ACCEPTANCE-GATE-V1":
        fail("campaign manifest must bind the Phase 3C acceptance gate")
    if campaign_props.get("validation_level", {}).get("const") != "V0":
        fail("campaign manifest must preserve V0 validation level")
    if campaign_props.get("validation_level_promoted_by_runtime", {}).get("const") is not False:
        fail("campaign manifest must forbid runtime validation-level promotion")
    if campaign_props.get("benchmark_identity_is_correctness_identity", {}).get("const") is not False:
        fail("campaign schema must keep benchmark identity separate from correctness identity")

    correctness_schema = load_json("schemas/correctness-receipt.schema.json")
    correctness_props = correctness_schema.get("properties", {})
    if correctness_props.get("inv_runtime_001", {}).get("const") != RUNTIME_INVARIANT:
        fail("correctness schema must embed INV-RUNTIME-001")
    if correctness_props.get("biological_validity_claimed", {}).get("const") is not False:
        fail("correctness schema must forbid biological-validity promotion")
    if correctness_props.get("verification_tolerance", {}).get("const") != 1e-12:
        fail("correctness schema must pin the Phase 3B residual tolerance")

    gate_schema = load_json("schemas/phase3c-gate-receipt.schema.json")
    gate_props = gate_schema.get("properties", {})
    if gate_props.get("schema", {}).get("const") != "IGM-PHASE3C-GATE-RECEIPT-V1":
        fail("unexpected Phase 3C gate receipt schema")
    if gate_props.get("gate_contract", {}).get("const") != "IGM-PHASE3C-ACCEPTANCE-GATE-V1":
        fail("unexpected Phase 3C gate contract")
    for key in (
        "profile_identity_preserved",
        "algorithm_identity_preserved",
        "phase3b_residual_gate_passed",
        "finite_and_bounded",
        "declared_slice_preserved",
        "correctness_identity_recomputed",
        "worker_independent_correctness_identity",
        "chunk_independent_correctness_identity",
        "benchmark_timing_excluded_from_correctness_identity",
        "accepted",
    ):
        if gate_props.get(key, {}).get("const") is not True:
            fail(f"Phase 3C gate schema must require {key}=true")
    for key in (
        "implementation_structures_biological_relationships_claimed",
        "validation_level_promoted_by_runtime",
        "biological_validity_claimed",
        "clinical_validity_claimed",
    ):
        if gate_props.get(key, {}).get("const") is not False:
            fail(f"Phase 3C gate schema must require {key}=false")

    registry = load_json("research/sources.json")
    if registry.get("schema") != "igm-source-registry/1":
        fail("unexpected source registry schema")
    seen: set[str] = set()
    for source in registry.get("sources", []):
        source_id = source.get("id")
        if not isinstance(source_id, str) or not source_id:
            fail("source without stable id")
        if source_id in seen:
            fail(f"duplicate source id: {source_id}")
        seen.add(source_id)
        url = source.get("url")
        if not isinstance(url, str) or not url.startswith("https://"):
            fail(f"source {source_id} must have an https URL")
        if source.get("class") in BIOLOGICAL_SOURCE_CLASSES:
            access = source.get("access")
            if not isinstance(access, dict):
                fail(f"biological source {source_id} requires explicit access metadata")
            if access.get("status") not in {
                "publicly-accessible",
                "open-access",
                "registration-required",
                "restricted",
            }:
                fail(f"biological source {source_id} has unknown access status")
            redistribution = access.get("redistribution")
            if not isinstance(redistribution, str) or not redistribution:
                fail(f"biological source {source_id} requires redistribution guidance")

    required_source_ids = {
        "structure.chen-2022-full-length-igm",
        "structure.emdb-13921",
        "governance.nhmrc-national-statement-2025",
        "governance.australian-code-2018",
        "governance.tga-software-medical-devices-2026",
        "governance.flinders-human-ethics",
    }
    missing = required_source_ids - seen
    if missing:
        fail(f"required source registry entries missing: {sorted(missing)}")

    snapshot_policy = load_json("research/source-snapshot-policy.json")
    if snapshot_policy.get("schema") != "IGM-SOURCE-SNAPSHOT-POLICY-V1":
        fail("unexpected Phase 4 source snapshot policy schema")
    if snapshot_policy.get("default_mode") != "reference-only":
        fail("Phase 4 source snapshot default must remain reference-only")

    v0_constants = load_json("research/v0-implementation-constants.json")
    if v0_constants.get("schema") != "IGM-V0-IMPLEMENTATION-CONSTANTS-V1":
        fail("unexpected V0 implementation constants schema")
    if v0_constants.get("source_informed_inheritance") != "forbidden":
        fail("source-informed profiles may not silently inherit V0 drawing constants")

    boundary = (ROOT / "docs/MEDICAL_RESEARCH_BOUNDARY.md").read_text(encoding="utf-8")
    for phrase in (
        "exploratory research software",
        "not represented by this project as a medical device",
        "Computational validity is not biological validity",
        "No institutional endorsement",
    ):
        if phrase not in boundary:
            fail(f"medical boundary missing required phrase: {phrase!r}")

    print("OK: IGM documentation/governance foundation validated through Phase 4")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
