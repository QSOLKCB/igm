#!/usr/bin/env python3
"""Validate IGM profiles against the repository JSON Schema without third-party packages.

This implements the Draft-2020-12 keyword subset used by model-profile.schema.json.
Cross-field project rules remain in validate_profile.py.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas" / "model-profile.schema.json"


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
    if wanted == "null": return value is None
    if wanted == "object": return isinstance(value, dict)
    if wanted == "array": return isinstance(value, list)
    if wanted == "string": return isinstance(value, str)
    if wanted == "boolean": return isinstance(value, bool)
    if wanted == "integer": return isinstance(value, int) and not isinstance(value, bool)
    if wanted == "number": return isinstance(value, (int, float)) and not isinstance(value, bool)
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


def validate(value: Any, schema: dict[str, Any], root: dict[str, Any], path: str = "$") -> None:
    if "$ref" in schema:
        validate(value, resolve_ref(root, schema["$ref"]), root, path)
        return

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
        required = schema.get("required", [])
        for key in required:
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


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print("usage: validate_json_schema.py PROFILE.json", file=sys.stderr)
        return 2
    try:
        schema = load(SCHEMA_PATH)
        profile = load(Path(argv[0]))
        validate(profile, schema, schema)
    except ValidationError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {argv[0]} conforms to schemas/model-profile.schema.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
