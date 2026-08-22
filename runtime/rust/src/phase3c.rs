// SPDX-License-Identifier: Apache-2.0
//! Phase 3C execution graph, GPU-shaped memory contract, and campaign receipts.
//!
//! This module is scheduling/reproducibility infrastructure for the existing
//! schematic V0 runtime. Execution adjacency is not biological adjacency.

use crate::phase3b::{
    run_penta_crt, verify_penta_crt, PentaCrtEngine, PentaCrtRunConfig,
    OPTIMIZATION_CONTRACT, OPTIMIZATION_NUMERICAL_PROFILE,
};
use crate::{ExecutionAddress, RuntimeError, EXECUTION_CELL_STATES, INV_BIO_001};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::mem::{align_of, size_of};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub const PHASE3C_CONTRACT: &str = "IGM-EXEC-CAMPAIGN-V1";
pub const EXECUTION_GRAPH_CONTRACT: &str = "IGM-EXEC-GRAPH-C5-K2-C3-V1";
pub const TRAVERSAL_RECEIPT_SCHEMA: &str = "IGM-EXEC-TRAVERSAL-RECEIPT-V1";
pub const MEMORY_LAYOUT_CONTRACT: &str = "IGM-WARP32-AOSOA-V1";
pub const MEMORY_PLAN_SCHEMA: &str = "IGM-MEMORY-PLAN-V1";
pub const CORRECTNESS_RECEIPT_SCHEMA: &str = "IGM-CAMPAIGN-CORRECTNESS-RECEIPT-V1";
pub const BENCHMARK_RECEIPT_SCHEMA: &str = "IGM-CAMPAIGN-BENCHMARK-RECEIPT-V1";
pub const ENVIRONMENT_RECEIPT_SCHEMA: &str = "IGM-CAMPAIGN-ENVIRONMENT-V1";
pub const CAMPAIGN_MANIFEST_SCHEMA: &str = "IGM-CAMPAIGN-MANIFEST-V1";
pub const REJECTION_RECEIPT_SCHEMA: &str = "IGM-CAMPAIGN-REJECTION-V1";
pub const INV_RUNTIME_001: &str = "Execution Adjacency Does Not Imply Biological Adjacency";
pub const WARP_WIDTH: usize = 32;
pub const MEANINGFUL_LANES: usize = 30;
pub const PADDING_LANES: usize = 2;
pub const DEFAULT_MEMORY_BUDGET_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_MEMORY_BUDGET_BYTES: u64 = 16 * 1024 * 1024 * 1024;
pub const MAX_CAMPAIGN_CHUNKS: u64 = 1_000_000;

const GRAPH_DOMAIN: &[u8] = b"IGM-EXEC-GRAPH-C5-K2-C3-V1\0";
const TRAVERSAL_DOMAIN: &[u8] = b"IGM-EXEC-TRAVERSAL-RECEIPT-V1\0";
const CORRECTNESS_DOMAIN: &[u8] = b"IGM-CAMPAIGN-CORRECTNESS-V1\0";
const MANIFEST_DOMAIN: &[u8] = b"IGM-CAMPAIGN-MANIFEST-V1\0";

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn parse_hex_u64(value: &str, label: &str) -> Result<u64, RuntimeError> {
    u64::from_str_radix(value, 16)
        .map_err(|e| err(format!("cannot parse {label} as u64 hex: {e}")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionEdgeKind {
    SectorPrevious,
    SectorNext,
    ArmFlip,
    LanePrevious,
    LaneNext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExecutionNeighbor {
    pub kind: ExecutionEdgeKind,
    pub sequence: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionGraphNode {
    pub sequence: u8,
    pub sector: u8,
    pub arm: u8,
    pub lane: u8,
    pub storage_index: u8,
    pub neighbors: [ExecutionNeighbor; 5],
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionGraphReceipt {
    pub schema: &'static str,
    pub graph_contract: &'static str,
    pub semantics: &'static str,
    pub inv_runtime_001: &'static str,
    pub node_count: usize,
    pub degree: usize,
    pub edge_count_undirected: usize,
    pub graph_sha256: String,
    pub traversal_sha256: String,
    pub nodes: Vec<ExecutionGraphNode>,
    pub biological_adjacency_claimed: bool,
}

fn sequence_for(sector: u8, arm: u8, lane: u8) -> Result<u8, RuntimeError> {
    ExecutionAddress { sector, arm, lane }.sequence()
}

pub fn execution_graph_nodes() -> Result<Vec<ExecutionGraphNode>, RuntimeError> {
    let mut nodes = Vec::with_capacity(MEANINGFUL_LANES);
    for sequence in 0..EXECUTION_CELL_STATES {
        let address = ExecutionAddress::from_sequence(sequence)?;
        let previous_sector = (address.sector + 4) % 5;
        let next_sector = (address.sector + 1) % 5;
        let flipped_arm = 1 - address.arm;
        let previous_lane = (address.lane + 2) % 3;
        let next_lane = (address.lane + 1) % 3;
        let neighbors = [
            ExecutionNeighbor {
                kind: ExecutionEdgeKind::SectorPrevious,
                sequence: sequence_for(previous_sector, address.arm, address.lane)?,
            },
            ExecutionNeighbor {
                kind: ExecutionEdgeKind::SectorNext,
                sequence: sequence_for(next_sector, address.arm, address.lane)?,
            },
            ExecutionNeighbor {
                kind: ExecutionEdgeKind::ArmFlip,
                sequence: sequence_for(address.sector, flipped_arm, address.lane)?,
            },
            ExecutionNeighbor {
                kind: ExecutionEdgeKind::LanePrevious,
                sequence: sequence_for(address.sector, address.arm, previous_lane)?,
            },
            ExecutionNeighbor {
                kind: ExecutionEdgeKind::LaneNext,
                sequence: sequence_for(address.sector, address.arm, next_lane)?,
            },
        ];
        nodes.push(ExecutionGraphNode {
            sequence,
            sector: address.sector,
            arm: address.arm,
            lane: address.lane,
            storage_index: address.storage_index()?,
            neighbors,
        });
    }
    Ok(nodes)
}

fn traversal_sha256(nodes: &[ExecutionGraphNode]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(TRAVERSAL_DOMAIN);
    for node in nodes {
        hasher.update([node.sequence, node.sector, node.arm, node.lane, node.storage_index]);
    }
    format!("{:x}", hasher.finalize())
}

fn graph_sha256(nodes: &[ExecutionGraphNode]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(GRAPH_DOMAIN);
    for node in nodes {
        hasher.update([node.sequence, node.sector, node.arm, node.lane, node.storage_index]);
        for neighbor in node.neighbors {
            hasher.update([neighbor.kind as u8, neighbor.sequence]);
        }
    }
    format!("{:x}", hasher.finalize())
}

pub fn execution_graph_receipt() -> Result<ExecutionGraphReceipt, RuntimeError> {
    let nodes = execution_graph_nodes()?;
    // C5 contributes degree 2, K2 degree 1, C3 degree 2: total degree 5.
    let degree = 5usize;
    let edge_count_undirected = nodes
        .len()
        .checked_mul(degree)
        .and_then(|value| value.checked_div(2))
        .ok_or_else(|| err("execution graph edge count overflow"))?;
    Ok(ExecutionGraphReceipt {
        schema: TRAVERSAL_RECEIPT_SCHEMA,
        graph_contract: EXECUTION_GRAPH_CONTRACT,
        semantics: "scheduling/execution graph only; not an IgM biological/model/provenance graph",
        inv_runtime_001: INV_RUNTIME_001,
        node_count: nodes.len(),
        degree,
        edge_count_undirected,
        graph_sha256: graph_sha256(&nodes),
        traversal_sha256: traversal_sha256(&nodes),
        nodes,
        biological_adjacency_claimed: false,
    })
}

/// GPU-shaped fixed-width execution cell. This is a scheduling/memory object,
/// not a molecular or biological object.
#[repr(C, align(128))]
#[derive(Clone)]
pub struct PaddedExecutionCell {
    pub active: [u8; WARP_WIDTH],
    pub sector: [u8; WARP_WIDTH],
    pub arm: [u8; WARP_WIDTH],
    pub lane: [u8; WARP_WIDTH],
    pub storage_index: [u8; WARP_WIDTH],
    pub reserved: [u8; WARP_WIDTH],
    pub value: [f64; WARP_WIDTH],
}

const _: () = {
    assert!(align_of::<PaddedExecutionCell>() == 128);
    assert!(size_of::<PaddedExecutionCell>() % 128 == 0);
};

impl PaddedExecutionCell {
    pub fn new() -> Result<Self, RuntimeError> {
        let mut cell = Self {
            active: [0; WARP_WIDTH],
            sector: [u8::MAX; WARP_WIDTH],
            arm: [u8::MAX; WARP_WIDTH],
            lane: [u8::MAX; WARP_WIDTH],
            storage_index: [u8::MAX; WARP_WIDTH],
            reserved: [0; WARP_WIDTH],
            value: [0.0; WARP_WIDTH],
        };
        for sequence in 0..EXECUTION_CELL_STATES {
            let index = usize::from(sequence);
            let address = ExecutionAddress::from_sequence(sequence)?;
            cell.active[index] = 1;
            cell.sector[index] = address.sector;
            cell.arm[index] = address.arm;
            cell.lane[index] = address.lane;
            cell.storage_index[index] = address.storage_index()?;
        }
        Ok(cell)
    }

    pub fn meaningful_lane_count(&self) -> usize {
        self.active.iter().map(|value| usize::from(*value)).sum()
    }

    pub fn padding_lane_count(&self) -> usize {
        WARP_WIDTH - self.meaningful_lane_count()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryLayoutReceipt {
    pub schema: &'static str,
    pub layout_contract: &'static str,
    pub warp_width: usize,
    pub meaningful_lanes: usize,
    pub padding_lanes: usize,
    pub padding_lanes_semantic: bool,
    pub cell_size_bytes: usize,
    pub cell_alignment_bytes: usize,
    pub active_lane_count_observed: usize,
    pub padding_lane_count_observed: usize,
    pub scientific_count_includes_padding: bool,
}

pub fn memory_layout_receipt() -> Result<MemoryLayoutReceipt, RuntimeError> {
    let cell = PaddedExecutionCell::new()?;
    Ok(MemoryLayoutReceipt {
        schema: "IGM-MEMORY-LAYOUT-RECEIPT-V1",
        layout_contract: MEMORY_LAYOUT_CONTRACT,
        warp_width: WARP_WIDTH,
        meaningful_lanes: MEANINGFUL_LANES,
        padding_lanes: PADDING_LANES,
        padding_lanes_semantic: false,
        cell_size_bytes: size_of::<PaddedExecutionCell>(),
        cell_alignment_bytes: align_of::<PaddedExecutionCell>(),
        active_lane_count_observed: cell.meaningful_lane_count(),
        padding_lane_count_observed: cell.padding_lane_count(),
        scientific_count_includes_padding: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CampaignChunk {
    pub ordinal: u64,
    pub start: u64,
    pub count: u64,
    pub end_exclusive: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryPlan {
    pub schema: &'static str,
    pub layout_contract: &'static str,
    pub requested_conformations: u64,
    pub memory_budget_bytes: u64,
    pub bytes_per_execution_cell: u64,
    pub resident_capacity_cells: u64,
    pub chunk_count: u64,
    pub last_chunk_cells: u64,
    pub meaningful_lanes_per_cell: usize,
    pub padding_lanes_per_cell: usize,
    pub padding_excluded_from_scientific_counts: bool,
}

pub fn plan_memory(requested_conformations: u64, memory_budget_bytes: u64) -> Result<MemoryPlan, RuntimeError> {
    if requested_conformations == 0 {
        return Err(err("memory plan requires at least one conformation"));
    }
    if memory_budget_bytes == 0 || memory_budget_bytes > MAX_MEMORY_BUDGET_BYTES {
        return Err(err("memory budget outside Phase 3C bound"));
    }
    let bytes_per_execution_cell = u64::try_from(size_of::<PaddedExecutionCell>())
        .map_err(|_| err("execution-cell size does not fit u64"))?;
    let resident_capacity_cells = memory_budget_bytes / bytes_per_execution_cell;
    if resident_capacity_cells == 0 {
        return Err(err(format!(
            "memory budget {memory_budget_bytes} is smaller than one execution cell ({bytes_per_execution_cell} bytes)"
        )));
    }
    let chunk_count = requested_conformations
        .checked_add(resident_capacity_cells - 1)
        .ok_or_else(|| err("memory-plan chunk rounding overflow"))?
        / resident_capacity_cells;
    if chunk_count > MAX_CAMPAIGN_CHUNKS {
        return Err(err("memory plan would exceed maximum bounded chunk count"));
    }
    let consumed_before_last = resident_capacity_cells
        .checked_mul(chunk_count.saturating_sub(1))
        .ok_or_else(|| err("memory-plan consumed count overflow"))?;
    let last_chunk_cells = requested_conformations
        .checked_sub(consumed_before_last)
        .ok_or_else(|| err("memory-plan last-chunk underflow"))?;
    Ok(MemoryPlan {
        schema: MEMORY_PLAN_SCHEMA,
        layout_contract: MEMORY_LAYOUT_CONTRACT,
        requested_conformations,
        memory_budget_bytes,
        bytes_per_execution_cell,
        resident_capacity_cells,
        chunk_count,
        last_chunk_cells,
        meaningful_lanes_per_cell: MEANINGFUL_LANES,
        padding_lanes_per_cell: PADDING_LANES,
        padding_excluded_from_scientific_counts: true,
    })
}

pub fn chunk_plan(start: u64, count: u64, memory: &MemoryPlan) -> Result<Vec<CampaignChunk>, RuntimeError> {
    if count == 0 || count != memory.requested_conformations {
        return Err(err("chunk plan count must match memory plan requested_conformations"));
    }
    let mut chunks = Vec::with_capacity(
        usize::try_from(memory.chunk_count).map_err(|_| err("chunk count does not fit usize"))?,
    );
    let mut cursor = 0_u64;
    let mut ordinal = 0_u64;
    while cursor < count {
        let remaining = count - cursor;
        let chunk_count = remaining.min(memory.resident_capacity_cells);
        let absolute_start = start
            .checked_add(cursor)
            .ok_or_else(|| err("campaign chunk absolute start overflow"))?;
        let end_exclusive = absolute_start
            .checked_add(chunk_count)
            .ok_or_else(|| err("campaign chunk end overflow"))?;
        chunks.push(CampaignChunk {
            ordinal,
            start: absolute_start,
            count: chunk_count,
            end_exclusive,
        });
        cursor = cursor
            .checked_add(chunk_count)
            .ok_or_else(|| err("campaign chunk cursor overflow"))?;
        ordinal += 1;
    }
    if chunks.len() as u64 != memory.chunk_count {
        return Err(err("chunk plan count disagrees with memory plan"));
    }
    Ok(chunks)
}

#[derive(Debug, Clone, Copy)]
pub struct CampaignConfig {
    pub start: u64,
    pub count: u64,
    pub requested_workers: usize,
    pub memory_budget_bytes: u64,
    pub verification_samples: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrectnessReceipt {
    pub schema: &'static str,
    pub campaign_contract: &'static str,
    pub optimization_contract: &'static str,
    pub numerical_profile: &'static str,
    pub graph_contract: &'static str,
    pub graph_sha256: String,
    pub traversal_sha256: String,
    pub layout_contract: &'static str,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub conformation_start: u64,
    pub conformation_count: u64,
    pub conformation_end_exclusive: u64,
    pub logical_pair_checks: u64,
    pub structured_distance_evaluations: u64,
    pub exact_z_residual_corrections: u64,
    pub diagnostic_xor_fnv1a64: String,
    pub min_pair_distance_squared: f64,
    pub max_pair_distance_squared: f64,
    pub verification_samples: usize,
    pub verification_max_geometry_residual: f64,
    pub verification_max_pair_residual: f64,
    pub verification_tolerance: f64,
    pub verification_accepted: bool,
    pub result_sha256: String,
    pub result_identity_worker_independent: bool,
    pub result_identity_chunk_independent: bool,
    pub inv_bio_001: &'static str,
    pub inv_runtime_001: &'static str,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkReceipt {
    pub schema: &'static str,
    pub campaign_contract: &'static str,
    pub elapsed_seconds: f64,
    pub conformations_per_second: f64,
    pub requested_workers: usize,
    pub memory_budget_bytes: u64,
    pub resident_capacity_cells: u64,
    pub chunk_count: u64,
    pub identity_bearing_correctness: bool,
    pub performance_claim: bool,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvironmentReceipt {
    pub schema: &'static str,
    pub os_family: &'static str,
    pub architecture: &'static str,
    pub rustc_version: Option<String>,
    pub cargo_version: Option<String>,
    pub available_parallelism: usize,
    pub hostname_included: bool,
    pub username_included: bool,
    pub raw_hardware_identifiers_included: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignExecution {
    pub correctness: CorrectnessReceipt,
    pub benchmark: BenchmarkReceipt,
    pub memory: MemoryPlan,
    pub graph: ExecutionGraphReceipt,
    pub layout: MemoryLayoutReceipt,
    pub environment: EnvironmentReceipt,
    pub chunks: Vec<CampaignChunk>,
}

fn command_version(program: &str) -> Option<String> {
    let output = Command::new(program).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn environment_receipt() -> EnvironmentReceipt {
    EnvironmentReceipt {
        schema: ENVIRONMENT_RECEIPT_SCHEMA,
        os_family: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        rustc_version: command_version("rustc"),
        cargo_version: command_version("cargo"),
        available_parallelism: std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1),
        hostname_included: false,
        username_included: false,
        raw_hardware_identifiers_included: false,
    }
}

fn correctness_sha256(
    engine: &PentaCrtEngine,
    graph: &ExecutionGraphReceipt,
    start: u64,
    count: u64,
    diagnostic: u64,
    min_d2: f64,
    max_d2: f64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(CORRECTNESS_DOMAIN);
    hasher.update(OPTIMIZATION_CONTRACT.as_bytes());
    hasher.update(OPTIMIZATION_NUMERICAL_PROFILE.as_bytes());
    hasher.update(EXECUTION_GRAPH_CONTRACT.as_bytes());
    hasher.update(graph.graph_sha256.as_bytes());
    hasher.update(graph.traversal_sha256.as_bytes());
    hasher.update(MEMORY_LAYOUT_CONTRACT.as_bytes());
    hasher.update(engine.model_profile_sha256().as_bytes());
    hasher.update(engine.optimization_profile_sha256().as_bytes());
    hasher.update(start.to_le_bytes());
    hasher.update(count.to_le_bytes());
    hasher.update(diagnostic.to_le_bytes());
    hasher.update(min_d2.to_bits().to_le_bytes());
    hasher.update(max_d2.to_bits().to_le_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn run_campaign(engine: &PentaCrtEngine, config: CampaignConfig) -> Result<CampaignExecution, RuntimeError> {
    if config.count == 0 {
        return Err(err("campaign conformation count must be positive"));
    }
    let end_exclusive = config
        .start
        .checked_add(config.count)
        .ok_or_else(|| err("campaign conformation range overflow"))?;
    if config.start >= engine.total_conformations() || end_exclusive > engine.total_conformations() {
        return Err(err("campaign slice exceeds PENTA-CRT execution domain"));
    }

    let verification = verify_penta_crt(engine, config.verification_samples)?;
    if !verification.accepted {
        return Err(err(format!(
            "campaign rejected: Phase 3B verification residual gate failed (geometry={}, pairs={}, tolerance={})",
            verification.max_geometry_lut_vs_reference_residual,
            verification.max_block_reuse_vs_brute_pair_residual,
            verification.residual_tolerance
        )));
    }

    let graph = execution_graph_receipt()?;
    let layout = memory_layout_receipt()?;
    if graph.node_count != MEANINGFUL_LANES || layout.active_lane_count_observed != MEANINGFUL_LANES {
        return Err(err("execution graph / padded layout meaningful-lane invariant failed"));
    }
    let memory = plan_memory(config.count, config.memory_budget_bytes)?;
    let chunks = chunk_plan(config.start, config.count, &memory)?;

    let started = Instant::now();
    let mut diagnostic = 0_u64;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    let mut logical_pair_checks = 0_u64;
    let mut structured_distance_evaluations = 0_u64;
    let mut exact_z_residual_corrections = 0_u64;

    for chunk in &chunks {
        let summary = run_penta_crt(
            engine,
            PentaCrtRunConfig::new(
                chunk.start,
                chunk.count,
                config.requested_workers,
                engine.total_conformations(),
            )?,
        )?;
        diagnostic ^= parse_hex_u64(&summary.diagnostic_xor_fnv1a64, "chunk diagnostic")?;
        min_d2 = min_d2.min(summary.min_pair_distance_squared);
        max_d2 = max_d2.max(summary.max_pair_distance_squared);
        logical_pair_checks = logical_pair_checks
            .checked_add(summary.logical_pair_checks)
            .ok_or_else(|| err("campaign logical-pair count overflow"))?;
        structured_distance_evaluations = structured_distance_evaluations
            .checked_add(summary.structured_distance_evaluations)
            .ok_or_else(|| err("campaign structured-evaluation count overflow"))?;
        exact_z_residual_corrections = exact_z_residual_corrections
            .checked_add(summary.exact_z_residual_corrections)
            .ok_or_else(|| err("campaign Z-correction count overflow"))?;
    }
    let elapsed_seconds = started.elapsed().as_secs_f64();
    if !min_d2.is_finite() || !max_d2.is_finite() || !elapsed_seconds.is_finite() {
        return Err(err("campaign produced non-finite summary"));
    }
    let conformations_per_second = if elapsed_seconds > 0.0 {
        config.count as f64 / elapsed_seconds
    } else {
        0.0
    };
    let result_sha256 = correctness_sha256(
        engine,
        &graph,
        config.start,
        config.count,
        diagnostic,
        min_d2,
        max_d2,
    );

    Ok(CampaignExecution {
        correctness: CorrectnessReceipt {
            schema: CORRECTNESS_RECEIPT_SCHEMA,
            campaign_contract: PHASE3C_CONTRACT,
            optimization_contract: OPTIMIZATION_CONTRACT,
            numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
            graph_contract: EXECUTION_GRAPH_CONTRACT,
            graph_sha256: graph.graph_sha256.clone(),
            traversal_sha256: graph.traversal_sha256.clone(),
            layout_contract: MEMORY_LAYOUT_CONTRACT,
            model_profile_sha256: engine.model_profile_sha256().to_string(),
            optimization_profile_sha256: engine.optimization_profile_sha256().to_string(),
            conformation_start: config.start,
            conformation_count: config.count,
            conformation_end_exclusive: end_exclusive,
            logical_pair_checks,
            structured_distance_evaluations,
            exact_z_residual_corrections,
            diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
            min_pair_distance_squared: min_d2,
            max_pair_distance_squared: max_d2,
            verification_samples: verification.samples_checked,
            verification_max_geometry_residual: verification.max_geometry_lut_vs_reference_residual,
            verification_max_pair_residual: verification.max_block_reuse_vs_brute_pair_residual,
            verification_tolerance: verification.residual_tolerance,
            verification_accepted: verification.accepted,
            result_sha256,
            result_identity_worker_independent: true,
            result_identity_chunk_independent: true,
            inv_bio_001: INV_BIO_001,
            inv_runtime_001: INV_RUNTIME_001,
            biological_validity_claimed: false,
            clinical_validity_claimed: false,
        },
        benchmark: BenchmarkReceipt {
            schema: BENCHMARK_RECEIPT_SCHEMA,
            campaign_contract: PHASE3C_CONTRACT,
            elapsed_seconds,
            conformations_per_second,
            requested_workers: config.requested_workers,
            memory_budget_bytes: config.memory_budget_bytes,
            resident_capacity_cells: memory.resident_capacity_cells,
            chunk_count: memory.chunk_count,
            identity_bearing_correctness: false,
            performance_claim: false,
            note: "Local timing is a benchmark observation only; it is excluded from correctness identity and establishes no biological or clinical claim.",
        },
        memory,
        graph,
        layout,
        environment: environment_receipt(),
        chunks,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactIdentity {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub role: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignManifest {
    pub schema: &'static str,
    pub campaign_contract: &'static str,
    pub correctness_result_sha256: String,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub graph_sha256: String,
    pub traversal_sha256: String,
    pub requested_workers: usize,
    pub chunk_count: u64,
    pub artifacts: Vec<ArtifactIdentity>,
    pub manifest_sha256: String,
    pub benchmark_identity_is_correctness_identity: bool,
    pub rejected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectionReceipt {
    pub schema: &'static str,
    pub campaign_contract: &'static str,
    pub stage: String,
    pub reason: String,
    pub preserved: bool,
    pub non_clinical: bool,
    pub biological_validity_claimed: bool,
    pub inv_bio_001: &'static str,
    pub inv_runtime_001: &'static str,
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, RuntimeError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| err(format!("cannot serialize campaign artifact: {e}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), RuntimeError> {
    if path.exists() {
        return Err(err(format!("refusing to overwrite existing campaign artifact: {}", path.display())));
    }
    fs::write(path, bytes)
        .map_err(|e| err(format!("cannot write campaign artifact {}: {e}", path.display())))
}

fn write_json_artifact<T: Serialize>(
    directory: &Path,
    filename: &str,
    role: &str,
    value: &T,
) -> Result<ArtifactIdentity, RuntimeError> {
    let bytes = json_bytes(value)?;
    let path = directory.join(filename);
    write_new(&path, &bytes)?;
    Ok(ArtifactIdentity {
        path: filename.to_string(),
        sha256: sha256_hex(&bytes),
        bytes: u64::try_from(bytes.len()).map_err(|_| err("artifact byte length overflow"))?,
        role: role.to_string(),
    })
}

fn campaign_manifest_sha256(
    execution: &CampaignExecution,
    artifacts: &[ArtifactIdentity],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MANIFEST_DOMAIN);
    hasher.update(execution.correctness.result_sha256.as_bytes());
    hasher.update(execution.correctness.model_profile_sha256.as_bytes());
    hasher.update(execution.correctness.optimization_profile_sha256.as_bytes());
    hasher.update(execution.correctness.graph_sha256.as_bytes());
    hasher.update(execution.correctness.traversal_sha256.as_bytes());
    hasher.update((execution.benchmark.requested_workers as u64).to_le_bytes());
    hasher.update(execution.memory.chunk_count.to_le_bytes());
    for artifact in artifacts {
        hasher.update(artifact.path.as_bytes());
        hasher.update(artifact.sha256.as_bytes());
        hasher.update(artifact.bytes.to_le_bytes());
        hasher.update(artifact.role.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn persist_accepted_campaign(directory: &Path, execution: &CampaignExecution) -> Result<CampaignManifest, RuntimeError> {
    if directory.exists() {
        return Err(err(format!("campaign output directory already exists: {}", directory.display())));
    }
    fs::create_dir_all(directory)
        .map_err(|e| err(format!("cannot create campaign directory {}: {e}", directory.display())))?;

    let mut artifacts = Vec::new();
    artifacts.push(write_json_artifact(
        directory,
        "correctness-receipt.json",
        "correctness",
        &execution.correctness,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "benchmark-receipt.json",
        "benchmark-observation",
        &execution.benchmark,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "execution-graph.json",
        "execution-graph-and-traversal",
        &execution.graph,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "memory-layout.json",
        "gpu-shaped-memory-contract",
        &execution.layout,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "memory-plan.json",
        "bounded-memory-plan",
        &execution.memory,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "environment.json",
        "privacy-safe-environment-observation",
        &execution.environment,
    )?);
    artifacts.push(write_json_artifact(
        directory,
        "chunks.json",
        "deterministic-chunk-plan",
        &execution.chunks,
    )?);

    let manifest_sha256 = campaign_manifest_sha256(execution, &artifacts);
    let manifest = CampaignManifest {
        schema: CAMPAIGN_MANIFEST_SCHEMA,
        campaign_contract: PHASE3C_CONTRACT,
        correctness_result_sha256: execution.correctness.result_sha256.clone(),
        model_profile_sha256: execution.correctness.model_profile_sha256.clone(),
        optimization_profile_sha256: execution.correctness.optimization_profile_sha256.clone(),
        graph_sha256: execution.correctness.graph_sha256.clone(),
        traversal_sha256: execution.correctness.traversal_sha256.clone(),
        requested_workers: execution.benchmark.requested_workers,
        chunk_count: execution.memory.chunk_count,
        artifacts: artifacts.clone(),
        manifest_sha256,
        benchmark_identity_is_correctness_identity: false,
        rejected: false,
    };
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

pub fn persist_rejected_campaign(
    directory: &Path,
    stage: impl Into<String>,
    reason: impl Into<String>,
) -> Result<RejectionReceipt, RuntimeError> {
    if directory.exists() {
        return Err(err(format!("campaign output directory already exists: {}", directory.display())));
    }
    fs::create_dir_all(directory)
        .map_err(|e| err(format!("cannot create rejected campaign directory {}: {e}", directory.display())))?;
    let receipt = RejectionReceipt {
        schema: REJECTION_RECEIPT_SCHEMA,
        campaign_contract: PHASE3C_CONTRACT,
        stage: stage.into(),
        reason: reason.into(),
        preserved: true,
        non_clinical: true,
        biological_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
        inv_runtime_001: INV_RUNTIME_001,
    };
    let artifact = write_json_artifact(directory, "rejected.json", "rejected-campaign", &receipt)?;
    let line = format!("{}  {}\n", artifact.sha256, artifact.path);
    write_new(&directory.join("SHA256SUMS"), line.as_bytes())?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        )
        .expect("Phase 3C test engine")
    }

    #[test]
    fn execution_graph_is_exact_regular_product_graph() {
        let receipt = execution_graph_receipt().unwrap();
        assert_eq!(receipt.node_count, 30);
        assert_eq!(receipt.degree, 5);
        assert_eq!(receipt.edge_count_undirected, 75);
        let sequences: BTreeSet<_> = receipt.nodes.iter().map(|node| node.sequence).collect();
        assert_eq!(sequences.len(), 30);
        for node in &receipt.nodes {
            let neighbors: BTreeSet<_> = node.neighbors.iter().map(|edge| edge.sequence).collect();
            assert_eq!(neighbors.len(), 5);
            assert!(!neighbors.contains(&node.sequence));
        }
    }

    #[test]
    fn padded_cell_has_exact_30_plus_2_lane_contract() {
        let cell = PaddedExecutionCell::new().unwrap();
        assert_eq!(align_of::<PaddedExecutionCell>(), 128);
        assert_eq!(size_of::<PaddedExecutionCell>() % 128, 0);
        assert_eq!(cell.meaningful_lane_count(), 30);
        assert_eq!(cell.padding_lane_count(), 2);
        assert_eq!(cell.active[30], 0);
        assert_eq!(cell.active[31], 0);
        assert_eq!(cell.sector[30], u8::MAX);
        assert_eq!(cell.sector[31], u8::MAX);
    }

    #[test]
    fn memory_plan_streams_when_budget_is_smaller_than_campaign() {
        let bytes = size_of::<PaddedExecutionCell>() as u64;
        let plan = plan_memory(257, bytes * 4).unwrap();
        assert_eq!(plan.resident_capacity_cells, 4);
        assert_eq!(plan.chunk_count, 65);
        let chunks = chunk_plan(100, 257, &plan).unwrap();
        assert_eq!(chunks.first().unwrap().start, 100);
        assert_eq!(chunks.last().unwrap().end_exclusive, 357);
        assert_eq!(chunks.iter().map(|chunk| chunk.count).sum::<u64>(), 257);
    }

    #[test]
    fn correctness_identity_is_independent_of_workers_and_chunk_budget() {
        let engine = engine();
        let cell = size_of::<PaddedExecutionCell>() as u64;
        let one = run_campaign(
            &engine,
            CampaignConfig {
                start: 100,
                count: 31,
                requested_workers: 1,
                memory_budget_bytes: cell * 2,
                verification_samples: 17,
            },
        )
        .unwrap();
        let seven = run_campaign(
            &engine,
            CampaignConfig {
                start: 100,
                count: 31,
                requested_workers: 7,
                memory_budget_bytes: cell * 31,
                verification_samples: 17,
            },
        )
        .unwrap();
        assert_eq!(one.correctness.result_sha256, seven.correctness.result_sha256);
        assert_eq!(
            one.correctness.diagnostic_xor_fnv1a64,
            seven.correctness.diagnostic_xor_fnv1a64
        );
        assert_eq!(
            one.correctness.min_pair_distance_squared.to_bits(),
            seven.correctness.min_pair_distance_squared.to_bits()
        );
        assert_eq!(
            one.correctness.max_pair_distance_squared.to_bits(),
            seven.correctness.max_pair_distance_squared.to_bits()
        );
        assert_ne!(one.memory.chunk_count, seven.memory.chunk_count);
    }

    #[test]
    fn too_small_memory_budget_fails_closed() {
        let cell = size_of::<PaddedExecutionCell>() as u64;
        assert!(plan_memory(10, cell - 1).is_err());
    }
}
