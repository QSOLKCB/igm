// SPDX-License-Identifier: Apache-2.0
//! Explicit Phase 3C acceptance gate.
//!
//! This layer turns the prose Phase 3C gate into an executable contract. An
//! accepted campaign must preserve model/algorithm identities, pass the fixed
//! Phase 3B residual gate, remain finite and bounded, preserve the declared
//! conformation slice, reproduce the canonical correctness identity, keep that
//! identity independent of workers/chunking, and exclude benchmark timing from
//! correctness identity.
//!
//! Execution topology, memory adjacency, warp/SIMD placement, chunk membership,
//! worker assignment, and future device assignment remain implementation
//! structures only. They cannot create biological relationships or promote the
//! validation level.

#[path = "phase3c_v2.rs"]
mod inner;

pub use inner::{
    chunk_plan, environment_receipt, execution_graph_nodes, execution_graph_receipt,
    memory_layout_receipt, persist_rejected_campaign, plan_memory, ArtifactIdentity,
    BenchmarkReceipt, CampaignChunk, CampaignConfig, CorrectnessReceipt, EnvironmentReceipt,
    ExecutionEdgeKind, ExecutionGraphNode, ExecutionGraphReceipt, ExecutionNeighbor,
    MemoryLayoutReceipt, MemoryPlan, PaddedExecutionCell, RejectionReceipt,
    BENCHMARK_RECEIPT_SCHEMA, CORRECTNESS_RECEIPT_SCHEMA, DEFAULT_MEMORY_BUDGET_BYTES,
    ENVIRONMENT_RECEIPT_SCHEMA, EXECUTION_GRAPH_CONTRACT, INV_RUNTIME_001,
    MAX_CAMPAIGN_CHUNKS, MAX_MEMORY_BUDGET_BYTES, MEANINGFUL_LANES, MEMORY_LAYOUT_CONTRACT,
    MEMORY_PLAN_SCHEMA, PADDING_LANES, PHASE3C_CONTRACT, REJECTION_RECEIPT_SCHEMA,
    TRAVERSAL_RECEIPT_SCHEMA, WARP_WIDTH,
};

use crate::phase3b::{
    PentaCrtEngine, BLOCK_REUSE_RESIDUAL_TOLERANCE, MAX_VERIFY_SAMPLES,
    OPTIMIZATION_CONTRACT, OPTIMIZATION_NUMERICAL_PROFILE,
};
use crate::{RuntimeError, INV_BIO_001, MAX_WORKERS};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const PHASE3C_GATE_CONTRACT: &str = "IGM-PHASE3C-ACCEPTANCE-GATE-V1";
pub const PHASE3C_GATE_RECEIPT_SCHEMA: &str = "IGM-PHASE3C-GATE-RECEIPT-V1";
pub const CAMPAIGN_MANIFEST_SCHEMA: &str = "IGM-CAMPAIGN-MANIFEST-V2";
pub const VALIDATION_LEVEL: &str = "V0";

const CORRECTNESS_DOMAIN: &[u8] = b"IGM-CAMPAIGN-CORRECTNESS-V1\0";
const GATE_DOMAIN: &[u8] = b"IGM-PHASE3C-GATE-RECEIPT-V1\0";
const MANIFEST_DOMAIN: &[u8] = b"IGM-CAMPAIGN-MANIFEST-V2\0";

const CORRECTNESS_INCLUDED_FIELDS: [&str; 14] = [
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
];

const CORRECTNESS_EXCLUDED_FIELDS: [&str; 6] = [
    "requested_workers",
    "memory_budget_bytes",
    "resident_capacity_cells",
    "chunk_count",
    "elapsed_seconds",
    "conformations_per_second",
];

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_hex_u64(value: &str, label: &str) -> Result<u64, RuntimeError> {
    if value.len() != 16 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(err(format!("{label} must be exactly 16 hexadecimal digits")));
    }
    u64::from_str_radix(value, 16).map_err(|e| err(format!("cannot parse {label}: {e}")))
}

fn recompute_correctness_identity(correctness: &CorrectnessReceipt) -> Result<String, RuntimeError> {
    let diagnostic = parse_hex_u64(&correctness.diagnostic_xor_fnv1a64, "correctness diagnostic")?;
    let mut hasher = Sha256::new();
    hasher.update(CORRECTNESS_DOMAIN);
    hasher.update(correctness.optimization_contract.as_bytes());
    hasher.update(correctness.numerical_profile.as_bytes());
    hasher.update(correctness.graph_contract.as_bytes());
    hasher.update(correctness.graph_sha256.as_bytes());
    hasher.update(correctness.traversal_sha256.as_bytes());
    hasher.update(correctness.layout_contract.as_bytes());
    hasher.update(correctness.model_profile_sha256.as_bytes());
    hasher.update(correctness.optimization_profile_sha256.as_bytes());
    hasher.update(correctness.conformation_start.to_le_bytes());
    hasher.update(correctness.conformation_count.to_le_bytes());
    hasher.update(diagnostic.to_le_bytes());
    hasher.update(correctness.min_pair_distance_squared.to_bits().to_le_bytes());
    hasher.update(correctness.max_pair_distance_squared.to_bits().to_le_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

fn chunks_preserve_slice(execution: &inner::CampaignExecution, config: CampaignConfig) -> bool {
    if execution.chunks.is_empty() || execution.chunks.len() as u64 != execution.memory.chunk_count {
        return false;
    }
    let mut expected_start = config.start;
    let mut total = 0_u64;
    for (ordinal, chunk) in execution.chunks.iter().enumerate() {
        if chunk.ordinal != ordinal as u64
            || chunk.count == 0
            || chunk.count > execution.memory.resident_capacity_cells
            || chunk.start != expected_start
            || chunk.end_exclusive != chunk.start.checked_add(chunk.count).unwrap_or(u64::MAX)
        {
            return false;
        }
        expected_start = chunk.end_exclusive;
        total = match total.checked_add(chunk.count) {
            Some(value) => value,
            None => return false,
        };
    }
    expected_start == config.start.checked_add(config.count).unwrap_or(u64::MAX)
        && total == config.count
}

fn finite_and_bounded(execution: &inner::CampaignExecution, config: CampaignConfig) -> bool {
    let correctness = &execution.correctness;
    let benchmark = &execution.benchmark;
    let memory = &execution.memory;

    let finite_correctness = correctness.min_pair_distance_squared.is_finite()
        && correctness.max_pair_distance_squared.is_finite()
        && correctness.verification_max_geometry_residual.is_finite()
        && correctness.verification_max_pair_residual.is_finite()
        && correctness.verification_tolerance.is_finite()
        && correctness.min_pair_distance_squared >= 0.0
        && correctness.min_pair_distance_squared <= correctness.max_pair_distance_squared;

    let finite_benchmark = benchmark.elapsed_seconds.is_finite()
        && benchmark.conformations_per_second.is_finite()
        && benchmark.elapsed_seconds >= 0.0
        && benchmark.conformations_per_second >= 0.0;

    let memory_bounded = memory.memory_budget_bytes > 0
        && memory.memory_budget_bytes <= MAX_MEMORY_BUDGET_BYTES
        && memory.bytes_per_execution_cell > 0
        && memory.resident_capacity_cells > 0
        && memory.chunk_count > 0
        && memory.chunk_count <= MAX_CAMPAIGN_CHUNKS
        && memory.last_chunk_cells > 0
        && memory.last_chunk_cells <= memory.resident_capacity_cells
        && memory.requested_conformations == config.count
        && memory.padding_excluded_from_scientific_counts;

    finite_correctness
        && finite_benchmark
        && memory_bounded
        && config.count > 0
        && config.requested_workers > 0
        && config.requested_workers <= MAX_WORKERS
        && config.verification_samples > 0
        && config.verification_samples <= MAX_VERIFY_SAMPLES
        && execution.graph.node_count == MEANINGFUL_LANES
        && execution.graph.degree == 5
        && execution.graph.edge_count_undirected == 75
        && execution.layout.warp_width == WARP_WIDTH
        && execution.layout.meaningful_lanes == MEANINGFUL_LANES
        && execution.layout.padding_lanes == PADDING_LANES
        && chunks_preserve_slice(execution, config)
}

#[derive(Debug, Clone, Serialize)]
pub struct Phase3cGateReceipt {
    pub schema: &'static str,
    pub gate_contract: &'static str,
    pub campaign_contract: &'static str,
    pub validation_level: &'static str,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub optimization_contract: &'static str,
    pub numerical_profile: &'static str,
    pub graph_contract: &'static str,
    pub layout_contract: &'static str,
    pub conformation_start: u64,
    pub conformation_count: u64,
    pub conformation_end_exclusive: u64,
    pub correctness_result_sha256: String,
    pub profile_identity_preserved: bool,
    pub algorithm_identity_preserved: bool,
    pub phase3b_residual_gate_passed: bool,
    pub finite_and_bounded: bool,
    pub declared_slice_preserved: bool,
    pub correctness_identity_recomputed: bool,
    pub worker_independent_correctness_identity: bool,
    pub chunk_independent_correctness_identity: bool,
    pub benchmark_timing_excluded_from_correctness_identity: bool,
    pub correctness_identity_included_fields: [&'static str; 14],
    pub correctness_identity_excluded_fields: [&'static str; 6],
    pub implementation_structures_biological_relationships_claimed: bool,
    pub validation_level_promoted_by_runtime: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub inv_bio_001: &'static str,
    pub inv_runtime_001: &'static str,
    pub accepted: bool,
    pub gate_identity_sha256: String,
}

fn gate_identity_sha256(receipt: &Phase3cGateReceipt) -> String {
    let mut hasher = Sha256::new();
    hasher.update(GATE_DOMAIN);
    for value in [
        receipt.gate_contract,
        receipt.campaign_contract,
        receipt.validation_level,
        receipt.model_profile_sha256.as_str(),
        receipt.optimization_profile_sha256.as_str(),
        receipt.optimization_contract,
        receipt.numerical_profile,
        receipt.graph_contract,
        receipt.layout_contract,
        receipt.correctness_result_sha256.as_str(),
        receipt.inv_bio_001,
        receipt.inv_runtime_001,
    ] {
        hasher.update(value.as_bytes());
    }
    hasher.update(receipt.conformation_start.to_le_bytes());
    hasher.update(receipt.conformation_count.to_le_bytes());
    hasher.update(receipt.conformation_end_exclusive.to_le_bytes());
    for value in [
        receipt.profile_identity_preserved,
        receipt.algorithm_identity_preserved,
        receipt.phase3b_residual_gate_passed,
        receipt.finite_and_bounded,
        receipt.declared_slice_preserved,
        receipt.correctness_identity_recomputed,
        receipt.worker_independent_correctness_identity,
        receipt.chunk_independent_correctness_identity,
        receipt.benchmark_timing_excluded_from_correctness_identity,
        receipt.implementation_structures_biological_relationships_claimed,
        receipt.validation_level_promoted_by_runtime,
        receipt.biological_validity_claimed,
        receipt.clinical_validity_claimed,
        receipt.accepted,
    ] {
        hasher.update([u8::from(value)]);
    }
    for field in receipt.correctness_identity_included_fields {
        hasher.update(field.as_bytes());
        hasher.update([0]);
    }
    for field in receipt.correctness_identity_excluded_fields {
        hasher.update(field.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn evaluate_gate(
    engine: &PentaCrtEngine,
    execution: &inner::CampaignExecution,
    config: CampaignConfig,
) -> Result<Phase3cGateReceipt, RuntimeError> {
    let correctness = &execution.correctness;
    let benchmark = &execution.benchmark;
    let expected_end = config
        .start
        .checked_add(config.count)
        .ok_or_else(|| err("Phase 3C gate conformation range overflow"))?;
    let recomputed = recompute_correctness_identity(correctness)?;

    let profile_identity_preserved = correctness.model_profile_sha256 == engine.model_profile_sha256()
        && correctness.optimization_profile_sha256 == engine.optimization_profile_sha256();

    let algorithm_identity_preserved = correctness.optimization_contract == OPTIMIZATION_CONTRACT
        && correctness.numerical_profile == OPTIMIZATION_NUMERICAL_PROFILE
        && correctness.graph_contract == EXECUTION_GRAPH_CONTRACT
        && correctness.layout_contract == MEMORY_LAYOUT_CONTRACT
        && correctness.graph_sha256 == execution.graph.graph_sha256
        && correctness.traversal_sha256 == execution.graph.traversal_sha256;

    let phase3b_residual_gate_passed = correctness.verification_accepted
        && correctness.verification_tolerance.to_bits()
            == BLOCK_REUSE_RESIDUAL_TOLERANCE.to_bits()
        && correctness.verification_max_geometry_residual.is_finite()
        && correctness.verification_max_pair_residual.is_finite()
        && correctness.verification_max_geometry_residual <= BLOCK_REUSE_RESIDUAL_TOLERANCE
        && correctness.verification_max_pair_residual <= BLOCK_REUSE_RESIDUAL_TOLERANCE;

    let bounded = finite_and_bounded(execution, config);

    let declared_slice_preserved = correctness.conformation_start == config.start
        && correctness.conformation_count == config.count
        && correctness.conformation_end_exclusive == expected_end
        && chunks_preserve_slice(execution, config);

    let correctness_identity_recomputed = recomputed == correctness.result_sha256;
    let worker_independent_correctness_identity = correctness_identity_recomputed
        && correctness.result_identity_worker_independent;
    let chunk_independent_correctness_identity = correctness_identity_recomputed
        && correctness.result_identity_chunk_independent;
    let benchmark_timing_excluded_from_correctness_identity = correctness_identity_recomputed
        && !benchmark.identity_bearing_correctness
        && !benchmark.performance_claim;

    let implementation_structures_biological_relationships_claimed = execution.graph.biological_adjacency_claimed
        || execution.layout.padding_lanes_semantic
        || execution.layout.scientific_count_includes_padding;

    let validation_level_promoted_by_runtime = correctness.biological_validity_claimed
        || correctness.clinical_validity_claimed;

    let accepted = profile_identity_preserved
        && algorithm_identity_preserved
        && phase3b_residual_gate_passed
        && bounded
        && declared_slice_preserved
        && correctness_identity_recomputed
        && worker_independent_correctness_identity
        && chunk_independent_correctness_identity
        && benchmark_timing_excluded_from_correctness_identity
        && !implementation_structures_biological_relationships_claimed
        && !validation_level_promoted_by_runtime;

    let mut receipt = Phase3cGateReceipt {
        schema: PHASE3C_GATE_RECEIPT_SCHEMA,
        gate_contract: PHASE3C_GATE_CONTRACT,
        campaign_contract: PHASE3C_CONTRACT,
        validation_level: VALIDATION_LEVEL,
        model_profile_sha256: correctness.model_profile_sha256.clone(),
        optimization_profile_sha256: correctness.optimization_profile_sha256.clone(),
        optimization_contract: OPTIMIZATION_CONTRACT,
        numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
        graph_contract: EXECUTION_GRAPH_CONTRACT,
        layout_contract: MEMORY_LAYOUT_CONTRACT,
        conformation_start: correctness.conformation_start,
        conformation_count: correctness.conformation_count,
        conformation_end_exclusive: correctness.conformation_end_exclusive,
        correctness_result_sha256: correctness.result_sha256.clone(),
        profile_identity_preserved,
        algorithm_identity_preserved,
        phase3b_residual_gate_passed,
        finite_and_bounded: bounded,
        declared_slice_preserved,
        correctness_identity_recomputed,
        worker_independent_correctness_identity,
        chunk_independent_correctness_identity,
        benchmark_timing_excluded_from_correctness_identity,
        correctness_identity_included_fields: CORRECTNESS_INCLUDED_FIELDS,
        correctness_identity_excluded_fields: CORRECTNESS_EXCLUDED_FIELDS,
        implementation_structures_biological_relationships_claimed,
        validation_level_promoted_by_runtime,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
        inv_runtime_001: INV_RUNTIME_001,
        accepted,
        gate_identity_sha256: String::new(),
    };
    receipt.gate_identity_sha256 = gate_identity_sha256(&receipt);
    Ok(receipt)
}

#[derive(Debug)]
pub struct CampaignExecution {
    pub correctness: CorrectnessReceipt,
    pub benchmark: BenchmarkReceipt,
    pub memory: MemoryPlan,
    pub graph: ExecutionGraphReceipt,
    pub layout: MemoryLayoutReceipt,
    pub environment: EnvironmentReceipt,
    pub chunks: Vec<CampaignChunk>,
    pub gate: Phase3cGateReceipt,
    _sealed: (),
}

/// Run a campaign and enforce the complete Phase 3C acceptance gate before an
/// accepted execution object can exist.
pub fn run_campaign(
    engine: &PentaCrtEngine,
    config: CampaignConfig,
) -> Result<CampaignExecution, RuntimeError> {
    let execution = inner::run_campaign(engine, config)?;
    let gate = evaluate_gate(engine, &execution, config)?;
    if !gate.accepted {
        return Err(err(format!(
            "Phase 3C acceptance gate rejected campaign (gate_identity_sha256={})",
            gate.gate_identity_sha256
        )));
    }
    Ok(CampaignExecution {
        correctness: execution.correctness,
        benchmark: execution.benchmark,
        memory: execution.memory,
        graph: execution.graph,
        layout: execution.layout,
        environment: execution.environment,
        chunks: execution.chunks,
        gate,
        _sealed: (),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignManifest {
    pub schema: &'static str,
    pub campaign_contract: &'static str,
    pub gate_contract: &'static str,
    pub gate_identity_sha256: String,
    pub phase3c_gate_artifact_sha256: String,
    pub validation_level: &'static str,
    pub validation_level_promoted_by_runtime: bool,
    pub correctness_result_sha256: String,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub optimization_contract: &'static str,
    pub numerical_profile: &'static str,
    pub graph_contract: &'static str,
    pub layout_contract: &'static str,
    pub graph_sha256: String,
    pub traversal_sha256: String,
    pub requested_workers: usize,
    pub chunk_count: u64,
    pub artifacts: Vec<ArtifactIdentity>,
    pub manifest_sha256: String,
    pub benchmark_identity_is_correctness_identity: bool,
    pub rejected: bool,
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, RuntimeError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| err(format!("cannot serialize Phase 3C artifact: {e}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    if path.exists() {
        return Err(err(format!(
            "refusing to overwrite existing Phase 3C artifact: {}",
            path.display()
        )));
    }
    fs::write(path, bytes)
        .map_err(|e| err(format!("cannot write Phase 3C artifact {}: {e}", path.display())))
}

fn write_json_artifact<T: Serialize>(
    directory: &Path,
    filename: &str,
    role: &str,
    value: &T,
) -> Result<ArtifactIdentity, RuntimeError> {
    let bytes = json_bytes(value)?;
    write_new(&directory.join(filename), &bytes)?;
    Ok(ArtifactIdentity {
        path: filename.to_string(),
        sha256: sha256_hex(&bytes),
        bytes: u64::try_from(bytes.len()).map_err(|_| err("artifact byte length overflow"))?,
        role: role.to_string(),
    })
}

fn validate_gate_for_persistence(execution: &CampaignExecution) -> Result<(), RuntimeError> {
    let gate = &execution.gate;
    if !gate.accepted
        || gate.schema != PHASE3C_GATE_RECEIPT_SCHEMA
        || gate.gate_contract != PHASE3C_GATE_CONTRACT
        || gate.campaign_contract != PHASE3C_CONTRACT
        || gate.validation_level != VALIDATION_LEVEL
        || gate.validation_level_promoted_by_runtime
        || gate.implementation_structures_biological_relationships_claimed
        || gate.correctness_result_sha256 != execution.correctness.result_sha256
        || gate.model_profile_sha256 != execution.correctness.model_profile_sha256
        || gate.optimization_profile_sha256 != execution.correctness.optimization_profile_sha256
        || gate.gate_identity_sha256 != gate_identity_sha256(gate)
        || recompute_correctness_identity(&execution.correctness)? != execution.correctness.result_sha256
    {
        return Err(err("refusing to persist campaign that does not satisfy the Phase 3C gate"));
    }
    Ok(())
}

fn manifest_sha256(manifest: &CampaignManifest) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MANIFEST_DOMAIN);
    for value in [
        manifest.campaign_contract,
        manifest.gate_contract,
        manifest.gate_identity_sha256.as_str(),
        manifest.phase3c_gate_artifact_sha256.as_str(),
        manifest.validation_level,
        manifest.correctness_result_sha256.as_str(),
        manifest.model_profile_sha256.as_str(),
        manifest.optimization_profile_sha256.as_str(),
        manifest.optimization_contract,
        manifest.numerical_profile,
        manifest.graph_contract,
        manifest.layout_contract,
        manifest.graph_sha256.as_str(),
        manifest.traversal_sha256.as_str(),
    ] {
        hasher.update(value.as_bytes());
    }
    hasher.update([u8::from(manifest.validation_level_promoted_by_runtime)]);
    hasher.update((manifest.requested_workers as u64).to_le_bytes());
    hasher.update(manifest.chunk_count.to_le_bytes());
    hasher.update([u8::from(manifest.benchmark_identity_is_correctness_identity)]);
    hasher.update([u8::from(manifest.rejected)]);
    for artifact in &manifest.artifacts {
        hasher.update(artifact.path.as_bytes());
        hasher.update(artifact.sha256.as_bytes());
        hasher.update(artifact.bytes.to_le_bytes());
        hasher.update(artifact.role.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

/// Persist an accepted campaign only after re-checking the explicit gate. The
/// gate receipt itself is identity-bearing and part of the campaign manifest.
pub fn persist_accepted_campaign(
    directory: &Path,
    execution: &CampaignExecution,
) -> Result<CampaignManifest, RuntimeError> {
    validate_gate_for_persistence(execution)?;
    if directory.exists() {
        return Err(err(format!("campaign output directory already exists: {}", directory.display())));
    }
    fs::create_dir_all(directory)
        .map_err(|e| err(format!("cannot create campaign directory {}: {e}", directory.display())))?;

    let mut artifacts = Vec::new();
    artifacts.push(write_json_artifact(directory, "correctness-receipt.json", "correctness", &execution.correctness)?);
    artifacts.push(write_json_artifact(directory, "benchmark-receipt.json", "benchmark-observation", &execution.benchmark)?);
    artifacts.push(write_json_artifact(directory, "execution-graph.json", "execution-graph-and-traversal", &execution.graph)?);
    artifacts.push(write_json_artifact(directory, "memory-layout.json", "gpu-shaped-memory-contract", &execution.layout)?);
    artifacts.push(write_json_artifact(directory, "memory-plan.json", "bounded-memory-plan", &execution.memory)?);
    artifacts.push(write_json_artifact(directory, "environment.json", "privacy-safe-environment-observation", &execution.environment)?);
    artifacts.push(write_json_artifact(directory, "chunks.json", "deterministic-chunk-plan", &execution.chunks)?);
    let gate_artifact = write_json_artifact(directory, "phase3c-gate.json", "phase3c-acceptance-gate", &execution.gate)?;
    let gate_artifact_sha256 = gate_artifact.sha256.clone();
    artifacts.push(gate_artifact);

    let mut manifest = CampaignManifest {
        schema: CAMPAIGN_MANIFEST_SCHEMA,
        campaign_contract: PHASE3C_CONTRACT,
        gate_contract: PHASE3C_GATE_CONTRACT,
        gate_identity_sha256: execution.gate.gate_identity_sha256.clone(),
        phase3c_gate_artifact_sha256: gate_artifact_sha256,
        validation_level: VALIDATION_LEVEL,
        validation_level_promoted_by_runtime: false,
        correctness_result_sha256: execution.correctness.result_sha256.clone(),
        model_profile_sha256: execution.correctness.model_profile_sha256.clone(),
        optimization_profile_sha256: execution.correctness.optimization_profile_sha256.clone(),
        optimization_contract: OPTIMIZATION_CONTRACT,
        numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
        graph_contract: EXECUTION_GRAPH_CONTRACT,
        layout_contract: MEMORY_LAYOUT_CONTRACT,
        graph_sha256: execution.correctness.graph_sha256.clone(),
        traversal_sha256: execution.correctness.traversal_sha256.clone(),
        requested_workers: execution.benchmark.requested_workers,
        chunk_count: execution.memory.chunk_count,
        artifacts: artifacts.clone(),
        manifest_sha256: String::new(),
        benchmark_identity_is_correctness_identity: false,
        rejected: false,
    };
    manifest.manifest_sha256 = manifest_sha256(&manifest);

    let manifest_identity = write_json_artifact(directory, "campaign-manifest.json", "campaign-manifest", &manifest)?;
    artifacts.push(manifest_identity);
    let mut checksum_lines = artifacts
        .iter()
        .map(|artifact| format!("{}  {}", artifact.sha256, artifact.path))
        .collect::<Vec<_>>();
    checksum_lines.sort();
    let mut checksum_bytes = checksum_lines.join("\n").into_bytes();
    checksum_bytes.push(b'\n');
    write_new(&directory.join("SHA256SUMS"), &checksum_bytes)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    use std::path::Path;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        )
        .expect("Phase 3C gate test engine")
    }

    fn config() -> CampaignConfig {
        CampaignConfig {
            start: 100,
            count: 7,
            requested_workers: 2,
            memory_budget_bytes: size_of::<PaddedExecutionCell>() as u64 * 2,
            verification_samples: 17,
        }
    }

    #[test]
    fn explicit_phase3c_gate_accepts_repository_fixture() {
        let engine = engine();
        let execution = run_campaign(&engine, config()).expect("repository fixture must pass gate");
        assert!(execution.gate.accepted);
        assert!(execution.gate.profile_identity_preserved);
        assert!(execution.gate.algorithm_identity_preserved);
        assert!(execution.gate.phase3b_residual_gate_passed);
        assert!(execution.gate.finite_and_bounded);
        assert!(execution.gate.declared_slice_preserved);
        assert!(execution.gate.correctness_identity_recomputed);
        assert!(execution.gate.worker_independent_correctness_identity);
        assert!(execution.gate.chunk_independent_correctness_identity);
        assert!(execution.gate.benchmark_timing_excluded_from_correctness_identity);
        assert!(!execution.gate.implementation_structures_biological_relationships_claimed);
        assert!(!execution.gate.validation_level_promoted_by_runtime);
    }

    #[test]
    fn gate_rejects_forged_worker_independence_claim() {
        let engine = engine();
        let mut execution = inner::run_campaign(&engine, config()).unwrap();
        execution.correctness.result_identity_worker_independent = false;
        let gate = evaluate_gate(&engine, &execution, config()).unwrap();
        assert!(!gate.accepted);
        assert!(!gate.worker_independent_correctness_identity);
    }

    #[test]
    fn gate_rejects_nonfinite_benchmark_even_though_timing_is_non_identity() {
        let engine = engine();
        let mut execution = inner::run_campaign(&engine, config()).unwrap();
        execution.benchmark.elapsed_seconds = f64::INFINITY;
        let gate = evaluate_gate(&engine, &execution, config()).unwrap();
        assert!(!gate.accepted);
        assert!(!gate.finite_and_bounded);
    }
}
