#!/usr/bin/env python3
"""Validate Phase 3C accepted/rejected campaign artifacts, including the explicit gate."""

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
OPTIMIZATION_CONTRACT = "IGM-PENTA-CRT-CPU-V1"
GRAPH_CONTRACT = "IGM-EXEC-GRAPH-C5-K2-C3-V1"
LAYOUT_CONTRACT = "IGM-WARP32-AOSOA-V1"
CAMPAIGN_CONTRACT = "IGM-EXEC-CAMPAIGN-V1"
GATE_CONTRACT = "IGM-PHASE3C-ACCEPTANCE-GATE-V1"
VERIFICATION_TOLERANCE = 1.0e-12
MAX_MEMORY_BUDGET_BYTES = 16 * 1024 * 1024 * 1024
MAX_CAMPAIGN_CHUNKS = 1_000_000
MAX_VERIFY_SAMPLES = 4096
MAX_WORKERS = 256

GRAPH_DOMAIN = b"IGM-EXEC-GRAPH-C5-K2-C3-V1\0"
TRAVERSAL_DOMAIN = b"IGM-EXEC-TRAVERSAL-RECEIPT-V1\0"
CORRECTNESS_DOMAIN = b"IGM-CAMPAIGN-CORRECTNESS-V1\0"
GATE_DOMAIN = b"IGM-PHASE3C-GATE-RECEIPT-V1\0"
MANIFEST_DOMAIN = b"IGM-CAMPAIGN-MANIFEST-V2\0"

CORRECTNESS_INCLUDED_FIELDS = [
    "optimization_contract",
    "numerical_profile",
    "graph_contract",
    "graph_sha256",
    "traversal_sha256",
    "layout_contract",
    "model_profile_sha256",
    "optimization_profile_sha256",
    "conformation_start",
    "conformation_count",
    "diagnostic_xor_fnv1a64",
    "min_pair_distance_squared",
    "max_pair_distance_squared",
    "domain_separator",
]
CORRECTNESS_EXCLUDED_FIELDS = [
    "requested_workers",
    "memory_budget_bytes",
    "resident_capacity_cells",
    "chunk_count",
    "elapsed_seconds",
    "conformations_per_second",
]

ARTIFACT_ROLES = [
    ("correctness-receipt.json", "correctness"),
    ("benchmark-receipt.json", "benchmark-observation"),
    ("execution-graph.json", "execution-graph-and-traversal"),
    ("memory-layout.json", "gpu-shaped-memory-contract"),
    ("memory-plan.json", "bounded-memory-plan"),
    ("environment.json", "privacy-safe-environment-observation"),
    ("chunks.json", "deterministic-chunk-plan"),
    ("phase3c-gate.json", "phase3c-acceptance-gate"),
]

ENVIRONMENT_KEYS = {
    "schema", "os_family", "architecture", "rustc_version", "cargo_version",
    "available_parallelism", "hostname_included", "username_included",
    "raw_hardware_identifiers_included",
}

GATE_KEYS = {
    "schema", "gate_contract", "campaign_contract", "validation_level",
    "model_profile_sha256", "optimization_profile_sha256", "optimization_contract",
    "numerical_profile", "graph_contract", "layout_contract", "conformation_start",
    "conformation_count", "conformation_end_exclusive", "correctness_result_sha256",
    "profile_identity_preserved", "algorithm_identity_preserved",
    "phase3b_residual_gate_passed", "finite_and_bounded", "declared_slice_preserved",
    "correctness_identity_recomputed", "worker_independent_correctness_identity",
    "chunk_independent_correctness_identity",
    "benchmark_timing_excluded_from_correctness_identity",
    "correctness_identity_included_fields", "correctness_identity_excluded_fields",
    "implementation_structures_biological_relationships_claimed",
    "validation_level_promoted_by_runtime", "biological_validity_claimed",
    "clinical_validity_claimed", "inv_bio_001", "inv_runtime_001", "accepted",
    "gate_identity_sha256",
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


def require(condition: bool, message: str) -> None:
    if not condition:
        raise CampaignError(message)


def reject_constant(value: str) -> None:
    raise CampaignError(f"non-standard JSON constant forbidden: {value}")


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, CampaignError) as exc:
        raise CampaignError(f"{path}: {exc}") from exc


def integer(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def finite_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def sha256_text(value: Any) -> bool:
    return isinstance(value, str) and len(value) == 64 and all(c in "0123456789abcdef" for c in value)


def u64(value: Any) -> bytes:
    require(integer(value) and 0 <= value < 2**64, "identity integer outside u64 domain")
    return value.to_bytes(8, "little", signed=False)


def f64_bits(value: Any) -> bytes:
    require(finite_number(value), "identity float must be finite")
    return struct.pack("<d", float(value))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(65536), b""):
            digest.update(block)
    return digest.hexdigest()


def validate_checksums(directory: Path) -> None:
    checksum_path = directory / "SHA256SUMS"
    require(checksum_path.is_file(), "SHA256SUMS missing")
    seen: set[str] = set()
    for line in checksum_path.read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        parts = line.split("  ", 1)
        require(len(parts) == 2, f"malformed SHA256SUMS line: {line!r}")
        digest, relative = parts
        require(sha256_text(digest), "invalid checksum digest")
        require(relative == Path(relative).name, f"checksum path must be local filename: {relative}")
        require(relative not in seen, f"duplicate checksum path: {relative}")
        seen.add(relative)
        path = directory / relative
        require(path.is_file(), f"checksummed artifact missing: {relative}")
        require(sha256_file(path) == digest, f"checksum mismatch: {relative}")
    expected = {p.name for p in directory.iterdir() if p.is_file() and p.name != "SHA256SUMS"}
    require(seen == expected, f"SHA256SUMS coverage mismatch: seen={sorted(seen)} expected={sorted(expected)}")


def validate_rejected(directory: Path) -> None:
    present = {p.name for p in directory.iterdir() if p.is_file()}
    require(present == {"rejected.json", "SHA256SUMS"}, "rejected campaign artifact set malformed")
    receipt = load_json(directory / "rejected.json")
    require(isinstance(receipt, dict), "rejection receipt must be object")
    require(receipt.get("schema") == "IGM-CAMPAIGN-REJECTION-V1", "unexpected rejection schema")
    require(receipt.get("campaign_contract") == CAMPAIGN_CONTRACT, "unexpected rejection campaign contract")
    require(receipt.get("preserved") is True, "rejection must be preserved")
    require(receipt.get("non_clinical") is True, "rejection must remain non-clinical")
    require(receipt.get("biological_validity_claimed") is False, "rejection cannot claim biological validity")
    require(receipt.get("inv_bio_001") == INV_BIO_001, "INV-BIO-001 missing from rejection")
    require(receipt.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from rejection")
    require(isinstance(receipt.get("stage"), str) and receipt["stage"], "rejection stage missing")
    require(isinstance(receipt.get("reason"), str) and receipt["reason"], "rejection reason missing")
    validate_checksums(directory)


def sequence_for(sector: int, arm: int, lane: int) -> int:
    return (6 * sector + 15 * arm + 10 * lane) % 30


def expected_node(sequence: int) -> tuple[int, int, int, int, list[tuple[str, int]]]:
    sector, arm, lane = sequence % 5, sequence % 2, sequence % 3
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
    require(graph.get("schema") == "IGM-EXEC-TRAVERSAL-RECEIPT-V1", "bad graph receipt schema")
    require(graph.get("graph_contract") == GRAPH_CONTRACT, "bad graph contract")
    require(graph.get("node_count") == 30 and graph.get("degree") == 5 and graph.get("edge_count_undirected") == 75, "bad graph topology summary")
    require(graph.get("biological_adjacency_claimed") is False, "execution graph cannot claim biological adjacency")
    require(graph.get("inv_runtime_001") == INV_RUNTIME_001, "INV-RUNTIME-001 missing from graph")
    nodes = graph.get("nodes")
    require(isinstance(nodes, list) and len(nodes) == 30, "graph node list malformed")
    gh = hashlib.sha256(); gh.update(GRAPH_DOMAIN)
    th = hashlib.sha256(); th.update(TRAVERSAL_DOMAIN)
    for sequence, node in enumerate(nodes):
        require(isinstance(node, dict), f"node {sequence} must be object")
        sector, arm, lane, storage, expected_neighbors = expected_node(sequence)
        require(node.get("sequence") == sequence, f"node {sequence} sequence mismatch")
        require((node.get("sector"), node.get("arm"), node.get("lane"), node.get("storage_index")) == (sector, arm, lane, storage), f"node {sequence} address mismatch")
        neighbors = node.get("neighbors")
        require(isinstance(neighbors, list) and len(neighbors) == 5, f"node {sequence} neighbor list malformed")
        actual: list[tuple[str, int]] = []
        for entry in neighbors:
            require(isinstance(entry, dict), "neighbor must be object")
            kind, target = entry.get("kind"), entry.get("sequence")
            require(kind in EDGE_KIND_IDS and integer(target) and 0 <= target < 30, "invalid graph edge")
            require(target != sequence, "execution graph self-edge forbidden")
            actual.append((kind, target))
        require(actual == expected_neighbors, f"node {sequence} does not match exact C5 x K2 x C3 topology")
        prefix = bytes([sequence, sector, arm, lane, storage])
        gh.update(prefix); th.update(prefix)
        for kind, target in actual:
            gh.update(bytes([EDGE_KIND_IDS[kind], target]))
    require(graph.get("graph_sha256") == gh.hexdigest(), "graph SHA mismatch")
    require(graph.get("traversal_sha256") == th.hexdigest(), "traversal SHA mismatch")


def validate_layout(layout: dict[str, Any]) -> None:
    require(layout.get("schema") == "IGM-MEMORY-LAYOUT-RECEIPT-V1", "bad layout schema")
    require(layout.get("layout_contract") == LAYOUT_CONTRACT, "bad layout contract")
    require(layout.get("warp_width") == 32 and layout.get("meaningful_lanes") == 30 and layout.get("padding_lanes") == 2, "30+2 lane contract broken")
    require(layout.get("active_lane_count_observed") == 30 and layout.get("padding_lane_count_observed") == 2, "observed lane counts broken")
    require(layout.get("padding_lanes_semantic") is False and layout.get("scientific_count_includes_padding") is False, "padding lanes must remain non-semantic")
    require(layout.get("cell_alignment_bytes") == 128, "cell alignment must be 128")
    require(integer(layout.get("cell_size_bytes")) and layout["cell_size_bytes"] > 0 and layout["cell_size_bytes"] % 128 == 0, "cell size invalid")


def validate_correctness(c: dict[str, Any], graph: dict[str, Any]) -> None:
    require(c.get("schema") == "IGM-CAMPAIGN-CORRECTNESS-RECEIPT-V1", "bad correctness schema")
    require(c.get("campaign_contract") == CAMPAIGN_CONTRACT, "bad correctness campaign contract")
    require(c.get("optimization_contract") == OPTIMIZATION_CONTRACT, "bad optimization contract")
    require(c.get("numerical_profile") == NUMERICAL_PROFILE, "unsupported numerical profile")
    require(c.get("graph_contract") == GRAPH_CONTRACT and c.get("layout_contract") == LAYOUT_CONTRACT, "correctness graph/layout contract mismatch")
    require(c.get("graph_sha256") == graph.get("graph_sha256") and c.get("traversal_sha256") == graph.get("traversal_sha256"), "correctness graph identity mismatch")
    for key in ("model_profile_sha256", "optimization_profile_sha256", "result_sha256"):
        require(sha256_text(c.get(key)), f"{key} must be SHA-256")
    diag = c.get("diagnostic_xor_fnv1a64")
    require(isinstance(diag, str) and len(diag) == 16 and all(ch in "0123456789abcdef" for ch in diag), "bad correctness diagnostic")
    for key in ("conformation_start", "conformation_count", "conformation_end_exclusive", "logical_pair_checks", "structured_distance_evaluations", "exact_z_residual_corrections", "verification_samples"):
        require(integer(c.get(key)) and c[key] >= 0, f"{key} must be non-negative integer")
    require(c["conformation_count"] >= 1 and c["conformation_end_exclusive"] == c["conformation_start"] + c["conformation_count"], "correctness slice arithmetic mismatch")
    require(c["logical_pair_checks"] == c["conformation_count"] * 120, "pair accounting mismatch")
    require(c["structured_distance_evaluations"] == c["conformation_count"] * 60, "structured accounting mismatch")
    require(c["exact_z_residual_corrections"] == c["conformation_count"] * 105, "Z correction accounting mismatch")
    require(1 <= c["verification_samples"] <= MAX_VERIFY_SAMPLES, "verification sample count outside bound")
    for key in ("min_pair_distance_squared", "max_pair_distance_squared", "verification_max_geometry_residual", "verification_max_pair_residual", "verification_tolerance"):
        require(finite_number(c.get(key)), f"{key} must be finite")
    require(0 <= c["min_pair_distance_squared"] <= c["max_pair_distance_squared"], "distance extrema invalid")
    require(c.get("verification_tolerance") == VERIFICATION_TOLERANCE, "verification tolerance not pinned")
    require(c.get("verification_accepted") is True and c["verification_max_geometry_residual"] <= VERIFICATION_TOLERANCE and c["verification_max_pair_residual"] <= VERIFICATION_TOLERANCE, "Phase 3B residual gate not passed")
    require(c.get("result_identity_worker_independent") is True and c.get("result_identity_chunk_independent") is True, "correctness independence contract broken")
    require(c.get("biological_validity_claimed") is False and c.get("clinical_validity_claimed") is False, "runtime cannot promote validation")
    require(c.get("inv_bio_001") == INV_BIO_001 and c.get("inv_runtime_001") == INV_RUNTIME_001, "correctness invariants missing")

    h = hashlib.sha256(); h.update(CORRECTNESS_DOMAIN)
    for key in ("optimization_contract", "numerical_profile", "graph_contract", "graph_sha256", "traversal_sha256", "layout_contract", "model_profile_sha256", "optimization_profile_sha256"):
        h.update(c[key].encode())
    h.update(u64(c["conformation_start"])); h.update(u64(c["conformation_count"])); h.update(int(diag, 16).to_bytes(8, "little"))
    h.update(f64_bits(c["min_pair_distance_squared"])); h.update(f64_bits(c["max_pair_distance_squared"]))
    require(c["result_sha256"] == h.hexdigest(), "correctness result identity does not recompute")


def validate_memory(memory: dict[str, Any], correctness: dict[str, Any], layout: dict[str, Any]) -> None:
    require(memory.get("schema") == "IGM-MEMORY-PLAN-V1" and memory.get("layout_contract") == LAYOUT_CONTRACT, "bad memory plan contract")
    require(memory.get("requested_conformations") == correctness.get("conformation_count"), "memory/correctness count mismatch")
    require(memory.get("meaningful_lanes_per_cell") == 30 and memory.get("padding_lanes_per_cell") == 2 and memory.get("padding_excluded_from_scientific_counts") is True, "memory lane semantics broken")
    for key in ("memory_budget_bytes", "bytes_per_execution_cell", "resident_capacity_cells", "chunk_count", "last_chunk_cells"):
        require(integer(memory.get(key)) and memory[key] >= 0, f"memory {key} invalid")
    budget, cell, requested = memory["memory_budget_bytes"], memory["bytes_per_execution_cell"], memory["requested_conformations"]
    require(0 < budget <= MAX_MEMORY_BUDGET_BYTES and cell == layout["cell_size_bytes"] and cell > 0, "memory budget/cell invalid")
    capacity = budget // cell
    require(capacity >= 1 and memory["resident_capacity_cells"] == capacity, "resident capacity not derivable")
    chunk_count = (requested + capacity - 1) // capacity
    require(1 <= chunk_count <= MAX_CAMPAIGN_CHUNKS and memory["chunk_count"] == chunk_count, "chunk count not derivable")
    last = requested - capacity * (chunk_count - 1)
    require(1 <= last <= capacity and memory["last_chunk_cells"] == last, "last chunk size not derivable")


def validate_chunks(chunks: Any, memory: dict[str, Any], correctness: dict[str, Any]) -> None:
    require(isinstance(chunks, list) and len(chunks) == memory["chunk_count"] and chunks, "chunk list malformed")
    expected_start, total = correctness["conformation_start"], 0
    for ordinal, chunk in enumerate(chunks):
        require(isinstance(chunk, dict), f"chunk {ordinal} must be object")
        for key in ("ordinal", "start", "count", "end_exclusive"):
            require(integer(chunk.get(key)) and chunk[key] >= 0, f"chunk {ordinal} {key} invalid")
        require(chunk["ordinal"] == ordinal and chunk["count"] > 0 and chunk["count"] <= memory["resident_capacity_cells"], f"chunk {ordinal} bounds invalid")
        require(chunk["start"] == expected_start and chunk["end_exclusive"] == chunk["start"] + chunk["count"], f"chunk {ordinal} continuity/arithmetic invalid")
        expected_count = memory["last_chunk_cells"] if ordinal == len(chunks) - 1 else memory["resident_capacity_cells"]
        require(chunk["count"] == expected_count, f"chunk {ordinal} size does not match deterministic plan")
        expected_start = chunk["end_exclusive"]; total += chunk["count"]
    require(total == correctness["conformation_count"] and expected_start == correctness["conformation_end_exclusive"], "chunk coverage mismatch")


def validate_benchmark(b: dict[str, Any], memory: dict[str, Any]) -> None:
    require(b.get("schema") == "IGM-CAMPAIGN-BENCHMARK-RECEIPT-V1" and b.get("campaign_contract") == CAMPAIGN_CONTRACT, "bad benchmark contract")
    require(b.get("identity_bearing_correctness") is False and b.get("performance_claim") is False, "benchmark/correctness boundary broken")
    require(finite_number(b.get("elapsed_seconds")) and b["elapsed_seconds"] >= 0 and finite_number(b.get("conformations_per_second")) and b["conformations_per_second"] >= 0, "benchmark timing must be finite non-negative")
    require(integer(b.get("requested_workers")) and 1 <= b["requested_workers"] <= MAX_WORKERS, "worker count outside bound")
    require(b.get("memory_budget_bytes") == memory["memory_budget_bytes"] and b.get("resident_capacity_cells") == memory["resident_capacity_cells"] and b.get("chunk_count") == memory["chunk_count"], "benchmark plan fields mismatch")


def validate_environment(e: dict[str, Any]) -> None:
    require(set(e) == ENVIRONMENT_KEYS, f"environment contains missing/extra fields: {sorted(set(e) ^ ENVIRONMENT_KEYS)}")
    require(e.get("schema") == "IGM-CAMPAIGN-ENVIRONMENT-V1", "bad environment schema")
    require(isinstance(e.get("os_family"), str) and e["os_family"] and isinstance(e.get("architecture"), str) and e["architecture"], "environment platform fields missing")
    for key in ("rustc_version", "cargo_version"):
        value = e.get(key)
        require(value is None or (isinstance(value, str) and bool(value)), f"environment {key} must be null or a non-empty string")
    require(integer(e.get("available_parallelism")) and e["available_parallelism"] >= 1, "environment parallelism invalid")
    require(e.get("hostname_included") is False and e.get("username_included") is False and e.get("raw_hardware_identifiers_included") is False, "raw environment identifiers forbidden")


def gate_identity(g: dict[str, Any]) -> str:
    h = hashlib.sha256(); h.update(GATE_DOMAIN)
    for key in ("gate_contract", "campaign_contract", "validation_level", "model_profile_sha256", "optimization_profile_sha256", "optimization_contract", "numerical_profile", "graph_contract", "layout_contract", "correctness_result_sha256", "inv_bio_001", "inv_runtime_001"):
        h.update(g[key].encode())
    h.update(u64(g["conformation_start"])); h.update(u64(g["conformation_count"])); h.update(u64(g["conformation_end_exclusive"]))
    for key in ("profile_identity_preserved", "algorithm_identity_preserved", "phase3b_residual_gate_passed", "finite_and_bounded", "declared_slice_preserved", "correctness_identity_recomputed", "worker_independent_correctness_identity", "chunk_independent_correctness_identity", "benchmark_timing_excluded_from_correctness_identity", "implementation_structures_biological_relationships_claimed", "validation_level_promoted_by_runtime", "biological_validity_claimed", "clinical_validity_claimed", "accepted"):
        require(isinstance(g.get(key), bool), f"gate {key} must be boolean")
        h.update(bytes([1 if g[key] else 0]))
    for field in g["correctness_identity_included_fields"]:
        h.update(field.encode()); h.update(b"\0")
    for field in g["correctness_identity_excluded_fields"]:
        h.update(field.encode()); h.update(b"\0")
    return h.hexdigest()


def validate_gate(g: dict[str, Any], correctness: dict[str, Any], benchmark: dict[str, Any], graph: dict[str, Any], layout: dict[str, Any], memory: dict[str, Any], chunks: list[dict[str, Any]]) -> None:
    require(set(g) == GATE_KEYS, f"gate receipt contains missing/extra fields: {sorted(set(g) ^ GATE_KEYS)}")
    require(g.get("schema") == "IGM-PHASE3C-GATE-RECEIPT-V1" and g.get("gate_contract") == GATE_CONTRACT and g.get("campaign_contract") == CAMPAIGN_CONTRACT, "bad Phase 3C gate contract")
    require(g.get("validation_level") == "V0" and g.get("validation_level_promoted_by_runtime") is False, "runtime validation-level promotion forbidden")
    require(g.get("model_profile_sha256") == correctness["model_profile_sha256"] and g.get("optimization_profile_sha256") == correctness["optimization_profile_sha256"], "gate profile identity mismatch")
    require(g.get("optimization_contract") == OPTIMIZATION_CONTRACT and g.get("numerical_profile") == NUMERICAL_PROFILE and g.get("graph_contract") == GRAPH_CONTRACT and g.get("layout_contract") == LAYOUT_CONTRACT, "gate algorithm identity mismatch")
    require((g.get("conformation_start"), g.get("conformation_count"), g.get("conformation_end_exclusive")) == (correctness["conformation_start"], correctness["conformation_count"], correctness["conformation_end_exclusive"]), "gate slice identity mismatch")
    require(g.get("correctness_result_sha256") == correctness["result_sha256"], "gate correctness identity mismatch")
    for key in ("profile_identity_preserved", "algorithm_identity_preserved", "phase3b_residual_gate_passed", "finite_and_bounded", "declared_slice_preserved", "correctness_identity_recomputed", "worker_independent_correctness_identity", "chunk_independent_correctness_identity", "benchmark_timing_excluded_from_correctness_identity", "accepted"):
        require(g.get(key) is True, f"Phase 3C gate requirement failed: {key}")
    require(g.get("implementation_structures_biological_relationships_claimed") is False and g.get("biological_validity_claimed") is False and g.get("clinical_validity_claimed") is False, "implementation structures cannot create biological/clinical claims")
    require(g.get("inv_bio_001") == INV_BIO_001 and g.get("inv_runtime_001") == INV_RUNTIME_001, "gate invariants missing")
    require(g.get("correctness_identity_included_fields") == CORRECTNESS_INCLUDED_FIELDS, "gate included correctness identity field contract changed")
    require(g.get("correctness_identity_excluded_fields") == CORRECTNESS_EXCLUDED_FIELDS, "gate excluded correctness identity field contract changed")
    require(g.get("benchmark_timing_excluded_from_correctness_identity") is True and benchmark["identity_bearing_correctness"] is False, "benchmark timing entered correctness identity")
    require(graph["biological_adjacency_claimed"] is False and layout["padding_lanes_semantic"] is False and layout["scientific_count_includes_padding"] is False, "implementation adjacency acquired biological semantics")
    require(memory["requested_conformations"] == correctness["conformation_count"] and sum(c["count"] for c in chunks) == correctness["conformation_count"], "gate bounded slice coverage mismatch")
    require(sha256_text(g.get("gate_identity_sha256")) and g["gate_identity_sha256"] == gate_identity(g), "gate identity does not recompute")


def validate_manifest(directory: Path, m: dict[str, Any], c: dict[str, Any], b: dict[str, Any], graph: dict[str, Any], memory: dict[str, Any], gate: dict[str, Any]) -> None:
    require(m.get("schema") == "IGM-CAMPAIGN-MANIFEST-V2" and m.get("campaign_contract") == CAMPAIGN_CONTRACT, "bad manifest V2 contract")
    require(m.get("gate_contract") == GATE_CONTRACT and m.get("gate_identity_sha256") == gate["gate_identity_sha256"], "manifest gate identity mismatch")
    require(m.get("phase3c_gate_artifact_sha256") == sha256_file(directory / "phase3c-gate.json"), "manifest gate artifact hash mismatch")
    require(m.get("validation_level") == "V0" and m.get("validation_level_promoted_by_runtime") is False, "manifest validation-level promotion forbidden")
    require(m.get("correctness_result_sha256") == c["result_sha256"] and m.get("model_profile_sha256") == c["model_profile_sha256"] and m.get("optimization_profile_sha256") == c["optimization_profile_sha256"], "manifest profile/correctness identity mismatch")
    require(m.get("optimization_contract") == OPTIMIZATION_CONTRACT and m.get("numerical_profile") == NUMERICAL_PROFILE and m.get("graph_contract") == GRAPH_CONTRACT and m.get("layout_contract") == LAYOUT_CONTRACT, "manifest algorithm identity mismatch")
    require(m.get("graph_sha256") == graph["graph_sha256"] == c["graph_sha256"] and m.get("traversal_sha256") == graph["traversal_sha256"] == c["traversal_sha256"], "manifest graph identity mismatch")
    require(m.get("requested_workers") == b["requested_workers"] and m.get("chunk_count") == memory["chunk_count"], "manifest execution plan mismatch")
    require(m.get("benchmark_identity_is_correctness_identity") is False and m.get("rejected") is False, "manifest acceptance/benchmark boundary broken")
    artifacts = m.get("artifacts")
    require(isinstance(artifacts, list) and len(artifacts) == len(ARTIFACT_ROLES), "manifest artifact set malformed")
    h = hashlib.sha256(); h.update(MANIFEST_DOMAIN)
    for key in ("campaign_contract", "gate_contract", "gate_identity_sha256", "phase3c_gate_artifact_sha256", "validation_level", "correctness_result_sha256", "model_profile_sha256", "optimization_profile_sha256", "optimization_contract", "numerical_profile", "graph_contract", "layout_contract", "graph_sha256", "traversal_sha256"):
        h.update(m[key].encode())
    h.update(bytes([1 if m["validation_level_promoted_by_runtime"] else 0])); h.update(u64(m["requested_workers"])); h.update(u64(m["chunk_count"])); h.update(bytes([1 if m["benchmark_identity_is_correctness_identity"] else 0])); h.update(bytes([1 if m["rejected"] else 0]))
    for artifact, (expected_path, expected_role) in zip(artifacts, ARTIFACT_ROLES):
        require(isinstance(artifact, dict) and set(artifact) == {"path", "sha256", "bytes", "role"}, f"manifest artifact {expected_path} malformed")
        require(artifact["path"] == expected_path and artifact["role"] == expected_role and sha256_text(artifact["sha256"]) and integer(artifact["bytes"]) and artifact["bytes"] > 0, f"manifest artifact {expected_path} identity malformed")
        path = directory / expected_path
        require(path.is_file() and path.stat().st_size == artifact["bytes"] and sha256_file(path) == artifact["sha256"], f"manifest artifact {expected_path} does not match file")
        h.update(expected_path.encode()); h.update(artifact["sha256"].encode()); h.update(u64(artifact["bytes"])); h.update(expected_role.encode())
    require(sha256_text(m.get("manifest_sha256")) and m["manifest_sha256"] == h.hexdigest(), "manifest identity does not recompute")


def validate_accepted(directory: Path) -> None:
    required = {p for p, _ in ARTIFACT_ROLES} | {"campaign-manifest.json", "SHA256SUMS"}
    present = {p.name for p in directory.iterdir() if p.is_file()}
    require(present == required, f"accepted campaign artifact set mismatch: missing={sorted(required-present)} extra={sorted(present-required)}")
    graph = load_json(directory / "execution-graph.json"); require(isinstance(graph, dict), "graph must be object"); validate_graph(graph)
    layout = load_json(directory / "memory-layout.json"); require(isinstance(layout, dict), "layout must be object"); validate_layout(layout)
    correctness = load_json(directory / "correctness-receipt.json"); require(isinstance(correctness, dict), "correctness must be object"); validate_correctness(correctness, graph)
    memory = load_json(directory / "memory-plan.json"); require(isinstance(memory, dict), "memory must be object"); validate_memory(memory, correctness, layout)
    chunks = load_json(directory / "chunks.json"); validate_chunks(chunks, memory, correctness)
    benchmark = load_json(directory / "benchmark-receipt.json"); require(isinstance(benchmark, dict), "benchmark must be object"); validate_benchmark(benchmark, memory)
    environment = load_json(directory / "environment.json"); require(isinstance(environment, dict), "environment must be object"); validate_environment(environment)
    gate = load_json(directory / "phase3c-gate.json"); require(isinstance(gate, dict), "gate must be object"); validate_gate(gate, correctness, benchmark, graph, layout, memory, chunks)
    manifest = load_json(directory / "campaign-manifest.json"); require(isinstance(manifest, dict), "manifest must be object"); validate_manifest(directory, manifest, correctness, benchmark, graph, memory, gate)
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
    print(f"OK: {argv[0]} passed IGM Phase 3C acceptance-gate validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
