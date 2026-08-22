#!/usr/bin/env python3
"""Validate the Phase 4 evidence-adapter contract and Phase 5 readiness state."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PHASE4_GATE = "Source ingestion must not silently convert observations into stronger claims than the source supports."

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
    "runtime/rust/src/evidence_main.rs",
    "runtime/rust/src/lib_v5.rs",
    "tools/validate_sources.py",
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


def main() -> int:
    for relative in REQUIRED_FILES:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size == 0:
            fail(f"required Phase 4 file missing or empty: {relative}")

    evidence_doc = (ROOT / "docs/EVIDENCE_ADAPTERS.md").read_text(encoding="utf-8")
    for fragment in (
        "IGM-SOURCE-ADAPTER-V1",
        "IGM-CRYO-EM-PARAMETER-ADAPTER-V1",
        "IGM-MD-TRAJECTORY-ADAPTER-V1",
        "IGM-BIOCHEMICAL-CALIBRATION-ADAPTER-V1",
        "IGM-PHASE4-EVIDENCE-BUNDLE-V1",
        "reference-only",
        "conflict",
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
    if "implemented in PR #9, pending review/merge" not in roadmap:
        fail("Phase 4 roadmap status must remain pending review/merge until PR #9 merges")

    readiness = (ROOT / "docs/PRE_PHASE5_READINESS.md").read_text(encoding="utf-8")
    if "Status: **READY_ON_PHASE4_MERGE**." not in readiness:
        fail("pre-Phase 5 readiness must be READY_ON_PHASE4_MERGE on this branch")
    # The document may discuss the string READY_ON_MAIN as a future state. Only
    # reject an actual readiness-status promotion before the Phase 4 merge.
    if "Status: **READY_ON_MAIN**." in readiness:
        fail("Phase 5 readiness must not claim READY_ON_MAIN before Phase 4 merge")

    model_schema = load_json("schemas/model-profile.schema.json")
    parameter_schema = model_schema["properties"]["parameters"]["items"]
    if "uncertainty" not in parameter_schema.get("properties", {}):
        fail("model profile schema must define evidence uncertainty")
    schema_text = (ROOT / "schemas/model-profile.schema.json").read_text(encoding="utf-8")
    if '"required": ["uncertainty"]' not in schema_text:
        fail("evidence-backed parameters must require uncertainty")

    v0 = load_json("research/v0-implementation-constants.json")
    if v0.get("source_informed_inheritance") != "forbidden":
        fail("V0 implementation constants may not silently flow into source-informed profiles")

    snapshot = load_json("research/source-snapshot-policy.json")
    if snapshot.get("default_mode") != "reference-only":
        fail("Phase 4 snapshot policy must fail closed to reference-only")

    source_schema = load_json("schemas/source-registry.schema.json")
    if source_schema.get("properties", {}).get("schema", {}).get("const") != "igm-source-registry/1":
        fail("source registry schema contract mismatch")

    evidence_input = load_json("schemas/evidence-input.schema.json")
    if evidence_input.get("properties", {}).get("schema", {}).get("const") != "IGM-EVIDENCE-INPUT-V1":
        fail("evidence input schema contract mismatch")

    bundle_schema = load_json("schemas/evidence-bundle.schema.json")
    props = bundle_schema.get("properties", {})
    if props.get("bundle_contract", {}).get("const") != "IGM-PHASE4-EVIDENCE-BUNDLE-V1":
        fail("evidence bundle contract mismatch")
    for key in (
        "reconciliation_performed",
        "claim_strengthening_detected",
        "validation_level_promoted_by_adapter",
        "biological_validity_claimed",
        "clinical_validity_claimed",
    ):
        if props.get(key, {}).get("const") is not False:
            fail(f"evidence bundle schema must pin {key}=false")

    print("OK: IGM Phase 4 evidence-adapter gate and Phase 5 readiness validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
