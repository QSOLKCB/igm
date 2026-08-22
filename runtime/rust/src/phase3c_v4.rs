// SPDX-License-Identifier: Apache-2.0
//! Sealed public Phase 3C acceptance boundary.
//!
//! `phase3c_v3` evaluates the executable Phase 3C gate. This wrapper makes the
//! resulting accepted campaign immutable to downstream library callers: all
//! receipt access is read-only and there is no mutable accessor or escape hatch.
//! Persistence therefore receives exactly the receipt set that passed the gate,
//! preventing post-gate claim or topology mutation before accepted artifacts are
//! written.

#[path = "phase3c_v3.rs"]
mod inner;

pub use inner::{
    chunk_plan, environment_receipt, execution_graph_nodes, execution_graph_receipt,
    memory_layout_receipt, persist_rejected_campaign, plan_memory, ArtifactIdentity,
    BenchmarkReceipt, CampaignChunk, CampaignConfig, CampaignManifest, CorrectnessReceipt,
    EnvironmentReceipt, ExecutionEdgeKind, ExecutionGraphNode, ExecutionGraphReceipt,
    ExecutionNeighbor, MemoryLayoutReceipt, MemoryPlan, PaddedExecutionCell,
    Phase3cGateReceipt, RejectionReceipt, BENCHMARK_RECEIPT_SCHEMA,
    CAMPAIGN_MANIFEST_SCHEMA, CORRECTNESS_RECEIPT_SCHEMA, DEFAULT_MEMORY_BUDGET_BYTES,
    ENVIRONMENT_RECEIPT_SCHEMA, EXECUTION_GRAPH_CONTRACT, INV_RUNTIME_001,
    MAX_CAMPAIGN_CHUNKS, MAX_MEMORY_BUDGET_BYTES, MEANINGFUL_LANES, MEMORY_LAYOUT_CONTRACT,
    MEMORY_PLAN_SCHEMA, PADDING_LANES, PHASE3C_CONTRACT, PHASE3C_GATE_CONTRACT,
    PHASE3C_GATE_RECEIPT_SCHEMA, REJECTION_RECEIPT_SCHEMA, TRAVERSAL_RECEIPT_SCHEMA,
    VALIDATION_LEVEL, WARP_WIDTH,
};

use crate::phase3b::PentaCrtEngine;
use crate::RuntimeError;
use std::path::Path;

/// Accepted Phase 3C campaign handle.
///
/// The inner receipt set is intentionally private. Callers can inspect receipts
/// through immutable accessors but cannot mutate correctness claims, execution
/// topology, layout semantics, gate predicates, or benchmark metadata after the
/// acceptance gate has run.
#[derive(Debug)]
pub struct CampaignExecution {
    inner: inner::CampaignExecution,
}

impl CampaignExecution {
    pub fn correctness(&self) -> &CorrectnessReceipt {
        &self.inner.correctness
    }

    pub fn benchmark(&self) -> &BenchmarkReceipt {
        &self.inner.benchmark
    }

    pub fn memory(&self) -> &MemoryPlan {
        &self.inner.memory
    }

    pub fn graph(&self) -> &ExecutionGraphReceipt {
        &self.inner.graph
    }

    pub fn layout(&self) -> &MemoryLayoutReceipt {
        &self.inner.layout
    }

    pub fn environment(&self) -> &EnvironmentReceipt {
        &self.inner.environment
    }

    pub fn chunks(&self) -> &[CampaignChunk] {
        &self.inner.chunks
    }

    pub fn gate(&self) -> &Phase3cGateReceipt {
        &self.inner.gate
    }
}

/// Run the Phase 3C gate and return an immutable accepted-campaign handle.
pub fn run_campaign(
    engine: &PentaCrtEngine,
    config: CampaignConfig,
) -> Result<CampaignExecution, RuntimeError> {
    inner::run_campaign(engine, config).map(|inner| CampaignExecution { inner })
}

/// Persist only the immutable receipt set that originally passed the gate.
pub fn persist_accepted_campaign(
    directory: &Path,
    execution: &CampaignExecution,
) -> Result<CampaignManifest, RuntimeError> {
    inner::persist_accepted_campaign(directory, &execution.inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        )
        .expect("Phase 3C sealed-boundary test engine")
    }

    #[test]
    fn accepted_campaign_is_exposed_read_only() {
        let engine = engine();
        let execution = run_campaign(
            &engine,
            CampaignConfig {
                start: 100,
                count: 7,
                requested_workers: 2,
                memory_budget_bytes: size_of::<PaddedExecutionCell>() as u64 * 2,
                verification_samples: 17,
            },
        )
        .expect("repository fixture must pass sealed gate");

        assert!(execution.gate().accepted);
        assert!(!execution.correctness().biological_validity_claimed);
        assert!(!execution.correctness().clinical_validity_claimed);
        assert!(!execution.graph().biological_adjacency_claimed);
        assert!(!execution.layout().padding_lanes_semantic);
        assert!(!execution.layout().scientific_count_includes_padding);
        assert!(!execution.benchmark().identity_bearing_correctness);
        assert!(!execution.benchmark().performance_claim);
    }
}
