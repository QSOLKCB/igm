// SPDX-License-Identifier: Apache-2.0
//! Dedicated scalar/reference-vs-optimized timing benchmark for Phase 3B.
//!
//! Timing is observational metadata only. It never enters correctness identity,
//! biological interpretation, validation level, or clinical claims.

use igm_runtime::phase3b::{
    run_penta_crt, verify_penta_crt, ConformationAddress, PentaCrtEngine,
    PentaCrtRunConfig, BLOCK_REUSE_RESIDUAL_TOLERANCE, MAX_VERIFY_SAMPLES,
    OPTIMIZATION_CONTRACT, OPTIMIZATION_NUMERICAL_PROFILE, PAIR_COUNT,
};
use igm_runtime::{
    ExecutionAddress, RuntimeError, Vec3, C72, EXECUTION_CELL_STATES, INV_BIO_001, S72,
};
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

const BENCHMARK_SCHEMA: &str = "IGM-PENTA-CRT-TIMING-BENCHMARK-V1";
const BENCHMARK_CONTRACT: &str = "IGM-PHASE3B-SCALAR-VS-OPTIMIZED-BENCHMARK-V1";
const REFERENCE_PROFILE: &str = "IGM-PENTA-CRT-F64-REFERENCE-BRUTE-V1";
const DEFAULT_COUNT: u64 = 4096;
const DEFAULT_REPETITIONS: usize = 9;
const DEFAULT_WARMUPS: usize = 2;
const DEFAULT_VERIFY_SAMPLES: usize = 1024;
const MIN_BENCHMARK_COUNT: u64 = 64;
const MAX_REPETITIONS: usize = 31;
const MAX_WARMUPS: usize = 8;
const V0_SUBUNIT_Z_AMPLITUDE: f64 = 0.08;
const V0_FAB_Z_OFFSET: f64 = 0.06;
const V0_JCHAIN_Y_RATIO: f64 = 0.35;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn default_model_profile() -> PathBuf {
    PathBuf::from("profiles/igm-schematic-pentamer-v0.json")
}

fn default_optimization_profile() -> PathBuf {
    PathBuf::from("runtime/profiles/igm-penta-crt-cpu-v1.json")
}

fn parse_value<T: std::str::FromStr>(
    args: &[String],
    index: &mut usize,
    flag: &str,
) -> Result<T, RuntimeError> {
    *index += 1;
    let value = args
        .get(*index)
        .ok_or_else(|| err(format!("{flag} requires a value")))?;
    value
        .parse()
        .map_err(|_| err(format!("invalid value for {flag}: {value}")))
}

#[derive(Debug, Clone)]
struct ReferenceInputs {
    core_radius: f64,
    fab_length: f64,
    base_spread_deg: f64,
    jchain_offset: f64,
    left_delta_deg: Vec<f64>,
    right_delta_deg: Vec<f64>,
    jchain_dx: Vec<f64>,
    jchain_dy: Vec<f64>,
}

fn load_json(path: &Path) -> Result<Value, RuntimeError> {
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("{} is not strict JSON: {e}", path.display())))
}

fn model_number(raw: &Value, name: &str) -> Result<f64, RuntimeError> {
    let parameters = raw
        .get("parameters")
        .and_then(Value::as_array)
        .ok_or_else(|| err("model profile parameters must be an array"))?;
    let value = parameters
        .iter()
        .find(|parameter| parameter.get("name").and_then(Value::as_str) == Some(name))
        .and_then(|parameter| parameter.get("value"))
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("missing numeric model parameter: {name}")))?;
    if !value.is_finite() {
        return Err(err(format!("model parameter {name} must be finite")));
    }
    Ok(value)
}

fn execution_values(raw: &Value, id: &str, expected: usize) -> Result<Vec<f64>, RuntimeError> {
    let dofs = raw
        .get("degrees_of_freedom")
        .and_then(Value::as_array)
        .ok_or_else(|| err("execution profile degrees_of_freedom must be an array"))?;
    let values = dofs
        .iter()
        .find(|dof| dof.get("id").and_then(Value::as_str) == Some(id))
        .and_then(|dof| dof.get("values"))
        .and_then(Value::as_array)
        .ok_or_else(|| err(format!("missing execution DoF: {id}")))?;
    if values.len() != expected {
        return Err(err(format!("execution DoF {id} must have {expected} values")));
    }
    values
        .iter()
        .map(|value| {
            let number = value
                .as_f64()
                .ok_or_else(|| err(format!("execution DoF {id} must contain numbers")))?;
            if !number.is_finite() {
                return Err(err(format!("execution DoF {id} contains non-finite value")));
            }
            Ok(number)
        })
        .collect()
}

fn load_reference_inputs(model: &Path, execution: &Path) -> Result<ReferenceInputs, RuntimeError> {
    let model_raw = load_json(model)?;
    let execution_raw = load_json(execution)?;
    Ok(ReferenceInputs {
        core_radius: model_number(&model_raw, "core_radius")?,
        fab_length: model_number(&model_raw, "fab_length")?,
        base_spread_deg: model_number(&model_raw, "fab_spread_deg")?,
        jchain_offset: model_number(&model_raw, "jchain_offset")?,
        left_delta_deg: execution_values(&execution_raw, "left_fab_delta_deg", 17)?,
        right_delta_deg: execution_values(&execution_raw, "right_fab_delta_deg", 17)?,
        jchain_dx: execution_values(&execution_raw, "jchain_dx", 9)?,
        jchain_dy: execution_values(&execution_raw, "jchain_dy", 9)?,
    })
}

#[derive(Debug, Clone, Copy)]
struct TrigPair {
    sin: f64,
    cos: f64,
}

fn deterministic_sin_cos(angle: f64) -> Result<TrigPair, RuntimeError> {
    if !angle.is_finite() {
        return Err(err("benchmark reference angle must be finite"));
    }
    let pi = std::f64::consts::PI;
    let tau = std::f64::consts::TAU;
    let mut x = angle % tau;
    if x > pi {
        x -= tau;
    } else if x < -pi {
        x += tau;
    }
    let x2 = x * x;
    let mut sin_sum = x;
    let mut sin_term = x;
    let mut cos_sum = 1.0_f64;
    let mut cos_term = 1.0_f64;
    for k in 1..14_u32 {
        let sf = f64::from(2 * k);
        sin_term *= -x2 / (sf * (sf + 1.0));
        sin_sum += sin_term;
        let cf = f64::from(2 * k - 1);
        cos_term *= -x2 / (cf * (cf + 1.0));
        cos_sum += cos_term;
    }
    if !sin_sum.is_finite() || !cos_sum.is_finite() {
        return Err(err("benchmark reference trig became non-finite"));
    }
    Ok(TrigPair {
        sin: sin_sum,
        cos: cos_sum,
    })
}

fn reference_geometry(
    inputs: &ReferenceInputs,
    address: ConformationAddress,
) -> Result<[Vec3; 16], RuntimeError> {
    let left_delta = *inputs
        .left_delta_deg
        .get(address.left_fab_bin as usize)
        .ok_or_else(|| err("left reference bin outside domain"))?;
    let right_delta = *inputs
        .right_delta_deg
        .get(address.right_fab_bin as usize)
        .ok_or_else(|| err("right reference bin outside domain"))?;
    let left = deterministic_sin_cos(
        (inputs.base_spread_deg + left_delta) * std::f64::consts::PI / 180.0,
    )?;
    let right = deterministic_sin_cos(
        (inputs.base_spread_deg + right_delta) * std::f64::consts::PI / 180.0,
    )?;

    let mut points = [Vec3::new(0.0, 0.0, 0.0); 16];
    let mut ux = 0.0_f64;
    let mut uy = -1.0_f64;
    for sector in 0..5_usize {
        let base = sector * 3;
        let sx = inputs.core_radius * ux;
        let sy = inputs.core_radius * uy;
        let sz = V0_SUBUNIT_Z_AMPLITUDE * (2.0 * ux * uy);
        points[base] = Vec3::new(sx, sy, sz);

        let left_dx = ux * left.cos + uy * left.sin;
        let left_dy = uy * left.cos - ux * left.sin;
        points[base + 1] = Vec3::new(
            sx + inputs.fab_length * left_dx,
            sy + inputs.fab_length * left_dy,
            sz - V0_FAB_Z_OFFSET * ux,
        );

        let right_dx = ux * right.cos - uy * right.sin;
        let right_dy = uy * right.cos + ux * right.sin;
        points[base + 2] = Vec3::new(
            sx + inputs.fab_length * right_dx,
            sy + inputs.fab_length * right_dy,
            sz + V0_FAB_Z_OFFSET * ux,
        );

        if sector != 4 {
            let next_x = C72 * ux - S72 * uy;
            let next_y = S72 * ux + C72 * uy;
            ux = next_x;
            uy = next_y;
        }
    }

    points[15] = Vec3::new(
        -inputs.jchain_offset
            + inputs.jchain_dx[address.jchain_dx_bin as usize],
        -inputs.jchain_offset * V0_JCHAIN_Y_RATIO
            + inputs.jchain_dy[address.jchain_dy_bin as usize],
        0.0,
    );
    if !points.iter().all(|point| point.is_finite()) {
        return Err(err("benchmark reference geometry became non-finite"));
    }
    Ok(points)
}

fn fnv_update(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn reference_record_hash(
    index: u64,
    address: ConformationAddress,
    pairs: &[f64; PAIR_COUNT],
) -> Result<u64, RuntimeError> {
    let mut hash = fnv_update(FNV_OFFSET, b"IGM-PENTA-CRT-CONFORMATION-V2\0");
    hash = fnv_update(hash, &index.to_le_bytes());
    for digit in [
        address.left_fab_bin,
        address.right_fab_bin,
        address.jchain_dx_bin,
        address.jchain_dy_bin,
    ] {
        hash = fnv_update(hash, &digit.to_le_bytes());
    }
    for d2 in pairs {
        if !d2.is_finite() {
            return Err(err("benchmark reference pair value became non-finite"));
        }
        hash = fnv_update(hash, &d2.to_bits().to_le_bytes());
    }
    for sequence in 0..EXECUTION_CELL_STATES {
        let execution = ExecutionAddress::from_sequence(sequence)?;
        hash = fnv_update(
            hash,
            &[
                execution.sector,
                execution.arm,
                execution.lane,
                execution.storage_index()?,
            ],
        );
    }
    Ok(hash)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ReferenceSummary {
    diagnostic: u64,
    min_d2: f64,
    max_d2: f64,
}

fn reference_slice(
    engine: &PentaCrtEngine,
    inputs: &ReferenceInputs,
    start: u64,
    count: u64,
) -> Result<ReferenceSummary, RuntimeError> {
    let end = start
        .checked_add(count)
        .ok_or_else(|| err("benchmark reference slice overflow"))?;
    if count < MIN_BENCHMARK_COUNT || start >= engine.total_conformations() || end > engine.total_conformations() {
        return Err(err("benchmark reference slice outside bounded domain"));
    }

    let mut diagnostic = 0_u64;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    for index in start..end {
        let address = engine.decode(index)?;
        let points = reference_geometry(inputs, address)?;
        let mut pairs = [0.0_f64; PAIR_COUNT];
        let mut cursor = 0_usize;
        for left in 0..16_usize {
            for right in left + 1..16_usize {
                let d2 = points[left].checked_squared_distance(points[right])?;
                pairs[cursor] = d2;
                cursor += 1;
                min_d2 = min_d2.min(d2);
                max_d2 = max_d2.max(d2);
            }
        }
        if cursor != PAIR_COUNT {
            return Err(err("benchmark reference pair-count invariant failed"));
        }
        diagnostic ^= reference_record_hash(index, address, &pairs)?;
    }
    if !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("benchmark reference summary became non-finite"));
    }
    Ok(ReferenceSummary {
        diagnostic,
        min_d2,
        max_d2,
    })
}

fn duration_ns(started: Instant) -> Result<u64, RuntimeError> {
    u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| err("benchmark duration does not fit u64 nanoseconds"))
}

fn median_ns(values: &[u64]) -> Result<u64, RuntimeError> {
    if values.is_empty() {
        return Err(err("benchmark median requires samples"));
    }
    let mut ordered = values.to_vec();
    ordered.sort_unstable();
    let mid = ordered.len() / 2;
    if ordered.len() % 2 == 1 {
        Ok(ordered[mid])
    } else {
        u64::try_from((u128::from(ordered[mid - 1]) + u128::from(ordered[mid])) / 2)
            .map_err(|_| err("benchmark median overflow"))
    }
}

#[derive(Debug, Serialize)]
struct TimingBenchmarkReport {
    schema: &'static str,
    benchmark_contract: &'static str,
    reference_profile: &'static str,
    optimized_contract: &'static str,
    optimized_numerical_profile: &'static str,
    model_profile_sha256: String,
    optimization_profile_sha256: String,
    validation_level: &'static str,
    conformation_start: u64,
    conformation_count: u64,
    conformation_end_exclusive: u64,
    warmups: usize,
    repetitions: usize,
    verification_samples: usize,
    phase3b_residual_gate_passed: bool,
    verification_max_geometry_residual: f64,
    verification_max_pair_residual: f64,
    verification_tolerance: f64,
    reference_times_ns: Vec<u64>,
    optimized_times_ns: Vec<u64>,
    reference_median_ns: u64,
    optimized_median_ns: u64,
    reference_conformations_per_second: f64,
    optimized_conformations_per_second: f64,
    observed_speedup_ratio: f64,
    reference_diagnostic_xor_fnv1a64: String,
    optimized_result_sha256: String,
    reference_min_pair_distance_squared: f64,
    reference_max_pair_distance_squared: f64,
    optimized_min_pair_distance_squared: f64,
    optimized_max_pair_distance_squared: f64,
    release_build: bool,
    single_threaded_comparison: bool,
    benchmark_timing_identity_bearing: bool,
    correctness_identity_includes_timing: bool,
    speedup_claimed: bool,
    performance_claim: bool,
    non_clinical: bool,
    biological_validity_claimed: bool,
    clinical_validity_claimed: bool,
    inv_bio_001: &'static str,
    note: &'static str,
}

fn run_benchmark(args: &[String]) -> Result<TimingBenchmarkReport, RuntimeError> {
    let model_path = default_model_profile();
    let optimization_path = default_optimization_profile();
    let engine = PentaCrtEngine::load(&model_path, &optimization_path)?;
    let inputs = load_reference_inputs(&model_path, &optimization_path)?;

    let mut start = 0_u64;
    let mut count = DEFAULT_COUNT.min(engine.total_conformations());
    let mut repetitions = DEFAULT_REPETITIONS;
    let mut warmups = DEFAULT_WARMUPS;
    let mut verification_samples = DEFAULT_VERIFY_SAMPLES.min(engine.total_conformations() as usize);
    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--start" => start = parse_value(args, &mut index, "--start")?,
            "--count" => count = parse_value(args, &mut index, "--count")?,
            "--repetitions" => repetitions = parse_value(args, &mut index, "--repetitions")?,
            "--warmups" => warmups = parse_value(args, &mut index, "--warmups")?,
            "--verify-samples" => {
                verification_samples = parse_value(args, &mut index, "--verify-samples")?
            }
            other => return Err(err(format!("unknown benchmark option: {other}"))),
        }
        index += 1;
    }

    if count < MIN_BENCHMARK_COUNT {
        return Err(err(format!("benchmark --count must be at least {MIN_BENCHMARK_COUNT}")));
    }
    let end = start
        .checked_add(count)
        .ok_or_else(|| err("benchmark conformation range overflow"))?;
    if start >= engine.total_conformations() || end > engine.total_conformations() {
        return Err(err("benchmark conformation slice exceeds execution domain"));
    }
    if repetitions < 3 || repetitions > MAX_REPETITIONS {
        return Err(err(format!("benchmark repetitions must be in [3,{MAX_REPETITIONS}]")));
    }
    if warmups == 0 || warmups > MAX_WARMUPS {
        return Err(err(format!("benchmark warmups must be in [1,{MAX_WARMUPS}]")));
    }
    if verification_samples == 0
        || verification_samples > MAX_VERIFY_SAMPLES
        || verification_samples as u64 > engine.total_conformations()
    {
        return Err(err("benchmark verification sample count outside Phase 3B bound"));
    }

    let verification = verify_penta_crt(&engine, verification_samples)?;
    if !verification.accepted
        || verification.residual_tolerance.to_bits()
            != BLOCK_REUSE_RESIDUAL_TOLERANCE.to_bits()
    {
        return Err(err("benchmark refused because Phase 3B residual gate did not pass"));
    }

    let optimized_config = || {
        PentaCrtRunConfig::new(start, count, 1, engine.total_conformations())
    };

    for _ in 0..warmups {
        black_box(reference_slice(&engine, &inputs, start, count)?);
        black_box(run_penta_crt(&engine, optimized_config()?)?);
    }

    let mut reference_times = Vec::with_capacity(repetitions);
    let mut optimized_times = Vec::with_capacity(repetitions);
    let mut reference_baseline: Option<ReferenceSummary> = None;
    let mut optimized_result_sha256: Option<String> = None;
    let mut optimized_min = None;
    let mut optimized_max = None;

    for repetition in 0..repetitions {
        let mut time_reference = || -> Result<(), RuntimeError> {
            let started = Instant::now();
            let summary = black_box(reference_slice(&engine, &inputs, start, count)?);
            let elapsed = duration_ns(started)?;
            if elapsed == 0 {
                return Err(err("reference benchmark duration was zero"));
            }
            if let Some(baseline) = reference_baseline {
                if summary.diagnostic != baseline.diagnostic
                    || summary.min_d2.to_bits() != baseline.min_d2.to_bits()
                    || summary.max_d2.to_bits() != baseline.max_d2.to_bits()
                {
                    return Err(err("reference benchmark became non-deterministic across repetitions"));
                }
            } else {
                reference_baseline = Some(summary);
            }
            reference_times.push(elapsed);
            Ok(())
        };

        let mut time_optimized = || -> Result<(), RuntimeError> {
            let started = Instant::now();
            let summary = black_box(run_penta_crt(&engine, optimized_config()?)?);
            let elapsed = duration_ns(started)?;
            if elapsed == 0 {
                return Err(err("optimized benchmark duration was zero"));
            }
            if let Some(expected) = optimized_result_sha256.as_deref() {
                if summary.result_sha256 != expected {
                    return Err(err("optimized benchmark result identity changed across repetitions"));
                }
            } else {
                optimized_result_sha256 = Some(summary.result_sha256.clone());
                optimized_min = Some(summary.min_pair_distance_squared);
                optimized_max = Some(summary.max_pair_distance_squared);
            }
            optimized_times.push(elapsed);
            Ok(())
        };

        if repetition % 2 == 0 {
            time_reference()?;
            time_optimized()?;
        } else {
            time_optimized()?;
            time_reference()?;
        }
    }

    let reference = reference_baseline.ok_or_else(|| err("missing reference benchmark summary"))?;
    let optimized_result_sha256 = optimized_result_sha256
        .ok_or_else(|| err("missing optimized benchmark result identity"))?;
    let optimized_min = optimized_min.ok_or_else(|| err("missing optimized benchmark minimum"))?;
    let optimized_max = optimized_max.ok_or_else(|| err("missing optimized benchmark maximum"))?;
    let reference_median = median_ns(&reference_times)?;
    let optimized_median = median_ns(&optimized_times)?;
    if reference_median == 0 || optimized_median == 0 {
        return Err(err("benchmark median duration cannot be zero"));
    }

    let reference_cps = count as f64 * 1.0e9 / reference_median as f64;
    let optimized_cps = count as f64 * 1.0e9 / optimized_median as f64;
    let speedup = reference_median as f64 / optimized_median as f64;
    if !reference_cps.is_finite() || !optimized_cps.is_finite() || !speedup.is_finite() {
        return Err(err("benchmark derived timing observation became non-finite"));
    }

    Ok(TimingBenchmarkReport {
        schema: BENCHMARK_SCHEMA,
        benchmark_contract: BENCHMARK_CONTRACT,
        reference_profile: REFERENCE_PROFILE,
        optimized_contract: OPTIMIZATION_CONTRACT,
        optimized_numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
        model_profile_sha256: engine.model_profile_sha256().to_string(),
        optimization_profile_sha256: engine.optimization_profile_sha256().to_string(),
        validation_level: "V0",
        conformation_start: start,
        conformation_count: count,
        conformation_end_exclusive: end,
        warmups,
        repetitions,
        verification_samples,
        phase3b_residual_gate_passed: true,
        verification_max_geometry_residual: verification.max_geometry_lut_vs_reference_residual,
        verification_max_pair_residual: verification.max_block_reuse_vs_brute_pair_residual,
        verification_tolerance: verification.residual_tolerance,
        reference_times_ns: reference_times,
        optimized_times_ns: optimized_times,
        reference_median_ns: reference_median,
        optimized_median_ns: optimized_median,
        reference_conformations_per_second: reference_cps,
        optimized_conformations_per_second: optimized_cps,
        observed_speedup_ratio: speedup,
        reference_diagnostic_xor_fnv1a64: format!("{:016x}", reference.diagnostic),
        optimized_result_sha256,
        reference_min_pair_distance_squared: reference.min_d2,
        reference_max_pair_distance_squared: reference.max_d2,
        optimized_min_pair_distance_squared: optimized_min,
        optimized_max_pair_distance_squared: optimized_max,
        release_build: !cfg!(debug_assertions),
        single_threaded_comparison: true,
        benchmark_timing_identity_bearing: false,
        correctness_identity_includes_timing: false,
        speedup_claimed: false,
        performance_claim: false,
        non_clinical: true,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
        note: "Local timing is an observation only. This report does not itself authorize a speedup claim, alter correctness identity, or establish biological/clinical validity.",
    })
}

fn usage() -> &'static str {
    "igm-benchmark\n\n\
usage:\n\
  igm-benchmark [--start N] [--count N] [--repetitions N] [--warmups N] [--verify-samples N]\n\n\
Compares the scalar deterministic reference path against the actual one-worker PENTA-CRT optimized runtime on the same V0 conformation slice. Timing is observation-only and excluded from correctness identity.\n"
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.first().map(String::as_str), Some("-h") | Some("--help")) {
        print!("{}", usage());
        return;
    }
    match run_benchmark(&args) {
        Ok(report) => match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{json}"),
            Err(error) => {
                eprintln!("FAIL: cannot serialize benchmark report: {error}");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("FAIL: {error}");
            std::process::exit(1);
        }
    }
}
