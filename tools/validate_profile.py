#!/usr/bin/env python3
"""Fail-closed semantic validation for IGM model profiles.

JSON Schema handles structural validation. This validator enforces repository-specific
cross-field rules that JSON Schema cannot express portably, including unique stable
identifiers, source-resolution rules, and participant references. It intentionally
remains dependency-free so it can run before any native or accelerator runtime.
"""

from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SOURCE_REGISTRY = ROOT / "research" / "sources.json"

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


def is_finite_number(value: Any) -> bool:
    return (
        isinstance(value, (int, float))
        and not isinstance(value, bool)
        and math.isfinite(value)
    )


def unique_string_set(values: Any, label: str) -> set[str]:
    if values is None:
        return set()
    if not isinstance(values, list):
        raise ProfileError(f"{label} must be an array")
    result: set[str] = set()
    for value in values:
        if not isinstance(value, str) or not value:
            raise ProfileError(f"{label} must contain non-empty strings")
        if value in result:
            raise ProfileError(f"{label} contains duplicate id: {value}")
        result.add(value)
    return result


def source_registry_ids() -> set[str]:
    registry = load_json(SOURCE_REGISTRY)
    if not isinstance(registry, dict) or not isinstance(registry.get("sources"), list):
        raise ProfileError("research/sources.json requires a sources array")
    ids: set[str] = set()
    for index, source in enumerate(registry["sources"]):
        if not isinstance(source, dict):
            raise ProfileError(f"source registry entry[{index}] must be an object")
        source_id = source.get("id")
        if not isinstance(source_id, str) or not source_id:
            raise ProfileError(f"source registry entry[{index}] requires id")
        if source_id in ids:
            raise ProfileError(f"duplicate source registry id: {source_id}")
        ids.add(source_id)
    return ids


def validate_parameter_bounds(parameter: dict[str, Any], index: int) -> None:
    has_lower = "lower_bound" in parameter and parameter["lower_bound"] is not None
    has_upper = "upper_bound" in parameter and parameter["upper_bound"] is not None
    if not (has_lower or has_upper):
        if "value" in parameter and isinstance(parameter["value"], float) and not math.isfinite(parameter["value"]):
            raise ProfileError(f"parameter[{index}] value must be finite")
        return

    if "value" not in parameter or not is_finite_number(parameter["value"]):
        raise ProfileError(
            f"parameter[{index}] declares numeric bounds and therefore requires a finite numeric value"
        )
    value = parameter["value"]

    lower = parameter.get("lower_bound")
    upper = parameter.get("upper_bound")
    if has_lower and not is_finite_number(lower):
        raise ProfileError(f"parameter[{index}] lower_bound must be finite numeric")
    if has_upper and not is_finite_number(upper):
        raise ProfileError(f"parameter[{index}] upper_bound must be finite numeric")
    if has_lower and has_upper and lower > upper:
        raise ProfileError(f"parameter[{index}] lower_bound exceeds upper_bound")
    if has_lower and value < lower:
        raise ProfileError(f"parameter[{index}] value is below lower_bound")
    if has_upper and value > upper:
        raise ProfileError(f"parameter[{index}] value is above upper_bound")


def validate_profile(profile: Any, registry_ids: set[str] | None = None) -> None:
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

    declared_sources = unique_string_set(profile.get("source_ids", []), "source_ids")
    if registry_ids is not None:
        missing = sorted(declared_sources - registry_ids)
        if missing:
            raise ProfileError(
                "profile source_ids do not resolve in research/sources.json: "
                + ", ".join(missing)
            )

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
        component_sources = unique_string_set(
            component.get("source_ids", []), f"component[{index}].source_ids"
        )
        missing = component_sources - declared_sources
        if missing:
            raise ProfileError(
                f"component[{index}] references undeclared source ids: {sorted(missing)}"
            )

    parameters = profile.get("parameters")
    if not isinstance(parameters, list):
        raise ProfileError("parameters must be an array")
    seen_parameter_names: set[str] = set()
    for index, parameter in enumerate(parameters):
        if not isinstance(parameter, dict):
            raise ProfileError(f"parameter[{index}] must be an object")
        name = parameter.get("name")
        if not isinstance(name, str) or not name:
            raise ProfileError(f"parameter[{index}] requires name")
        if name in seen_parameter_names:
            raise ProfileError(f"duplicate parameter name: {name}")
        seen_parameter_names.add(name)

        status = parameter.get("status")
        if status == "unknown" and "value" in parameter:
            raise ProfileError(
                f"parameter[{index}] is unknown and therefore may not carry a value"
            )
        source_id = parameter.get("source_id")
        if source_id is not None:
            if not isinstance(source_id, str) or not source_id:
                raise ProfileError(f"parameter[{index}] source_id must be null or non-empty string")
            if source_id not in declared_sources:
                raise ProfileError(
                    f"parameter[{index}] source_id is not declared by profile: {source_id}"
                )
        if status in EVIDENCE_BACKED:
            derivation = parameter.get("derivation")
            if not isinstance(source_id, str) or not source_id:
                raise ProfileError(
                    f"parameter[{index}] with status {status} requires source_id"
                )
            if source_id not in declared_sources:
                raise ProfileError(
                    f"parameter[{index}] with status {status} has unresolved source_id"
                )
            if not isinstance(derivation, str) or not derivation:
                raise ProfileError(
                    f"parameter[{index}] with status {status} requires derivation"
                )
        if status == "observed" and parameter.get("derivation") != "direct":
            raise ProfileError("observed parameters require derivation=direct")
        if status == "calibrated" and parameter.get("derivation") != "calibrated":
            raise ProfileError("calibrated parameters require derivation=calibrated")
        if status == "unknown":
            if source_id is not None or parameter.get("derivation") not in (None, "unknown"):
                raise ProfileError("unknown parameters must not claim source provenance")
        validate_parameter_bounds(parameter, index)

    constraints = profile.get("constraints", [])
    if not isinstance(constraints, list):
        raise ProfileError("constraints must be an array")
    seen_constraint_ids: set[str] = set()
    for index, constraint in enumerate(constraints):
        if not isinstance(constraint, dict):
            raise ProfileError(f"constraint[{index}] must be an object")
        constraint_id = constraint.get("id")
        if not isinstance(constraint_id, str) or not constraint_id:
            raise ProfileError(f"constraint[{index}] requires id")
        if constraint_id in seen_constraint_ids:
            raise ProfileError(f"duplicate constraint id: {constraint_id}")
        seen_constraint_ids.add(constraint_id)
        constraint_sources = unique_string_set(
            constraint.get("source_ids", []), f"constraint[{index}].source_ids"
        )
        missing = constraint_sources - declared_sources
        if missing:
            raise ProfileError(
                f"constraint[{index}] references undeclared source ids: {sorted(missing)}"
            )

        definition = constraint.get("definition")
        if isinstance(definition, dict) and "participants" in definition:
            participant_list = definition["participants"]
            if not isinstance(participant_list, list) or not participant_list:
                raise ProfileError(f"constraint[{index}] participants must be non-empty array")
            participants: set[str] = set()
            for participant in participant_list:
                if not isinstance(participant, str) or not participant:
                    raise ProfileError(f"constraint[{index}] participant must be non-empty string")
                if participant in participants:
                    raise ProfileError(
                        f"constraint[{index}] contains duplicate participant: {participant}"
                    )
                if participant not in seen_ids:
                    raise ProfileError(
                        f"constraint[{index}] references missing component: {participant}"
                    )
                participants.add(participant)

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
        "source_ids": [],
        "claims": {
            "biological_validity_claimed": False,
            "clinical_validity_claimed": False,
            "medical_device_claimed": False,
            "diagnostic_use_claimed": False,
            "treatment_use_claimed": False,
        },
    }
    validate_profile(base, set())

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

    unresolved_provenance = json.loads(json.dumps(base))
    unresolved_provenance["parameters"] = [
        {
            "name": "source-backed",
            "status": "observed",
            "value": 1.0,
            "source_id": "missing.source",
            "derivation": "direct",
        }
    ]
    probes.append(("source id not declared", unresolved_provenance))

    out_of_bounds = json.loads(json.dumps(base))
    out_of_bounds["parameters"] = [
        {
            "name": "bounded",
            "status": "assumed",
            "value": 12.0,
            "lower_bound": 0.0,
            "upper_bound": 10.0,
        }
    ]
    probes.append(("out-of-bounds parameter", out_of_bounds))

    nonnumeric_bounded = json.loads(json.dumps(base))
    nonnumeric_bounded["parameters"] = [
        {
            "name": "bounded",
            "status": "assumed",
            "value": False,
            "lower_bound": 0.0,
            "upper_bound": 10.0,
        }
    ]
    probes.append(("nonnumeric bounded parameter", nonnumeric_bounded))

    duplicate_participant = json.loads(json.dumps(base))
    duplicate_participant["constraints"] = [
        {
            "id": "constraint:test",
            "kind": "synthetic",
            "status": "assumed",
            "definition": {"participants": ["core:a", "core:a"]},
        }
    ]
    probes.append(("duplicate constraint participant", duplicate_participant))

    missing_participant = json.loads(json.dumps(base))
    missing_participant["constraints"] = [
        {
            "id": "constraint:test",
            "kind": "synthetic",
            "status": "assumed",
            "definition": {"participants": ["missing:node"]},
        }
    ]
    probes.append(("missing constraint participant", missing_participant))

    for label, probe in probes:
        try:
            validate_profile(probe, set())
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
        registry_ids = source_registry_ids()
        validate_profile(load_json(Path(argv[0])), registry_ids)
    except ProfileError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {argv[0]} passed IGM semantic profile validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
