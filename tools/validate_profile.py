#!/usr/bin/env python3
"""Fail-closed semantic validation for IGM model profiles.

JSON Schema handles structural validation. This validator enforces repository-specific
cross-field rules that JSON Schema cannot express portably, especially uniqueness by
component identifier. It intentionally remains dependency-free so it can run before any
future native or accelerator runtime.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

SEMVER_RE = re.compile(
    r"^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"
    r"(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)
EVIDENCE_BACKED = {"observed", "source-derived", "calibrated"}
EARLY_LEVELS = {"V0", "V1", "V2"}
FORBIDDEN_UPSTREAM_CLAIMS = {
    "clinical_validity_claimed",
    "medical_device_claimed",
    "diagnostic_use_claimed",
    "treatment_use_claimed",
}


class ProfileError(ValueError):
    pass


def reject_json_constant(value: str) -> None:
    raise ProfileError(f"non-standard JSON constant is forbidden: {value}")


def load_json(path: Path) -> Any:
    try:
        return json.loads(
            path.read_text(encoding="utf-8"),
            parse_constant=reject_json_constant,
        )
    except (OSError, json.JSONDecodeError, ProfileError) as exc:
        raise ProfileError(f"{path}: {exc}") from exc


def validate_profile(profile: Any) -> None:
    if not isinstance(profile, dict):
        raise ProfileError("profile root must be an object")
    if profile.get("schema") != "IGM-MODEL-PROFILE-V1":
        raise ProfileError("unexpected profile schema")

    version = profile.get("version")
    if not isinstance(version, str) or not SEMVER_RE.fullmatch(version):
        raise ProfileError("version must be a complete semantic version")

    level = profile.get("validation_level")
    if level not in {"V0", "V1", "V2", "V3", "V4"}:
        raise ProfileError("validation_level must be V0..V4")

    components = profile.get("components")
    if not isinstance(components, list) or not components:
        raise ProfileError("components must be a non-empty array")
    seen_ids: set[str] = set()
    for index, component in enumerate(components):
        if not isinstance(component, dict):
            raise ProfileError(f"component[{index}] must be an object")
        component_id = component.get("id")
        if not isinstance(component_id, str) or not component_id:
            raise ProfileError(f"component[{index}] requires a stable id")
        if component_id in seen_ids:
            raise ProfileError(f"duplicate component id: {component_id}")
        seen_ids.add(component_id)

    parameters = profile.get("parameters")
    if not isinstance(parameters, list):
        raise ProfileError("parameters must be an array")
    for index, parameter in enumerate(parameters):
        if not isinstance(parameter, dict):
            raise ProfileError(f"parameter[{index}] must be an object")
        status = parameter.get("status")
        if status == "unknown" and "value" in parameter:
            raise ProfileError(
                f"parameter[{index}] is unknown and therefore may not carry a value"
            )
        if status in EVIDENCE_BACKED:
            source_id = parameter.get("source_id")
            derivation = parameter.get("derivation")
            if not isinstance(source_id, str) or not source_id:
                raise ProfileError(
                    f"parameter[{index}] with status {status} requires source_id"
                )
            if not isinstance(derivation, str) or not derivation:
                raise ProfileError(
                    f"parameter[{index}] with status {status} requires derivation"
                )
        if status == "observed" and parameter.get("derivation") != "direct":
            raise ProfileError("observed parameters require derivation=direct")
        if status == "calibrated" and parameter.get("derivation") != "calibrated":
            raise ProfileError("calibrated parameters require derivation=calibrated")

    claims = profile.get("claims")
    if not isinstance(claims, dict):
        raise ProfileError("claims must be an object")
    for key in FORBIDDEN_UPSTREAM_CLAIMS:
        if claims.get(key) is not False:
            raise ProfileError(f"upstream profile must set {key}=false")
    if level in EARLY_LEVELS and claims.get("biological_validity_claimed") is not False:
        raise ProfileError(
            f"{level} lacks external biological validation; biological_validity_claimed must be false"
        )


def self_test() -> None:
    base = {
        "schema": "IGM-MODEL-PROFILE-V1",
        "model_id": "IGM-SCHEMATIC-PENTAMER-V0",
        "version": "0.1.0-rc.1+cpu",
        "validation_level": "V0",
        "representation": {"primary": "articulated-geometry"},
        "components": [
            {"id": "core:a", "kind": "schematic", "source_status": "assumed"}
        ],
        "parameters": [{"name": "alpha", "status": "unknown"}],
        "claims": {
            "biological_validity_claimed": False,
            "clinical_validity_claimed": False,
            "medical_device_claimed": False,
            "diagnostic_use_claimed": False,
            "treatment_use_claimed": False,
        },
    }
    validate_profile(base)

    probes: list[tuple[str, dict[str, Any]]] = []

    duplicate = json.loads(json.dumps(base))
    duplicate["components"].append(
        {"id": "core:a", "kind": "different", "source_status": "assumed"}
    )
    probes.append(("duplicate component id", duplicate))

    unknown_value = json.loads(json.dumps(base))
    unknown_value["parameters"] = [
        {"name": "unknown-but-valued", "status": "unknown", "value": 37}
    ]
    probes.append(("unknown parameter value", unknown_value))

    unsupported_bio = json.loads(json.dumps(base))
    unsupported_bio["claims"]["biological_validity_claimed"] = True
    probes.append(("V0 biological validity claim", unsupported_bio))

    missing_provenance = json.loads(json.dumps(base))
    missing_provenance["validation_level"] = "V1"
    missing_provenance["parameters"] = [
        {"name": "source-backed", "status": "source-derived", "value": 1.0}
    ]
    probes.append(("source-derived without provenance", missing_provenance))

    for label, probe in probes:
        try:
            validate_profile(probe)
        except ProfileError:
            continue
        raise ProfileError(f"self-test failed to reject: {label}")


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        self_test()
        print("OK: IGM profile semantic validator self-test passed")
        return 0
    if len(argv) != 1:
        print("usage: validate_profile.py PROFILE.json | --self-test", file=sys.stderr)
        return 2
    try:
        validate_profile(load_json(Path(argv[0])))
    except ProfileError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {argv[0]} passed IGM semantic profile validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
