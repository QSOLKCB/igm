#!/usr/bin/env python3
"""Fail-closed validation for persisted IGM Phase 3C campaign artifacts."""

from __future__ import annotations

import hashlib
import json
import math
import struct
import sys
from pathlib import Path
from typing import Any

INV_BIO_001 = "Perfect Mathematics Does Not Equal Perfect Biological Reality"
INV_RUNTIME_001 = "Execution Adjacency Does Not Imply Biological Adjacency"
NUMERICAL_PROFILE = "IGM-PENTA-CRT-F64-LUT-BLOCK-CIRCULANT-ZRESIDUAL-V1"
VERIFICATION_TOLERANCE = 1.0e-12
MAX_MEMORY_BUDGET_BYTES = 16 * 1024 * 1024 * 1024
MAX_CAMPAIGN_CHUNKS = 1_000_000
MAX_VERIFY_SAMPLES = 4096
GRAPH_DOMAIN = b"IGM-EXEC-GRAPH-C5-K2-C3-V1\0"
TRAVERSAL_DOMAIN = b"IGM-EXEC-TRAVERSAL-RECEIPT-V1\0"
CORRECTNESS_DOMAIN = b"IGM-CAMPAIGN-CORRECTNESS-V1\0"
MANIFEST_DOMAIN = b"IGM-CAMPAIGN-MANIFEST-V1\0"

ARTIFACT_ROLES = [
    ("correctness-receipt.json", "correctness"),
    ("benchmark-receipt.json", "benchmark-observation"),
    ("execution-graph.json", "execution-graph-and-traversal"),
    ("memory-layout.json", "gpu-shaped-memory-contract"),
    ("memory-plan.json", "bounded-memory-plan"),
    ("environment.json", "privacy-safe-environment-observation"),
    ("chunks.json", "deterministic-chunk-plan"),
]

ENVIRONMENT_KEYS = {
    "schema",
    "os_family",
    "architecture",
    "rustc_version",
    "cargo_version",
    "available_parallelism",
    "hostname_included",
    "username_included",
    "raw_hardware_identifiers_included",
}

EDGE_KIND_IDS = {
    "sector-previous": 0,
    "sector-next": 1,
    "arm-flip": 2,
    "lane-previous": 3,
    "lane-next": 4,
}


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


def integer(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def sha256_text(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def u64(value: int) -> bytes:
    require(integer(value) and 0 <= value < 2**64, "identity integer outside u64 domain")
    return value.to_bytes(8, "little", signed=False)


def validate_checksums(directory: Path) -> None:
    checksum_path = directory / "SHA256SUMS"
    require(checksum_path.is_file(), "SHA256SUMS missing")
    lines = [line for line in checksum_path.read_text(encoding="utf-8").splitlines() if line]
    seen: set[str] = set()
    for line in lines:
        parts = line.split("  ", 1)
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, relative = parts
        require(sha256_text(digest), "invalid checksum digest")
        require(relative not in seen, f"duplicate checksum path: {relative}")
        require(relative == Path(relative).name, f"checksum path must be a local filename: {relative}")
        seen.add(relative)
        path = directory / relative
        require(path.is_file(), f"checksummed artifact missing: {relative}")
        require(sha256_file(path) == digest, f"checksum mismatch: {relative}")

    expected = {path.name for path in directory.iterdir() if path.is_file() and path.name != "SHA256SUMS"}
    require(seen == expected, f"SHA256SUMS coverage mismatch: seen={sorted(seen)} expected={sorted(expected)}")


def validate_rejected(directory: Path) -> None:
    present = {path.name for path in directory.iterdir() if path.is_file()}
    require(present == {"rejected.json", "SHA256SUMS"}, "rejected campaign may contain only rejection receipt and checksum file")
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


def sequence_for(sector: int, arm: int, lane: int) -> int:
    return (6 * sector + 15 * arm + 10 * lane) % 30


def expected_node(sequence: int) -> tuple[int, int, int, int, list[tuple[str, int]]]:
    sector = sequence % 5
    arm = sequence % 2
    lane = sequence % 3
    storage = 6 * sector + 3 * arm + lane
    neighbors = [
        ("sector-previous", sequence_for((sector + 4) % 5, arm, lane)),
        ("sector-next", sequence_for((sector + 1) % 5, arm, lane)),
        ("arm-flip", sequence_for(sector, 1 - arm, lane)),
        ("lane-previous", sequence_for(sector, arm, (lane + 2) % 3)),
        ("lane-next", sequence_for(sector, arm, (lane + 1) % 3)),
    ]
    return sector, arm, lane, storage, neighbors


def validate_graph(graph: dict[str, Any]) -> None:
    require(graph.get("schema") == "IGM-EXEC-TRAVERSAL-RECEIPT-V1", "bad execution graph receipt schema")
    require(graph.get("graph_contract") == "IGM-EXEC-GRAPH-C5-K2-C3-V1", "bad execution graph contract")
    require(graph.get("node_count") == 30, "execution graph must have 30 nodes")
    require(graph.get("degree") == 5, "execution graph must be degree 5")
    require(graph.get("edge_count_undirected") == 75, "execution graph must have 75 undirected edges")
    require(graph.get("biological_adjacency_claimed") is False, "execution graph cannot claim biological adjacency")
    require(graph.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from graph receipt")
    nodes = graph.get("nodes")
    require(isinstance(nodes, list) and len(nodes) == 30, "execution graph nodes malformed")

    graph_hasher = hashlib.sha256()
    graph_hasher.update(GRAPH_DOMAIN)
    traversal_hasher = hashlib.sha256()
    traversal_hasher.update(TRAVERSAL_DOMAIN)

    for sequence, node in enumerate(nodes):
        require(isinstance(node, dict), f"execution node {sequence} must be an object")
        sector, arm, lane, storage, expected_neighbors = expected_node(sequence)
        require(node.get("sequence") == sequence, f"execution node order/sequence mismatch at {sequence}")
        require(node.get("sector") == sector, f"execution node {sequence} sector mismatch")
        require(node.get("arm") == arm, f"execution node {sequence} arm mismatch")
        require(node.get("lane") == lane, f"execution node {sequence} lane mismatch")
        require(node.get("storage_index") == storage, f"execution node {sequence} storage index mismatch")
        neighbors = node.get("neighbors")
        require(isinstance(neighbors, list) and len(neighbors) == 5, "execution node must have five neighbors")
        actual_neighbors = []
        for entry in neighbors:
            require(isinstance(entry, dict), "execution neighbor must be an object")
            kind = entry.get("kind")
            target = entry.get("sequence")
            require(kind in EDGE_KIND_IDS, f"unknown execution edge kind: {kind!r}")
            require(integer(target) and 0 <= target < 30, "execution neighbor sequence outside [0,29]")
            require(target != sequence, "execution graph may not contain self-neighbor")
            actual_neighbors.append((kind, target))
        require(actual_neighbors == expected_neighbors, f"execution node {sequence} does not match exact C5 x K2 x C3 neighbors")

        prefix = bytes([sequence, sector, arm, lane, storage])
        graph_hasher.update(prefix)
        traversal_hasher.update(prefix)
        for kind, target in actual_neighbors:
            graph_hasher.update(bytes([EDGE_KIND_IDS[kind], target]))

    computed_graph = graph_hasher.hexdigest()
    computed_traversal = traversal_hasher.hexdigest()
    require(graph.get("graph_sha256") == computed_graph, "execution graph SHA-256 does not match exact graph topology")
    require(graph.get("traversal_sha256") == computed_traversal, "execution traversal SHA-256 does not match exact traversal")


def validate_layout(layout: dict[str, Any]) -> None:
    require(layout.get("schema") == "IGM-MEMORY-LAYOUT-RECEIPT-V1", "bad memory layout receipt schema")
    require(layout.get("layout_contract") == "IGM-WARP32-AOSOA-V1", "bad layout contract")
    require(layout.get("warp_width") == 32, "warp width must be 32")
    require(layout.get("meaningful_lanes") == 30, "meaningful lanes must be 30")
    require(layout.get("padding_lanes") == 2, "padding lanes must be 2")
    require(layout.get("active_lane_count_observed") == 30, "active lane observation mismatch")
    require(layout.get("padding_lane_count_observed") == 2, "padding lane observation mismatch")
    require(layout.get("scientific_count_includes_padding") is False, "padding lanes cannot enter scientific counts")
    require(layout.get("padding_lanes_semantic") is False, "padding lanes cannot be semantic")
    require(layout.get("cell_alignment_bytes") == 128, "execution cell must retain 128-byte alignment")
    require(integer(layout.get("cell_size_bytes")) and layout["cell_size_bytes"] > 0, "execution cell size must be positive integer")
    require(layout["cell_size_bytes"] % 128 == 0, "execution cell size must be alignment multiple")


def validate_correctness(correctness: dict[str, Any], graph: dict[str, Any]) -> None:
    require(correctness.get("schema") == "IGM-CAMPAIGN-CORRECTNESS-RECEIPT-V1", "bad correctness schema")
    require(correctness.get("campaign_contract") == "IGM-EXEC-CAMPAIGN-V1", "bad campaign contract")
    require(correctness.get("optimization_contract") == "IGM-PENTA-CRT-CPU-V1", "bad optimization contract")
    require(correctness.get("numerical_profile") == NUMERICAL_PROFILE, "unsupported correctness numerical profile")
    require(correctness.get("graph_contract") == "IGM-EXEC-GRAPH-C5-K2-C3-V1", "bad graph contract")
    require(correctness.get("layout_contract") == "IGM-WARP32-AOSOA-V1", "bad memory layout contract")
    require(correctness.get("graph_sha256") == graph.get("graph_sha256"), "correctness graph identity mismatch")
    require(correctness.get("traversal_sha256") == graph.get("traversal_sha256"), "correctness traversal identity mismatch")
    require(correctness.get("verification_accepted") is True, "correctness receipt must carry accepted verification")
    require(correctness.get("result_identity_worker_independent") is True, "result must be worker-independent")
    require(correctness.get("result_identity_chunk_independent") is True, "result must be chunk-independent")
    require(correctness.get("biological_validity_claimed") is False, "correctness cannot claim biological validity")
    require(correctness.get("clinical_validity_claimed") is False, "correctness cannot claim clinical validity")
    require(correctness.get("inv_bio_001") == INV_BIO_001, "INV-BIO-001 missing from correctness")
    require(correctness.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from correctness")
    for key in ("model_profile_sha256", "optimization_profile_sha256", "result_sha256"):
        require(sha256_text(correctness.get(key)), f"correctness {key} must be lowercase SHA-256")
    require(isinstance(correctness.get("diagnostic_xor_fnv1a64"), str) and len(correctness["diagnostic_xor_fnv1a64"]) == 16 and all(c in "0123456789abcdef" for c in correctness["diagnostic_xor_fnv1a64"]), "bad correctness diagnostic")
    for key in ("conformation_start", "conformation_count", "conformation_end_exclusive", "logical_pair_checks", "structured_distance_evaluations", "exact_z_residual_corrections", "verification_samples"):
        require(integer(correctness.get(key)) and correctness[key] >= 0, f"correctness {key} must be non-negative integer")
    require(correctness["conformation_count"] >= 1, "correctness conformation_count must be positive")
    require(correctness["conformation_end_exclusive"] == correctness["conformation_start"] + correctness["conformation_count"], "correctness conformation range arithmetic mismatch")
    require(correctness["logical_pair_checks"] == correctness["conformation_count"] * 120, "logical pair accounting mismatch")
    require(correctness["structured_distance_evaluations"] == correctness["conformation_count"] * 60, "structured distance accounting mismatch")
    require(correctness["exact_z_residual_corrections"] == correctness["conformation_count"] * 105, "Z residual accounting mismatch")
    require(1 <= correctness["verification_samples"] <= MAX_VERIFY_SAMPLES, "verification sample count outside admitted bound")
    for key in ("min_pair_distance_squared", "max_pair_distance_squared", "verification_max_geometry_residual", "verification_max_pair_residual", "verification_tolerance"):
        require(finite_number(correctness.get(key)), f"correctness {key} must be finite numeric")
    require(correctness["min_pair_distance_squared"] <= correctness["max_pair_distance_squared"], "distance extrema inverted")
    require(correctness["verification_tolerance"] == VERIFICATION_TOLERANCE, "verification tolerance does not match admitted numerical profile")
    require(correctness["verification_max_geometry_residual"] <= VERIFICATION_TOLERANCE, "geometry residual exceeds fixed tolerance")
    require(correctness["verification_max_pair_residual"] <= VERIFICATION_TOLERANCE, "pair residual exceeds fixed tolerance")

    hasher = hashlib.sha256()
    hasher.update(CORRECTNESS_DOMAIN)
    for value in (
        correctness["optimization_contract"],
        correctness["numerical_profile"],
        correctness["graph_contract"],
        correctness["graph_sha256"],
        correctness["traversal_sha256"],
        correctness["layout_contract"],
        correctness["model_profile_sha256"],
        correctness["optimization_profile_sha256"],
    ):
        hasher.update(value.encode("utf-8"))
    hasher.update(u64(correctness["conformation_start"]))
    hasher.update(u64(correctness["conformation_count"]))
    hasher.update(int(correctness["diagnostic_xor_fnv1a64"], 16).to_bytes(8, "little"))
    hasher.update(struct.pack("<d", float(correctness["min_pair_distance_squared"])))
    hasher.update(struct.pack("<d", float(correctness["max_pair_distance_squared"])))
    require(correctness["result_sha256"] == hasher.hexdigest(), "correctness result_sha256 is not reconstructible from receipt")


def validate_benchmark(benchmark: dict[str, Any], memory: dict[str, Any] | None = None) -> None:
    require(benchmark.get("schema") == "IGM-CAMPAIGN-BENCHMARK-RECEIPT-V1", "bad benchmark schema")
    require(benchmark.get("campaign_contract") == "IGM-EXEC-CAMPAIGN-V1", "bad benchmark campaign contract")
    require(benchmark.get("identity_bearing_correctness") is False, "benchmark must not be correctness identity")
    require(benchmark.get("performance_claim") is False, "benchmark must remain observation-only")
    require(finite_number(benchmark.get("elapsed_seconds")) and benchmark["elapsed_seconds"] >= 0, "elapsed_seconds must be finite non-negative")
    require(finite_number(benchmark.get("conformations_per_second")) and benchmark["conformations_per_second"] >= 0, "throughput must be finite non-negative")
    require(integer(benchmark.get("requested_workers")) and 1 <= benchmark["requested_workers"] <= 256, "benchmark requested_workers outside bound")
    if memory is not None:
        require(benchmark.get("memory_budget_bytes") == memory.get("memory_budget_bytes"), "benchmark memory budget mismatch")
        require(benchmark.get("resident_capacity_cells") == memory.get("resident_capacity_cells"), "benchmark resident capacity mismatch")
        require(benchmark.get("chunk_count") == memory.get("chunk_count"), "benchmark chunk count mismatch")


def validate_memory(memory: dict[str, Any], correctness: dict[str, Any], layout: dict[str, Any]) -> None:
    require(memory.get("schema") == "IGM-MEMORY-PLAN-V1", "bad memory plan schema")
    require(memory.get("layout_contract") == "IGM-WARP32-AOSOA-V1", "memory plan layout contract mismatch")
    require(memory.get("requested_conformations") == correctness.get("conformation_count"), "memory plan count mismatch")
    require(memory.get("padding_excluded_from_scientific_counts") is True, "memory plan must exclude padding")
    require(memory.get("meaningful_lanes_per_cell") == 30 and memory.get("padding_lanes_per_cell") == 2, "memory lane counts mismatch")
    for key in ("memory_budget_bytes", "bytes_per_execution_cell", "resident_capacity_cells", "chunk_count", "last_chunk_cells"):
        require(integer(memory.get(key)) and memory[key] >= 0, f"memory plan {key} must be non-negative integer")
    budget = memory["memory_budget_bytes"]
    cell_size = memory["bytes_per_execution_cell"]
    requested = memory["requested_conformations"]
    require(0 < budget <= MAX_MEMORY_BUDGET_BYTES, "memory budget outside Phase 3C bound")
    require(cell_size == layout["cell_size_bytes"] and cell_size > 0, "memory plan cell size disagrees with layout")
    expected_capacity = budget // cell_size
    require(expected_capacity >= 1, "memory budget is too small for one execution cell")
    require(memory["resident_capacity_cells"] == expected_capacity, "resident capacity is not derivable from budget/cell size")
    expected_chunks = (requested + expected_capacity - 1) // expected_capacity
    require(1 <= expected_chunks <= MAX_CAMPAIGN_CHUNKS, "derived chunk count outside bounded domain")
    require(memory["chunk_count"] == expected_chunks, "memory plan chunk count is not derivable from budget")
    expected_last = requested - expected_capacity * (expected_chunks - 1)
    require(1 <= expected_last <= expected_capacity, "derived last chunk size invalid")
    require(memory["last_chunk_cells"] == expected_last, "memory plan last chunk is not derivable from budget")


def validate_chunks(chunks: Any, memory: dict[str, Any], correctness: dict[str, Any]) -> None:
    require(isinstance(chunks, list) and len(chunks) == memory["chunk_count"], "chunk list count mismatch")
    require(chunks, "chunk plan cannot be empty")
    expected_start = correctness["conformation_start"]
    total = 0
    for ordinal, chunk in enumerate(chunks):
        require(isinstance(chunk, dict), f"chunk {ordinal} must be an object")
        for key in ("ordinal", "start", "count", "end_exclusive"):
            require(integer(chunk.get(key)) and chunk[key] >= 0, f"chunk {ordinal} {key} must be non-negative integer")
        require(chunk["ordinal"] == ordinal, f"chunk {ordinal} ordinal mismatch")
        require(chunk["count"] > 0, f"chunk {ordinal} must have positive count")
        require(chunk["count"] <= memory["resident_capacity_cells"], f"chunk {ordinal} exceeds resident capacity")
        require(chunk["start"] == expected_start, f"chunk {ordinal} start breaks deterministic continuity")
        require(chunk["end_exclusive"] == chunk["start"] + chunk["count"], f"chunk {ordinal} end arithmetic mismatch")
        if ordinal < len(chunks) - 1:
            require(chunk["count"] == memory["resident_capacity_cells"], f"non-final chunk {ordinal} must fill resident capacity")
        else:
            require(chunk["count"] == memory["last_chunk_cells"], "final chunk size mismatch")
        expected_start = chunk["end_exclusive"]
        total += chunk["count"]
    require(total == correctness["conformation_count"], "chunk coverage count mismatch")
    require(expected_start == correctness["conformation_end_exclusive"], "chunk end mismatch")


def validate_environment(environment: dict[str, Any]) -> None:
    require(isinstance(environment, dict), "environment receipt must be an object")
    require(set(environment) == ENVIRONMENT_KEYS, f"environment receipt contains missing/extra fields: {sorted(set(environment) ^ ENVIRONMENT_KEYS)}")
    require(environment.get("schema") == "IGM-CAMPAIGN-ENVIRONMENT-V1", "bad environment schema")
    require(isinstance(environment.get("os_family"), str) and environment["os_family"], "environment os_family missing")
    require(isinstance(environment.get("architecture"), str) and environment["architecture"], "environment architecture missing")
    for key in ("rustc_version", "cargo_version"):
        require(environment[key] is None or (isinstance(environment[key], str) and environment[key]), f"environment {key} must be null or non-empty string")
    require(integer(environment.get("available_parallelism")) and environment["available_parallelism"] >= 1, "environment available_parallelism invalid")
    require(environment.get("hostname_included") is False, "hostname must not be included")
    require(environment.get("username_included") is False, "username must not be included")
    require(environment.get("raw_hardware_identifiers_included") is False, "raw hardware identifiers must not be included")


def validate_manifest(directory: Path, manifest: dict[str, Any], correctness: dict[str, Any], benchmark: dict[str, Any], graph: dict[str, Any], memory: dict[str, Any]) -> None:
    require(manifest.get("schema") == "IGM-CAMPAIGN-MANIFEST-V1", "bad campaign manifest schema")
    require(manifest.get("campaign_contract") == "IGM-EXEC-CAMPAIGN-V1", "bad manifest campaign contract")
    require(manifest.get("rejected") is False, "accepted manifest cannot be rejected")
    require(manifest.get("benchmark_identity_is_correctness_identity") is False, "benchmark/correctness identity boundary broken")
    require(manifest.get("correctness_result_sha256") == correctness["result_sha256"], "manifest correctness identity mismatch")
    require(manifest.get("model_profile_sha256") == correctness["model_profile_sha256"], "manifest model profile identity mismatch")
    require(manifest.get("optimization_profile_sha256") == correctness["optimization_profile_sha256"], "manifest optimization profile identity mismatch")
    require(manifest.get("graph_sha256") == graph["graph_sha256"] == correctness["graph_sha256"], "manifest graph identity mismatch")
    require(manifest.get("traversal_sha256") == graph["traversal_sha256"] == correctness["traversal_sha256"], "manifest traversal identity mismatch")
    require(manifest.get("requested_workers") == benchmark["requested_workers"], "manifest requested_workers mismatch")
    require(manifest.get("chunk_count") == memory["chunk_count"] == benchmark["chunk_count"], "manifest chunk count mismatch")
    require(sha256_text(manifest.get("manifest_sha256")), "manifest_sha256 must be lowercase SHA-256")

    artifacts = manifest.get("artifacts")
    require(isinstance(artifacts, list) and len(artifacts) == len(ARTIFACT_ROLES), "manifest artifact set malformed")
    expected_paths = [path for path, _ in ARTIFACT_ROLES]
    actual_paths = [artifact.get("path") if isinstance(artifact, dict) else None for artifact in artifacts]
    require(actual_paths == expected_paths, "manifest artifact order/path set differs from canonical campaign artifact set")

    hasher = hashlib.sha256()
    hasher.update(MANIFEST_DOMAIN)
    for value in (
        manifest["correctness_result_sha256"],
        manifest["model_profile_sha256"],
        manifest["optimization_profile_sha256"],
        manifest["graph_sha256"],
        manifest["traversal_sha256"],
    ):
        hasher.update(value.encode("utf-8"))
    hasher.update(u64(manifest["requested_workers"]))
    hasher.update(u64(manifest["chunk_count"]))

    for artifact, (expected_path, expected_role) in zip(artifacts, ARTIFACT_ROLES):
        require(isinstance(artifact, dict), "manifest artifact entry must be an object")
        require(set(artifact) == {"path", "sha256", "bytes", "role"}, f"manifest artifact {expected_path} fields malformed")
        require(artifact["path"] == expected_path, f"manifest artifact path mismatch: {expected_path}")
        require(artifact["role"] == expected_role, f"manifest artifact role mismatch: {expected_path}")
        require(sha256_text(artifact["sha256"]), f"manifest artifact hash invalid: {expected_path}")
        require(integer(artifact["bytes"]) and artifact["bytes"] > 0, f"manifest artifact byte count invalid: {expected_path}")
        path = directory / expected_path
        require(path.is_file(), f"manifest artifact missing: {expected_path}")
        require(path.stat().st_size == artifact["bytes"], f"manifest byte count mismatch: {expected_path}")
        require(sha256_file(path) == artifact["sha256"], f"manifest hash mismatch: {expected_path}")
        hasher.update(expected_path.encode("utf-8"))
        hasher.update(artifact["sha256"].encode("utf-8"))
        hasher.update(u64(artifact["bytes"]))
        hasher.update(expected_role.encode("utf-8"))

    require(manifest["manifest_sha256"] == hasher.hexdigest(), "manifest_sha256 does not match domain-separated manifest identity")


def validate_accepted(directory: Path) -> None:
    required_files = {path for path, _ in ARTIFACT_ROLES} | {"campaign-manifest.json", "SHA256SUMS"}
    present = {path.name for path in directory.iterdir() if path.is_file()}
    require(present == required_files, f"accepted campaign artifact set mismatch: missing={sorted(required_files - present)} extra={sorted(present - required_files)}")

    graph = load_json(directory / "execution-graph.json")
    require(isinstance(graph, dict), "execution graph receipt must be an object")
    validate_graph(graph)

    layout = load_json(directory / "memory-layout.json")
    require(isinstance(layout, dict), "memory layout receipt must be an object")
    validate_layout(layout)

    correctness = load_json(directory / "correctness-receipt.json")
    require(isinstance(correctness, dict), "correctness receipt must be an object")
    validate_correctness(correctness, graph)

    memory = load_json(directory / "memory-plan.json")
    require(isinstance(memory, dict), "memory plan must be an object")
    validate_memory(memory, correctness, layout)

    benchmark = load_json(directory / "benchmark-receipt.json")
    require(isinstance(benchmark, dict), "benchmark receipt must be an object")
    validate_benchmark(benchmark, memory)

    chunks = load_json(directory / "chunks.json")
    validate_chunks(chunks, memory, correctness)

    environment = load_json(directory / "environment.json")
    require(isinstance(environment, dict), "environment receipt must be an object")
    validate_environment(environment)

    manifest = load_json(directory / "campaign-manifest.json")
    require(isinstance(manifest, dict), "campaign manifest must be an object")
    validate_manifest(directory, manifest, correctness, benchmark, graph, memory)

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
