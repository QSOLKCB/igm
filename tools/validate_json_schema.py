#!/usr/bin/env python3
"""Validate IGM profiles against the repository JSON Schema without third-party packages.

This implements the Draft-2020-12 keyword subset used by model-profile.schema.json.
Cross-field project rules remain in validate_profile.py.

The validator fails closed if the schema introduces a validation keyword this
implementation does not understand. Annotation keywords and x-igm-* extensions
may be present without affecting validation semantics.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "model-profile.schema.json"

ANNOTATION_KEYS = {
    "$schema",
    "$id",
    "$comment",
    "title",
    "description",
    "default",
    "examples",
    "deprecated",
    "readOnly",
    "writeOnly",
}
SUPPORTED_VALIDATION_KEYS = {
    "$ref",
    "$defs",
    "type",
    "const",
    "enum",
    "required",
    "properties",
    "additionalProperties",
    "minLength",
    "pattern",
    "minItems",
    "uniqueItems",
    "items",
    "not",
    "allOf",
    "if",
    "then",
}


class ValidationError(ValueError):
    pass


def reject_constant(value: str) -> None:
    raise ValidationError(f"non-standard JSON constant is forbidden: {value}")


def load(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, ValidationError) as exc:
        raise ValidationError(f"{path}: {exc}") from exc


def type_matches(value: Any, wanted: str) -> bool:
    if wanted == "null":
        return value is None
    if wanted == "object":
        return isinstance(value, dict)
    if wanted == "array":
        return isinstance(value, list)
    if wanted == "string":
        return isinstance(value, str)
    if wanted == "boolean":
        return isinstance(value, bool)
    if wanted == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if wanted == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    raise ValidationError(f"unsupported schema type keyword: {wanted}")


def resolve_ref(root: dict[str, Any], ref: str) -> dict[str, Any]:
    if not ref.startswith("#/"):
        raise ValidationError(f"external $ref not supported: {ref}")
    node: Any = root
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        if not isinstance(node, dict) or part not in node:
            raise ValidationError(f"unresolved $ref: {ref}")
        node = node[part]
    if not isinstance(node, dict):
        raise ValidationError(f"$ref target must be object: {ref}")
    return node


def audit_schema(schema: Any, path: str = "$schema") -> None:
    """Reject schema semantics this dependency-free validator would ignore."""
    if not isinstance(schema, dict):
        raise ValidationError(f"{path}: schema node must be an object")

    for key in schema:
        if key in ANNOTATION_KEYS or key in SUPPORTED_VALIDATION_KEYS or key.startswith("x-igm-"):
            continue
        raise ValidationError(
            f"{path}: unsupported JSON Schema keyword {key!r}; implement it before relying on it"
        )

    additional = schema.get("additionalProperties")
    if additional is not None and not isinstance(additional, bool):
        raise ValidationError(
            f"{path}.additionalProperties: schema-valued additionalProperties is not supported"
        )

    properties = schema.get("properties")
    if properties is not None:
        if not isinstance(properties, dict):
            raise ValidationError(f"{path}.properties must be an object")
        for name, child in properties.items():
            audit_schema(child, f"{path}.properties[{name!r}]")

    defs = schema.get("$defs")
    if defs is not None:
        if not isinstance(defs, dict):
            raise ValidationError(f"{path}.$defs must be an object")
        for name, child in defs.items():
            audit_schema(child, f"{path}.$defs[{name!r}]")

    for key in ("items", "not", "if", "then"):
        if key in schema:
            audit_schema(schema[key], f"{path}.{key}")

    if "allOf" in schema:
        if not isinstance(schema["allOf"], list):
            raise ValidationError(f"{path}.allOf must be an array")
        for index, child in enumerate(schema["allOf"]):
            audit_schema(child, f"{path}.allOf[{index}]")


def validate(value: Any, schema: dict[str, Any], root: dict[str, Any], path: str = "$") -> None:
    if "$ref" in schema:
        validate(value, resolve_ref(root, schema["$ref"]), root, path)

    if "const" in schema and value != schema["const"]:
        raise ValidationError(f"{path}: expected const {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        raise ValidationError(f"{path}: value {value!r} not in enum")

    if "type" in schema:
        wanted = schema["type"]
        types = wanted if isinstance(wanted, list) else [wanted]
        if not any(type_matches(value, item) for item in types):
            raise ValidationError(f"{path}: expected type {types}, got {type(value).__name__}")

    if isinstance(value, str):
        if "minLength" in schema and len(value) < schema["minLength"]:
            raise ValidationError(f"{path}: string shorter than minLength")
        if "pattern" in schema and re.fullmatch(schema["pattern"], value) is None:
            raise ValidationError(f"{path}: string does not match pattern")

    if isinstance(value, list):
        if "minItems" in schema and len(value) < schema["minItems"]:
            raise ValidationError(f"{path}: array shorter than minItems")
        if schema.get("uniqueItems"):
            canonical = [json.dumps(item, sort_keys=True, separators=(",", ":")) for item in value]
            if len(canonical) != len(set(canonical)):
                raise ValidationError(f"{path}: array items must be unique")
        if isinstance(schema.get("items"), dict):
            for index, item in enumerate(value):
                validate(item, schema["items"], root, f"{path}[{index}]")

    if isinstance(value, dict):
        for key in schema.get("required", []):
            if key not in value:
                raise ValidationError(f"{path}: missing required property {key!r}")
        props = schema.get("properties", {})
        if schema.get("additionalProperties") is False:
            extras = set(value) - set(props)
            if extras:
                raise ValidationError(f"{path}: additional properties forbidden: {sorted(extras)}")
        for key, subschema in props.items():
            if key in value and isinstance(subschema, dict):
                validate(value[key], subschema, root, f"{path}.{key}")

    if "not" in schema:
        try:
            validate(value, schema["not"], root, path)
        except ValidationError:
            pass
        else:
            raise ValidationError(f"{path}: matched forbidden `not` schema")

    for sub in schema.get("allOf", []):
        if "if" in sub:
            try:
                validate(value, sub["if"], root, path)
            except ValidationError:
                continue
            if "then" in sub:
                validate(value, sub["then"], root, path)
        else:
            validate(value, sub, root, path)


def self_test() -> None:
    schema = load(SCHEMA_PATH)
    audit_schema(schema)
    for probe in (
        {"type": "number", "minimum": 0},
        {"anyOf": [{"type": "string"}, {"type": "number"}]},
    ):
        try:
            audit_schema(probe, "$self-test")
        except ValidationError:
            continue
        raise ValidationError("self-test failed to reject unsupported validation keyword")


def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        try:
            self_test()
        except ValidationError as exc:
            print(f"FAIL: {exc}", file=sys.stderr)
            return 1
        print("OK: IGM JSON Schema subset validator self-test passed")
        return 0
    if len(argv) != 1:
        print("usage: validate_json_schema.py PROFILE.json | --self-test", file=sys.stderr)
        return 2
    try:
        schema = load(SCHEMA_PATH)
        audit_schema(schema)
        profile = load(Path(argv[0]))
        validate(profile, schema, schema)
    except ValidationError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {argv[0]} conforms to schemas/model-profile.schema.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
