#!/usr/bin/env python3
"""Fail-closed validation for IGM Phase 5 representation contracts and bundles."""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PHASE5_GATE = "A representation earns scientific interpretation only from explicit evidence and validation"
CONTRACT = "IGM-PHASE5-REPRESENTATION-V1"
CONFIG_SCHEMA = "IGM-PHASE5-REPRESENTATION-CONFIG-V1"
GATE_CONTRACT = "IGM-PHASE5-REPRESENTATION-GATE-V1"
GRAPH_CONTRACTS = {
    "model_graph": "IGM-MODEL-GRAPH-V1",
    "execution_graph": "IGM-EXEC-GRAPH-C5-K2-C3-V1",
    "tensor_factor_graph": "IGM-TENSOR-FACTOR-GRAPH-V1",
    "visualization_graph": "IGM-VISUALIZATION-GRAPH-V1",
}


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


def validate_repository_contract() -> None:
    required = [
        "docs/PHASE5_REPRESENTATIONS.md",
        "runtime/rust/src/phase5.rs",
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
        if threshold.get("status") != "assumed" or threshold.get("scientific_interpretation_claimed") is not False:
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
    if bundle.get("phase4_source_adapter_contract") != "IGM-SOURCE-ADAPTER-V1" or bundle.get("phase4_boundary_consumed") is not True:
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
    if not isinstance(arrays, list) or len(arrays) < 2:
        fail("Phase 5 bundle requires explicit numerical arrays")
    for array in arrays:
        if array.get("contract") != "IGM-NUMERICAL-ARRAY-PROJECTION-V1":
            fail("unexpected numerical-array contract")
        if array.get("tensor_claimed") is not False:
            fail("plain numerical arrays may not be promoted to tensors")
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

    tensor = bundle.get("centered_coordinate_tensor", {})
    if tensor.get("contract") != "IGM-DECLARED-TENSOR-V1" or tensor.get("tensor_claimed") is not True:
        fail("true tensor requires the declared tensor contract")
    semantics = tensor.get("transform_semantics", {})
    if semantics.get("exact_semantics_declared") is not True:
        fail("tensor transformation semantics must be explicit")
    if tensor.get("rank") != 2 or tensor.get("shape", [None, None])[1] != 3:
        fail("reference centered tensor must have declared rank 2 and Cartesian width 3")

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

    hyper = bundle.get("model_hypergraph", {})
    if hyper.get("contract") != "IGM-MODEL-HYPERGRAPH-V1" or hyper.get("pairwise_expansion_performed") is not False:
        fail("model hypergraph must preserve n-ary constraints without forced pairwise expansion")

    observables = bundle.get("observables", {})
    pairs = observables.get("pairs")
    if observables.get("contract") != "IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1":
        fail("unexpected Phase 5 observable contract")
    if not isinstance(pairs, list) or len(pairs) != 120:
        fail("16-node V0 representation requires 120 unique pair observables")
    for pair in pairs:
        if pair.get("biological_contact_claimed") is not False:
            fail("computational contact may not become biological contact")
        if not finite(pair.get("distance")) or not finite(pair.get("distance_squared")):
            fail("pair observable distances must remain finite")
    accessibility = observables.get("accessibility")
    if not isinstance(accessibility, list) or len(accessibility) != 16:
        fail("V0 accessibility projection requires 16 component entries")
    if any(item.get("biochemical_accessibility_claimed") is not False for item in accessibility):
        fail("geometric accessibility may not become biochemical accessibility")

    ensemble = bundle.get("ensemble_statistics", {})
    if ensemble.get("contract") != "IGM-ENSEMBLE-STATISTICS-V1":
        fail("unexpected ensemble statistics contract")
    if ensemble.get("sampling") != "explicit-index-set" or ensemble.get("phase3b_residual_gate_passed") is not True:
        fail("ensemble must use explicit indices after Phase 3B residual admission")
    if ensemble.get("population_variance_used") is not True or ensemble.get("biological_ensemble_claimed") is not False:
        fail("ensemble numerical assumptions/nonclaim mismatch")
    for metric in ("min_pair_distance", "max_pair_distance"):
        stats = ensemble.get(metric, {})
        for key in ("minimum", "maximum", "mean", "median", "population_variance", "population_standard_deviation"):
            if not finite(stats.get(key)):
                fail(f"ensemble {metric}.{key} must be finite")
        if stats.get("count") != len(ensemble.get("explicit_indices", [])):
            fail(f"ensemble {metric} count must match explicit indices")

    uncertainties = bundle.get("uncertainties", {})
    if uncertainties.get("contract") != "IGM-COMPUTATIONAL-UNCERTAINTY-V1":
        fail("unexpected computational uncertainty contract")
    if uncertainties.get("supported_kinds") != ["unknown", "interval", "distribution", "ensemble"]:
        fail("Phase 5 uncertainty kinds must remain explicit")
    if uncertainties.get("evidence_strength_promoted") is not False:
        fail("computational uncertainty may not promote evidence strength")

    assessment = bundle.get("tensor_network_assessment", {})
    if assessment.get("contract") != "IGM-TENSOR-NETWORK-ASSESSMENT-V1":
        fail("unexpected tensor-network assessment contract")
    if assessment.get("exact_reconstruction_verified") is not True:
        fail("reference tensor-network assessment must establish exact identity reconstruction")
    if assessment.get("material_reduction") is not False or assessment.get("admitted") is not False:
        fail("reference tensor-network factorization must be rejected because it is not materially smaller")
    if assessment.get("performance_claim") is not False:
        fail("tensor-network assessment may not make a performance claim")

    vortex = bundle.get("vortex_projection")
    if vortex is not None:
        if vortex.get("contract") != "IGM-VORTEX-INSPIRED-PROJECTION-V1":
            fail("unexpected vortex-inspired projection contract")
        if vortex.get("biological_ontology_claimed") is not False or vortex.get("scientific_interpretation_claimed") is not False:
            fail("vortex-inspired projection must remain representational only")

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

    identity = bundle.get("bundle_sha256")
    if not isinstance(identity, str) or len(identity) != 64 or any(ch not in "0123456789abcdef" for ch in identity):
        fail("Phase 5 bundle requires lowercase SHA-256 identity")


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
