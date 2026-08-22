// SPDX-License-Identifier: Apache-2.0
//! Seeded property-based fuzzing for the Phase 3A deterministic Rust runtime.
//!
//! This is implementation testing for the schematic V0 fixture only. Passing
//! generated properties does not create biological validity, molecular-dynamics
//! evidence, clinical validity, or a source-informed model.

use igm_runtime::{
    bounded_rotate_z, load_profile, partition_ranges, run_structural_fixture, ExecutionAddress,
    RunConfig, SquaredDistanceGate, Vec3, EXECUTION_CELL_STATES, INV_BIO_001, MAX_WORKERS,
};
use std::fmt::Display;
use std::path::Path;

const DEFAULT_SEED: u64 = 0x4947_4d50_524f_5037; // "IGMPROP7"
const DEFAULT_CASES: usize = 512;
const MAX_CASES: usize = 16_384;
const MAX_RUNTIME_CASES: usize = 32;

#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn bounded_u64(&mut self, exclusive_upper: u64) -> u64 {
        assert!(exclusive_upper > 0);
        self.next_u64() % exclusive_upper
    }

    fn bounded_usize(&mut self, exclusive_upper: usize) -> usize {
        self.bounded_u64(exclusive_upper as u64) as usize
    }

    fn exact_binary_f64(&mut self, magnitude: i32) -> f64 {
        let span = i64::from(magnitude) * 16 + 1;
        let raw = self.bounded_u64(span as u64) as i64 - i64::from(magnitude) * 8;
        raw as f64 / 8.0
    }
}

fn parse_seed() -> u64 {
    match std::env::var("IGM_PROPERTY_FUZZ_SEED") {
        Ok(value) => {
            let trimmed = value.trim();
            if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
                u64::from_str_radix(hex, 16).expect("IGM_PROPERTY_FUZZ_SEED must be u64 decimal/hex")
            } else {
                trimmed
                    .parse::<u64>()
                    .expect("IGM_PROPERTY_FUZZ_SEED must be u64 decimal/hex")
            }
        }
        Err(_) => DEFAULT_SEED,
    }
}

fn case_count() -> usize {
    match std::env::var("IGM_PROPERTY_FUZZ_CASES") {
        Ok(value) => value
            .parse::<usize>()
            .expect("IGM_PROPERTY_FUZZ_CASES must be a positive integer")
            .clamp(1, MAX_CASES),
        Err(_) => DEFAULT_CASES,
    }
}

fn rng_for(domain: u64) -> SplitMix64 {
    SplitMix64::new(parse_seed() ^ domain.rotate_left(17))
}

fn fixture_path() -> &'static Path {
    Path::new("profiles/igm-schematic-pentamer-v0.json")
}

fn fuzz_ok<T, E: Display>(
    result: Result<T, E>,
    seed: u64,
    case: usize,
    property: &str,
    operation: &str,
) -> T {
    result.unwrap_or_else(|error| {
        panic!(
            "property={property} seed=0x{seed:016x} case={case} operation={operation}: {error}"
        )
    })
}

#[test]
fn property_crt_addressing_is_total_only_on_declared_domains() {
    const PROPERTY: &str = "crt-addressing";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x4352_5433_3041_4444); // CRT30ADD

    for case in 0..cases {
        let sequence = rng.bounded_u64(256) as u8;
        match ExecutionAddress::from_sequence(sequence) {
            Ok(address) => {
                assert!(
                    sequence < EXECUTION_CELL_STATES,
                    "property={PROPERTY} seed=0x{seed:016x} case={case}: out-of-domain sequence accepted"
                );
                let round_trip = fuzz_ok(address.sequence(), seed, case, PROPERTY, "address.sequence");
                assert_eq!(
                    round_trip, sequence,
                    "property={PROPERTY} seed=0x{seed:016x} case={case}: sequence round trip mismatch"
                );
                let storage = fuzz_ok(
                    address.storage_index(),
                    seed,
                    case,
                    PROPERTY,
                    "address.storage_index",
                );
                assert!(
                    storage < EXECUTION_CELL_STATES,
                    "property={PROPERTY} seed=0x{seed:016x} case={case}: storage index out of domain"
                );
            }
            Err(_) => assert!(
                sequence >= EXECUTION_CELL_STATES,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: valid sequence rejected"
            ),
        }

        let address = ExecutionAddress {
            sector: rng.bounded_u64(8) as u8,
            arm: rng.bounded_u64(5) as u8,
            lane: rng.bounded_u64(6) as u8,
        };
        let valid = address.sector < 5 && address.arm < 2 && address.lane < 3;
        let sequence_result = address.sequence();
        let storage_result = address.storage_index();

        if valid {
            let seq = fuzz_ok(
                sequence_result,
                seed,
                case,
                PROPERTY,
                "valid address.sequence",
            );
            let storage = fuzz_ok(
                storage_result,
                seed,
                case,
                PROPERTY,
                "valid address.storage_index",
            );
            let reconstructed = fuzz_ok(
                ExecutionAddress::from_sequence(seq),
                seed,
                case,
                PROPERTY,
                "ExecutionAddress::from_sequence(round_trip)",
            );
            assert_eq!(
                reconstructed, address,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: address round trip mismatch"
            );
            assert!(
                storage < EXECUTION_CELL_STATES,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: valid storage index out of domain"
            );
        } else {
            assert!(
                sequence_result.is_err(),
                "property={PROPERTY} seed=0x{seed:016x} case={case}: sequence() accepted invalid address sector={} arm={} lane={}",
                address.sector,
                address.arm,
                address.lane
            );
            assert!(
                storage_result.is_err(),
                "property={PROPERTY} seed=0x{seed:016x} case={case}: storage_index() accepted invalid address sector={} arm={} lane={}",
                address.sector,
                address.arm,
                address.lane
            );
        }
    }
}

#[test]
fn property_partition_ranges_are_gap_free_bounded_and_balanced() {
    const PROPERTY: &str = "partition-ranges";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x5041_5254_4954_494f); // PARTITIO

    for case in 0..cases {
        let items = 1 + rng.bounded_u64(100_000);
        let requested = 1 + rng.bounded_usize(MAX_WORKERS);
        let ranges = fuzz_ok(
            partition_ranges(items, requested),
            seed,
            case,
            PROPERTY,
            "partition_ranges",
        );
        let effective = requested.min(items as usize);
        assert_eq!(
            ranges.len(), effective,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: effective worker count mismatch"
        );
        assert!(
            !ranges.is_empty(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: partition unexpectedly empty"
        );
        assert_eq!(
            ranges[0].start, 0,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: first range does not start at zero"
        );
        assert_eq!(
            ranges[ranges.len() - 1].end,
            items,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: last range does not end at item count"
        );

        let mut total = 0_u64;
        let mut min_len = u64::MAX;
        let mut max_len = 0_u64;
        for (worker, range) in ranges.iter().enumerate() {
            assert_eq!(
                range.worker, worker,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: worker ordinal mismatch"
            );
            assert!(
                range.length > 0,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: zero-length range"
            );
            assert_eq!(
                range.end - range.start,
                range.length,
                "property={PROPERTY} seed=0x{seed:016x} case={case}: range arithmetic mismatch"
            );
            if worker > 0 {
                assert_eq!(
                    ranges[worker - 1].end,
                    range.start,
                    "property={PROPERTY} seed=0x{seed:016x} case={case}: partition gap/overlap"
                );
            }
            total += range.length;
            min_len = min_len.min(range.length);
            max_len = max_len.max(range.length);
        }
        assert_eq!(
            total, items,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: partition coverage mismatch"
        );
        assert!(
            max_len - min_len <= 1,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: quotient/remainder balance violated"
        );
    }
}

#[test]
fn property_squared_distance_is_symmetric_and_translation_invariant() {
    const PROPERTY: &str = "squared-distance";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x4449_5354_414e_4345); // DISTANCE

    for case in 0..cases {
        let a = Vec3::new(
            rng.exact_binary_f64(128),
            rng.exact_binary_f64(128),
            rng.exact_binary_f64(128),
        );
        let b = Vec3::new(
            rng.exact_binary_f64(128),
            rng.exact_binary_f64(128),
            rng.exact_binary_f64(128),
        );
        let t = Vec3::new(
            rng.exact_binary_f64(16),
            rng.exact_binary_f64(16),
            rng.exact_binary_f64(16),
        );
        let d_ab = fuzz_ok(
            a.checked_squared_distance(b),
            seed,
            case,
            PROPERTY,
            "a.checked_squared_distance(b)",
        );
        let d_ba = fuzz_ok(
            b.checked_squared_distance(a),
            seed,
            case,
            PROPERTY,
            "b.checked_squared_distance(a)",
        );
        assert_eq!(
            d_ab.to_bits(),
            d_ba.to_bits(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: symmetry mismatch"
        );
        assert!(
            d_ab >= 0.0 && d_ab.is_finite(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: distance not finite/non-negative"
        );

        let at = Vec3::new(a.x + t.x, a.y + t.y, a.z + t.z);
        let bt = Vec3::new(b.x + t.x, b.y + t.y, b.z + t.z);
        let translated = fuzz_ok(
            at.checked_squared_distance(bt),
            seed,
            case,
            PROPERTY,
            "translated checked_squared_distance",
        );
        assert_eq!(
            d_ab.to_bits(),
            translated.to_bits(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: translation invariance mismatch"
        );
    }
}

#[test]
fn property_squared_distance_gate_matches_direct_predicate() {
    const PROPERTY: &str = "squared-distance-gate";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x4449_5354_4741_5445); // DISTGATE

    for case in 0..cases {
        let a = Vec3::new(
            rng.exact_binary_f64(64),
            rng.exact_binary_f64(64),
            rng.exact_binary_f64(64),
        );
        let b = Vec3::new(
            rng.exact_binary_f64(64),
            rng.exact_binary_f64(64),
            rng.exact_binary_f64(64),
        );
        let cutoff = (1 + rng.bounded_u64(512)) as f64 / 8.0;
        let gate = fuzz_ok(
            SquaredDistanceGate::new(cutoff),
            seed,
            case,
            PROPERTY,
            "SquaredDistanceGate::new",
        );
        let direct = fuzz_ok(
            a.checked_squared_distance(b),
            seed,
            case,
            PROPERTY,
            "direct checked_squared_distance",
        ) < cutoff * cutoff;
        let gated = fuzz_ok(gate.below(a, b), seed, case, PROPERTY, "gate.below");
        assert_eq!(
            gated, direct,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: gate/direct predicate mismatch"
        );
    }
}

#[test]
fn property_bounded_rotation_preserves_z_and_radius_about_pivot() {
    const PROPERTY: &str = "bounded-rotation";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x524f_5441_5445_5a31); // ROTATEZ1

    for case in 0..cases {
        let point = Vec3::new(
            rng.exact_binary_f64(32),
            rng.exact_binary_f64(32),
            rng.exact_binary_f64(32),
        );
        let pivot = Vec3::new(
            rng.exact_binary_f64(8),
            rng.exact_binary_f64(8),
            rng.exact_binary_f64(8),
        );
        let angle = (rng.bounded_u64(3001) as i64 - 1500) as f64 / 1000.0;
        let rotated = fuzz_ok(
            bounded_rotate_z(point, pivot, angle, -1.5, 1.5),
            seed,
            case,
            PROPERTY,
            "bounded_rotate_z",
        );
        assert_eq!(
            rotated.z.to_bits(),
            point.z.to_bits(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: Z coordinate changed"
        );

        let before = (point.x - pivot.x).powi(2) + (point.y - pivot.y).powi(2);
        let after = (rotated.x - pivot.x).powi(2) + (rotated.y - pivot.y).powi(2);
        let tolerance = 2.0e-12 * before.max(1.0);
        assert!(
            (before - after).abs() <= tolerance,
            "property={PROPERTY} seed=0x{seed:016x} case={case} before={before:e} after={after:e} tol={tolerance:e}"
        );
    }
}

#[test]
fn property_structural_result_identity_is_worker_independent_and_nonclinical() {
    const PROPERTY: &str = "worker-independent-structural-identity";
    let seed = parse_seed();
    let cases = (case_count() / 16).clamp(8, MAX_RUNTIME_CASES);
    let mut rng = rng_for(0x5255_4e49_4445_4e54); // RUNIDENT
    let loaded = load_profile(fixture_path()).unwrap_or_else(|error| {
        panic!(
            "property={PROPERTY} seed=0x{seed:016x}: repository V0 profile failed to load: {error}"
        )
    });

    for case in 0..cases {
        let work_items = 1 + rng.bounded_u64(129);
        let workers_a = 1 + rng.bounded_usize(32);
        let workers_b = 1 + rng.bounded_usize(32);
        let config_a = fuzz_ok(
            RunConfig::new(work_items, workers_a),
            seed,
            case,
            PROPERTY,
            "RunConfig::new(a)",
        );
        let config_b = fuzz_ok(
            RunConfig::new(work_items, workers_b),
            seed,
            case,
            PROPERTY,
            "RunConfig::new(b)",
        );
        let a = fuzz_ok(
            run_structural_fixture(&loaded, config_a),
            seed,
            case,
            PROPERTY,
            "run_structural_fixture(a)",
        );
        let b = fuzz_ok(
            run_structural_fixture(&loaded, config_b),
            seed,
            case,
            PROPERTY,
            "run_structural_fixture(b)",
        );

        assert_eq!(
            a.result_sha256, b.result_sha256,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: result identity differs by workers"
        );
        assert_eq!(
            a.diagnostic_xor_fnv1a64, b.diagnostic_xor_fnv1a64,
            "property={PROPERTY} seed=0x{seed:016x} case={case}: diagnostic differs by workers"
        );
        assert_eq!(
            a.min_pair_distance_squared.to_bits(),
            b.min_pair_distance_squared.to_bits(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: min distance differs by workers"
        );
        assert_eq!(
            a.max_pair_distance_squared.to_bits(),
            b.max_pair_distance_squared.to_bits(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: max distance differs by workers"
        );

        for (run_label, summary) in [("a", &a), ("b", &b)] {
            assert!(
                summary.result_identity_worker_independent,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: worker-independence flag false"
            );
            assert_eq!(
                summary.validation_level, "V0",
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: validation level promoted"
            );
            assert!(
                summary.non_clinical,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: non_clinical flag false"
            );
            assert_eq!(
                summary.inv_bio_001, INV_BIO_001,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: INV-BIO-001 changed"
            );
            assert!(
                !summary.biological_validity_claimed,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: biological validity promoted"
            );
            assert!(
                !summary.clinical_validity_claimed,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: clinical validity promoted"
            );
            assert!(
                !summary.performance_claim,
                "property={PROPERTY} seed=0x{seed:016x} case={case} run={run_label}: performance claim promoted"
            );
        }
    }
}

#[test]
fn property_nonfinite_and_out_of_domain_inputs_fail_closed() {
    const PROPERTY: &str = "fail-closed-inputs";
    let seed = parse_seed();
    let cases = case_count();
    let mut rng = rng_for(0x4641_494c_434c_4f53); // FAILCLOS

    for case in 0..cases {
        let finite = Vec3::new(
            rng.exact_binary_f64(8),
            rng.exact_binary_f64(8),
            rng.exact_binary_f64(8),
        );
        let bad = match rng.bounded_u64(3) {
            0 => Vec3::new(f64::NAN, 0.0, 0.0),
            1 => Vec3::new(0.0, f64::INFINITY, 0.0),
            _ => Vec3::new(0.0, 0.0, f64::NEG_INFINITY),
        };
        assert!(
            finite.checked_squared_distance(bad).is_err(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: non-finite distance input accepted"
        );
        assert!(
            SquaredDistanceGate::new(f64::NAN).is_err(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: NaN cutoff accepted"
        );
        assert!(
            bounded_rotate_z(finite, Vec3::new(0.0, 0.0, 0.0), 2.0, -1.0, 1.0).is_err(),
            "property={PROPERTY} seed=0x{seed:016x} case={case}: out-of-bounds rotation accepted"
        );
    }
}
