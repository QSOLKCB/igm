// SPDX-License-Identifier: Apache-2.0

use igm_runtime::phase3b::PentaCrtEngine;
use igm_runtime::phase3c::{
    chunk_plan, execution_graph_receipt, memory_layout_receipt, persist_accepted_campaign,
    persist_rejected_campaign, plan_memory, run_campaign, CampaignConfig, DEFAULT_MEMORY_BUDGET_BYTES,
    INV_RUNTIME_001, PHASE3C_CONTRACT, PHASE3C_GATE_CONTRACT,
};
use igm_runtime::{RuntimeError, INV_BIO_001};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};

fn default_model_profile() -> PathBuf {
    PathBuf::from("profiles/igm-schematic-pentamer-v0.json")
}

fn default_execution_profile() -> PathBuf {
    PathBuf::from("runtime/profiles/igm-penta-crt-cpu-v1.json")
}

fn usage() -> &'static str {
    "igm-campaign 0.1.0\n\n\
commands:\n\
  graph\n\
  layout\n\
  plan --count N [--start N] [--budget-bytes N]\n\
  run [--model PATH] [--execution PATH] [--start N] [--count N]\n\
      [--workers N] [--budget-bytes N] [--verify-samples N] --out DIR\n\n\
Phase 3C is scheduling/reproducibility infrastructure only.\n\
Execution adjacency does not imply biological adjacency.\n"
}

fn parse_u64(text: &str, label: &str) -> Result<u64, RuntimeError> {
    text.parse::<u64>()
        .map_err(|_| RuntimeError(format!("{label} must be a non-negative integer")))
}

fn parse_usize(text: &str, label: &str) -> Result<usize, RuntimeError> {
    text.parse::<usize>()
        .map_err(|_| RuntimeError(format!("{label} must be a non-negative integer")))
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), RuntimeError> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_graph() -> Result<(), RuntimeError> {
    print_json(&execution_graph_receipt()?)
}

fn command_layout() -> Result<(), RuntimeError> {
    print_json(&memory_layout_receipt()?)
}

fn command_plan(args: &[String]) -> Result<(), RuntimeError> {
    let mut start = 0_u64;
    let mut count = None;
    let mut budget = DEFAULT_MEMORY_BUDGET_BYTES;
    let mut index = 0usize;
    while index < args.len() {
        let key = args[index].as_str();
        index += 1;
        let value = args
            .get(index)
            .ok_or_else(|| RuntimeError(format!("{key} requires a value")))?;
        match key {
            "--start" => start = parse_u64(value, "--start")?,
            "--count" => count = Some(parse_u64(value, "--count")?),
            "--budget-bytes" => budget = parse_u64(value, "--budget-bytes")?,
            other => return Err(RuntimeError(format!("unknown plan option: {other}"))),
        }
        index += 1;
    }
    let count = count.ok_or_else(|| RuntimeError("plan requires --count N".into()))?;
    let memory = plan_memory(count, budget)?;
    let chunks = chunk_plan(start, count, &memory)?;
    print_json(&json!({
        "schema": "IGM-CAMPAIGN-PLAN-PREVIEW-V1",
        "campaign_contract": PHASE3C_CONTRACT,
        "memory": memory,
        "chunks": chunks,
        "inv_bio_001": INV_BIO_001,
        "inv_runtime_001": INV_RUNTIME_001,
        "biological_validity_claimed": false
    }))
}

#[derive(Debug)]
struct RunArgs {
    model: PathBuf,
    execution: PathBuf,
    start: u64,
    count: Option<u64>,
    workers: usize,
    budget: u64,
    verify_samples: usize,
    out: PathBuf,
}

fn parse_run(args: &[String]) -> Result<RunArgs, RuntimeError> {
    let mut parsed = RunArgs {
        model: default_model_profile(),
        execution: default_execution_profile(),
        start: 0,
        count: None,
        workers: std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1)
            .min(64),
        budget: DEFAULT_MEMORY_BUDGET_BYTES,
        verify_samples: 257,
        out: PathBuf::new(),
    };
    let mut index = 0usize;
    while index < args.len() {
        let key = args[index].as_str();
        index += 1;
        let value = args
            .get(index)
            .ok_or_else(|| RuntimeError(format!("{key} requires a value")))?;
        match key {
            "--model" => parsed.model = PathBuf::from(value),
            "--execution" => parsed.execution = PathBuf::from(value),
            "--start" => parsed.start = parse_u64(value, "--start")?,
            "--count" => parsed.count = Some(parse_u64(value, "--count")?),
            "--workers" => parsed.workers = parse_usize(value, "--workers")?,
            "--budget-bytes" => parsed.budget = parse_u64(value, "--budget-bytes")?,
            "--verify-samples" => parsed.verify_samples = parse_usize(value, "--verify-samples")?,
            "--out" => parsed.out = PathBuf::from(value),
            other => return Err(RuntimeError(format!("unknown run option: {other}"))),
        }
        index += 1;
    }
    if parsed.out.as_os_str().is_empty() {
        return Err(RuntimeError("run requires --out DIR".into()));
    }
    Ok(parsed)
}

fn preserve_rejection(out: &Path, stage: &str, error: &RuntimeError) -> RuntimeError {
    match persist_rejected_campaign(out, stage, error.to_string()) {
        Ok(_) => RuntimeError(format!(
            "{}; rejected campaign preserved at {}",
            error,
            out.display()
        )),
        Err(persist_error) => RuntimeError(format!(
            "{}; additionally failed to preserve rejected campaign at {}: {}",
            error,
            out.display(),
            persist_error
        )),
    }
}

fn command_run(args: &[String]) -> Result<(), RuntimeError> {
    let parsed = parse_run(args)?;
    if parsed.out.exists() {
        return Err(RuntimeError(format!(
            "campaign output path already exists: {}",
            parsed.out.display()
        )));
    }

    let engine = match PentaCrtEngine::load(&parsed.model, &parsed.execution) {
        Ok(engine) => engine,
        Err(error) => return Err(preserve_rejection(&parsed.out, "engine-admission", &error)),
    };

    if parsed.start >= engine.total_conformations() {
        let error = RuntimeError(format!(
            "--start {} is outside execution domain [0,{})",
            parsed.start,
            engine.total_conformations()
        ));
        return Err(preserve_rejection(&parsed.out, "range-admission", &error));
    }

    let count = match parsed.count {
        Some(value) => value,
        None => engine.total_conformations() - parsed.start,
    };
    let end_exclusive = match parsed.start.checked_add(count) {
        Some(value) if count > 0 && value <= engine.total_conformations() => value,
        _ => {
            let error = RuntimeError(format!(
                "campaign range start={} count={} exceeds execution domain [0,{})",
                parsed.start,
                count,
                engine.total_conformations()
            ));
            return Err(preserve_rejection(&parsed.out, "range-admission", &error));
        }
    };
    debug_assert!(end_exclusive > parsed.start);

    let config = CampaignConfig {
        start: parsed.start,
        count,
        requested_workers: parsed.workers,
        memory_budget_bytes: parsed.budget,
        verification_samples: parsed.verify_samples,
    };
    let execution = match run_campaign(&engine, config) {
        Ok(execution) => execution,
        Err(error) => return Err(preserve_rejection(&parsed.out, "phase3c-gate", &error)),
    };
    let manifest = persist_accepted_campaign(&parsed.out, &execution)?;
    print_json(&json!({
        "schema": "IGM-CAMPAIGN-COMPLETION-V2",
        "campaign_contract": PHASE3C_CONTRACT,
        "phase3c_gate_contract": PHASE3C_GATE_CONTRACT,
        "phase3c_gate_accepted": execution.gate.accepted,
        "phase3c_gate_identity_sha256": execution.gate.gate_identity_sha256,
        "output_directory": parsed.out,
        "correctness_result_sha256": execution.correctness.result_sha256,
        "manifest_sha256": manifest.manifest_sha256,
        "chunk_count": execution.memory.chunk_count,
        "requested_workers": execution.benchmark.requested_workers,
        "verification_accepted": execution.correctness.verification_accepted,
        "benchmark_timing_excluded_from_correctness_identity": execution.gate.benchmark_timing_excluded_from_correctness_identity,
        "validation_level_promoted_by_runtime": false,
        "performance_claim": false,
        "biological_validity_claimed": false,
        "inv_bio_001": INV_BIO_001,
        "inv_runtime_001": INV_RUNTIME_001
    }))
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("graph") => command_graph(),
        Some("layout") => command_layout(),
        Some("plan") => command_plan(&args[1..]),
        Some("run") => command_run(&args[1..]),
        Some("-h") | Some("--help") | None => {
            print!("{}", usage());
            Ok(())
        }
        Some(other) => Err(RuntimeError(format!(
            "unknown command: {other}\n\n{}",
            usage()
        ))),
    };
    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}
