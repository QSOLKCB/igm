// SPDX-License-Identifier: Apache-2.0
//! Phase 5 tensor, graph, observable, ensemble, and uncertainty representations.
//!
//! This module is a representational/computational layer only. It consumes the
//! validated model/profile and Phase 4 provenance boundary, but it cannot turn
//! representation convenience into biological or clinical authority.

use crate::evidence::SOURCE_ADAPTER_CONTRACT;
use crate::phase3b::{
    run_penta_crt, verify_penta_crt, PentaCrtEngine, PentaCrtRunConfig,
    MAX_VERIFY_SAMPLES, OPTIMIZATION_CONTRACT,
};
use crate::{build_geometry, load_profile, GeometryState, LoadedProfile, Profile, RuntimeError, Vec3, INV_BIO_001};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component as PathComponent, Path, PathBuf};

pub const PHASE5_CONTRACT: &str = "IGM-PHASE5-REPRESENTATION-V1";
pub const PHASE5_CONFIG_SCHEMA: &str = "IGM-PHASE5-REPRESENTATION-CONFIG-V1";
pub const PHASE5_CONFIG_ID: &str = "IGM-PHASE5-V0-REPRESENTATION";
pub const NUMERICAL_ARRAY_CONTRACT: &str = "IGM-NUMERICAL-ARRAY-PROJECTION-V1";
pub const DECLARED_TENSOR_CONTRACT: &str = "IGM-DECLARED-TENSOR-V1";
pub const MODEL_GRAPH_CONTRACT: &str = "IGM-MODEL-GRAPH-V1";
pub const MODEL_HYPERGRAPH_CONTRACT: &str = "IGM-MODEL-HYPERGRAPH-V1";
pub const PROVENANCE_GRAPH_CONTRACT: &str = "IGM-PROVENANCE-GRAPH-V1";
pub const TENSOR_FACTOR_GRAPH_CONTRACT: &str = "IGM-TENSOR-FACTOR-GRAPH-V1";
pub const VISUALIZATION_GRAPH_CONTRACT: &str = "IGM-VISUALIZATION-GRAPH-V1";
pub const EXECUTION_GRAPH_CONTRACT: &str = "IGM-EXEC-GRAPH-C5-K2-C3-V1";
pub const OBSERVABLE_CONTRACT: &str = "IGM-PAIR-ACCESSIBILITY-OBSERVABLES-V1";
pub const ENSEMBLE_CONTRACT: &str = "IGM-ENSEMBLE-STATISTICS-V1";
pub const UNCERTAINTY_CONTRACT: &str = "IGM-COMPUTATIONAL-UNCERTAINTY-V1";
pub const TENSOR_NETWORK_ASSESSMENT_CONTRACT: &str = "IGM-TENSOR-NETWORK-ASSESSMENT-V1";
pub const VORTEX_PROJECTION_CONTRACT: &str = "IGM-VORTEX-INSPIRED-PROJECTION-V1";
pub const PHASE5_GATE_CONTRACT: &str = "IGM-PHASE5-REPRESENTATION-GATE-V1";
pub const PHASE5_GATE: &str =
    "A representation earns scientific interpretation only from explicit evidence and validation";
pub const INV_MATH_002: &str = "A Multidimensional Array Is Not Automatically a Tensor";
pub const MAX_PHASE5_CONFIG_BYTES: u64 = 128 * 1024;
pub const MAX_ENSEMBLE_MEMBERS: usize = 4096;

const BUNDLE_DOMAIN: &[u8] = b"IGM-PHASE5-REPRESENTATION-V1\0";
const GATE_DOMAIN: &[u8] = b"IGM-PHASE5-REPRESENTATION-GATE-V1\0";

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn canonical_json(value: &Value) -> Result<String, RuntimeError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).map_err(|e| err(e.to_string()))
        }
        Value::Array(values) => {
            let mut out = String::from("[");
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                out.push_str(&canonical_json(item)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
            keys.sort();
            let mut out = String::from("{");
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).map_err(|e| err(e.to_string()))?);
                out.push(':');
                out.push_str(&canonical_json(map.get(key).expect("canonical key exists"))?);
            }
            out.push('}');
            Ok(out)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssumedThreshold {
    pub value: f64,
    pub unit: String,
    pub status: String,
    pub scientific_interpretation_claimed: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnsembleConfig {
    pub execution_profile_path: String,
    pub indices: Vec<u64>,
    pub verification_samples: usize,
    pub sampling: String,
    pub population_variance: bool,
    pub status: String,
    pub biological_ensemble_claimed: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TensorNetworkConfig {
    pub assessment_enabled: bool,
    pub require_exact_reconstruction: bool,
    pub require_material_reduction: bool,
    pub performance_claim: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase5Claims {
    pub scientific_interpretation_claimed: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub performance_claim: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Phase5Config {
    pub schema: String,
    pub profile_id: String,
    pub version: String,
    pub model_id: String,
    pub validation_level: String,
    pub contact_cutoff: AssumedThreshold,
    pub accessibility_clearance: AssumedThreshold,
    pub ensemble: EnsembleConfig,
    pub tensor_network: TensorNetworkConfig,
    pub vortex_inspired_projection_enabled: bool,
    pub claims: Phase5Claims,
    pub notes: String,
}

#[derive(Debug)]
struct LoadedPhase5Config {
    config: Phase5Config,
    sha256: String,
}

fn safe_repo_relative(path: &str) -> Result<PathBuf, RuntimeError> {
    if path.is_empty() {
        return Err(err("Phase 5 repository-relative path may not be empty"));
    }
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Err(err("Phase 5 repository path must be relative"));
    }
    if candidate.components().any(|part| !matches!(part, PathComponent::Normal(_))) {
        return Err(err("Phase 5 repository path may not contain traversal or special components"));
    }
    if !path.starts_with("runtime/profiles/") {
        return Err(err("Phase 5 execution profile must live under runtime/profiles/"));
    }
    Ok(candidate.to_path_buf())
}

fn validate_threshold(threshold: &AssumedThreshold, label: &str) -> Result<(), RuntimeError> {
    if !threshold.value.is_finite() || threshold.value <= 0.0 {
        return Err(err(format!("{label} must be finite and > 0")));
    }
    if threshold.unit != "model-unit"
        || threshold.status != "assumed"
        || threshold.scientific_interpretation_claimed
        || threshold.notes.trim().is_empty()
    {
        return Err(err(format!("{label} must remain an explicit non-scientific V0 assumption")));
    }
    Ok(())
}

fn validate_config(config: &Phase5Config) -> Result<(), RuntimeError> {
    if config.schema != PHASE5_CONFIG_SCHEMA
        || config.profile_id != PHASE5_CONFIG_ID
        || config.version != "0.1.0"
        || config.model_id != crate::SUPPORTED_MODEL_ID
        || config.validation_level != "V0"
    {
        return Err(err("unsupported Phase 5 representation config identity"));
    }
    validate_threshold(&config.contact_cutoff, "contact_cutoff")?;
    validate_threshold(&config.accessibility_clearance, "accessibility_clearance")?;

    let ensemble = &config.ensemble;
    safe_repo_relative(&ensemble.execution_profile_path)?;
    if ensemble.indices.is_empty() || ensemble.indices.len() > MAX_ENSEMBLE_MEMBERS {
        return Err(err("Phase 5 ensemble index count outside bounded domain"));
    }
    let unique = ensemble.indices.iter().copied().collect::<BTreeSet<_>>();
    if unique.len() != ensemble.indices.len() {
        return Err(err("Phase 5 ensemble indices must be unique"));
    }
    if ensemble.verification_samples == 0 || ensemble.verification_samples > MAX_VERIFY_SAMPLES {
        return Err(err("Phase 5 ensemble verification sample count outside Phase 3B bound"));
    }
    if ensemble.sampling != "explicit-index-set"
        || !ensemble.population_variance
        || ensemble.status != "assumed"
        || ensemble.biological_ensemble_claimed
        || ensemble.notes.trim().is_empty()
    {
        return Err(err("Phase 5 ensemble must remain explicit, deterministic, assumed, and non-biological"));
    }

    if !config.tensor_network.assessment_enabled
        || !config.tensor_network.require_exact_reconstruction
        || !config.tensor_network.require_material_reduction
        || config.tensor_network.performance_claim
        || config.tensor_network.notes.trim().is_empty()
    {
        return Err(err("Phase 5 tensor-network assessment must require exactness and material reduction without a performance claim"));
    }
    if config.claims.scientific_interpretation_claimed
        || config.claims.biological_validity_claimed
        || config.claims.clinical_validity_claimed
        || config.claims.performance_claim
    {
        return Err(err("Phase 5 V0 representation config may not promote scientific/biological/clinical/performance claims"));
    }
    Ok(())
}

fn load_config(path: &Path) -> Result<LoadedPhase5Config, RuntimeError> {
    let metadata = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat Phase 5 config {}: {e}", path.display())))?;
    if !metadata.is_file() || metadata.len() > MAX_PHASE5_CONFIG_BYTES {
        return Err(err("Phase 5 config is not a bounded regular file"));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read Phase 5 config {}: {e}", path.display())))?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("Phase 5 config is not strict JSON: {e}")))?;
    let canonical = canonical_json(&raw)?;
    let config: Phase5Config = serde_json::from_value(raw)
        .map_err(|e| err(format!("Phase 5 config structural error: {e}")))?;
    validate_config(&config)?;
    Ok(LoadedPhase5Config {
        config,
        sha256: sha256_hex(canonical.as_bytes()),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct NumericalArrayProjection {
    pub contract: &'static str,
    pub name: &'static str,
    pub shape: Vec<usize>,
    pub axis_labels: Vec<String>,
    pub component_ids: Vec<String>,
    pub data: Vec<f64>,
    pub layout: &'static str,
    pub tensor_claimed: bool,
    pub semantics: &'static str,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TensorTransformSemantics {
    pub component_axis: &'static str,
    pub coordinate_axis: &'static str,
    pub translation_behavior: &'static str,
    pub scalar_field: &'static str,
    pub exact_semantics_declared: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeclaredTensor {
    pub contract: &'static str,
    pub name: &'static str,
    pub rank: usize,
    pub shape: Vec<usize>,
    pub component_ids: Vec<String>,
    pub coordinate_labels: Vec<&'static str>,
    pub data: Vec<f64>,
    pub transform_semantics: TensorTransformSemantics,
    pub tensor_claimed: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub source_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub relationship_type: String,
    pub derived: bool,
    pub biological_relationship_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedGraph {
    pub contract: &'static str,
    pub name: &'static str,
    pub namespace: &'static str,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hyperedge {
    pub id: String,
    pub relationship_type: String,
    pub participants: Vec<String>,
    pub status: String,
    pub source_ids: Vec<String>,
    pub biological_relationship_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypedHypergraph {
    pub contract: &'static str,
    pub name: &'static str,
    pub namespace: &'static str,
    pub node_ids: Vec<String>,
    pub hyperedges: Vec<Hyperedge>,
    pub pairwise_expansion_performed: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphSeparationReceipt {
    pub model_graph_contract: &'static str,
    pub execution_graph_contract: &'static str,
    pub tensor_factor_graph_contract: &'static str,
    pub visualization_graph_contract: &'static str,
    pub model_execution_merged: bool,
    pub model_tensor_factor_merged: bool,
    pub model_visualization_merged: bool,
    pub execution_tensor_factor_merged: bool,
    pub execution_visualization_merged: bool,
    pub tensor_factor_visualization_merged: bool,
    pub cross_namespace_semantic_promotion_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PairObservable {
    pub left_index: usize,
    pub right_index: usize,
    pub left_id: String,
    pub right_id: String,
    pub distance_squared: f64,
    pub distance: f64,
    pub computational_contact: bool,
    pub biological_contact_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityObservable {
    pub component_id: String,
    pub nearest_neighbor_distance: f64,
    pub clearance_threshold: f64,
    pub geometric_accessibility: bool,
    pub biochemical_accessibility_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservableSet {
    pub contract: &'static str,
    pub contact_cutoff: f64,
    pub contact_cutoff_unit: String,
    pub accessibility_clearance: f64,
    pub accessibility_clearance_unit: String,
    pub pairs: Vec<PairObservable>,
    pub accessibility: Vec<AccessibilityObservable>,
    pub assumptions_explicit: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScalarStatistics {
    pub count: usize,
    pub minimum: f64,
    pub maximum: f64,
    pub mean: f64,
    pub median: f64,
    pub population_variance: f64,
    pub population_standard_deviation: f64,
    pub numerical_assumption: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnsembleStatisticsSet {
    pub contract: &'static str,
    pub source_execution_contract: &'static str,
    pub sampling: String,
    pub explicit_indices: Vec<u64>,
    pub verification_samples: usize,
    pub phase3b_residual_gate_passed: bool,
    pub min_pair_distance: ScalarStatistics,
    pub max_pair_distance: ScalarStatistics,
    pub population_variance_used: bool,
    pub biological_ensemble_claimed: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ComputationalUncertainty {
    Unknown {
        reason: String,
    },
    Interval {
        lower: f64,
        upper: f64,
        unit: String,
    },
    Distribution {
        family: String,
        parameters: BTreeMap<String, f64>,
        unit: String,
        sampling_performed: bool,
    },
    Ensemble {
        member_count: usize,
        minimum: f64,
        maximum: f64,
        statistic: String,
        unit: String,
    },
}

impl ComputationalUncertainty {
    pub fn validate(&self) -> Result<(), RuntimeError> {
        match self {
            Self::Unknown { reason } => {
                if reason.trim().is_empty() {
                    return Err(err("unknown computational uncertainty requires a reason"));
                }
            }
            Self::Interval { lower, upper, unit } => {
                if !lower.is_finite() || !upper.is_finite() || lower > upper || unit.is_empty() {
                    return Err(err("interval uncertainty requires finite ordered bounds and unit"));
                }
            }
            Self::Distribution {
                family,
                parameters,
                unit,
                sampling_performed,
            } => {
                if family.trim().is_empty()
                    || parameters.is_empty()
                    || unit.trim().is_empty()
                    || *sampling_performed
                    || parameters.values().any(|v| !v.is_finite())
                {
                    return Err(err("distribution uncertainty is metadata-only in Phase 5 and requires finite explicit parameters"));
                }
                if family == "normal" {
                    let sd = parameters
                        .get("standard_deviation")
                        .ok_or_else(|| err("normal uncertainty requires standard_deviation"))?;
                    if *sd < 0.0 {
                        return Err(err("normal uncertainty standard_deviation may not be negative"));
                    }
                }
            }
            Self::Ensemble {
                member_count,
                minimum,
                maximum,
                statistic,
                unit,
            } => {
                if *member_count == 0
                    || !minimum.is_finite()
                    || !maximum.is_finite()
                    || minimum > maximum
                    || statistic.trim().is_empty()
                    || unit.trim().is_empty()
                {
                    return Err(err("ensemble uncertainty requires finite ordered statistics and non-zero membership"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UncertaintySet {
    pub contract: &'static str,
    pub supported_kinds: Vec<&'static str>,
    pub records: Vec<ComputationalUncertainty>,
    pub evidence_strength_promoted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TensorNetworkAssessment {
    pub contract: &'static str,
    pub source_tensor_contract: &'static str,
    pub source_tensor_shape: Vec<usize>,
    pub candidate_rank: usize,
    pub dense_elements: usize,
    pub factorized_elements: usize,
    pub exact_reconstruction_verified: bool,
    pub material_reduction: bool,
    pub admitted: bool,
    pub performance_claim: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TensorFactorGraph {
    pub contract: &'static str,
    pub name: &'static str,
    pub namespace: &'static str,
    pub source_tensor: &'static str,
    pub candidate_nodes: Vec<String>,
    pub candidate_edges: Vec<GraphEdge>,
    pub factorization_admitted: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct VortexProjectionNode {
    pub id: String,
    pub radial_squared: f64,
    pub phase_embedding_x: f64,
    pub phase_embedding_y: f64,
    pub axial_z: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct VortexInspiredProjection {
    pub contract: &'static str,
    pub name: &'static str,
    pub nodes: Vec<VortexProjectionNode>,
    pub phase_semantics: &'static str,
    pub biological_ontology_claimed: bool,
    pub scientific_interpretation_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Phase5GateReceipt {
    pub schema: &'static str,
    pub gate_contract: &'static str,
    pub phase4_boundary_consumed: bool,
    pub arrays_not_automatically_tensors: bool,
    pub tensor_transform_semantics_declared: bool,
    pub graph_namespaces_separated: bool,
    pub observable_assumptions_explicit: bool,
    pub ensemble_assumptions_explicit: bool,
    pub uncertainty_kinds_explicit: bool,
    pub tensor_network_requires_exact_material_reduction: bool,
    pub vortex_projection_is_optional: bool,
    pub vortex_biological_ontology_claimed: bool,
    pub scientific_interpretation_claimed: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub accepted: bool,
    pub inv_bio_001: &'static str,
    pub phase5_gate: &'static str,
    pub gate_identity_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RepresentationBundle {
    pub schema: &'static str,
    pub representation_contract: &'static str,
    pub model_id: String,
    pub validation_level: String,
    pub model_profile_sha256: String,
    pub representation_config_sha256: String,
    pub representation_config_id: String,
    pub phase4_source_adapter_contract: &'static str,
    pub phase4_boundary_consumed: bool,
    pub arrays: Vec<NumericalArrayProjection>,
    pub centered_coordinate_tensor: DeclaredTensor,
    pub model_graph: TypedGraph,
    pub model_hypergraph: TypedHypergraph,
    pub provenance_graph: TypedGraph,
    pub tensor_factor_graph: TensorFactorGraph,
    pub visualization_graph: TypedGraph,
    pub graph_separation: GraphSeparationReceipt,
    pub observables: ObservableSet,
    pub ensemble_statistics: EnsembleStatisticsSet,
    pub uncertainties: UncertaintySet,
    pub tensor_network_assessment: TensorNetworkAssessment,
    pub vortex_projection: Option<VortexInspiredProjection>,
    pub gate: Phase5GateReceipt,
    pub scientific_interpretation_claimed: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub performance_claim: bool,
    pub inv_bio_001: &'static str,
    pub inv_math_002: &'static str,
    pub phase5_gate: &'static str,
    pub bundle_sha256: String,
}

fn positions_array(geometry: &GeometryState) -> NumericalArrayProjection {
    let mut data = Vec::with_capacity(geometry.nodes.len() * 3);
    for node in &geometry.nodes {
        data.extend([node.position.x, node.position.y, node.position.z]);
    }
    NumericalArrayProjection {
        contract: NUMERICAL_ARRAY_CONTRACT,
        name: "cartesian-position-array",
        shape: vec![geometry.nodes.len(), 3],
        axis_labels: vec!["component".into(), "cartesian-coordinate".into()],
        component_ids: geometry.nodes.iter().map(|node| node.id.clone()).collect(),
        data,
        layout: "row-major",
        tensor_claimed: false,
        semantics: "plain numerical projection of validated model geometry",
        scientific_interpretation_claimed: false,
    }
}

fn pair_distance_squared_array(geometry: &GeometryState) -> Result<NumericalArrayProjection, RuntimeError> {
    let n = geometry.nodes.len();
    let mut data = Vec::with_capacity(n * n);
    for left in &geometry.nodes {
        for right in &geometry.nodes {
            data.push(left.position.checked_squared_distance(right.position)?);
        }
    }
    Ok(NumericalArrayProjection {
        contract: NUMERICAL_ARRAY_CONTRACT,
        name: "pair-distance-squared-array",
        shape: vec![n, n],
        axis_labels: vec!["left-component".into(), "right-component".into()],
        component_ids: geometry.nodes.iter().map(|node| node.id.clone()).collect(),
        data,
        layout: "row-major",
        tensor_claimed: false,
        semantics: "symmetric scalar observable array; no tensor transformation claim",
        scientific_interpretation_claimed: false,
    })
}

fn centered_tensor(geometry: &GeometryState) -> Result<DeclaredTensor, RuntimeError> {
    if geometry.nodes.is_empty() {
        return Err(err("centered tensor requires at least one geometry node"));
    }
    let n = geometry.nodes.len() as f64;
    let mut cx = 0.0_f64;
    let mut cy = 0.0_f64;
    let mut cz = 0.0_f64;
    for node in &geometry.nodes {
        cx += node.position.x;
        cy += node.position.y;
        cz += node.position.z;
    }
    cx /= n;
    cy /= n;
    cz /= n;
    if !cx.is_finite() || !cy.is_finite() || !cz.is_finite() {
        return Err(err("centered tensor centroid became non-finite"));
    }
    let mut data = Vec::with_capacity(geometry.nodes.len() * 3);
    for node in &geometry.nodes {
        data.extend([
            node.position.x - cx,
            node.position.y - cy,
            node.position.z - cz,
        ]);
    }
    if data.iter().any(|value| !value.is_finite()) {
        return Err(err("centered tensor contains non-finite value"));
    }
    Ok(DeclaredTensor {
        contract: DECLARED_TENSOR_CONTRACT,
        name: "centered-cartesian-coordinate-tensor",
        rank: 2,
        shape: vec![geometry.nodes.len(), 3],
        component_ids: geometry.nodes.iter().map(|node| node.id.clone()).collect(),
        coordinate_labels: vec!["x", "y", "z"],
        data,
        transform_semantics: TensorTransformSemantics {
            component_axis: "finite-basis permutation representation under component reindexing",
            coordinate_axis: "Cartesian vector representation under orthogonal frame transformations",
            translation_behavior: "translation removed by centroid subtraction before tensor projection",
            scalar_field: "R/f64",
            exact_semantics_declared: true,
        },
        tensor_claimed: true,
        scientific_interpretation_claimed: false,
    })
}

fn participants(value: &Value) -> Vec<String> {
    value
        .get("participants")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn model_graph(profile: &Profile) -> TypedGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut domains = BTreeSet::new();
    for component in &profile.components {
        nodes.push(GraphNode {
            id: component.id.clone(),
            node_type: "component".into(),
            source_status: Some(component.source_status.clone()),
        });
        domains.insert(component.kind.clone());
    }
    for domain in domains {
        let domain_id = format!("domain:{domain}");
        nodes.push(GraphNode {
            id: domain_id.clone(),
            node_type: "domain".into(),
            source_status: None,
        });
        for component in profile.components.iter().filter(|component| component.kind == domain) {
            edges.push(GraphEdge {
                source: component.id.clone(),
                target: domain_id.clone(),
                relationship_type: "member-of-declared-domain".into(),
                derived: false,
                biological_relationship_claimed: false,
            });
        }
    }
    for constraint in &profile.constraints {
        let constraint_node = format!("constraint-node:{}", constraint.id);
        nodes.push(GraphNode {
            id: constraint_node.clone(),
            node_type: "constraint".into(),
            source_status: Some(constraint.status.clone()),
        });
        for participant in participants(&constraint.definition) {
            edges.push(GraphEdge {
                source: participant,
                target: constraint_node.clone(),
                relationship_type: "participates-in-declared-constraint".into(),
                derived: false,
                biological_relationship_claimed: false,
            });
        }
    }
    TypedGraph {
        contract: MODEL_GRAPH_CONTRACT,
        name: "model-graph",
        namespace: "model",
        nodes,
        edges,
        scientific_interpretation_claimed: false,
    }
}

fn model_hypergraph(profile: &Profile) -> TypedHypergraph {
    let hyperedges = profile
        .constraints
        .iter()
        .filter_map(|constraint| {
            let p = participants(&constraint.definition);
            if p.is_empty() {
                None
            } else {
                Some(Hyperedge {
                    id: constraint.id.clone(),
                    relationship_type: constraint.kind.clone(),
                    participants: p,
                    status: constraint.status.clone(),
                    source_ids: constraint.source_ids.clone(),
                    biological_relationship_claimed: false,
                })
            }
        })
        .collect();
    TypedHypergraph {
        contract: MODEL_HYPERGRAPH_CONTRACT,
        name: "model-hypergraph",
        namespace: "model-hypergraph",
        node_ids: profile.components.iter().map(|component| component.id.clone()).collect(),
        hyperedges,
        pairwise_expansion_performed: false,
        scientific_interpretation_claimed: false,
    }
}

fn provenance_graph(profile: &Profile) -> TypedGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for source in &profile.source_ids {
        nodes.push(GraphNode {
            id: format!("source:{source}"),
            node_type: "source".into(),
            source_status: Some("registered-source".into()),
        });
    }
    for component in &profile.components {
        let id = format!("entity:component:{}", component.id);
        nodes.push(GraphNode {
            id: id.clone(),
            node_type: "component-provenance-subject".into(),
            source_status: Some(component.source_status.clone()),
        });
        for source in &component.source_ids {
            edges.push(GraphEdge {
                source: id.clone(),
                target: format!("source:{source}"),
                relationship_type: "supported-by-source".into(),
                derived: false,
                biological_relationship_claimed: false,
            });
        }
    }
    for parameter in &profile.parameters {
        let id = format!("entity:parameter:{}", parameter.name);
        nodes.push(GraphNode {
            id: id.clone(),
            node_type: "parameter-provenance-subject".into(),
            source_status: Some(parameter.status.clone()),
        });
        if let Some(source) = &parameter.source_id {
            edges.push(GraphEdge {
                source: id,
                target: format!("source:{source}"),
                relationship_type: "supported-by-source".into(),
                derived: false,
                biological_relationship_claimed: false,
            });
        }
    }
    for constraint in &profile.constraints {
        let id = format!("entity:constraint:{}", constraint.id);
        nodes.push(GraphNode {
            id: id.clone(),
            node_type: "constraint-provenance-subject".into(),
            source_status: Some(constraint.status.clone()),
        });
        for source in &constraint.source_ids {
            edges.push(GraphEdge {
                source: id.clone(),
                target: format!("source:{source}"),
                relationship_type: "supported-by-source".into(),
                derived: false,
                biological_relationship_claimed: false,
            });
        }
    }
    TypedGraph {
        contract: PROVENANCE_GRAPH_CONTRACT,
        name: "provenance-graph",
        namespace: "provenance",
        nodes,
        edges,
        scientific_interpretation_claimed: false,
    }
}

fn observables(
    geometry: &GeometryState,
    config: &Phase5Config,
) -> Result<ObservableSet, RuntimeError> {
    let mut pairs = Vec::new();
    let n = geometry.nodes.len();
    let mut nearest = vec![f64::INFINITY; n];
    for left in 0..n {
        for right in left + 1..n {
            let d2 = geometry.nodes[left]
                .position
                .checked_squared_distance(geometry.nodes[right].position)?;
            if d2 < 0.0 || !d2.is_finite() {
                return Err(err("pair observable distance squared outside finite non-negative domain"));
            }
            let distance = d2.sqrt();
            if !distance.is_finite() {
                return Err(err("pair observable distance became non-finite"));
            }
            nearest[left] = nearest[left].min(distance);
            nearest[right] = nearest[right].min(distance);
            pairs.push(PairObservable {
                left_index: left,
                right_index: right,
                left_id: geometry.nodes[left].id.clone(),
                right_id: geometry.nodes[right].id.clone(),
                distance_squared: d2,
                distance,
                computational_contact: distance <= config.contact_cutoff.value,
                biological_contact_claimed: false,
            });
        }
    }
    let accessibility = geometry
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| AccessibilityObservable {
            component_id: node.id.clone(),
            nearest_neighbor_distance: nearest[index],
            clearance_threshold: config.accessibility_clearance.value,
            geometric_accessibility: nearest[index] >= config.accessibility_clearance.value,
            biochemical_accessibility_claimed: false,
        })
        .collect::<Vec<_>>();
    if accessibility
        .iter()
        .any(|item| !item.nearest_neighbor_distance.is_finite())
    {
        return Err(err("accessibility observable requires at least two finite geometry nodes"));
    }
    Ok(ObservableSet {
        contract: OBSERVABLE_CONTRACT,
        contact_cutoff: config.contact_cutoff.value,
        contact_cutoff_unit: config.contact_cutoff.unit.clone(),
        accessibility_clearance: config.accessibility_clearance.value,
        accessibility_clearance_unit: config.accessibility_clearance.unit.clone(),
        pairs,
        accessibility,
        assumptions_explicit: true,
        scientific_interpretation_claimed: false,
    })
}

fn visualization_graph(geometry: &GeometryState, observables: &ObservableSet) -> TypedGraph {
    let nodes = geometry
        .nodes
        .iter()
        .map(|node| GraphNode {
            id: node.id.clone(),
            node_type: "visualized-component".into(),
            source_status: Some(node.source_status.clone()),
        })
        .collect();
    let edges = observables
        .pairs
        .iter()
        .filter(|pair| pair.computational_contact)
        .map(|pair| GraphEdge {
            source: pair.left_id.clone(),
            target: pair.right_id.clone(),
            relationship_type: "computational-contact-under-explicit-cutoff".into(),
            derived: true,
            biological_relationship_claimed: false,
        })
        .collect();
    TypedGraph {
        contract: VISUALIZATION_GRAPH_CONTRACT,
        name: "visualization-graph",
        namespace: "visualization",
        nodes,
        edges,
        scientific_interpretation_claimed: false,
    }
}

fn scalar_statistics(mut values: Vec<f64>) -> Result<ScalarStatistics, RuntimeError> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err(err("ensemble statistics require a non-empty finite sample"));
    }
    values.sort_by(f64::total_cmp);
    let count = values.len();
    let minimum = values[0];
    let maximum = values[count - 1];
    let mean = values.iter().copied().sum::<f64>() / count as f64;
    let median = if count % 2 == 1 {
        values[count / 2]
    } else {
        (values[count / 2 - 1] + values[count / 2]) / 2.0
    };
    let population_variance = values
        .iter()
        .map(|value| {
            let d = *value - mean;
            d * d
        })
        .sum::<f64>()
        / count as f64;
    let population_standard_deviation = population_variance.sqrt();
    if !mean.is_finite()
        || !median.is_finite()
        || !population_variance.is_finite()
        || !population_standard_deviation.is_finite()
    {
        return Err(err("ensemble statistics produced non-finite result"));
    }
    Ok(ScalarStatistics {
        count,
        minimum,
        maximum,
        mean,
        median,
        population_variance,
        population_standard_deviation,
        numerical_assumption: "population variance over the explicit deterministic index set; no sampling-population inference",
    })
}

fn ensemble_statistics(
    penta: &PentaCrtEngine,
    config: &EnsembleConfig,
) -> Result<EnsembleStatisticsSet, RuntimeError> {
    let verification = verify_penta_crt(penta, config.verification_samples)?;
    if !verification.accepted {
        return Err(err("Phase 5 ensemble refuses a Phase 3B engine that fails the fixed residual gate"));
    }
    let total = penta.total_conformations();
    let mut min_distances = Vec::with_capacity(config.indices.len());
    let mut max_distances = Vec::with_capacity(config.indices.len());
    for index in &config.indices {
        if *index >= total {
            return Err(err(format!("Phase 5 ensemble index {index} exceeds Phase 3B domain {total}")));
        }
        let run = run_penta_crt(penta, PentaCrtRunConfig::new(*index, 1, 1, total)?)?;
        if !run.min_pair_distance_squared.is_finite()
            || !run.max_pair_distance_squared.is_finite()
            || run.min_pair_distance_squared < 0.0
            || run.max_pair_distance_squared < 0.0
        {
            return Err(err("Phase 5 ensemble received invalid Phase 3B pair-distance summary"));
        }
        min_distances.push(run.min_pair_distance_squared.sqrt());
        max_distances.push(run.max_pair_distance_squared.sqrt());
    }
    Ok(EnsembleStatisticsSet {
        contract: ENSEMBLE_CONTRACT,
        source_execution_contract: OPTIMIZATION_CONTRACT,
        sampling: config.sampling.clone(),
        explicit_indices: config.indices.clone(),
        verification_samples: config.verification_samples,
        phase3b_residual_gate_passed: true,
        min_pair_distance: scalar_statistics(min_distances)?,
        max_pair_distance: scalar_statistics(max_distances)?,
        population_variance_used: true,
        biological_ensemble_claimed: false,
        scientific_interpretation_claimed: false,
    })
}

fn tensor_network_assessment(tensor: &DeclaredTensor, config: &TensorNetworkConfig) -> TensorNetworkAssessment {
    let dense_elements = tensor.data.len();
    let candidate_rank = 3_usize;
    // Exact identity factorization X = X * I_3 is a rigorously specified
    // candidate. It is intentionally rejected because it stores more numbers
    // than the dense tensor, demonstrating that "tensor network" is not an
    // automatic promotion or optimization.
    let factorized_elements = dense_elements + candidate_rank * candidate_rank;
    let exact_reconstruction_verified = true;
    let material_reduction = factorized_elements < dense_elements;
    let admitted = config.assessment_enabled
        && (!config.require_exact_reconstruction || exact_reconstruction_verified)
        && (!config.require_material_reduction || material_reduction);
    let reason = if admitted {
        "candidate is exact and materially reduces declared storage".to_string()
    } else if !material_reduction {
        "exact identity factorization is not admitted because it does not materially reduce storage".to_string()
    } else {
        "candidate rejected by Phase 5 admission requirements".to_string()
    };
    TensorNetworkAssessment {
        contract: TENSOR_NETWORK_ASSESSMENT_CONTRACT,
        source_tensor_contract: DECLARED_TENSOR_CONTRACT,
        source_tensor_shape: tensor.shape.clone(),
        candidate_rank,
        dense_elements,
        factorized_elements,
        exact_reconstruction_verified,
        material_reduction,
        admitted,
        performance_claim: false,
        reason,
    }
}

fn tensor_factor_graph(assessment: &TensorNetworkAssessment) -> TensorFactorGraph {
    TensorFactorGraph {
        contract: TENSOR_FACTOR_GRAPH_CONTRACT,
        name: "tensor-factor-graph",
        namespace: "tensor-factor",
        source_tensor: "centered-cartesian-coordinate-tensor",
        candidate_nodes: vec![
            "tensor:centered-coordinate".into(),
            "factor:left-original".into(),
            "factor:right-identity3".into(),
        ],
        candidate_edges: vec![
            GraphEdge {
                source: "factor:left-original".into(),
                target: "tensor:centered-coordinate".into(),
                relationship_type: "candidate-factor".into(),
                derived: true,
                biological_relationship_claimed: false,
            },
            GraphEdge {
                source: "factor:right-identity3".into(),
                target: "tensor:centered-coordinate".into(),
                relationship_type: "candidate-factor".into(),
                derived: true,
                biological_relationship_claimed: false,
            },
        ],
        factorization_admitted: assessment.admitted,
        scientific_interpretation_claimed: false,
    }
}

fn vortex_projection(geometry: &GeometryState, enabled: bool) -> Result<Option<VortexInspiredProjection>, RuntimeError> {
    if !enabled {
        return Ok(None);
    }
    let mut nodes = Vec::with_capacity(geometry.nodes.len());
    for node in &geometry.nodes {
        let radial_squared = node.position.x * node.position.x + node.position.y * node.position.y;
        if !radial_squared.is_finite() {
            return Err(err("vortex-inspired radial projection became non-finite"));
        }
        nodes.push(VortexProjectionNode {
            id: node.id.clone(),
            radial_squared,
            phase_embedding_x: node.position.x,
            phase_embedding_y: node.position.y,
            axial_z: node.position.z,
        });
    }
    Ok(Some(VortexInspiredProjection {
        contract: VORTEX_PROJECTION_CONTRACT,
        name: "optional-vortex-inspired-coordinate-projection",
        nodes,
        phase_semantics: "representational radial-squared plus Cartesian phase embedding; not a biological ontology",
        biological_ontology_claimed: false,
        scientific_interpretation_claimed: false,
    }))
}

fn uncertainties(ensemble: &EnsembleStatisticsSet) -> Result<UncertaintySet, RuntimeError> {
    let mut normal = BTreeMap::new();
    normal.insert("mean".into(), ensemble.min_pair_distance.mean);
    normal.insert(
        "standard_deviation".into(),
        ensemble.min_pair_distance.population_standard_deviation,
    );
    let records = vec![
        ComputationalUncertainty::Unknown {
            reason: "V0 representation does not establish biological uncertainty or biological validity".into(),
        },
        ComputationalUncertainty::Interval {
            lower: ensemble.min_pair_distance.minimum,
            upper: ensemble.min_pair_distance.maximum,
            unit: "model-unit".into(),
        },
        ComputationalUncertainty::Distribution {
            family: "normal".into(),
            parameters: normal,
            unit: "model-unit".into(),
            sampling_performed: false,
        },
        ComputationalUncertainty::Ensemble {
            member_count: ensemble.min_pair_distance.count,
            minimum: ensemble.min_pair_distance.minimum,
            maximum: ensemble.min_pair_distance.maximum,
            statistic: "minimum-pair-distance across explicit Phase 3B indices".into(),
            unit: "model-unit".into(),
        },
    ];
    for record in &records {
        record.validate()?;
    }
    Ok(UncertaintySet {
        contract: UNCERTAINTY_CONTRACT,
        supported_kinds: vec!["unknown", "interval", "distribution", "ensemble"],
        records,
        evidence_strength_promoted: false,
    })
}

fn graph_separation() -> GraphSeparationReceipt {
    GraphSeparationReceipt {
        model_graph_contract: MODEL_GRAPH_CONTRACT,
        execution_graph_contract: EXECUTION_GRAPH_CONTRACT,
        tensor_factor_graph_contract: TENSOR_FACTOR_GRAPH_CONTRACT,
        visualization_graph_contract: VISUALIZATION_GRAPH_CONTRACT,
        model_execution_merged: false,
        model_tensor_factor_merged: false,
        model_visualization_merged: false,
        execution_tensor_factor_merged: false,
        execution_visualization_merged: false,
        tensor_factor_visualization_merged: false,
        cross_namespace_semantic_promotion_claimed: false,
    }
}

fn gate_identity(gate: &Phase5GateReceipt) -> Result<String, RuntimeError> {
    let mut value = serde_json::to_value(gate).map_err(|e| err(e.to_string()))?;
    if let Value::Object(map) = &mut value {
        map.insert("gate_identity_sha256".into(), Value::String(String::new()));
    }
    Ok(sha256_hex(format!("{}{}", String::from_utf8_lossy(GATE_DOMAIN), canonical_json(&value)?).as_bytes()))
}

fn make_gate(
    config: &Phase5Config,
    tensor: &DeclaredTensor,
    assessment: &TensorNetworkAssessment,
    vortex: &Option<VortexInspiredProjection>,
) -> Result<Phase5GateReceipt, RuntimeError> {
    let vortex_biological_ontology_claimed = vortex
        .as_ref()
        .map(|projection| projection.biological_ontology_claimed)
        .unwrap_or(false);
    let mut gate = Phase5GateReceipt {
        schema: "IGM-PHASE5-GATE-RECEIPT-V1",
        gate_contract: PHASE5_GATE_CONTRACT,
        phase4_boundary_consumed: true,
        arrays_not_automatically_tensors: true,
        tensor_transform_semantics_declared: tensor.transform_semantics.exact_semantics_declared,
        graph_namespaces_separated: true,
        observable_assumptions_explicit: true,
        ensemble_assumptions_explicit: true,
        uncertainty_kinds_explicit: true,
        tensor_network_requires_exact_material_reduction: config.tensor_network.require_exact_reconstruction
            && config.tensor_network.require_material_reduction
            && (!assessment.admitted || (assessment.exact_reconstruction_verified && assessment.material_reduction)),
        vortex_projection_is_optional: true,
        vortex_biological_ontology_claimed,
        scientific_interpretation_claimed: false,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        accepted: false,
        inv_bio_001: INV_BIO_001,
        phase5_gate: PHASE5_GATE,
        gate_identity_sha256: String::new(),
    };
    gate.accepted = gate.phase4_boundary_consumed
        && gate.arrays_not_automatically_tensors
        && gate.tensor_transform_semantics_declared
        && gate.graph_namespaces_separated
        && gate.observable_assumptions_explicit
        && gate.ensemble_assumptions_explicit
        && gate.uncertainty_kinds_explicit
        && gate.tensor_network_requires_exact_material_reduction
        && gate.vortex_projection_is_optional
        && !gate.vortex_biological_ontology_claimed
        && !gate.scientific_interpretation_claimed
        && !gate.biological_validity_claimed
        && !gate.clinical_validity_claimed;
    if !gate.accepted {
        return Err(err("Phase 5 representation gate rejected the bundle"));
    }
    gate.gate_identity_sha256 = gate_identity(&gate)?;
    Ok(gate)
}

fn bundle_identity(bundle: &RepresentationBundle) -> Result<String, RuntimeError> {
    let mut value = serde_json::to_value(bundle).map_err(|e| err(e.to_string()))?;
    if let Value::Object(map) = &mut value {
        map.insert("bundle_sha256".into(), Value::String(String::new()));
    }
    let canonical = canonical_json(&value)?;
    let mut hasher = Sha256::new();
    hasher.update(BUNDLE_DOMAIN);
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug)]
pub struct Phase5Engine {
    model: LoadedProfile,
    config: LoadedPhase5Config,
    penta: PentaCrtEngine,
}

impl Phase5Engine {
    pub fn load(model_path: &Path, config_path: &Path, repository_root: &Path) -> Result<Self, RuntimeError> {
        let model = load_profile(model_path)?;
        let config = load_config(config_path)?;
        if model.profile().model_id != config.config.model_id
            || model.profile().validation_level != config.config.validation_level
        {
            return Err(err("Phase 5 config/model identity mismatch"));
        }
        if model.profile().claims.biological_validity_claimed
            || model.profile().claims.clinical_validity_claimed
        {
            return Err(err("Phase 5 V0 engine refuses model validity promotion"));
        }
        let execution_rel = safe_repo_relative(&config.config.ensemble.execution_profile_path)?;
        let execution_path = repository_root.join(execution_rel);
        let penta = PentaCrtEngine::load(model_path, &execution_path)?;
        Ok(Self { model, config, penta })
    }

    pub fn bundle(&self) -> Result<RepresentationBundle, RuntimeError> {
        let geometry = build_geometry(self.model.profile())?;
        let arrays = vec![positions_array(&geometry), pair_distance_squared_array(&geometry)?];
        let tensor = centered_tensor(&geometry)?;
        let observables = observables(&geometry, &self.config.config)?;
        let model_graph = model_graph(self.model.profile());
        let model_hypergraph = model_hypergraph(self.model.profile());
        let provenance_graph = provenance_graph(self.model.profile());
        let visualization_graph = visualization_graph(&geometry, &observables);
        let assessment = tensor_network_assessment(&tensor, &self.config.config.tensor_network);
        let tensor_factor_graph = tensor_factor_graph(&assessment);
        let ensemble = ensemble_statistics(&self.penta, &self.config.config.ensemble)?;
        let uncertainties = uncertainties(&ensemble)?;
        let vortex = vortex_projection(
            &geometry,
            self.config.config.vortex_inspired_projection_enabled,
        )?;
        let gate = make_gate(&self.config.config, &tensor, &assessment, &vortex)?;
        let mut bundle = RepresentationBundle {
            schema: "IGM-PHASE5-REPRESENTATION-BUNDLE-V1",
            representation_contract: PHASE5_CONTRACT,
            model_id: self.model.profile().model_id.clone(),
            validation_level: self.model.profile().validation_level.clone(),
            model_profile_sha256: self.model.profile_sha256().to_string(),
            representation_config_sha256: self.config.sha256.clone(),
            representation_config_id: self.config.config.profile_id.clone(),
            phase4_source_adapter_contract: SOURCE_ADAPTER_CONTRACT,
            phase4_boundary_consumed: true,
            arrays,
            centered_coordinate_tensor: tensor,
            model_graph,
            model_hypergraph,
            provenance_graph,
            tensor_factor_graph,
            visualization_graph,
            graph_separation: graph_separation(),
            observables,
            ensemble_statistics: ensemble,
            uncertainties,
            tensor_network_assessment: assessment,
            vortex_projection: vortex,
            gate,
            scientific_interpretation_claimed: false,
            biological_validity_claimed: false,
            clinical_validity_claimed: false,
            performance_claim: false,
            inv_bio_001: INV_BIO_001,
            inv_math_002: INV_MATH_002,
            phase5_gate: PHASE5_GATE,
            bundle_sha256: String::new(),
        };
        bundle.bundle_sha256 = bundle_identity(&bundle)?;
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Phase5Engine {
        Phase5Engine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-phase5-v0.json"),
            Path::new("."),
        )
        .expect("repository Phase 5 engine")
    }

    #[test]
    fn arrays_are_not_silently_promoted_to_tensors() {
        let bundle = engine().bundle().unwrap();
        assert!(bundle.arrays.iter().all(|array| !array.tensor_claimed));
        assert!(bundle.centered_coordinate_tensor.tensor_claimed);
        assert!(bundle
            .centered_coordinate_tensor
            .transform_semantics
            .exact_semantics_declared);
    }

    #[test]
    fn graph_namespaces_are_separate() {
        let bundle = engine().bundle().unwrap();
        let receipt = bundle.graph_separation;
        assert!(!receipt.model_execution_merged);
        assert!(!receipt.model_tensor_factor_merged);
        assert!(!receipt.model_visualization_merged);
        assert!(!receipt.cross_namespace_semantic_promotion_claimed);
        assert!(!bundle.model_hypergraph.pairwise_expansion_performed);
    }

    #[test]
    fn v0_factorization_is_rigorously_assessed_and_not_admitted_without_reduction() {
        let bundle = engine().bundle().unwrap();
        let assessment = bundle.tensor_network_assessment;
        assert!(assessment.exact_reconstruction_verified);
        assert!(!assessment.material_reduction);
        assert!(!assessment.admitted);
        assert!(!assessment.performance_claim);
    }

    #[test]
    fn observables_remain_computational_nonclaims() {
        let bundle = engine().bundle().unwrap();
        assert_eq!(bundle.observables.pairs.len(), 120);
        assert!(bundle
            .observables
            .pairs
            .iter()
            .all(|pair| !pair.biological_contact_claimed));
        assert!(bundle
            .observables
            .accessibility
            .iter()
            .all(|item| !item.biochemical_accessibility_claimed));
    }

    #[test]
    fn ensemble_statistics_are_explicit_and_phase3b_gated() {
        let bundle = engine().bundle().unwrap();
        assert!(bundle.ensemble_statistics.phase3b_residual_gate_passed);
        assert_eq!(bundle.ensemble_statistics.sampling, "explicit-index-set");
        assert_eq!(bundle.ensemble_statistics.min_pair_distance.count, 3);
        assert!(!bundle.ensemble_statistics.biological_ensemble_claimed);
    }

    #[test]
    fn uncertainty_contract_supports_all_four_phase5_kinds() {
        let bundle = engine().bundle().unwrap();
        assert_eq!(
            bundle.uncertainties.supported_kinds,
            vec!["unknown", "interval", "distribution", "ensemble"]
        );
        assert_eq!(bundle.uncertainties.records.len(), 4);
        for record in &bundle.uncertainties.records {
            record.validate().unwrap();
        }
        assert!(!bundle.uncertainties.evidence_strength_promoted);
    }

    #[test]
    fn vortex_projection_is_optional_and_nonontological() {
        let mut config = load_config(Path::new("runtime/profiles/igm-phase5-v0.json"))
            .unwrap()
            .config;
        assert!(!config.vortex_inspired_projection_enabled);
        let model = load_profile(Path::new("profiles/igm-schematic-pentamer-v0.json")).unwrap();
        let geometry = build_geometry(model.profile()).unwrap();
        assert!(vortex_projection(&geometry, false).unwrap().is_none());
        config.vortex_inspired_projection_enabled = true;
        let projected = vortex_projection(&geometry, true).unwrap().unwrap();
        assert!(!projected.biological_ontology_claimed);
        assert!(!projected.scientific_interpretation_claimed);
    }

    #[test]
    fn phase5_gate_is_nonpromoting() {
        let bundle = engine().bundle().unwrap();
        assert!(bundle.gate.accepted);
        assert!(!bundle.gate.scientific_interpretation_claimed);
        assert!(!bundle.gate.biological_validity_claimed);
        assert!(!bundle.gate.clinical_validity_claimed);
        assert_eq!(bundle.phase5_gate, PHASE5_GATE);
        assert_eq!(bundle.inv_bio_001, INV_BIO_001);
    }
}
