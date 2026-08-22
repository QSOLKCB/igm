#!/usr/bin/env python3
"""Validate persisted IGM Phase 3C campaign artifacts without external dependencies."""

from __future__ import annotations

import hashlib
import json
import math
import sys
from pathlib import Path
from typing import Any

INV_BIO_001 = "Perfect Mathematics Does Not Equal Perfect Biological Reality"
INV_RUNTIME_001 = "Execution Adjacency Does Not Imply Biological Adjacency"


class CampaignError(ValueError):
    pass


def reject_constant(value: str) -> None:
    raise CampaignError(f"non-standard JSON constant forbidden: {value}")


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, CampaignError) as exc:
        raise CampaignError(f"{path}: {exc}") from exc


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(65536), b""):
            digest.update(block)
    return digest.hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise CampaignError(message)


def finite_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def validate_checksums(directory: Path) -> None:
    checksum_path = directory / "SHA256SUMS"
    require(checksum_path.is_file(), "SHA256SUMS missing")
    lines = [line for line in checksum_path.read_text(encoding="utf-8").splitlines() if line]
    seen: set[str] = set()
    for line in lines:
        parts = line.split("  ", 1)
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, relative = parts
        require(len(digest) == 64 and all(c in "0123456789abcdef" for c in digest), "invalid checksum digest")
        require(relative not in seen, f"duplicate checksum path: {relative}")
        seen.add(relative)
        path = directory / relative
        require(path.is_file(), f"checksummed artifact missing: {relative}")
        require(sha256_file(path) == digest, f"checksum mismatch: {relative}")

    expected = {
        path.name
        for path in directory.iterdir()
        if path.is_file() and path.name != "SHA256SUMS"
    }
    require(seen == expected, f"SHA256SUMS coverage mismatch: seen={sorted(seen)} expected={sorted(expected)}")


def validate_rejected(directory: Path) -> None:
    receipt = load_json(directory / "rejected.json")
    require(receipt.get("schema") == "IGM-CAMPAIGN-REJECTION-V1", "unexpected rejection schema")
    require(receipt.get("campaign_contract") == "IGM-EXEC-CAMPAIGN-V1", "unexpected campaign contract")
    require(receipt.get("preserved") is True, "rejected receipt must be preserved=true")
    require(receipt.get("non_clinical") is True, "rejected receipt must remain non-clinical")
    require(receipt.get("biological_validity_claimed") is False, "rejected receipt cannot claim biological validity")
    require(receipt.get("inv_bio_001") == INV_BIO_001, "INV-BIO-001 missing from rejection")
    require(receipt.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from rejection")
    require(isinstance(receipt.get("stage"), str) and receipt["stage"], "rejection stage missing")
    require(isinstance(receipt.get("reason"), str) and receipt["reason"], "rejection reason missing")
    validate_checksums(directory)


def validate_accepted(directory: Path) -> None:
    required_files = {
        "correctness-receipt.json",
        "benchmark-receipt.json",
        "execution-graph.json",
        "memory-layout.json",
        "memory-plan.json",
        "environment.json",
        "chunks.json",
        "campaign-manifest.json",
        "SHA256SUMS",
    }
    present = {path.name for path in directory.iterdir() if path.is_file()}
    require(required_files <= present, f"accepted campaign missing files: {sorted(required_files - present)}")

    correctness = load_json(directory / "correctness-receipt.json")
    require(correctness.get("schema") == "IGM-CAMPAIGN-CORRECTNESS-RECEIPT-V1", "bad correctness schema")
    require(correctness.get("campaign_contract") == "IGM-EXEC-CAMPAIGN-V1", "bad campaign contract")
    require(correctness.get("graph_contract") == "IGM-EXEC-GRAPH-C5-K2-C3-V1", "bad graph contract")
    require(correctness.get("layout_contract") == "IGM-WARP32-AOSOA-V1", "bad memory layout contract")
    require(correctness.get("verification_accepted") is True, "correctness receipt must carry accepted verification")
    require(correctness.get("result_identity_worker_independent") is True, "result must be worker-independent")
    require(correctness.get("result_identity_chunk_independent") is True, "result must be chunk-independent")
    require(correctness.get("biological_validity_claimed") is False, "correctness cannot claim biological validity")
    require(correctness.get("clinical_validity_claimed") is False, "correctness cannot claim clinical validity")
    require(correctness.get("inv_bio_001") == INV_BIO_001, "INV-BIO-001 missing from correctness")
    require(correctness.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from correctness")
    for key in (
        "min_pair_distance_squared",
        "max_pair_distance_squared",
        "verification_max_geometry_residual",
        "verification_max_pair_residual",
        "verification_tolerance",
    ):
        require(finite_number(correctness.get(key)), f"correctness {key} must be finite numeric")
    require(correctness["min_pair_distance_squared"] <= correctness["max_pair_distance_squared"], "distance extrema inverted")
    require(correctness["verification_max_geometry_residual"] <= correctness["verification_tolerance"], "geometry residual exceeds tolerance")
    require(correctness["verification_max_pair_residual"] <= correctness["verification_tolerance"], "pair residual exceeds tolerance")

    benchmark = load_json(directory / "benchmark-receipt.json")
    require(benchmark.get("schema") == "IGM-CAMPAIGN-BENCHMARK-RECEIPT-V1", "bad benchmark schema")
    require(benchmark.get("identity_bearing_correctness") is False, "benchmark must not be correctness identity")
    require(benchmark.get("performance_claim") is False, "benchmark must remain observation-only")
    require(finite_number(benchmark.get("elapsed_seconds")), "elapsed_seconds must be finite")
    require(finite_number(benchmark.get("conformations_per_second")), "throughput must be finite")

    graph = load_json(directory / "execution-graph.json")
    require(graph.get("schema") == "IGM-EXEC-TRAVERSAL-RECEIPT-V1", "bad execution graph receipt schema")
    require(graph.get("graph_contract") == "IGM-EXEC-GRAPH-C5-K2-C3-V1", "bad execution graph contract")
    require(graph.get("node_count") == 30, "execution graph must have 30 nodes")
    require(graph.get("degree") == 5, "execution graph must be degree 5")
    require(graph.get("edge_count_undirected") == 75, "execution graph must have 75 undirected edges")
    require(graph.get("biological_adjacency_claimed") is False, "execution graph cannot claim biological adjacency")
    require(graph.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from graph receipt")
    nodes = graph.get("nodes")
    require(isinstance(nodes, list) and len(nodes) == 30, "execution graph nodes malformed")
    require({node.get("sequence") for node in nodes} == set(range(30)), "execution graph sequence coverage invalid")
    for node in nodes:
        neighbors = node.get("neighbors")
        require(isinstance(neighbors, list) and len(neighbors) == 5, "execution node must have five neighbors")
        require(len({entry.get("sequence") for entry in neighbors}) == 5, "execution node neighbors must be distinct")

    layout = load_json(directory / "memory-layout.json")
    require(layout.get("layout_contract") == "IGM-WARP32-AOSOA-V1", "bad layout contract")
    require(layout.get("warp_width") == 32, "warp width must be 32")
    require(layout.get("meaningful_lanes") == 30, "meaningful lanes must be 30")
    require(layout.get("padding_lanes") == 2, "padding lanes must be 2")
    require(layout.get("active_lane_count_observed") == 30, "active lane observation mismatch")
    require(layout.get("padding_lane_count_observed") == 2, "padding lane observation mismatch")
    require(layout.get("scientific_count_includes_padding") is False, "padding lanes cannot enter scientific counts")
    require(layout.get("padding_lanes_semantic") is False, "padding lanes cannot be semantic")
    require(layout.get("cell_alignment_bytes") == 128, "execution cell must retain 128-byte alignment")
    require(layout.get("cell_size_bytes") % 128 == 0, "execution cell size must be alignment multiple")

    memory = load_json(directory / "memory-plan.json")
    require(memory.get("schema") == "IGM-MEMORY-PLAN-V1", "bad memory plan schema")
    require(memory.get("requested_conformations") == correctness.get("conformation_count"), "memory plan count mismatch")
    require(memory.get("padding_excluded_from_scientific_counts") is True, "memory plan must exclude padding")
    require(memory.get("resident_capacity_cells", 0) >= 1, "resident capacity must be positive")
    require(memory.get("chunk_count", 0) >= 1, "chunk count must be positive")

    chunks = load_json(directory / "chunks.json")
    require(isinstance(chunks, list) and len(chunks) == memory["chunk_count"], "chunk list count mismatch")
    require(sum(chunk["count"] for chunk in chunks) == correctness["conformation_count"], "chunk coverage count mismatch")
    require(chunks[0]["start"] == correctness["conformation_start"], "chunk start mismatch")
    require(chunks[-1]["end_exclusive"] == correctness["conformation_end_exclusive"], "chunk end mismatch")
    for left, right in zip(chunks, chunks[1:]):
        require(left["end_exclusive"] == right["start"], "chunk plan contains gap/overlap")

    environment = load_json(directory / "environment.json")
    require(environment.get("schema") == "IGM-CAMPAIGN-ENVIRONMENT-V1", "bad environment schema")
    require(environment.get("hostname_included") is False, "hostname must not be included")
    require(environment.get("username_included") is False, "username must not be included")
    require(environment.get("raw_hardware_identifiers_included") is False, "raw hardware identifiers must not be included")

    manifest = load_json(directory / "campaign-manifest.json")
    require(manifest.get("schema") == "IGM-CAMPAIGN-MANIFEST-V1", "bad campaign manifest schema")
    require(manifest.get("rejected") is False, "accepted manifest cannot be rejected")
    require(manifest.get("benchmark_identity_is_correctness_identity") is False, "benchmark/correctness identity boundary broken")
    require(manifest.get("correctness_result_sha256") == correctness.get("result_sha256"), "manifest correctness identity mismatch")
    require(manifest.get("graph_sha256") == graph.get("graph_sha256"), "manifest graph identity mismatch")
    require(manifest.get("traversal_sha256") == graph.get("traversal_sha256"), "manifest traversal identity mismatch")
    require(manifest.get("chunk_count") == memory.get("chunk_count"), "manifest chunk count mismatch")
    artifacts = manifest.get("artifacts")
    require(isinstance(artifacts, list) and artifacts, "manifest artifacts missing")
    for artifact in artifacts:
        path = directory / artifact["path"]
        require(path.is_file(), f"manifest artifact missing: {artifact['path']}")
        require(path.stat().st_size == artifact["bytes"], f"manifest byte count mismatch: {artifact['path']}")
        require(sha256_file(path) == artifact["sha256"], f"manifest hash mismatch: {artifact['path']}")

    validate_checksums(directory)


def validate(directory: Path) -> None:
    require(directory.is_dir(), f"campaign directory not found: {directory}")
    if (directory / "rejected.json").is_file():
        validate_rejected(directory)
    else:
        validate_accepted(directory)


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print("usage: validate_campaign.py CAMPAIGN_DIR", file=sys.stderr)
        return 2
    try:
        validate(Path(argv[0]))
    except CampaignError as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 1
    print(f"OK: {argv[0]} passed IGM Phase 3C campaign validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
