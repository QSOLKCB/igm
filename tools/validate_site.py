#!/usr/bin/env python3
"""Fail-closed static validation for the IGM Pages visual laboratory."""

from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
POLICY = ROOT / "governance" / "policy.json"
PROFILE = ROOT / "profiles" / "igm-schematic-pentamer-v0.json"

INVARIANTS = {
    "INV-BIO-001": "Perfect Mathematics Does Not Equal Perfect Biological Reality",
    "INV-MATH-002": "A Multidimensional Array Is Not Automatically a Tensor",
    "INV-MATH-003": "Coordinate Presentation Must Not Alter Coordinate-Invariant Observables",
    "INV-GRAPH-001": "Graph Representation Must Match Declared Relationship Semantics",
    "INV-GRAPH-002": "Topology Is Measured or Sourced, Never Assumed",
    "INV-VIZ-001": "Visualization Layout Must Not Alter Model Semantics",
    "INV-VIZ-002": "Visual Proximity Does Not Imply Biological Proximity",
}


def reject_constant(value: str) -> None:
    raise ValueError(f"non-standard JSON constant forbidden: {value}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: {message}")


def main() -> int:
    required = [SITE / "index.html", SITE / "style.css", SITE / "app.mjs", SITE / "igm-model.mjs", PROFILE]
    for path in required:
        require(path.is_file() and path.stat().st_size > 0, f"missing/empty {path.relative_to(ROOT)}")

    html = (SITE / "index.html").read_text(encoding="utf-8")
    model = (SITE / "igm-model.mjs").read_text(encoding="utf-8")
    app = (SITE / "app.mjs").read_text(encoding="utf-8")
    combined = "\n".join([html, model, app])

    require("V0 SCHEMATIC" in html, "UI must expose V0 SCHEMATIC")
    require("NOT CLINICAL" in html, "UI must expose NOT CLINICAL")
    require(INVARIANTS["INV-BIO-001"] in html, "UI must expose INV-BIO-001 phrase")
    require("Math.random" not in combined, "canonical site code may not use Math.random")
    require("biofabric" not in app.lower(), "app implementation must not contain BioFabric implementation code")
    require("BioFabric source code or assets" in html, "clean-room attribution boundary missing")

    remote_runtime = re.findall(r"<(?:script|link)[^>]+(?:src|href)=[\"']https?://", html, flags=re.I)
    require(not remote_runtime, "site may not load remote runtime scripts/styles")

    profile = load(PROFILE)
    require(profile.get("model_id") == "IGM-SCHEMATIC-PENTAMER-V0", "unexpected V0 profile id")
    require(profile.get("validation_level") == "V0", "Pages fixture must remain V0")
    require(profile.get("claims", {}).get("biological_validity_claimed") is False, "V0 biological validity claim forbidden")
    require(len(profile.get("components", [])) == 16, "V0 profile must contain 5 sectors + 10 arms + 1 J marker")
    unknown = [p for p in profile.get("parameters", []) if p.get("status") == "unknown"]
    require(all("value" not in p for p in unknown), "unknown Pages parameters may not carry values")

    policy = load(POLICY)
    indexed = {item.get("id"): item.get("name") for item in policy.get("core_invariants", [])}
    for invariant_id, name in INVARIANTS.items():
        require(indexed.get(invariant_id) == name, f"governance missing {invariant_id}")
        require(name in model or invariant_id == "INV-BIO-001", f"model missing invariant phrase {invariant_id}")

    print("OK: IGM Pages visual laboratory passed static governance/determinism validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
