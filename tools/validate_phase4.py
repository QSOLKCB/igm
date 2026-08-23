#!/usr/bin/env python3
"""Validate the merged Phase 4 evidence-adapter contract and Phase 5 readiness state."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PHASE4_GATE = "Source ingestion must not silently convert observations into stronger claims than the source supports."
EVIDENCE_BACKED = {"observed", "source-derived", "calibrated"}

REQUIRED_FILES = [
    "docs/EVIDENCE_ADAPTERS.md",
    "docs/PRE_PHASE5_READINESS.md",
    "schemas/source-registry.schema.json",
    "schemas/evidence-input.schema.json",
    "schemas/evidence-bundle.schema.json",
    "schemas/source-snapshot-policy.schema.json",
    "research/source-snapshot-policy.json",
    "research/v0-implementation-constants.json",
    "research/evidence/cryo-em-pentamer-count.json",
    "runtime/rust/src/phase4.rs",
    "runtime/rust/src/phase4_v2.rs",
    "runtime/rust/src/evidence_main.rs",
    "runtime/rust/src/lib_v5.rs",
    "tools/validate_sources.py",
    "tools/validate_json_schema.py",
]

PHASE4_CHECKBOXES = [
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
]


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_json(relative: str):
    try:
        return json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"{relative}: {exc}")


def uncertainty_requirement_present(parameter_schema: dict) -> bool:
    required: set[str] = set()
    for rule in parameter_schema.get("allOf", []):
        condition = rule.get("if", {})
        statuses = condition.get("properties", {}).get("status", {}).get("enum", [])
        if EVIDENCE_BACKED.issubset(set(statuses)):
            required.update(rule.get("then", {}).get("required", []))
    return {"source_id", "derivation", "uncertainty"}.issubset(required)


def main() -> int:
    for relative in REQUIRED_FILES:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size == 0:
            fail(f"required Phase 4 file missing or empty: {relative}")

    evidence_doc = (ROOT / "docs/EVIDENCE_ADAPTERS.md").read_text(encoding="utf-8")
    for fragment in (
        "Status: **complete and merged in PR #9**.",
        "IGM-SOURCE-ADAPTER-V1",
        "IGM-CRYO-EM-PARAMETER-ADAPTER-V1",
        "IGM-MD-TRAJECTORY-ADAPTER-V1",
        "IGM-BIOCHEMICAL-CALIBRATION-ADAPTER-V1",
        "IGM-PHASE4-EVIDENCE-BUNDLE-V1",
        "reference-only",
        "duplicate candidate identities",
        "actual bytes",
        PHASE4_GATE,
    ):
        if fragment not in evidence_doc:
            fail(f"Phase 4 evidence documentation missing: {fragment!r}")

    roadmap = (ROOT / "ROADMAP.md").read_text(encoding="utf-8")
    for item in PHASE4_CHECKBOXES:
        if f"- [x] {item}" not in roadmap:
            fail(f"Phase 4 roadmap item not marked implemented: {item}")
    if PHASE4_GATE not in roadmap:
        fail("Phase 4 gate text missing or changed in ROADMAP.md")
    if "Status: **complete and merged in PR #9**." not in roadmap:
        fail("Phase 4 roadmap status must record merged completion")
    if "Status: **READY_ON_MAIN**." not in roadmap:
        fail("ROADMAP.md must record Phase 5 architectural readiness on main")

    readiness = (ROOT / "docs/PRE_PHASE5_READINESS.md").read_text(encoding="utf-8")
    if "Status: **READY_ON_MAIN**." not in readiness:
        fail("pre-Phase 5 readiness must be READY_ON_MAIN after Phase 4 merge")
    for fragment in (
        "Phase 4 is complete and merged in PR #9",
        PHASE4_GATE,
        "does **not** mean that IGM now has a validated source-informed IgM model",
    ):
        if fragment not in readiness:
            fail(f"pre-Phase 5 readiness missing post-merge boundary: {fragment!r}")

    model_schema = load_json("schemas/model-profile.schema.json")
    parameter_schema = model_schema["properties"]["parameters"]["items"]
    if "uncertainty" not in parameter_schema.get("properties", {}):
        fail("model profile schema must define evidence uncertainty")
    if not uncertainty_requirement_present(parameter_schema):
        fail("evidence-backed parameters must structurally require source_id, derivation, and uncertainty")
    uncertainty = model_schema.get("$defs", {}).get("evidenceUncertainty", {})
    rules = uncertainty.get("allOf", [])
    rule_text = json.dumps(rules, sort_keys=True)
    for required_fragment in ('"lower"', '"upper"', '"notes"', '"value"'):
        if required_fragment not in rule_text:
            fail(f"evidence uncertainty kind rules missing {required_fragment}")
    if uncertainty.get("properties", {}).get("value", {}).get("minimum") != 0:
        fail("standard-deviation uncertainty must have a non-negative numeric floor")

    source_schema = load_json("schemas/source-registry.schema.json")
    if source_schema.get("properties", {}).get("schema", {}).get("const") != "igm-source-registry/1":
        fail("source registry schema contract mismatch")
    source_def = source_schema.get("$defs", {}).get("source", {})
    if source_def.get("additionalProperties") is not False:
        fail("source registry entries must reject unknown fields")
    if "evidence_mappings" not in source_def.get("properties", {}):
        fail("source registry must bind support statements to structured evidence mappings")

    evidence_input = load_json("schemas/evidence-input.schema.json")
    if evidence_input.get("properties", {}).get("schema", {}).get("const") != "IGM-EVIDENCE-INPUT-V1":
        fail("evidence input schema contract mismatch")
    snapshot_def = evidence_input.get("$defs", {}).get("snapshot", {})
    if "external_payload_path" not in snapshot_def.get("properties", {}):
        fail("packaged evidence input must support a repository-relative payload path")

    snapshot_schema = load_json("schemas/source-snapshot-policy.schema.json")
    record_props = (
        snapshot_schema.get("properties", {})
        .get("records", {})
        .get("items", {})
        .get("properties", {})
    )
    if "external_payload_path" not in record_props:
        fail("snapshot policy must bind packaged payload path as well as digest")

    v0 = load_json("research/v0-implementation-constants.json")
    if v0.get("source_informed_inheritance") != "forbidden":
        fail("V0 implementation constants may not silently flow into source-informed profiles")

    snapshot = load_json("research/source-snapshot-policy.json")
    if snapshot.get("default_mode") != "reference-only":
        fail("Phase 4 snapshot policy must fail closed to reference-only")

    bundle_schema = load_json("schemas/evidence-bundle.schema.json")
    props = bundle_schema.get("properties", {})
    if props.get("bundle_contract", {}).get("const") != "IGM-PHASE4-EVIDENCE-BUNDLE-V1":
        fail("evidence bundle contract mismatch")
    if props.get("inv_bio_001", {}).get("const") != "Perfect Mathematics Does Not Equal Perfect Biological Reality":
        fail("evidence bundle schema must declare INV-BIO-001 emitted by runtime")
    for key in (
        "reconciliation_performed",
        "claim_strengthening_detected",
        "validation_level_promoted_by_adapter",
        "biological_validity_claimed",
        "clinical_validity_claimed",
    ):
        if props.get(key, {}).get("const") is not False:
            fail(f"evidence bundle schema must pin {key}=false")

    print("OK: merged IGM Phase 4 gate validated; Phase 5 architecture is READY_ON_MAIN")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
