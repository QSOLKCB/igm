// SPDX-License-Identifier: Apache-2.0
//! Hardened public Phase 3C boundary.
//!
//! Memory and chunk admission are deliberately checked before the potentially
//! expensive Phase 3B reference-verification workload. The inner implementation
//! remains responsible for repeating its own checks before execution.

#[path = "phase3c.rs"]
mod inner;

pub use inner::{
    chunk_plan, environment_receipt, execution_graph_nodes, execution_graph_receipt,
    memory_layout_receipt, persist_accepted_campaign, persist_rejected_campaign, plan_memory,
    ArtifactIdentity, BenchmarkReceipt, CampaignChunk, CampaignConfig, CampaignExecution,
    CampaignManifest, CorrectnessReceipt, EnvironmentReceipt, ExecutionEdgeKind,
    ExecutionGraphNode, ExecutionGraphReceipt, ExecutionNeighbor, MemoryLayoutReceipt,
    MemoryPlan, PaddedExecutionCell, RejectionReceipt, BENCHMARK_RECEIPT_SCHEMA,
    CAMPAIGN_MANIFEST_SCHEMA, CORRECTNESS_RECEIPT_SCHEMA, DEFAULT_MEMORY_BUDGET_BYTES,
    ENVIRONMENT_RECEIPT_SCHEMA, EXECUTION_GRAPH_CONTRACT, INV_RUNTIME_001,
    MAX_CAMPAIGN_CHUNKS, MAX_MEMORY_BUDGET_BYTES, MEANINGFUL_LANES, MEMORY_LAYOUT_CONTRACT,
    MEMORY_PLAN_SCHEMA, PADDING_LANES, PHASE3C_CONTRACT, REJECTION_RECEIPT_SCHEMA,
    TRAVERSAL_RECEIPT_SCHEMA, WARP_WIDTH,
};

use crate::phase3b::PentaCrtEngine;
use crate::RuntimeError;

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

/// Execute an admitted Phase 3C campaign.
///
/// The public boundary validates the conformation range, resident-memory budget,
/// and deterministic chunk plan before invoking the inner verification path.
/// This prevents a campaign that is already known to be inadmissible from
/// spending compute on reference verification first.
pub fn run_campaign(
    engine: &PentaCrtEngine,
    config: CampaignConfig,
) -> Result<CampaignExecution, RuntimeError> {
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

    let memory = inner::plan_memory(config.count, config.memory_budget_bytes)?;
    let chunks = inner::chunk_plan(config.start, config.count, &memory)?;
    if chunks.len() as u64 != memory.chunk_count {
        return Err(err("campaign chunk preflight disagrees with memory plan"));
    }

    inner::run_campaign(engine, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase3b::PentaCrtEngine;
    use std::mem::size_of;
    use std::path::Path;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        )
        .expect("Phase 3C test engine")
    }

    #[test]
    fn public_campaign_boundary_rejects_memory_before_inner_execution() {
        let engine = engine();
        let cell = size_of::<PaddedExecutionCell>() as u64;
        let error = run_campaign(
            &engine,
            CampaignConfig {
                start: 0,
                count: 1,
                requested_workers: 1,
                memory_budget_bytes: cell - 1,
                verification_samples: 17,
            },
        )
        .expect_err("sub-cell memory budget must fail at preflight");
        assert!(error.to_string().contains("smaller than one execution cell"));
    }
}
