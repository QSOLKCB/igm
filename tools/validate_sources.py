#!/usr/bin/env python3
"""Fail-closed Phase 4 source/evidence validation for IGM."""

from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "research" / "sources.json"
SNAPSHOT_POLICY = ROOT / "research" / "source-snapshot-policy.json"
V0_CONSTANTS = ROOT / "research" / "v0-implementation-constants.json"
CRYO_FIXTURE = ROOT / "research" / "evidence" / "cryo-em-pentamer-count.json"

DOI_RE = re.compile(r"^10\.[0-9]{4,9}/\S+$")
PDB_RE = re.compile(r"^[0-9][A-Za-z0-9]{3}$")
EMDB_RE = re.compile(r"^EMD-[0-9]+$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")

EVIDENCE_CLASSES = {
    "public-structure",
    "peer-reviewed-literature",
    "preprint",
    "molecular-dynamics",
    "biochemical-measurement",
    "background-only",
}
STRUCTURAL_AUTHORITIES = {"structural-source"}
VALID_ACCESS = {
    "publicly-accessible",
    "open-access",
    "registration-required",
    "restricted",
}
VALID_SNAPSHOT_MODES = {"reference-only", "hash-only", "packaged"}


class SourceError(ValueError):
    pass


def fail(message: str) -> None:
    raise SourceError(message)


def reject_constant(value: str) -> None:
    raise SourceError(f"non-standard JSON constant forbidden: {value}")


def load(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, SourceError) as exc:
        raise SourceError(f"{path}: {exc}") from exc


def finite_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def validate_registry(registry: Any) -> dict[str, dict[str, Any]]:
    if not isinstance(registry, dict) or registry.get("schema") != "igm-source-registry/1":
        fail("unexpected source registry schema")
    if registry.get("project") != "QSOLKCB/igm":
        fail("unexpected source registry project")
    sources = registry.get("sources")
    if not isinstance(sources, list) or not sources:
        fail("source registry requires a non-empty sources array")

    by_id: dict[str, dict[str, Any]] = {}
    external_ids: dict[tuple[str, str], str] = {}
    for index, source in enumerate(sources):
        if not isinstance(source, dict):
            fail(f"source[{index}] must be an object")
        source_id = source.get("id")
        if not isinstance(source_id, str) or not source_id:
            fail(f"source[{index}] requires id")
        if source_id in by_id:
            fail(f"duplicate source id: {source_id}")
        by_id[source_id] = source

        url = source.get("url")
        if not isinstance(url, str) or not url.startswith("https://"):
            fail(f"source {source_id} requires https URL")
        source_class = source.get("class")
        authority = source.get("authority")
        if not isinstance(source_class, str) or not source_class:
            fail(f"source {source_id} requires class")
        if not isinstance(authority, str) or not authority:
            fail(f"source {source_id} requires authority")

        for field, pattern in (("doi", DOI_RE), ("pdb", PDB_RE), ("emdb", EMDB_RE)):
            value = source.get(field)
            if value is None:
                continue
            if not isinstance(value, str) or not pattern.fullmatch(value):
                fail(f"source {source_id} has malformed {field}: {value!r}")
            key = (field, value.lower())
            previous = external_ids.get(key)
            if previous is not None and previous != source_id:
                fail(f"duplicate external identifier {field}={value}: {previous}, {source_id}")
            external_ids[key] = source_id

        if authority in STRUCTURAL_AUTHORITIES:
            if not any(source.get(field) for field in ("doi", "pdb", "emdb")):
                fail(f"structural source {source_id} requires DOI, PDB, or EMDB identifier")

        if source_class in EVIDENCE_CLASSES:
            access = source.get("access")
            if not isinstance(access, dict):
                fail(f"evidence source {source_id} requires access metadata")
            if access.get("status") not in VALID_ACCESS:
                fail(f"evidence source {source_id} has invalid access status")
            redistribution = access.get("redistribution")
            if not isinstance(redistribution, str) or not redistribution:
                fail(f"evidence source {source_id} requires redistribution guidance")
            for key in ("supports", "does_not_support"):
                values = source.get(key)
                if not isinstance(values, list) or not values:
                    fail(f"evidence source {source_id} requires non-empty {key}")
                if not all(isinstance(item, str) and item for item in values):
                    fail(f"evidence source {source_id} {key} must contain non-empty strings")
                if len(values) != len(set(values)):
                    fail(f"evidence source {source_id} {key} contains duplicates")

    structural = [s for s in sources if s.get("authority") == "structural-source"]
    if not any(s.get("doi") for s in structural):
        fail("structural registry must contain at least one DOI")
    if not any(s.get("pdb") for s in structural):
        fail("structural registry must contain at least one PDB identifier")
    if not any(s.get("emdb") for s in structural):
        fail("structural registry must contain at least one EMDB identifier")
    return by_id


def validate_snapshot_policy(policy: Any, sources: dict[str, dict[str, Any]]) -> None:
    if not isinstance(policy, dict) or policy.get("schema") != "IGM-SOURCE-SNAPSHOT-POLICY-V1":
        fail("unexpected source snapshot policy schema")
    if policy.get("default_mode") != "reference-only":
        fail("source snapshot default must remain reference-only")
    records = policy.get("records")
    if not isinstance(records, list):
        fail("source snapshot policy requires records array")
    seen: set[str] = set()
    for index, record in enumerate(records):
        if not isinstance(record, dict):
            fail(f"snapshot record[{index}] must be object")
        source_id = record.get("source_id")
        if source_id not in sources:
            fail(f"snapshot policy references unknown source: {source_id}")
        if source_id in seen:
            fail(f"duplicate snapshot policy record: {source_id}")
        seen.add(source_id)
        mode = record.get("mode")
        if mode not in VALID_SNAPSHOT_MODES:
            fail(f"snapshot policy {source_id} has invalid mode")
        permission = record.get("redistribution_permission_verified")
        committed = record.get("external_payload_committed")
        digest = record.get("external_payload_sha256")
        if not isinstance(permission, bool) or not isinstance(committed, bool):
            fail(f"snapshot policy {source_id} requires boolean permission/commit flags")
        if mode == "reference-only":
            if permission or committed or digest is not None:
                fail(f"reference-only snapshot {source_id} cannot claim permission, payload, or hash")
        elif mode == "hash-only":
            if committed or not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                fail(f"hash-only snapshot {source_id} requires SHA-256 and no committed payload")
        elif mode == "packaged":
            if not permission or not committed or not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                fail(f"packaged snapshot {source_id} requires verified permission, committed payload, and SHA-256")
        if not isinstance(record.get("reason"), str) or not record["reason"]:
            fail(f"snapshot policy {source_id} requires reason")


def validate_v0_constants(manifest: Any) -> None:
    if not isinstance(manifest, dict) or manifest.get("schema") != "IGM-V0-IMPLEMENTATION-CONSTANTS-V1":
        fail("unexpected V0 implementation-constants schema")
    if manifest.get("validation_level") != "V0" or manifest.get("biological_meaning_claimed") is not False:
        fail("V0 implementation constants must remain non-biological V0 metadata")
    if manifest.get("source_informed_inheritance") != "forbidden":
        fail("source-informed profiles must be forbidden from silently inheriting V0 constants")
    expected = {
        "v0_subunit_z_amplitude": 0.08,
        "v0_fab_z_offset": 0.06,
        "v0_jchain_y_ratio": 0.35,
    }
    constants = manifest.get("constants")
    if not isinstance(constants, list) or len(constants) != len(expected):
        fail("V0 implementation constants manifest must contain exactly the three legacy constants")
    found: dict[str, float] = {}
    for item in constants:
        if not isinstance(item, dict):
            fail("V0 implementation constant must be object")
        constant_id = item.get("id")
        value = item.get("value")
        if constant_id not in expected or not finite_number(value):
            fail(f"unexpected V0 implementation constant: {constant_id}")
        if item.get("status") != "assumed":
            fail(f"V0 implementation constant {constant_id} must remain assumed")
        rule = item.get("source_informed_rule")
        if not isinstance(rule, str) or "may not inherit" not in rule:
            fail(f"V0 implementation constant {constant_id} lacks source-informed non-inheritance rule")
        found[constant_id] = float(value)
    for key, value in expected.items():
        if found.get(key) != value:
            fail(f"V0 implementation constant drift: {key}")


def validate_uncertainty(value: Any) -> None:
    if not isinstance(value, dict):
        fail("evidence uncertainty must be object")
    kind = value.get("kind")
    if kind not in {"unknown", "interval", "standard-deviation", "confidence-interval", "source-reported"}:
        fail("evidence uncertainty kind invalid")
    if kind in {"unknown", "source-reported"}:
        if not isinstance(value.get("notes"), str) or not value["notes"]:
            fail(f"uncertainty kind {kind} requires explanatory notes")
    if kind in {"interval", "confidence-interval"}:
        lower, upper = value.get("lower"), value.get("upper")
        if not finite_number(lower) or not finite_number(upper) or lower > upper:
            fail(f"uncertainty kind {kind} requires finite ordered lower/upper")
    if kind == "standard-deviation":
        sigma = value.get("value")
        if not finite_number(sigma) or sigma < 0:
            fail("standard-deviation uncertainty requires finite non-negative value")


def validate_cryo_fixture(fixture: Any, sources: dict[str, dict[str, Any]], policy: Any) -> None:
    if not isinstance(fixture, dict) or fixture.get("schema") != "IGM-EVIDENCE-INPUT-V1":
        fail("unexpected cryo-EM evidence fixture schema")
    if fixture.get("adapter_id") != "IGM-CRYO-EM-PARAMETER-ADAPTER-V1":
        fail("cryo fixture must target the cryo-EM parameter adapter")
    source_id = fixture.get("source_id")
    source = sources.get(source_id)
    if source is None:
        fail("cryo fixture references missing source")
    if source.get("class") not in {"public-structure", "peer-reviewed-literature"}:
        fail("cryo adapter requires structural/public literature source")
    support = fixture.get("support_statement")
    if support not in source.get("supports", []):
        fail("cryo fixture support_statement must exactly match a registry supports statement")
    target = fixture.get("target")
    if not isinstance(target, dict) or target.get("kind") != "parameter" or not target.get("id"):
        fail("cryo fixture must target a parameter")
    if fixture.get("derivation") != "direct":
        fail("repository cryo cardinality fixture must remain a direct observation")
    validate_uncertainty(fixture.get("uncertainty"))

    snapshot = fixture.get("snapshot")
    if not isinstance(snapshot, dict):
        fail("cryo fixture snapshot must be object")
    records = {item.get("source_id"): item for item in policy.get("records", []) if isinstance(item, dict)}
    record = records.get(source_id)
    if record is None:
        fail("cryo fixture source requires explicit snapshot policy record")
    for key in ("mode", "external_payload_committed", "redistribution_permission_verified"):
        if snapshot.get(key) != record.get(key):
            fail(f"cryo fixture snapshot does not match source policy: {key}")
    if snapshot.get("external_payload_sha256") != record.get("external_payload_sha256"):
        fail("cryo fixture payload hash does not match source policy")


def self_test() -> None:
    # Minimal malformed probes exercise the fail-closed validators without touching repository files.
    try:
        validate_uncertainty({"kind": "interval", "lower": 2.0, "upper": 1.0})
    except SourceError:
        pass
    else:
        fail("self-test failed to reject reversed uncertainty interval")

    try:
        validate_snapshot_policy(
            {
                "schema": "IGM-SOURCE-SNAPSHOT-POLICY-V1",
                "default_mode": "reference-only",
                "records": [{
                    "source_id": "x",
                    "mode": "packaged",
                    "redistribution_permission_verified": False,
                    "external_payload_committed": True,
                    "external_payload_sha256": "0" * 64,
                    "reason": "probe",
                }],
            },
            {"x": {}},
        )
    except SourceError:
        pass
    else:
        fail("self-test failed to reject packaged snapshot without verified permission")


def main(argv: list[str]) -> int:
    try:
        if argv == ["--self-test"]:
            self_test()
            print("OK: IGM Phase 4 source validator self-test passed")
            return 0
        if argv:
            print("usage: validate_sources.py [--self-test]", file=sys.stderr)
            return 2
        registry = load(REGISTRY)
        sources = validate_registry(registry)
        policy = load(SNAPSHOT_POLICY)
        validate_snapshot_policy(policy, sources)
        validate_v0_constants(load(V0_CONSTANTS))
        validate_cryo_fixture(load(CRYO_FIXTURE), sources, policy)
    except SourceError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    structural = [source for source in sources.values() if source.get("authority") == "structural-source"]
    print(
        "OK: IGM Phase 4 source registry/evidence policy validated "
        f"({len(sources)} sources, {len(structural)} structural sources)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
