#!/usr/bin/env python3
"""Deterministic Phase-1 governance/documentation validation for IGM."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVARIANT = "Perfect Mathematics Does Not Equal Perfect Biological Reality"

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
    "governance/policy.json",
    "research/sources.json",
    "schemas/model-profile.schema.json",
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


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_json(path: str):
    try:
        return json.loads((ROOT / path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
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
    invariants = {item.get("id"): item for item in policy.get("core_invariants", [])}
    inv = invariants.get("INV-BIO-001")
    if not inv or inv.get("name") != INVARIANT or inv.get("normative") is not True:
        fail("INV-BIO-001 missing, renamed, or non-normative")

    schema = load_json("schemas/model-profile.schema.json")
    if schema.get("properties", {}).get("schema", {}).get("const") != "IGM-MODEL-PROFILE-V1":
        fail("model profile schema contract changed unexpectedly")
    claims = schema.get("properties", {}).get("claims", {}).get("properties", {})
    for key in (
        "clinical_validity_claimed",
        "medical_device_claimed",
        "diagnostic_use_claimed",
        "treatment_use_claimed",
    ):
        if claims.get(key, {}).get("const") is not False:
            fail(f"upstream model schema must hard-code {key}=false")

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

    boundary = (ROOT / "docs/MEDICAL_RESEARCH_BOUNDARY.md").read_text(encoding="utf-8")
    for phrase in (
        "exploratory research software",
        "not represented by this project as a medical device",
        "Computational validity is not biological validity",
        "No institutional endorsement",
    ):
        if phrase not in boundary:
            fail(f"medical boundary missing required phrase: {phrase!r}")

    print("OK: IGM Phase-1 documentation/governance foundation validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
