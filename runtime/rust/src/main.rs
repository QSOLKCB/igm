// SPDX-License-Identifier: Apache-2.0

use igm_runtime::{
    browser_v0_reference, build_geometry, default_evaluated_count, load_profile,
    logical_ensemble_size, run_structural_fixture, ExecutionAddress, RunConfig,
    RuntimeError, EXECUTION_CELL_STATES, INV_BIO_001, RUNTIME_CONTRACT, VERSION,
};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

fn usage() -> &'static str {
    "igm-runtime 0.1.0\n\n\
commands:\n\
  validate [PROFILE]\n\
  geometry [PROFILE]\n\
  address SEQUENCE\n\
  run [PROFILE] [--work-items N] [--workers N]\n\n\
PROFILE defaults to profiles/igm-schematic-pentamer-v0.json.\n\
The validate, geometry, and run commands fail closed unless the repository\n\
JSON-Schema and semantic profile gates can be located and pass.\n"
}

fn default_profile() -> PathBuf {
    PathBuf::from("profiles/igm-schematic-pentamer-v0.json")
}

fn discover_repo_root(profile: &Path) -> Result<PathBuf, RuntimeError> {
    let mut candidates = Vec::new();
    if let Ok(current) = env::current_dir() {
        candidates.extend(current.ancestors().map(Path::to_path_buf));
    }
    if let Ok(canonical) = profile.canonicalize() {
        if let Some(parent) = canonical.parent() {
            candidates.extend(parent.ancestors().map(Path::to_path_buf));
        }
    }
    for candidate in candidates {
        if candidate.join("tools/validate_json_schema.py").is_file()
            && candidate.join("tools/validate_profile.py").is_file()
            && candidate.join("schemas/model-profile.schema.json").is_file()
        {
            return Ok(candidate);
        }
    }
    Err(RuntimeError(
        "cannot locate repository validation gates; run from an IGM checkout or set the working directory to one"
            .into(),
    ))
}

fn run_gate(root: &Path, script: &str, profile: &Path) -> Result<(), RuntimeError> {
    let output = Command::new("python3")
        .arg(root.join(script))
        .arg(profile)
        .current_dir(root)
        .output()
        .map_err(|e| RuntimeError(format!("failed to execute {script}: {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(RuntimeError(format!(
            "repository validation gate failed: {script}: {}{}",
            stdout.trim(),
            stderr.trim()
        )));
    }
    Ok(())
}

fn validate_repository_contracts(profile: &Path) -> Result<(), RuntimeError> {
    let root = discover_repo_root(profile)?;
    let profile = profile
        .canonicalize()
        .map_err(|e| RuntimeError(format!("cannot resolve profile path: {e}")))?;
    run_gate(&root, "tools/validate_json_schema.py", &profile)?;
    run_gate(&root, "tools/validate_profile.py", &profile)?;
    Ok(())
}

fn command_validate(profile: &Path) -> Result<(), RuntimeError> {
    validate_repository_contracts(profile)?;
    let loaded = load_profile(profile)?;
    let geometry = build_geometry(&loaded.profile)?;
    let residual = geometry.max_pairwise_distance_residual(&browser_v0_reference())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-RUST-PROFILE-VALIDATION-V1",
            "runtime_contract": RUNTIME_CONTRACT,
            "runtime_version": VERSION,
            "model_id": loaded.profile.model_id,
            "model_version": loaded.profile.version,
            "validation_level": loaded.profile.validation_level,
            "profile_sha256": loaded.profile_sha256,
            "component_count": geometry.nodes.len(),
            "browser_reference_max_component_residual": residual,
            "non_clinical": true,
            "biological_validity_claimed": false,
            "inv_bio_001": INV_BIO_001
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_geometry(profile: &Path) -> Result<(), RuntimeError> {
    validate_repository_contracts(profile)?;
    let loaded = load_profile(profile)?;
    let geometry = build_geometry(&loaded.profile)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-RUST-GEOMETRY-V1",
            "runtime_contract": RUNTIME_CONTRACT,
            "model_id": loaded.profile.model_id,
            "model_version": loaded.profile.version,
            "validation_level": loaded.profile.validation_level,
            "profile_sha256": loaded.profile_sha256,
            "jchain_participants": &geometry.jchain_participants,
            "nodes": &geometry.nodes,
            "notice": format!("V0 · NOT CLINICAL · {INV_BIO_001}")
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_address(text: &str) -> Result<(), RuntimeError> {
    let sequence: u8 = text
        .parse()
        .map_err(|_| RuntimeError("SEQUENCE must be an integer in [0,29]".into()))?;
    let address = ExecutionAddress::from_sequence(sequence)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-CRT-PENTAFOLD-ADDRESS-V1",
            "sequence": sequence,
            "sector": address.sector,
            "arm": address.arm,
            "lane": address.lane,
            "storage_index": address.storage_index()?,
            "inverse_sequence": address.sequence()?,
            "execution_cell_states": EXECUTION_CELL_STATES,
            "semantics": "execution scheduling only; not a biological graph walk"
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn parse_run(args: &[String]) -> Result<(PathBuf, Option<u64>, Option<usize>), RuntimeError> {
    let mut profile = default_profile();
    let mut work_items = None;
    let mut workers = None;
    let mut index = 0;
    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            profile = PathBuf::from(first);
            index = 1;
        }
    }
    while index < args.len() {
        match args[index].as_str() {
            "--work-items" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| RuntimeError("--work-items requires N".into()))?;
                work_items = Some(
                    value
                        .parse()
                        .map_err(|_| RuntimeError("invalid --work-items".into()))?,
                );
            }
            "--workers" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| RuntimeError("--workers requires N".into()))?;
                workers = Some(
                    value
                        .parse()
                        .map_err(|_| RuntimeError("invalid --workers".into()))?,
                );
            }
            other => return Err(RuntimeError(format!("unknown run option: {other}"))),
        }
        index += 1;
    }
    Ok((profile, work_items, workers))
}

fn command_run(args: &[String]) -> Result<(), RuntimeError> {
    let (profile_path, requested_work_items, requested_workers) = parse_run(args)?;
    validate_repository_contracts(&profile_path)?;
    let loaded = load_profile(&profile_path)?;
    let logical = logical_ensemble_size(&loaded.profile)?;
    let work_items = requested_work_items.unwrap_or(default_evaluated_count(&loaded.profile)?);
    let workers = requested_workers.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .min(64)
    });
    let config = RunConfig::new(work_items, workers, logical)?;
    let started = Instant::now();
    let summary = run_structural_fixture(&loaded, config)?;
    let elapsed = started.elapsed().as_secs_f64();
    let throughput = if elapsed > 0.0 {
        summary.work_items as f64 / elapsed
    } else {
        0.0
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "summary": summary,
            "performance_observation": {
                "elapsed_seconds": elapsed,
                "work_items_per_second": throughput,
                "identity_bearing": false,
                "performance_claim": false,
                "note": "Local timing is an observation only and is excluded from deterministic result identity."
            }
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("validate") => {
            let profile = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_profile);
            command_validate(&profile)
        }
        Some("geometry") => {
            let profile = args
                .get(1)
                .map(PathBuf::from)
                .unwrap_or_else(default_profile);
            command_geometry(&profile)
        }
        Some("address") => args
            .get(1)
            .ok_or_else(|| RuntimeError("address requires SEQUENCE".into()))
            .and_then(|value| command_address(value)),
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
