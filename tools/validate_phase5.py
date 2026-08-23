#!/usr/bin/env python3
"""Fail-closed validation for IGM Phase 5 representation contracts and bundles."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PHASE5_GATE = "A representation earns scientific interpretation only from explicit evidence and validation"
CONTRACT = "IGM-PHASE5-REPRESENTATION-V1"
CONFIG_SCHEMA = "IGM-PHASE5-REPRESENTATION-CONFIG-V1"
GATE_CONTRACT = "IGM-PHASE5-REPRESENTATION-GATE-V1"
BUNDLE_DOMAIN = b"IGM-PHASE5-REPRESENTATION-V1\0"
GATE_DOMAIN = b"IGM-PHASE5-REPRESENTATION-GATE-V1\0"
GRAPH_CONTRACTS = {
    "model_graph": "IGM-MODEL-GRAPH-V1",
    "execution_graph": "IGM-EXEC-GRAPH-C5-K2-C3-V1",
    "tensor_factor_graph": "IGM-TENSOR-FACTOR-GRAPH-V1",
    "visualization_graph": "IGM-VISUALIZATION-GRAPH-V1",
}
GRAPH_OBJECTS = {
    "model_graph": ("IGM-MODEL-GRAPH-V1", "model-graph", "model"),
    "provenance_graph": ("IGM-PROVENANCE-GRAPH-V1", "provenance-graph", "provenance"),
    "visualization_graph": (
        "IGM-VISUALIZATION-GRAPH-V1",
        "visualization-graph",
        "visualization",
    ),
}
ABS_TOL = 1.0e-12


def fail(message: str) -> None:
    print(f"FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def reject_constant(value: str) -> None:
    raise ValueError(f"non-standard JSON constant forbidden: {value}")


def load(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        fail(f"{path}: {exc}")


def finite(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def close(left: float, right: float, tolerance: float = ABS_TOL) -> bool:
    return math.isclose(float(left), float(right), rel_tol=0.0, abs_tol=tolerance)


def _canonical_number(value: int | float) -> str:
    if isinstance(value, bool) or not finite(value):
        raise ValueError("canonical JSON requires finite numeric values")
    if isinstance(value, int):
        return str(value)
    text = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(",", ":"))
    # serde_json uses Ryu's compact exponent spelling. Python pads small
    # exponents and emits an explicit plus sign, so normalize those spelling
    # differences while preserving the same IEEE-754 value.
    text = text.replace("e+", "e")
    text = re.sub(r"e(-?)0+([0-9]+)$", r"e\1\2", text)
    return text


def canonical_json(value: Any) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        return _canonical_number(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if isinstance(value, list):
        return "[" + ",".join(canonical_json(item) for item in value) + "]"
    if isinstance(value, dict):
        parts = []
        for key in sorted(value):
            if not isinstance(key, str):
                raise ValueError("canonical JSON object keys must be strings")
            parts.append(
                json.dumps(key, ensure_ascii=False, separators=(",", ":"))
                + ":"
                + canonical_json(value[key])
            )
        return "{" + ",".join(parts) + "}"
    raise ValueError(f"unsupported canonical JSON value: {type(value).__name__}")


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def compute_bundle_identity(bundle: dict[str, Any]) -> str:
    blanked = copy.deepcopy(bundle)
    blanked["bundle_sha256"] = ""
    canonical = canonical_json(blanked).encode("utf-8")
    return sha256_hex(BUNDLE_DOMAIN + canonical)


def compute_gate_identity(gate: dict[str, Any]) -> str:
    blanked = copy.deepcopy(gate)
    blanked["gate_identity_sha256"] = ""
    canonical = canonical_json(blanked).encode("utf-8")
    return sha256_hex(GATE_DOMAIN + canonical)


def canonical_file_sha256(path: Path) -> str:
    value = load(path)
    try:
        canonical = canonical_json(value).encode("utf-8")
    except ValueError as exc:
        fail(f"{path}: {exc}")
    return sha256_hex(canonical)


def valid_hash(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(ch in "0123456789abcdef" for ch in value)
    )


def validate_repository_contract() -> None:
    required = [
        "docs/PHASE5_REPRESENTATIONS.md",
        "runtime/rust/src/phase5.rs",
        "runtime/rust/src/phase5_v2.rs",
        "runtime/rust/src/representation_main.rs",
        "runtime/rust/src/lib_v6.rs",
        "runtime/profiles/igm-phase5-v0.json",
        "schemas/phase5-representation-config.schema.json",
        "schemas/phase5-representation-bundle.schema.json",
    ]
    for relative in required:
        path = ROOT / relative
        if not path.is_file() or path.stat().st_size == 0:
            fail(f"missing/empty Phase 5 file: {relative}")

    doc = (ROOT / "docs/PHASE5_REPRESENTATIONS.md").read_text(encoding="utf-8")
    for phrase in (
        CONTRACT,
        "IGM-NUMERICAL-ARRAY-PROJECTION-V1",
        "IGM-DECLARED-TENSOR-V1",
        "IGM-MODEL-GRAPH-V1",
        "IGM-MODEL-HYPERGRAPH-V1",
        "IGM-PROVENANCE-GRAPH-V1",
        "IGM-TENSOR-FACTOR-GRAPH-V1",
        "IGM-VISUALIZATION-GRAPH-V1",
        "IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1",
        "IGM-ENSEMBLE-STATISTICS-V1",
        "IGM-COMPUTATIONAL-UNCERTAINTY-V1",
        "IGM-TENSOR-NETWORK-ASSESSMENT-V1",
        "IGM-VORTEX-INSPIRED-PROJECTION-V1",
        GATE_CONTRACT,
        PHASE5_GATE,
        "pairwise_expansion_performed = false",
        "biological_ensemble_claimed = false",
        "evidence_strength_promoted = false",
        "material_reduction = false",
        "admitted = false",
        "biological_ontology_claimed = false",
    ):
        if phrase not in doc:
            fail(f"Phase 5 documentation missing required contract/boundary text: {phrase!r}")

    config = load(ROOT / "runtime/profiles/igm-phase5-v0.json")
    if config.get("schema") != CONFIG_SCHEMA:
        fail("Phase 5 config schema identity mismatch")
    if config.get("profile_id") != "IGM-PHASE5-V0-REPRESENTATION":
        fail("Phase 5 config profile identity mismatch")
    if config.get("validation_level") != "V0":
        fail("Phase 5 reference representation must remain V0")
    for key in ("contact_cutoff", "accessibility_clearance"):
        threshold = config.get(key)
        if not isinstance(threshold, dict):
            fail(f"Phase 5 config missing {key}")
        if not finite(threshold.get("value")) or threshold["value"] <= 0:
            fail(f"Phase 5 {key} must be positive finite")
        if (
            threshold.get("status") != "assumed"
            or threshold.get("scientific_interpretation_claimed") is not False
        ):
            fail(f"Phase 5 {key} must remain an explicit non-scientific assumption")

    ensemble = config.get("ensemble", {})
    indices = ensemble.get("indices")
    if not isinstance(indices, list) or not indices or len(indices) != len(set(indices)):
        fail("Phase 5 ensemble requires unique explicit indices")
    if ensemble.get("sampling") != "explicit-index-set":
        fail("Phase 5 ensemble sampling must be explicit-index-set")
    if ensemble.get("population_variance") is not True:
        fail("Phase 5 ensemble must explicitly choose population variance")
    if ensemble.get("biological_ensemble_claimed") is not False:
        fail("Phase 5 ensemble may not claim a biological ensemble")

    execution_path = ROOT / ensemble.get("execution_profile_path", "")
    if not execution_path.is_file():
        fail("Phase 5 configured execution profile is missing")
    execution = load(execution_path)
    if execution.get("profile_id") != "IGM-PENTA-CRT-SYMMETRIC-V0":
        fail("Phase 5 configured execution profile identity mismatch")

    tensor_network = config.get("tensor_network", {})
    if tensor_network.get("require_exact_reconstruction") is not True:
        fail("tensor-network assessment must require exact reconstruction")
    if tensor_network.get("require_material_reduction") is not True:
        fail("tensor-network assessment must require material reduction")
    if tensor_network.get("performance_claim") is not False:
        fail("tensor-network assessment may not make a performance claim")

    claims = config.get("claims", {})
    for key in (
        "scientific_interpretation_claimed",
        "biological_validity_claimed",
        "clinical_validity_claimed",
        "performance_claim",
    ):
        if claims.get(key) is not False:
            fail(f"Phase 5 config must set {key}=false")

    config_schema = load(ROOT / "schemas/phase5-representation-config.schema.json")
    if config_schema.get("properties", {}).get("schema", {}).get("const") != CONFIG_SCHEMA:
        fail("Phase 5 config JSON Schema contract mismatch")
    bundle_schema = load(ROOT / "schemas/phase5-representation-bundle.schema.json")
    props = bundle_schema.get("properties", {})
    if props.get("representation_contract", {}).get("const") != CONTRACT:
        fail("Phase 5 bundle JSON Schema contract mismatch")
    if props.get("phase5_gate", {}).get("const") != PHASE5_GATE:
        fail("Phase 5 bundle schema must pin the representation gate")
    for key in (
        "scientific_interpretation_claimed",
        "biological_validity_claimed",
        "clinical_validity_claimed",
        "performance_claim",
    ):
        if props.get(key, {}).get("const") is not False:
            fail(f"Phase 5 bundle schema must pin {key}=false")

    defs = bundle_schema.get("$defs", {})
    for name in (
        "positionArray",
        "pairDistanceArray",
        "declaredTensor",
        "modelGraph",
        "modelHypergraph",
        "provenanceGraph",
        "tensorFactorGraph",
        "visualizationGraph",
        "graphSeparation",
        "observables",
        "ensembleStatistics",
        "uncertainties",
        "tensorNetworkAssessment",
        "vortexProjection",
        "gate",
    ):
        node = defs.get(name)
        if not isinstance(node, dict) or node.get("additionalProperties") is not False:
            fail(f"Phase 5 nested bundle schema must fail closed for {name}")
    ensemble_schema = defs.get("ensembleStatistics", {}).get("properties", {})
    if "source_execution_profile_id" not in ensemble_schema or "source_execution_profile_sha256" not in ensemble_schema:
        fail("Phase 5 ensemble schema must retain exact execution-profile identity")


def validate_graph(graph: Any, label: str, contract: str, name: str, namespace: str) -> set[str]:
    if not isinstance(graph, dict):
        fail(f"{label} must be an object")
    if graph.get("contract") != contract or graph.get("name") != name or graph.get("namespace") != namespace:
        fail(f"{label} contract/name/namespace mismatch")
    if graph.get("scientific_interpretation_claimed") is not False:
        fail(f"{label} may not claim scientific interpretation")
    nodes = graph.get("nodes")
    edges = graph.get("edges")
    if not isinstance(nodes, list) or not isinstance(edges, list):
        fail(f"{label} requires node and edge arrays")
    node_ids: set[str] = set()
    for node in nodes:
        if not isinstance(node, dict):
            fail(f"{label} node must be object")
        node_id = node.get("id")
        if not isinstance(node_id, str) or not node_id or node_id in node_ids:
            fail(f"{label} requires unique non-empty node ids")
        node_ids.add(node_id)
    for edge in edges:
        if not isinstance(edge, dict):
            fail(f"{label} edge must be object")
        if edge.get("source") not in node_ids or edge.get("target") not in node_ids:
            fail(f"{label} edge endpoint does not resolve in its own namespace")
        if not isinstance(edge.get("relationship_type"), str) or not edge["relationship_type"]:
            fail(f"{label} edge requires relationship_type")
        if edge.get("biological_relationship_claimed") is not False:
            fail(f"{label} edge may not promote a representation edge to biology")
    return node_ids


def extract_reference_geometry(arrays: list[dict[str, Any]]) -> tuple[list[str], list[tuple[float, float, float]], list[float]]:
    if len(arrays) != 2:
        fail("reference Phase 5 bundle requires exactly two explicit numerical arrays")
    by_name: dict[str, dict[str, Any]] = {}
    for array in arrays:
        name = array.get("name")
        if not isinstance(name, str) or name in by_name:
            fail("Phase 5 numerical arrays require unique names")
        by_name[name] = array
    if set(by_name) != {"cartesian-position-array", "pair-distance-squared-array"}:
        fail("reference Phase 5 bundle requires position and pair-distance arrays")

    positions_array = by_name["cartesian-position-array"]
    pair_array = by_name["pair-distance-squared-array"]
    if positions_array.get("shape") != [16, 3] or pair_array.get("shape") != [16, 16]:
        fail("reference Phase 5 numerical-array shapes must be [16,3] and [16,16]")
    component_ids = positions_array.get("component_ids")
    if (
        not isinstance(component_ids, list)
        or len(component_ids) != 16
        or len(set(component_ids)) != 16
        or any(not isinstance(item, str) or not item for item in component_ids)
    ):
        fail("position array requires 16 unique component ids")
    if pair_array.get("component_ids") != component_ids:
        fail("pair-distance array component ordering must match the position array")

    position_data = positions_array.get("data")
    pair_data = pair_array.get("data")
    if not isinstance(position_data, list) or len(position_data) != 48 or any(not finite(v) for v in position_data):
        fail("position array requires exactly 48 finite f64 values")
    if not isinstance(pair_data, list) or len(pair_data) != 256 or any(not finite(v) for v in pair_data):
        fail("pair-distance array requires exactly 256 finite f64 values")

    positions = [
        (float(position_data[i * 3]), float(position_data[i * 3 + 1]), float(position_data[i * 3 + 2]))
        for i in range(16)
    ]
    for left in range(16):
        for right in range(16):
            dx = positions[left][0] - positions[right][0]
            dy = positions[left][1] - positions[right][1]
            dz = positions[left][2] - positions[right][2]
            expected = dx * dx + dy * dy + dz * dz
            actual = float(pair_data[left * 16 + right])
            if not close(actual, expected):
                fail(f"pair-distance array disagrees with position array at ({left},{right})")
            if not close(actual, float(pair_data[right * 16 + left])):
                fail("pair-distance array must remain symmetric")
    return list(component_ids), positions, [float(v) for v in pair_data]


def validate_observables(
    observables: Any,
    component_ids: list[str],
    positions: list[tuple[float, float, float]],
) -> None:
    if not isinstance(observables, dict) or observables.get("contract") != "IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1":
        fail("unexpected Phase 5 observable contract")
    cutoff = observables.get("contact_cutoff")
    clearance = observables.get("accessibility_clearance")
    if not finite(cutoff) or cutoff <= 0 or not finite(clearance) or clearance <= 0:
        fail("observable thresholds must be positive finite assumptions")
    if observables.get("contact_cutoff_unit") != "model-unit" or observables.get("accessibility_clearance_unit") != "model-unit":
        fail("observable threshold units must remain model-unit")
    if observables.get("assumptions_explicit") is not True or observables.get("scientific_interpretation_claimed") is not False:
        fail("observable assumptions/nonclaim receipt mismatch")

    expected_pairs = {(left, right) for left in range(16) for right in range(left + 1, 16)}
    pairs = observables.get("pairs")
    if not isinstance(pairs, list) or len(pairs) != len(expected_pairs):
        fail("16-node V0 representation requires 120 unique pair observables")
    seen: set[tuple[int, int]] = set()
    nearest = [math.inf] * 16
    for pair in pairs:
        if not isinstance(pair, dict):
            fail("pair observable must be object")
        left = pair.get("left_index")
        right = pair.get("right_index")
        if (
            not isinstance(left, int)
            or isinstance(left, bool)
            or not isinstance(right, int)
            or isinstance(right, bool)
            or (left, right) not in expected_pairs
            or (left, right) in seen
        ):
            fail("pair observable indices must provide unique complete i<j coverage")
        seen.add((left, right))
        if pair.get("left_id") != component_ids[left] or pair.get("right_id") != component_ids[right]:
            fail("pair observable component ids do not match declared indices")
        dx = positions[left][0] - positions[right][0]
        dy = positions[left][1] - positions[right][1]
        dz = positions[left][2] - positions[right][2]
        expected_d2 = dx * dx + dy * dy + dz * dz
        expected_distance = math.sqrt(expected_d2)
        if not finite(pair.get("distance_squared")) or not close(pair["distance_squared"], expected_d2):
            fail("pair observable distance_squared disagrees with position array")
        if not finite(pair.get("distance")) or not close(pair["distance"], expected_distance):
            fail("pair observable distance disagrees with distance_squared/position array")
        if pair.get("computational_contact") is not (expected_distance <= float(cutoff)):
            fail("pair observable computational_contact disagrees with declared cutoff")
        if pair.get("biological_contact_claimed") is not False:
            fail("computational contact may not become biological contact")
        nearest[left] = min(nearest[left], expected_distance)
        nearest[right] = min(nearest[right], expected_distance)
    if seen != expected_pairs:
        fail("pair observable coverage is incomplete")

    accessibility = observables.get("accessibility")
    if not isinstance(accessibility, list) or len(accessibility) != 16:
        fail("V0 accessibility projection requires 16 component entries")
    by_id: dict[str, dict[str, Any]] = {}
    for item in accessibility:
        component_id = item.get("component_id") if isinstance(item, dict) else None
        if not isinstance(component_id, str) or component_id in by_id:
            fail("accessibility entries require unique component ids")
        by_id[component_id] = item
    if set(by_id) != set(component_ids):
        fail("accessibility component coverage must match the position array")
    for index, component_id in enumerate(component_ids):
        item = by_id[component_id]
        if not finite(item.get("nearest_neighbor_distance")) or not close(
            item["nearest_neighbor_distance"], nearest[index]
        ):
            fail("accessibility nearest-neighbour distance disagrees with pair geometry")
        if not finite(item.get("clearance_threshold")) or not close(item["clearance_threshold"], clearance):
            fail("accessibility clearance threshold disagrees with observable contract")
        if item.get("geometric_accessibility") is not (nearest[index] >= float(clearance)):
            fail("geometric_accessibility disagrees with declared clearance threshold")
        if item.get("biochemical_accessibility_claimed") is not False:
            fail("geometric accessibility may not become biochemical accessibility")


def validate_statistics(stats: Any, count: int, label: str) -> None:
    if not isinstance(stats, dict) or stats.get("count") != count:
        fail(f"ensemble {label} count must match explicit indices")
    for key in (
        "minimum",
        "maximum",
        "mean",
        "median",
        "population_variance",
        "population_standard_deviation",
    ):
        if not finite(stats.get(key)):
            fail(f"ensemble {label}.{key} must be finite")
    if stats["minimum"] > stats["maximum"]:
        fail(f"ensemble {label} minimum exceeds maximum")
    if not (stats["minimum"] - ABS_TOL <= stats["mean"] <= stats["maximum"] + ABS_TOL):
        fail(f"ensemble {label} mean outside range")
    if not (stats["minimum"] - ABS_TOL <= stats["median"] <= stats["maximum"] + ABS_TOL):
        fail(f"ensemble {label} median outside range")
    if stats["population_variance"] < 0 or stats["population_standard_deviation"] < 0:
        fail(f"ensemble {label} variance/standard deviation must be non-negative")
    if not close(
        stats["population_standard_deviation"] ** 2,
        stats["population_variance"],
        tolerance=1.0e-10,
    ):
        fail(f"ensemble {label} standard deviation is inconsistent with variance")
    if not isinstance(stats.get("numerical_assumption"), str) or not stats["numerical_assumption"]:
        fail(f"ensemble {label} must declare numerical assumption")


def validate_uncertainties(uncertainties: Any, ensemble_count: int) -> None:
    if not isinstance(uncertainties, dict) or uncertainties.get("contract") != "IGM-COMPUTATIONAL-UNCERTAINTY-V1":
        fail("unexpected computational uncertainty contract")
    if uncertainties.get("supported_kinds") != ["unknown", "interval", "distribution", "ensemble"]:
        fail("Phase 5 uncertainty kinds must remain explicit")
    if uncertainties.get("evidence_strength_promoted") is not False:
        fail("computational uncertainty may not promote evidence strength")
    records = uncertainties.get("records")
    if not isinstance(records, list) or len(records) != 4:
        fail("reference Phase 5 bundle requires exactly one record per uncertainty kind")
    by_kind: dict[str, dict[str, Any]] = {}
    for record in records:
        if not isinstance(record, dict) or record.get("kind") in by_kind:
            fail("computational uncertainty records require unique kinds")
        by_kind[record.get("kind")] = record
    if set(by_kind) != {"unknown", "interval", "distribution", "ensemble"}:
        fail("computational uncertainty records must cover all supported kinds")
    if not isinstance(by_kind["unknown"].get("reason"), str) or not by_kind["unknown"]["reason"].strip():
        fail("unknown computational uncertainty requires a reason")
    interval = by_kind["interval"]
    if not finite(interval.get("lower")) or not finite(interval.get("upper")) or interval["lower"] > interval["upper"]:
        fail("interval computational uncertainty requires finite ordered bounds")
    distribution = by_kind["distribution"]
    params = distribution.get("parameters")
    if distribution.get("family") != "normal" or distribution.get("sampling_performed") is not False or not isinstance(params, dict):
        fail("reference distribution uncertainty must be metadata-only normal")
    if not finite(params.get("mean")) or not finite(params.get("standard_deviation")) or params["standard_deviation"] < 0:
        fail("normal distribution uncertainty requires finite mean and non-negative standard deviation")
    ensemble = by_kind["ensemble"]
    if ensemble.get("member_count") != ensemble_count:
        fail("ensemble uncertainty member count must match explicit ensemble")
    if not finite(ensemble.get("minimum")) or not finite(ensemble.get("maximum")) or ensemble["minimum"] > ensemble["maximum"]:
        fail("ensemble uncertainty requires finite ordered bounds")


def validate_bundle(bundle_path: Path) -> None:
    bundle = load(bundle_path)
    if not isinstance(bundle, dict):
        fail("Phase 5 bundle root must be object")
    if bundle.get("schema") != "IGM-PHASE5-REPRESENTATION-BUNDLE-V1":
        fail("unexpected Phase 5 bundle schema")
    if bundle.get("representation_contract") != CONTRACT:
        fail("unexpected Phase 5 representation contract")
    if bundle.get("validation_level") != "V0":
        fail("Phase 5 V0 bundle must remain V0")
    if (
        bundle.get("phase4_source_adapter_contract") != "IGM-SOURCE-ADAPTER-V1"
        or bundle.get("phase4_boundary_consumed") is not True
    ):
        fail("Phase 5 bundle must consume the Phase 4 evidence boundary")
    for key in (
        "scientific_interpretation_claimed",
        "biological_validity_claimed",
        "clinical_validity_claimed",
        "performance_claim",
    ):
        if bundle.get(key) is not False:
            fail(f"Phase 5 bundle must set {key}=false")
    if bundle.get("phase5_gate") != PHASE5_GATE:
        fail("Phase 5 gate text changed")

    arrays = bundle.get("arrays")
    if not isinstance(arrays, list):
        fail("Phase 5 bundle requires explicit numerical arrays")
    for array in arrays:
        if not isinstance(array, dict) or array.get("contract") != "IGM-NUMERICAL-ARRAY-PROJECTION-V1":
            fail("unexpected numerical-array contract")
        if array.get("tensor_claimed") is not False or array.get("scientific_interpretation_claimed") is not False:
            fail("plain numerical arrays may not be promoted to tensors/scientific interpretation")
        shape = array.get("shape")
        data = array.get("data")
        if not isinstance(shape, list) or not shape or not isinstance(data, list):
            fail("numerical array requires shape/data")
        expected = 1
        for dimension in shape:
            if not isinstance(dimension, int) or isinstance(dimension, bool) or dimension <= 0:
                fail("numerical array shape must be positive integers")
            expected *= dimension
        if len(data) != expected or any(not finite(value) for value in data):
            fail("numerical array data does not match finite declared shape")
    component_ids, positions, _pair_matrix = extract_reference_geometry(arrays)

    tensor = bundle.get("centered_coordinate_tensor", {})
    if tensor.get("contract") != "IGM-DECLARED-TENSOR-V1" or tensor.get("tensor_claimed") is not True:
        fail("true tensor requires the declared tensor contract")
    semantics = tensor.get("transform_semantics", {})
    if semantics.get("exact_semantics_declared") is not True:
        fail("tensor transformation semantics must be explicit")
    if tensor.get("rank") != 2 or tensor.get("shape") != [16, 3]:
        fail("reference centered tensor must have declared rank 2 and shape [16,3]")
    if tensor.get("component_ids") != component_ids:
        fail("centered tensor component ordering must match position array")
    tensor_data = tensor.get("data")
    if not isinstance(tensor_data, list) or len(tensor_data) != 48 or any(not finite(v) for v in tensor_data):
        fail("centered tensor requires exactly 48 finite values")
    for axis in range(3):
        axis_sum = sum(float(tensor_data[index * 3 + axis]) for index in range(16))
        if not close(axis_sum, 0.0, tolerance=1.0e-10):
            fail("centered tensor must remove translation by zero-centroid projection")

    namespaces: list[str] = []
    graph_node_ids: dict[str, set[str]] = {}
    for key, (contract, name, namespace) in GRAPH_OBJECTS.items():
        graph_node_ids[key] = validate_graph(bundle.get(key), key, contract, name, namespace)
        namespaces.append(namespace)
    if graph_node_ids["visualization_graph"] != set(component_ids):
        fail("visualization graph node set must match represented components")

    tensor_factor = bundle.get("tensor_factor_graph")
    if not isinstance(tensor_factor, dict):
        fail("tensor_factor_graph must be object")
    if (
        tensor_factor.get("contract") != "IGM-TENSOR-FACTOR-GRAPH-V1"
        or tensor_factor.get("name") != "tensor-factor-graph"
        or tensor_factor.get("namespace") != "tensor-factor"
        or tensor_factor.get("scientific_interpretation_claimed") is not False
    ):
        fail("tensor-factor graph contract/name/namespace mismatch")
    factor_nodes = tensor_factor.get("candidate_nodes")
    factor_edges = tensor_factor.get("candidate_edges")
    if not isinstance(factor_nodes, list) or len(factor_nodes) != len(set(factor_nodes)) or not isinstance(factor_edges, list):
        fail("tensor-factor graph requires unique candidate nodes and edge array")
    factor_node_set = set(factor_nodes)
    for edge in factor_edges:
        if edge.get("source") not in factor_node_set or edge.get("target") not in factor_node_set:
            fail("tensor-factor edge endpoint must resolve in tensor-factor namespace")
        if edge.get("biological_relationship_claimed") is not False:
            fail("tensor-factor edges may not become biological relationships")
    namespaces.append("tensor-factor")

    hyper = bundle.get("model_hypergraph", {})
    if (
        hyper.get("contract") != "IGM-MODEL-HYPERGRAPH-V1"
        or hyper.get("name") != "model-hypergraph"
        or hyper.get("namespace") != "model-hypergraph"
        or hyper.get("pairwise_expansion_performed") is not False
        or hyper.get("scientific_interpretation_claimed") is not False
    ):
        fail("model hypergraph must preserve n-ary constraints without pairwise/scientific promotion")
    node_ids = hyper.get("node_ids")
    if not isinstance(node_ids, list) or set(node_ids) != set(component_ids) or len(node_ids) != len(set(node_ids)):
        fail("model hypergraph nodes must match represented components")
    hyperedge_ids: set[str] = set()
    for edge in hyper.get("hyperedges", []):
        edge_id = edge.get("id") if isinstance(edge, dict) else None
        participants = edge.get("participants") if isinstance(edge, dict) else None
        if not isinstance(edge_id, str) or not edge_id or edge_id in hyperedge_ids:
            fail("model hypergraph requires unique hyperedge ids")
        hyperedge_ids.add(edge_id)
        if not isinstance(participants, list) or not participants or len(participants) != len(set(participants)):
            fail("model hyperedges require unique non-empty participant lists")
        if any(participant not in set(component_ids) for participant in participants):
            fail("model hyperedge participant does not resolve to a component")
        if edge.get("biological_relationship_claimed") is not False:
            fail("model hyperedge may not promote representation to biological relationship")
    namespaces.append("model-hypergraph")
    if len(namespaces) != len(set(namespaces)):
        fail("Phase 5 graph objects may not reuse a semantic namespace")

    separation = bundle.get("graph_separation", {})
    for key, expected in GRAPH_CONTRACTS.items():
        field = f"{key}_contract"
        if separation.get(field) != expected:
            fail(f"graph separation receipt mismatch for {field}")
    for key, value in separation.items():
        if key.endswith("_merged") and value is not False:
            fail(f"graph namespaces must remain separate: {key}")
    if separation.get("cross_namespace_semantic_promotion_claimed") is not False:
        fail("graph namespace separation may not promote semantics")

    validate_observables(bundle.get("observables"), component_ids, positions)

    ensemble = bundle.get("ensemble_statistics", {})
    if ensemble.get("contract") != "IGM-ENSEMBLE-STATISTICS-V1":
        fail("unexpected ensemble statistics contract")
    if ensemble.get("source_execution_contract") != "IGM-PENTA-CRT-CPU-V1":
        fail("ensemble source execution contract mismatch")
    if ensemble.get("source_execution_profile_id") != "IGM-PENTA-CRT-SYMMETRIC-V0":
        fail("ensemble must retain exact Phase 3B execution profile id")
    if not valid_hash(ensemble.get("source_execution_profile_sha256")):
        fail("ensemble must retain exact Phase 3B execution profile SHA-256")
    config = load(ROOT / "runtime/profiles/igm-phase5-v0.json")
    execution_path = ROOT / config["ensemble"]["execution_profile_path"]
    if ensemble["source_execution_profile_sha256"] != canonical_file_sha256(execution_path):
        fail("ensemble execution-profile SHA-256 does not match configured Phase 3B bytes")
    indices = ensemble.get("explicit_indices")
    if not isinstance(indices, list) or not indices or len(indices) != len(set(indices)):
        fail("ensemble explicit indices must be unique and non-empty")
    if ensemble.get("sampling") != "explicit-index-set" or ensemble.get("phase3b_residual_gate_passed") is not True:
        fail("ensemble must use explicit indices after Phase 3B residual admission")
    if ensemble.get("population_variance_used") is not True or ensemble.get("biological_ensemble_claimed") is not False:
        fail("ensemble numerical assumptions/nonclaim mismatch")
    if ensemble.get("scientific_interpretation_claimed") is not False:
        fail("ensemble statistics may not claim scientific interpretation")
    validate_statistics(ensemble.get("min_pair_distance"), len(indices), "min_pair_distance")
    validate_statistics(ensemble.get("max_pair_distance"), len(indices), "max_pair_distance")

    validate_uncertainties(bundle.get("uncertainties"), len(indices))

    assessment = bundle.get("tensor_network_assessment", {})
    if assessment.get("contract") != "IGM-TENSOR-NETWORK-ASSESSMENT-V1":
        fail("unexpected tensor-network assessment contract")
    if assessment.get("exact_reconstruction_verified") is not True:
        fail("reference tensor-network assessment must establish exact identity reconstruction")
    if assessment.get("material_reduction") is not False or assessment.get("admitted") is not False:
        fail("reference tensor-network factorization must be rejected because it is not materially smaller")
    if assessment.get("performance_claim") is not False:
        fail("tensor-network assessment may not make a performance claim")
    if assessment.get("dense_elements") != len(tensor_data):
        fail("tensor-network dense element count must match source tensor")
    rank = assessment.get("candidate_rank")
    if not isinstance(rank, int) or isinstance(rank, bool) or rank <= 0:
        fail("tensor-network candidate rank must be positive integer")
    if assessment.get("factorized_elements") != assessment["dense_elements"] + rank * rank:
        fail("reference tensor-network factorized element accounting mismatch")
    if tensor_factor.get("factorization_admitted") is not assessment.get("admitted"):
        fail("tensor-factor graph admission must match tensor-network assessment")

    vortex = bundle.get("vortex_projection")
    if vortex is not None:
        if vortex.get("contract") != "IGM-VORTEX-INSPIRED-PROJECTION-V1":
            fail("unexpected vortex-inspired projection contract")
        if (
            vortex.get("biological_ontology_claimed") is not False
            or vortex.get("scientific_interpretation_claimed") is not False
        ):
            fail("vortex-inspired projection must remain representational only")
        nodes = vortex.get("nodes")
        if not isinstance(nodes, list) or len(nodes) != 16:
            fail("vortex projection must preserve represented component count")
        vortex_ids = {node.get("id") for node in nodes if isinstance(node, dict)}
        if vortex_ids != set(component_ids):
            fail("vortex projection component ids must match represented geometry")

    gate = bundle.get("gate", {})
    if gate.get("gate_contract") != GATE_CONTRACT or gate.get("accepted") is not True:
        fail("Phase 5 representation gate must be explicitly accepted")
    for key in (
        "phase4_boundary_consumed",
        "arrays_not_automatically_tensors",
        "tensor_transform_semantics_declared",
        "graph_namespaces_separated",
        "observable_assumptions_explicit",
        "ensemble_assumptions_explicit",
        "uncertainty_kinds_explicit",
        "tensor_network_requires_exact_material_reduction",
        "vortex_projection_is_optional",
    ):
        if gate.get(key) is not True:
            fail(f"Phase 5 gate must require {key}=true")
    for key in (
        "vortex_biological_ontology_claimed",
        "scientific_interpretation_claimed",
        "biological_validity_claimed",
        "clinical_validity_claimed",
    ):
        if gate.get(key) is not False:
            fail(f"Phase 5 gate must require {key}=false")
    gate_identity = gate.get("gate_identity_sha256")
    if not valid_hash(gate_identity) or gate_identity != compute_gate_identity(gate):
        fail("Phase 5 gate identity does not match its canonical gate contents")

    identity = bundle.get("bundle_sha256")
    if not valid_hash(identity):
        fail("Phase 5 bundle requires lowercase SHA-256 identity")
    try:
        expected_identity = compute_bundle_identity(bundle)
    except ValueError as exc:
        fail(f"cannot canonicalize Phase 5 bundle identity: {exc}")
    if identity != expected_identity:
        fail("Phase 5 bundle SHA-256 does not match canonical domain-separated bundle contents")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bundle", type=Path)
    args = parser.parse_args()
    validate_repository_contract()
    if args.bundle is not None:
        validate_bundle(args.bundle)
        print(f"OK: {args.bundle} satisfies the Phase 5 representation gate")
    else:
        print("OK: IGM Phase 5 repository representation contracts validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
