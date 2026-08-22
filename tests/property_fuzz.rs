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

#[test]
fn property_crt_addressing_is_total_only_on_declared_domains() {
    let cases = case_count();
    let mut rng = rng_for(0x4352_5433_3041_4444); // CRT30ADD

    for case in 0..cases {
        let sequence = rng.bounded_u64(256) as u8;
        match ExecutionAddress::from_sequence(sequence) {
            Ok(address) => {
                assert!(sequence < EXECUTION_CELL_STATES, "seed={} case={case}", parse_seed());
                assert_eq!(address.sequence().unwrap(), sequence, "seed={} case={case}", parse_seed());
                let storage = address.storage_index().unwrap();
                assert!(storage < EXECUTION_CELL_STATES, "seed={} case={case}", parse_seed());
            }
            Err(_) => assert!(sequence >= EXECUTION_CELL_STATES, "seed={} case={case}", parse_seed()),
        }

        let address = ExecutionAddress {
            sector: rng.bounded_u64(8) as u8,
            arm: rng.bounded_u64(5) as u8,
            lane: rng.bounded_u64(6) as u8,
        };
        let valid = address.sector < 5 && address.arm < 2 && address.lane < 3;
        match (address.sequence(), address.storage_index()) {
            (Ok(seq), Ok(storage)) => {
                assert!(valid, "seed={} case={case}", parse_seed());
                assert_eq!(ExecutionAddress::from_sequence(seq).unwrap(), address, "seed={} case={case}", parse_seed());
                assert!(storage < EXECUTION_CELL_STATES, "seed={} case={case}", parse_seed());
            }
            _ => assert!(!valid, "seed={} case={case}", parse_seed()),
        }
    }
}

#[test]
fn property_partition_ranges_are_gap_free_bounded_and_balanced() {
    let cases = case_count();
    let mut rng = rng_for(0x5041_5254_4954_494f); // PARTITIO

    for case in 0..cases {
        let items = 1 + rng.bounded_u64(100_000);
        let requested = 1 + rng.bounded_usize(MAX_WORKERS);
        let ranges = partition_ranges(items, requested).unwrap();
        let effective = requested.min(items as usize);
        assert_eq!(ranges.len(), effective, "seed={} case={case}", parse_seed());
        assert_eq!(ranges.first().unwrap().start, 0, "seed={} case={case}", parse_seed());
        assert_eq!(ranges.last().unwrap().end, items, "seed={} case={case}", parse_seed());

        let mut total = 0_u64;
        let mut min_len = u64::MAX;
        let mut max_len = 0_u64;
        for (worker, range) in ranges.iter().enumerate() {
            assert_eq!(range.worker, worker, "seed={} case={case}", parse_seed());
            assert!(range.length > 0, "seed={} case={case}", parse_seed());
            assert_eq!(range.end - range.start, range.length, "seed={} case={case}", parse_seed());
            if worker > 0 {
                assert_eq!(ranges[worker - 1].end, range.start, "seed={} case={case}", parse_seed());
            }
            total += range.length;
            min_len = min_len.min(range.length);
            max_len = max_len.max(range.length);
        }
        assert_eq!(total, items, "seed={} case={case}", parse_seed());
        assert!(max_len - min_len <= 1, "seed={} case={case}", parse_seed());
    }
}

#[test]
fn property_squared_distance_is_symmetric_and_translation_invariant() {
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
        let d_ab = a.checked_squared_distance(b).unwrap();
        let d_ba = b.checked_squared_distance(a).unwrap();
        assert_eq!(d_ab.to_bits(), d_ba.to_bits(), "seed={} case={case}", parse_seed());
        assert!(d_ab >= 0.0 && d_ab.is_finite(), "seed={} case={case}", parse_seed());

        let at = Vec3::new(a.x + t.x, a.y + t.y, a.z + t.z);
        let bt = Vec3::new(b.x + t.x, b.y + t.y, b.z + t.z);
        let translated = at.checked_squared_distance(bt).unwrap();
        assert_eq!(d_ab.to_bits(), translated.to_bits(), "seed={} case={case}", parse_seed());
    }
}

#[test]
fn property_squared_distance_gate_matches_direct_predicate() {
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
        let gate = SquaredDistanceGate::new(cutoff).unwrap();
        let direct = a.checked_squared_distance(b).unwrap() < cutoff * cutoff;
        assert_eq!(gate.below(a, b).unwrap(), direct, "seed={} case={case}", parse_seed());
    }
}

#[test]
fn property_bounded_rotation_preserves_z_and_radius_about_pivot() {
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
        let rotated = bounded_rotate_z(point, pivot, angle, -1.5, 1.5).unwrap();
        assert_eq!(rotated.z.to_bits(), point.z.to_bits(), "seed={} case={case}", parse_seed());

        let before = (point.x - pivot.x).powi(2) + (point.y - pivot.y).powi(2);
        let after = (rotated.x - pivot.x).powi(2) + (rotated.y - pivot.y).powi(2);
        let tolerance = 2.0e-12 * before.max(1.0);
        assert!((before - after).abs() <= tolerance, "seed={} case={case} before={before:e} after={after:e} tol={tolerance:e}", parse_seed());
    }
}

#[test]
fn property_structural_result_identity_is_worker_independent_and_nonclinical() {
    let cases = (case_count() / 16).clamp(8, MAX_RUNTIME_CASES);
    let mut rng = rng_for(0x5255_4e49_4445_4e54); // RUNIDENT
    let loaded = load_profile(fixture_path()).expect("repository V0 profile must load");

    for case in 0..cases {
        let work_items = 1 + rng.bounded_u64(129);
        let workers_a = 1 + rng.bounded_usize(32);
        let workers_b = 1 + rng.bounded_usize(32);
        let a = run_structural_fixture(&loaded, RunConfig::new(work_items, workers_a).unwrap()).unwrap();
        let b = run_structural_fixture(&loaded, RunConfig::new(work_items, workers_b).unwrap()).unwrap();

        assert_eq!(a.result_sha256, b.result_sha256, "seed={} case={case}", parse_seed());
        assert_eq!(a.diagnostic_xor_fnv1a64, b.diagnostic_xor_fnv1a64, "seed={} case={case}", parse_seed());
        assert_eq!(a.min_pair_distance_squared.to_bits(), b.min_pair_distance_squared.to_bits(), "seed={} case={case}", parse_seed());
        assert_eq!(a.max_pair_distance_squared.to_bits(), b.max_pair_distance_squared.to_bits(), "seed={} case={case}", parse_seed());
        assert!(a.result_identity_worker_independent && b.result_identity_worker_independent);
        assert_eq!(a.validation_level, "V0");
        assert!(a.non_clinical);
        assert_eq!(a.inv_bio_001, INV_BIO_001);
        assert!(!a.biological_validity_claimed);
        assert!(!a.clinical_validity_claimed);
        assert!(!a.performance_claim);
    }
}

#[test]
fn property_nonfinite_and_out_of_domain_inputs_fail_closed() {
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
        assert!(finite.checked_squared_distance(bad).is_err(), "seed={} case={case}", parse_seed());
        assert!(SquaredDistanceGate::new(f64::NAN).is_err(), "seed={} case={case}", parse_seed());
        assert!(bounded_rotate_z(finite, Vec3::new(0.0, 0.0, 0.0), 2.0, -1.0, 1.0).is_err(), "seed={} case={case}", parse_seed());
    }
}
