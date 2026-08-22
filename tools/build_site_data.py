#!/usr/bin/env python3
"""Build deterministic local data for the static IGM Pages visual laboratory.

The builder is a fail-closed packaging boundary: it validates the source profile
against both the repository JSON Schema subset and the semantic profile gate
before any profile bytes are copied into the deployable Pages artifact.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from validate_json_schema import (
    ValidationError as SchemaValidationError,
    audit_schema,
    load as load_schema_json,
    validate as validate_schema,
)
from validate_profile import ProfileError, validate_profile

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
DATA = SITE / "data"
PROFILE = ROOT / "profiles" / "igm-schematic-pentamer-v0.json"
SOURCES = ROOT / "research" / "sources.json"
SCHEMA = ROOT / "schemas" / "model-profile.schema.json"


def reject_constant(value: str) -> None:
    raise ValueError(f"non-standard JSON constant forbidden: {value}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)


def canonical_bytes(value) -> bytes:
    return (
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
    ).encode("utf-8")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def validate_source_profile(profile) -> None:
    try:
        schema = load_schema_json(SCHEMA)
        audit_schema(schema)
        validate_schema(profile, schema, schema)
        validate_profile(profile)
    except (SchemaValidationError, ProfileError) as exc:
        raise SystemExit(f"FAIL: refusing to package invalid Pages profile: {exc}") from exc


def main() -> int:
    profile = load(PROFILE)
    validate_source_profile(profile)
    sources = load(SOURCES)

    DATA.mkdir(parents=True, exist_ok=True)
    (DATA / "profile.json").write_bytes(canonical_bytes(profile))
    (DATA / "sources.json").write_bytes(canonical_bytes(sources))

    manifest_entries = []
    for path in sorted(SITE.rglob("*")):
        if not path.is_file() or path.name == "manifest.json":
            continue
        rel = path.relative_to(SITE).as_posix()
        manifest_entries.append(
            {"path": rel, "sha256": sha256(path), "bytes": path.stat().st_size}
        )
    manifest = {
        "schema": "igm-pages-manifest/1",
        "profile": profile["model_id"],
        "validation_level": profile["validation_level"],
        "non_clinical": True,
        "entries": manifest_entries,
    }
    (SITE / "manifest.json").write_bytes(canonical_bytes(manifest))
    print(f"OK: built IGM Pages data with {len(manifest_entries)} hashed files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
