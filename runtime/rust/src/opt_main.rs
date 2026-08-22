// SPDX-License-Identifier: Apache-2.0

use igm_runtime::phase3b::{
    describe_address, run_penta_crt, verify_penta_crt, PentaCrtEngine, PentaCrtRunConfig,
    OPTIMIZATION_CONTRACT, OPTIMIZATION_NUMERICAL_PROFILE,
};
use igm_runtime::{RuntimeError, INV_BIO_001};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn default_model_profile() -> PathBuf {
    PathBuf::from("profiles/igm-schematic-pentamer-v0.json")
}

fn default_optimization_profile() -> PathBuf {
    PathBuf::from("runtime/profiles/igm-penta-crt-cpu-v1.json")
}

fn usage() -> &'static str {
    "igm-penta-crt\n\n\
commands:\n\
  profile\n\
  address INDEX\n\
  verify [--samples N]\n\
  run [--start N] [--count N] [--workers N]\n\n\
Uses the repository V0 model profile plus runtime/profiles/igm-penta-crt-cpu-v1.json.\n\
Execution states are synthetic computational fixtures, not biological conformations.\n"
}

fn load_engine() -> Result<PentaCrtEngine, RuntimeError> {
    PentaCrtEngine::load(&default_model_profile(), &default_optimization_profile())
}

fn parse_value<T: std::str::FromStr>(args: &[String], index: &mut usize, flag: &str) -> Result<T, RuntimeError> {
    *index += 1;
    let value = args
        .get(*index)
        .ok_or_else(|| RuntimeError(format!("{flag} requires a value")))?;
    value
        .parse()
        .map_err(|_| RuntimeError(format!("invalid value for {flag}: {value}")))
}

fn command_profile() -> Result<(), RuntimeError> {
    let engine = load_engine()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-PENTA-CRT-PROFILE-INFO-V1",
            "optimization_contract": OPTIMIZATION_CONTRACT,
            "numerical_profile": OPTIMIZATION_NUMERICAL_PROFILE,
            "model_profile_sha256": engine.model_profile_sha256(),
            "optimization_profile_sha256": engine.optimization_profile_sha256(),
            "optimization_profile_id": engine.optimization_profile_id(),
            "radices": engine.radices(),
            "dof_ids": engine.dof_ids(),
            "total_conformations": engine.total_conformations(),
            "base_spread_deg": engine.base_spread_deg(),
            "non_clinical": true,
            "biological_validity_claimed": false,
            "inv_bio_001": INV_BIO_001
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_address(text: &str) -> Result<(), RuntimeError> {
    let engine = load_engine()?;
    let index: u64 = text
        .parse()
        .map_err(|_| RuntimeError("INDEX must be a non-negative integer".into()))?;
    let address = engine.decode(index)?;
    let round_trip = engine.encode(address)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-PENTA-CRT-CONFORMATION-ADDRESS-V1",
            "index": index,
            "address": address,
            "round_trip_index": round_trip,
            "description": describe_address(&engine, index)?,
            "total_conformations": engine.total_conformations(),
            "semantics": "execution coordinate only; not a biological graph walk or validated biological conformation"
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_verify(args: &[String]) -> Result<(), RuntimeError> {
    let mut samples = 257usize;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--samples" => samples = parse_value(args, &mut index, "--samples")?,
            other => return Err(RuntimeError(format!("unknown verify option: {other}"))),
        }
        index += 1;
    }
    let engine = load_engine()?;
    let report = verify_penta_crt(&engine, samples)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|e| RuntimeError(e.to_string()))?
    );
    if !report.accepted {
        return Err(RuntimeError(
            "PENTA-CRT structured reuse failed its declared reference residual tolerance".into(),
        ));
    }
    Ok(())
}

fn command_run(args: &[String]) -> Result<(), RuntimeError> {
    let engine = load_engine()?;
    let mut start = 0u64;
    let mut count = 4096u64.min(engine.total_conformations());
    let mut workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(64);
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--start" => start = parse_value(args, &mut index, "--start")?,
            "--count" => count = parse_value(args, &mut index, "--count")?,
            "--workers" => workers = parse_value(args, &mut index, "--workers")?,
            other => return Err(RuntimeError(format!("unknown run option: {other}"))),
        }
        index += 1;
    }
    let config = PentaCrtRunConfig::new(start, count, workers, engine.total_conformations())?;
    let started = Instant::now();
    let summary = run_penta_crt(&engine, config)?;
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let conformations_per_second = if elapsed_seconds > 0.0 {
        summary.conformation_count as f64 / elapsed_seconds
    } else {
        0.0
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "summary": summary,
            "performance_observation": {
                "elapsed_seconds": elapsed_seconds,
                "conformations_per_second": conformations_per_second,
                "identity_bearing": false,
                "performance_claim": false,
                "note": "Local timing is excluded from correctness identity and does not establish biological validity."
            }
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("profile") => command_profile(),
        Some("address") => args
            .get(1)
            .ok_or_else(|| RuntimeError("address requires INDEX".into()))
            .and_then(|value| command_address(value)),
        Some("verify") => command_verify(&args[1..]),
        Some("run") => command_run(&args[1..]),
        Some("-h") | Some("--help") | None => {
            print!("{}", usage());
            Ok(())
        }
        Some(other) => Err(RuntimeError(format!("unknown command: {other}\n\n{}", usage()))),
    };
    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}
