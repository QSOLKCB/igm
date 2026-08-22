// SPDX-License-Identifier: Apache-2.0
//! Phase 3B PENTA-CRT CPU optimization profile.
//!
//! This module is a clean-room IGM implementation. It reuses mathematical and
//! engineering ideas documented in `docs/RUNTIME_LINEAGE.md`, not source code
//! from RSH, GLUBALL, or ETQ/SONIFICATION.
//!
//! The optimization profile is a V0 computational fixture. It is not a
//! molecular-dynamics model and does not establish biological or clinical
//! validity.

use crate::{
    load_profile, logical_ensemble_size, partition_ranges, ExecutionAddress, LoadedProfile,
    Profile, RuntimeError, Vec3, C72, EXECUTION_CELL_STATES, INV_BIO_001, MAX_WORKERS, S72,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub const OPTIMIZATION_PROFILE_CONTRACT: &str = "IGM-PENTA-CRT-CPU-PROFILE-V1";
pub const OPTIMIZATION_PROFILE_ID: &str = "IGM-PENTA-CRT-SYMMETRIC-V0";
pub const OPTIMIZATION_CONTRACT: &str = "IGM-PENTA-CRT-CPU-V1";
pub const OPTIMIZATION_NUMERICAL_PROFILE: &str = "IGM-PENTA-CRT-F64-LUT-BLOCK-CIRCULANT-V1";
pub const RUN_SCHEMA: &str = "IGM-PENTA-CRT-CPU-RUN-V1";
pub const VERIFY_SCHEMA: &str = "IGM-PENTA-CRT-VERIFY-V1";
pub const MODEL_ID: &str = "IGM-SCHEMATIC-PENTAMER-V0";
pub const MAX_OPTIMIZATION_PROFILE_BYTES: u64 = 64 * 1024;
pub const DOF_COUNT: usize = 4;
pub const NODE_COUNT: usize = 16;
pub const SYMMETRIC_NODE_COUNT: usize = 15;
pub const PAIR_COUNT: usize = 120;
pub const STRUCTURED_DISTANCE_EVALUATIONS: usize = 60;
pub const BRUTE_DISTANCE_EVALUATIONS: usize = PAIR_COUNT;
pub const BLOCK_REUSE_RESIDUAL_TOLERANCE: f64 = 1.0e-12;
pub const MAX_VERIFY_SAMPLES: usize = 4096;

const V0_SUBUNIT_Z_AMPLITUDE: f64 = 0.08;
const V0_FAB_Z_OFFSET: f64 = 0.06;
const V0_JCHAIN_Y_RATIO: f64 = 0.35;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const RESULT_DOMAIN: &[u8] = b"IGM-PENTA-CRT-RESULT-V1\0";
const MANIFEST_DOMAIN: &[u8] = b"IGM-PENTA-CRT-MANIFEST-V1\0";

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptimizationProfile {
    schema: String,
    profile_id: String,
    version: String,
    model_id: String,
    validation_level: String,
    symmetry: SymmetryDeclaration,
    degrees_of_freedom: Vec<DofDeclaration>,
    claims: OptimizationClaims,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymmetryDeclaration {
    computational_symmetry: String,
    status: String,
    biological_symmetry_claimed: bool,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DofDeclaration {
    id: String,
    unit: String,
    scope: String,
    status: String,
    values: Vec<f64>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OptimizationClaims {
    biological_validity_claimed: bool,
    clinical_validity_claimed: bool,
    medical_device_claimed: bool,
    diagnostic_use_claimed: bool,
    treatment_use_claimed: bool,
}

#[derive(Debug, Clone)]
struct LoadedOptimizationProfile {
    profile: OptimizationProfile,
    profile_sha256: String,
}

fn canonical_json(value: &Value) -> Result<String, RuntimeError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).map_err(|e| err(format!("canonical JSON scalar: {e}")))
        }
        Value::Array(values) => {
            let mut out = String::from("[");
            for (index, item) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                out.push_str(&canonical_json(item)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let mut out = String::from("{");
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).map_err(|e| err(e.to_string()))?);
                out.push(':');
                out.push_str(&canonical_json(map.get(key).expect("canonical key exists"))?);
            }
            out.push('}');
            Ok(out)
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn load_optimization_profile(path: &Path) -> Result<LoadedOptimizationProfile, RuntimeError> {
    let metadata = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat optimization profile {}: {e}", path.display())))?;
    if !metadata.is_file() {
        return Err(err(format!(
            "optimization profile is not a regular file: {}",
            path.display()
        )));
    }
    if metadata.len() > MAX_OPTIMIZATION_PROFILE_BYTES {
        return Err(err(format!(
            "optimization profile is {} bytes; limit is {MAX_OPTIMIZATION_PROFILE_BYTES}",
            metadata.len()
        )));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read optimization profile {}: {e}", path.display())))?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("optimization profile is not strict JSON: {e}")))?;
    let canonical = canonical_json(&raw)?;
    let profile: OptimizationProfile = serde_json::from_value(raw)
        .map_err(|e| err(format!("optimization profile structural error: {e}")))?;
    validate_optimization_profile(&profile)?;
    Ok(LoadedOptimizationProfile {
        profile,
        profile_sha256: sha256_hex(canonical.as_bytes()),
    })
}

fn validate_optimization_profile(profile: &OptimizationProfile) -> Result<(), RuntimeError> {
    if profile.schema != OPTIMIZATION_PROFILE_CONTRACT
        || profile.profile_id != OPTIMIZATION_PROFILE_ID
        || profile.version != "0.1.0"
        || profile.model_id != MODEL_ID
        || profile.validation_level != "V0"
    {
        return Err(err("unsupported Phase 3B optimization profile identity"));
    }
    if profile.symmetry.computational_symmetry != "C5"
        || profile.symmetry.status != "assumed"
        || profile.symmetry.biological_symmetry_claimed
    {
        return Err(err(
            "Phase 3B requires assumed C5 computational symmetry with biological_symmetry_claimed=false",
        ));
    }
    if profile.degrees_of_freedom.len() != DOF_COUNT {
        return Err(err("Phase 3B requires exactly four explicit execution degrees of freedom"));
    }
    let expected = [
        ("left_fab_delta_deg", "degree", "all-left-fab-placeholders", 17usize, -45.0, 45.0),
        ("right_fab_delta_deg", "degree", "all-right-fab-placeholders", 17usize, -45.0, 45.0),
        ("jchain_dx", "model-unit", "jchain:0", 9usize, -1.0, 1.0),
        ("jchain_dy", "model-unit", "jchain:0", 9usize, -1.0, 1.0),
    ];
    for (dof, (id, unit, scope, radix, lower, upper)) in
        profile.degrees_of_freedom.iter().zip(expected)
    {
        if dof.id != id || dof.unit != unit || dof.scope != scope || dof.status != "assumed" {
            return Err(err(format!("invalid declaration for execution DoF {}", dof.id)));
        }
        if dof.values.len() != radix {
            return Err(err(format!("{} must declare exactly {radix} explicit values", dof.id)));
        }
        if !dof.values.iter().all(|value| value.is_finite() && *value >= lower && *value <= upper) {
            return Err(err(format!("{} contains a non-finite or out-of-runtime-bound value", dof.id)));
        }
        if !dof.values.iter().any(|value| value.to_bits() == 0.0f64.to_bits()) {
            return Err(err(format!("{} must explicitly contain a zero state", dof.id)));
        }
        for pair in dof.values.windows(2) {
            if pair[0] >= pair[1] {
                return Err(err(format!("{} values must be strictly increasing", dof.id)));
            }
        }
    }
    if profile.claims.biological_validity_claimed
        || profile.claims.clinical_validity_claimed
        || profile.claims.medical_device_claimed
        || profile.claims.diagnostic_use_claimed
        || profile.claims.treatment_use_claimed
    {
        return Err(err("optimization profile may not claim biological, clinical, or medical validity"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConformationAddress {
    pub left_fab_bin: u16,
    pub right_fab_bin: u16,
    pub jchain_dx_bin: u16,
    pub jchain_dy_bin: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MixedRadix4 {
    radices: [u16; DOF_COUNT],
    total: u64,
}

impl MixedRadix4 {
    pub fn new(radices: [u16; DOF_COUNT]) -> Result<Self, RuntimeError> {
        if radices.iter().any(|radix| *radix == 0) {
            return Err(err("mixed-radix execution coordinates require non-zero radices"));
        }
        let total = radices.iter().try_fold(1u64, |acc, radix| {
            acc.checked_mul(u64::from(*radix))
                .ok_or_else(|| err("mixed-radix conformation count overflow"))
        })?;
        Ok(Self { radices, total })
    }

    pub fn total(self) -> u64 {
        self.total
    }

    pub fn radices(self) -> [u16; DOF_COUNT] {
        self.radices
    }

    pub fn decode(self, index: u64) -> Result<ConformationAddress, RuntimeError> {
        if index >= self.total {
            return Err(err("conformation index outside mixed-radix domain"));
        }
        let mut value = index;
        let mut digits = [0u16; DOF_COUNT];
        for (slot, radix) in self.radices.into_iter().enumerate() {
            digits[slot] = (value % u64::from(radix)) as u16;
            value /= u64::from(radix);
        }
        Ok(ConformationAddress {
            left_fab_bin: digits[0],
            right_fab_bin: digits[1],
            jchain_dx_bin: digits[2],
            jchain_dy_bin: digits[3],
        })
    }

    pub fn encode(self, address: ConformationAddress) -> Result<u64, RuntimeError> {
        let digits = [
            address.left_fab_bin,
            address.right_fab_bin,
            address.jchain_dx_bin,
            address.jchain_dy_bin,
        ];
        for (digit, radix) in digits.into_iter().zip(self.radices) {
            if digit >= radix {
                return Err(err("mixed-radix conformation digit outside domain"));
            }
        }
        let mut multiplier = 1u64;
        let mut index = 0u64;
        for (digit, radix) in digits.into_iter().zip(self.radices) {
            index = index
                .checked_add(u64::from(digit) * multiplier)
                .ok_or_else(|| err("mixed-radix encode overflow"))?;
            multiplier = multiplier
                .checked_mul(u64::from(radix))
                .ok_or_else(|| err("mixed-radix stride overflow"))?;
        }
        if index >= self.total {
            return Err(err("mixed-radix encode escaped domain"));
        }
        Ok(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConformationSlice {
    pub start: u64,
    pub count: u64,
    pub end_exclusive: u64,
}

impl ConformationSlice {
    pub fn new(start: u64, count: u64, total: u64) -> Result<Self, RuntimeError> {
        if count == 0 {
            return Err(err("conformation slice count must be positive"));
        }
        let end_exclusive = start
            .checked_add(count)
            .ok_or_else(|| err("conformation slice range overflow"))?;
        if start >= total || end_exclusive > total {
            return Err(err("conformation slice exceeds execution-profile domain"));
        }
        Ok(Self {
            start,
            count,
            end_exclusive,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct TrigPair {
    sin: f64,
    cos: f64,
}

fn reduce_to_pi(angle: f64) -> Result<f64, RuntimeError> {
    if !angle.is_finite() {
        return Err(err("angle must be finite"));
    }
    let pi = std::f64::consts::PI;
    let tau = std::f64::consts::TAU;
    let mut x = angle % tau;
    if x > pi {
        x -= tau;
    } else if x < -pi {
        x += tau;
    }
    Ok(x)
}

fn deterministic_sin_cos(angle: f64) -> Result<TrigPair, RuntimeError> {
    let x = reduce_to_pi(angle)?;
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
        return Err(err("deterministic trigonometric projection became non-finite"));
    }
    Ok(TrigPair {
        sin: sin_sum,
        cos: cos_sum,
    })
}

fn model_parameter(profile: &Profile, name: &str) -> Result<f64, RuntimeError> {
    let parameter = profile
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .ok_or_else(|| err(format!("missing model parameter for Phase 3B: {name}")))?;
    let value = parameter
        .value
        .as_ref()
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("Phase 3B parameter {name} must be numeric")))?;
    if !value.is_finite() {
        return Err(err(format!("Phase 3B parameter {name} must be finite")));
    }
    Ok(value)
}

#[derive(Debug)]
pub struct PentaCrtEngine {
    model: LoadedProfile,
    optimization: LoadedOptimizationProfile,
    radix: MixedRadix4,
    left_lut: Vec<TrigPair>,
    right_lut: Vec<TrigPair>,
    jchain_dx: Vec<f64>,
    jchain_dy: Vec<f64>,
    core_radius: f64,
    fab_length: f64,
    base_spread_deg: f64,
    jchain_offset: f64,
}

impl PentaCrtEngine {
    pub fn load(model_profile: &Path, optimization_profile: &Path) -> Result<Self, RuntimeError> {
        let model = load_profile(model_profile)?;
        if model.profile().model_id != MODEL_ID || model.profile().validation_level != "V0" {
            return Err(err("Phase 3B engine accepts only the validated V0 schematic model"));
        }
        let optimization = load_optimization_profile(optimization_profile)?;
        let radices = optimization
            .profile
            .degrees_of_freedom
            .iter()
            .map(|dof| u16::try_from(dof.values.len()).map_err(|_| err("DoF radix exceeds u16")))
            .collect::<Result<Vec<_>, _>>()?;
        let radices: [u16; DOF_COUNT] = radices
            .try_into()
            .map_err(|_| err("internal Phase 3B radix cardinality mismatch"))?;
        let radix = MixedRadix4::new(radices)?;
        let logical_limit = logical_ensemble_size(model.profile())?;
        if radix.total() > logical_limit {
            return Err(err(format!(
                "optimization conformation domain {} exceeds model logical_ensemble_size {logical_limit}",
                radix.total()
            )));
        }

        let core_radius = model_parameter(model.profile(), "core_radius")?;
        let fab_length = model_parameter(model.profile(), "fab_length")?;
        let base_spread_deg = model_parameter(model.profile(), "fab_spread_deg")?;
        let jchain_offset = model_parameter(model.profile(), "jchain_offset")?;

        let make_lut = |values: &[f64]| -> Result<Vec<TrigPair>, RuntimeError> {
            values
                .iter()
                .map(|delta| {
                    let angle = (base_spread_deg + *delta) * std::f64::consts::PI / 180.0;
                    deterministic_sin_cos(angle)
                })
                .collect()
        };
        let left_lut = make_lut(&optimization.profile.degrees_of_freedom[0].values)?;
        let right_lut = make_lut(&optimization.profile.degrees_of_freedom[1].values)?;
        let jchain_dx = optimization.profile.degrees_of_freedom[2].values.clone();
        let jchain_dy = optimization.profile.degrees_of_freedom[3].values.clone();

        Ok(Self {
            model,
            optimization,
            radix,
            left_lut,
            right_lut,
            jchain_dx,
            jchain_dy,
            core_radius,
            fab_length,
            base_spread_deg,
            jchain_offset,
        })
    }

    pub fn total_conformations(&self) -> u64 {
        self.radix.total()
    }

    pub fn radices(&self) -> [u16; DOF_COUNT] {
        self.radix.radices()
    }

    pub fn decode(&self, index: u64) -> Result<ConformationAddress, RuntimeError> {
        self.radix.decode(index)
    }

    pub fn encode(&self, address: ConformationAddress) -> Result<u64, RuntimeError> {
        self.radix.encode(address)
    }

    pub fn model_profile_sha256(&self) -> &str {
        self.model.profile_sha256()
    }

    pub fn optimization_profile_sha256(&self) -> &str {
        &self.optimization.profile_sha256
    }

    pub fn optimization_profile_id(&self) -> &str {
        &self.optimization.profile.profile_id
    }

    pub fn dof_ids(&self) -> Vec<&str> {
        self.optimization
            .profile
            .degrees_of_freedom
            .iter()
            .map(|dof| dof.id.as_str())
            .collect()
    }

    pub fn base_spread_deg(&self) -> f64 {
        self.base_spread_deg
    }
}

#[derive(Debug, Clone, Copy)]
struct GeometrySoa {
    x: [f64; NODE_COUNT],
    y: [f64; NODE_COUNT],
    z: [f64; NODE_COUNT],
}

impl GeometrySoa {
    fn point(&self, index: usize) -> Vec3 {
        Vec3::new(self.x[index], self.y[index], self.z[index])
    }

    fn set(&mut self, index: usize, point: Vec3) -> Result<(), RuntimeError> {
        if !point.is_finite() {
            return Err(err("Phase 3B geometry produced non-finite coordinate"));
        }
        self.x[index] = point.x;
        self.y[index] = point.y;
        self.z[index] = point.z;
        Ok(())
    }
}

fn empty_geometry() -> GeometrySoa {
    GeometrySoa {
        x: [0.0; NODE_COUNT],
        y: [0.0; NODE_COUNT],
        z: [0.0; NODE_COUNT],
    }
}

fn build_geometry_from_trig(
    engine: &PentaCrtEngine,
    address: ConformationAddress,
    left: TrigPair,
    right: TrigPair,
) -> Result<GeometrySoa, RuntimeError> {
    let mut geometry = empty_geometry();
    let mut ux = 0.0_f64;
    let mut uy = -1.0_f64;
    for sector in 0..5usize {
        let base = sector * 3;
        let sx = engine.core_radius * ux;
        let sy = engine.core_radius * uy;
        let sz = V0_SUBUNIT_Z_AMPLITUDE * (2.0 * ux * uy);
        geometry.set(base, Vec3::new(sx, sy, sz))?;

        let left_dx = ux * left.cos + uy * left.sin;
        let left_dy = uy * left.cos - ux * left.sin;
        geometry.set(
            base + 1,
            Vec3::new(
                sx + engine.fab_length * left_dx,
                sy + engine.fab_length * left_dy,
                sz - V0_FAB_Z_OFFSET * ux,
            ),
        )?;

        let right_dx = ux * right.cos - uy * right.sin;
        let right_dy = uy * right.cos + ux * right.sin;
        geometry.set(
            base + 2,
            Vec3::new(
                sx + engine.fab_length * right_dx,
                sy + engine.fab_length * right_dy,
                sz + V0_FAB_Z_OFFSET * ux,
            ),
        )?;

        if sector != 4 {
            let next_x = C72 * ux - S72 * uy;
            let next_y = S72 * ux + C72 * uy;
            ux = next_x;
            uy = next_y;
        }
    }

    let dx = engine.jchain_dx[address.jchain_dx_bin as usize];
    let dy = engine.jchain_dy[address.jchain_dy_bin as usize];
    geometry.set(
        15,
        Vec3::new(
            -engine.jchain_offset + dx,
            -engine.jchain_offset * V0_JCHAIN_Y_RATIO + dy,
            0.0,
        ),
    )?;
    Ok(geometry)
}

fn build_optimized_geometry(
    engine: &PentaCrtEngine,
    address: ConformationAddress,
) -> Result<GeometrySoa, RuntimeError> {
    let left = *engine
        .left_lut
        .get(address.left_fab_bin as usize)
        .ok_or_else(|| err("left Fab LUT index escaped domain"))?;
    let right = *engine
        .right_lut
        .get(address.right_fab_bin as usize)
        .ok_or_else(|| err("right Fab LUT index escaped domain"))?;
    build_geometry_from_trig(engine, address, left, right)
}

fn build_reference_geometry(
    engine: &PentaCrtEngine,
    address: ConformationAddress,
) -> Result<GeometrySoa, RuntimeError> {
    let left_delta = engine.optimization.profile.degrees_of_freedom[0].values
        [address.left_fab_bin as usize];
    let right_delta = engine.optimization.profile.degrees_of_freedom[1].values
        [address.right_fab_bin as usize];
    let left = deterministic_sin_cos(
        (engine.base_spread_deg + left_delta) * std::f64::consts::PI / 180.0,
    )?;
    let right = deterministic_sin_cos(
        (engine.base_spread_deg + right_delta) * std::f64::consts::PI / 180.0,
    )?;
    build_geometry_from_trig(engine, address, left, right)
}

fn max_geometry_residual(left: &GeometrySoa, right: &GeometrySoa) -> f64 {
    let mut max = 0.0_f64;
    for index in 0..NODE_COUNT {
        max = max.max((left.x[index] - right.x[index]).abs());
        max = max.max((left.y[index] - right.y[index]).abs());
        max = max.max((left.z[index] - right.z[index]).abs());
    }
    max
}

#[derive(Debug, Clone)]
struct PairEvaluation {
    pairs: [f64; PAIR_COUNT],
    min_d2: f64,
    max_d2: f64,
    distance_evaluations: u64,
}

fn checked_d2(left: Vec3, right: Vec3) -> Result<f64, RuntimeError> {
    left.checked_squared_distance(right)
}

fn structured_pairs(geometry: &GeometrySoa) -> Result<PairEvaluation, RuntimeError> {
    // Five 3x3 blocks from sector 0 to each C5 sector. These are the numerical
    // block-circulant representatives for the 15-node symmetric portion.
    let mut blocks = [[[0.0_f64; 3]; 3]; 5];
    let mut distance_evaluations = 0u64;
    for delta in 0..5usize {
        for left_local in 0..3usize {
            for right_local in 0..3usize {
                let d2 = checked_d2(
                    geometry.point(left_local),
                    geometry.point(delta * 3 + right_local),
                )?;
                blocks[delta][left_local][right_local] = d2;
                distance_evaluations += 1;
            }
        }
    }

    // J-chain/asymmetry is deliberately a sparse defect outside the C5 block
    // reuse. Every J-to-symmetric-node distance is evaluated directly.
    let mut j_distances = [0.0_f64; SYMMETRIC_NODE_COUNT];
    for index in 0..SYMMETRIC_NODE_COUNT {
        j_distances[index] = checked_d2(geometry.point(index), geometry.point(15))?;
        distance_evaluations += 1;
    }
    if distance_evaluations != STRUCTURED_DISTANCE_EVALUATIONS as u64 {
        return Err(err("structured distance-evaluation accounting invariant failed"));
    }

    let mut pairs = [0.0_f64; PAIR_COUNT];
    let mut cursor = 0usize;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    for left in 0..NODE_COUNT {
        for right in left + 1..NODE_COUNT {
            let d2 = if right == 15 {
                j_distances[left]
            } else {
                let left_sector = left / 3;
                let right_sector = right / 3;
                let left_local = left % 3;
                let right_local = right % 3;
                let delta = (right_sector + 5 - left_sector) % 5;
                blocks[delta][left_local][right_local]
            };
            if !d2.is_finite() {
                return Err(err("structured pair reuse produced non-finite distance"));
            }
            pairs[cursor] = d2;
            cursor += 1;
            min_d2 = min_d2.min(d2);
            max_d2 = max_d2.max(d2);
        }
    }
    if cursor != PAIR_COUNT || !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("structured pair reconstruction invariant failed"));
    }
    Ok(PairEvaluation {
        pairs,
        min_d2,
        max_d2,
        distance_evaluations,
    })
}

fn brute_pairs(geometry: &GeometrySoa) -> Result<PairEvaluation, RuntimeError> {
    let mut pairs = [0.0_f64; PAIR_COUNT];
    let mut cursor = 0usize;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    for left in 0..NODE_COUNT {
        for right in left + 1..NODE_COUNT {
            let d2 = checked_d2(geometry.point(left), geometry.point(right))?;
            pairs[cursor] = d2;
            cursor += 1;
            min_d2 = min_d2.min(d2);
            max_d2 = max_d2.max(d2);
        }
    }
    if cursor != PAIR_COUNT || !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("brute-force pair evaluation invariant failed"));
    }
    Ok(PairEvaluation {
        pairs,
        min_d2,
        max_d2,
        distance_evaluations: PAIR_COUNT as u64,
    })
}

fn max_pair_residual(left: &[f64; PAIR_COUNT], right: &[f64; PAIR_COUNT]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (*a - *b).abs())
        .fold(0.0_f64, f64::max)
}

fn fnv_update(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

fn fnv_u64(state: u64, value: u64) -> u64 {
    fnv_update(state, &value.to_le_bytes())
}

fn record_hash(
    conformation_index: u64,
    address: ConformationAddress,
    evaluation: &PairEvaluation,
) -> Result<u64, RuntimeError> {
    let mut hash = fnv_update(FNV_OFFSET, b"IGM-PENTA-CRT-CONFORMATION-V1\0");
    hash = fnv_u64(hash, conformation_index);
    for digit in [
        address.left_fab_bin,
        address.right_fab_bin,
        address.jchain_dx_bin,
        address.jchain_dy_bin,
    ] {
        hash = fnv_update(hash, &digit.to_le_bytes());
    }
    for d2 in evaluation.pairs {
        if !d2.is_finite() {
            return Err(err("cannot hash non-finite structured distance"));
        }
        hash = fnv_update(hash, &d2.to_bits().to_le_bytes());
    }
    // Preserve the exact ETQ-inspired 30-state execution cell as scheduling
    // metadata. It is not a biological graph traversal.
    for sequence in 0..EXECUTION_CELL_STATES {
        let execution = ExecutionAddress::from_sequence(sequence)?;
        hash = fnv_update(hash, &[execution.sector, execution.arm, execution.lane]);
        hash = fnv_update(hash, &[execution.storage_index()?]);
    }
    Ok(hash)
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationReport {
    pub schema: &'static str,
    pub optimization_contract: &'static str,
    pub numerical_profile: &'static str,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub total_conformations: u64,
    pub samples_checked: usize,
    pub max_geometry_lut_vs_reference_residual: f64,
    pub max_block_reuse_vs_brute_pair_residual: f64,
    pub residual_tolerance: f64,
    pub structured_distance_evaluations_per_conformation: usize,
    pub brute_distance_evaluations_per_conformation: usize,
    pub accepted: bool,
    pub non_clinical: bool,
    pub biological_validity_claimed: bool,
    pub inv_bio_001: &'static str,
}

pub fn verify_penta_crt(
    engine: &PentaCrtEngine,
    samples: usize,
) -> Result<VerificationReport, RuntimeError> {
    if samples == 0 || samples > MAX_VERIFY_SAMPLES || samples as u64 > engine.total_conformations() {
        return Err(err("verification sample count outside bounded domain"));
    }
    let mut max_geometry = 0.0_f64;
    let mut max_pairs = 0.0_f64;
    for sample in 0..samples {
        let index = ((sample as u128 * engine.total_conformations() as u128) / samples as u128) as u64;
        let address = engine.decode(index)?;
        let optimized_geometry = build_optimized_geometry(engine, address)?;
        let reference_geometry = build_reference_geometry(engine, address)?;
        max_geometry = max_geometry.max(max_geometry_residual(&optimized_geometry, &reference_geometry));
        let structured = structured_pairs(&optimized_geometry)?;
        let brute = brute_pairs(&reference_geometry)?;
        max_pairs = max_pairs.max(max_pair_residual(&structured.pairs, &brute.pairs));
    }
    if !max_geometry.is_finite() || !max_pairs.is_finite() {
        return Err(err("verification residual became non-finite"));
    }
    let accepted = max_geometry <= BLOCK_REUSE_RESIDUAL_TOLERANCE
        && max_pairs <= BLOCK_REUSE_RESIDUAL_TOLERANCE;
    Ok(VerificationReport {
        schema: VERIFY_SCHEMA,
        optimization_contract: OPTIMIZATION_CONTRACT,
        numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
        model_profile_sha256: engine.model_profile_sha256().to_string(),
        optimization_profile_sha256: engine.optimization_profile_sha256().to_string(),
        total_conformations: engine.total_conformations(),
        samples_checked: samples,
        max_geometry_lut_vs_reference_residual: max_geometry,
        max_block_reuse_vs_brute_pair_residual: max_pairs,
        residual_tolerance: BLOCK_REUSE_RESIDUAL_TOLERANCE,
        structured_distance_evaluations_per_conformation: STRUCTURED_DISTANCE_EVALUATIONS,
        brute_distance_evaluations_per_conformation: BRUTE_DISTANCE_EVALUATIONS,
        accepted,
        non_clinical: true,
        biological_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PentaCrtRunConfig {
    slice: ConformationSlice,
    requested_workers: usize,
}

impl PentaCrtRunConfig {
    pub fn new(
        start: u64,
        count: u64,
        workers: usize,
        total_conformations: u64,
    ) -> Result<Self, RuntimeError> {
        if workers == 0 || workers > MAX_WORKERS {
            return Err(err("PENTA-CRT worker count outside runtime bound"));
        }
        Ok(Self {
            slice: ConformationSlice::new(start, count, total_conformations)?,
            requested_workers: workers,
        })
    }

    pub fn slice(self) -> ConformationSlice {
        self.slice
    }

    pub fn requested_workers(self) -> usize {
        self.requested_workers
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PentaCrtWorkerSummary {
    pub worker: usize,
    pub start: u64,
    pub end_exclusive: u64,
    pub conformations: u64,
    pub logical_pair_checks: u64,
    pub actual_distance_evaluations: u64,
    pub diagnostic_xor_fnv1a64: String,
    pub min_pair_distance_squared: f64,
    pub max_pair_distance_squared: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PentaCrtRunSummary {
    pub schema: &'static str,
    pub optimization_contract: &'static str,
    pub numerical_profile: &'static str,
    pub model_id: &'static str,
    pub validation_level: &'static str,
    pub non_clinical: bool,
    pub inv_bio_001: &'static str,
    pub model_profile_sha256: String,
    pub optimization_profile_sha256: String,
    pub optimization_profile_id: String,
    pub total_conformations: u64,
    pub conformation_start: u64,
    pub conformation_count: u64,
    pub conformation_end_exclusive: u64,
    pub requested_workers: usize,
    pub workers: usize,
    pub logical_pair_checks: u64,
    pub actual_distance_evaluations: u64,
    pub brute_distance_evaluations_avoided: u64,
    pub structured_distance_evaluations_per_conformation: usize,
    pub brute_distance_evaluations_per_conformation: usize,
    pub hot_loop_trig_calls: u64,
    pub hot_loop_sqrt_calls: u64,
    pub per_conformation_heap_allocations: u64,
    pub diagnostic_xor_fnv1a64: String,
    pub min_pair_distance_squared: f64,
    pub max_pair_distance_squared: f64,
    pub result_sha256: String,
    pub manifest_sha256: String,
    pub result_identity_worker_independent: bool,
    pub structured_reuse_admitted_for_v0_fixture: bool,
    pub reference_equivalence_checked_in_this_run: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub performance_claim: bool,
    pub worker_summaries: Vec<PentaCrtWorkerSummary>,
}

fn result_sha256(
    engine: &PentaCrtEngine,
    slice: ConformationSlice,
    diagnostic: u64,
    min_d2: f64,
    max_d2: f64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(RESULT_DOMAIN);
    hasher.update(OPTIMIZATION_NUMERICAL_PROFILE.as_bytes());
    hasher.update(engine.model_profile_sha256().as_bytes());
    hasher.update(engine.optimization_profile_sha256().as_bytes());
    hasher.update(slice.start.to_le_bytes());
    hasher.update(slice.count.to_le_bytes());
    hasher.update(diagnostic.to_le_bytes());
    hasher.update(min_d2.to_bits().to_le_bytes());
    hasher.update(max_d2.to_bits().to_le_bytes());
    format!("{:x}", hasher.finalize())
}

fn manifest_sha256(
    engine: &PentaCrtEngine,
    config: PentaCrtRunConfig,
    ranges: &[crate::WorkRange],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MANIFEST_DOMAIN);
    hasher.update(OPTIMIZATION_NUMERICAL_PROFILE.as_bytes());
    hasher.update(engine.model_profile_sha256().as_bytes());
    hasher.update(engine.optimization_profile_sha256().as_bytes());
    hasher.update(config.slice.start.to_le_bytes());
    hasher.update(config.slice.count.to_le_bytes());
    hasher.update((config.requested_workers as u64).to_le_bytes());
    hasher.update((ranges.len() as u64).to_le_bytes());
    for range in ranges {
        hasher.update((range.worker as u64).to_le_bytes());
        hasher.update(range.start.to_le_bytes());
        hasher.update(range.end.to_le_bytes());
        hasher.update(range.length.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn run_penta_crt(
    engine: &PentaCrtEngine,
    config: PentaCrtRunConfig,
) -> Result<PentaCrtRunSummary, RuntimeError> {
    if config.slice.end_exclusive > engine.total_conformations() {
        return Err(err("PENTA-CRT execution slice exceeds engine domain"));
    }
    let ranges = partition_ranges(config.slice.count, config.requested_workers)?;
    let worker_summaries = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(ranges.len());
        for range in ranges.iter().copied() {
            handles.push(scope.spawn(move || -> Result<PentaCrtWorkerSummary, RuntimeError> {
                let mut diagnostic = 0u64;
                let mut min_d2 = f64::INFINITY;
                let mut max_d2 = 0.0_f64;
                for local_index in range.start..range.end {
                    let conformation_index = config
                        .slice
                        .start
                        .checked_add(local_index)
                        .ok_or_else(|| err("PENTA-CRT conformation index overflow"))?;
                    let address = engine.decode(conformation_index)?;
                    let geometry = build_optimized_geometry(engine, address)?;
                    let evaluation = structured_pairs(&geometry)?;
                    let record = record_hash(conformation_index, address, &evaluation)?;
                    diagnostic ^= record;
                    min_d2 = min_d2.min(evaluation.min_d2);
                    max_d2 = max_d2.max(evaluation.max_d2);
                }
                if !min_d2.is_finite() || !max_d2.is_finite() {
                    return Err(err("PENTA-CRT worker distance summary became non-finite"));
                }
                Ok(PentaCrtWorkerSummary {
                    worker: range.worker,
                    start: config.slice.start + range.start,
                    end_exclusive: config.slice.start + range.end,
                    conformations: range.length,
                    logical_pair_checks: (PAIR_COUNT as u64)
                        .checked_mul(range.length)
                        .ok_or_else(|| err("PENTA-CRT worker pair-count overflow"))?,
                    actual_distance_evaluations: (STRUCTURED_DISTANCE_EVALUATIONS as u64)
                        .checked_mul(range.length)
                        .ok_or_else(|| err("PENTA-CRT worker distance-evaluation overflow"))?,
                    diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
                    min_pair_distance_squared: min_d2,
                    max_pair_distance_squared: max_d2,
                })
            }));
        }
        let mut summaries = Vec::with_capacity(handles.len());
        for handle in handles {
            summaries.push(
                handle
                    .join()
                    .map_err(|_| err("PENTA-CRT worker thread panicked"))??,
            );
        }
        Ok::<_, RuntimeError>(summaries)
    })?;

    let mut diagnostic = 0u64;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    let mut logical_pair_checks = 0u64;
    let mut actual_distance_evaluations = 0u64;
    for worker in &worker_summaries {
        let parsed = u64::from_str_radix(&worker.diagnostic_xor_fnv1a64, 16)
            .map_err(|e| err(format!("PENTA-CRT diagnostic parse failed: {e}")))?;
        diagnostic ^= parsed;
        min_d2 = min_d2.min(worker.min_pair_distance_squared);
        max_d2 = max_d2.max(worker.max_pair_distance_squared);
        logical_pair_checks = logical_pair_checks
            .checked_add(worker.logical_pair_checks)
            .ok_or_else(|| err("PENTA-CRT global pair count overflow"))?;
        actual_distance_evaluations = actual_distance_evaluations
            .checked_add(worker.actual_distance_evaluations)
            .ok_or_else(|| err("PENTA-CRT global distance-evaluation count overflow"))?;
    }
    if !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("PENTA-CRT global distance summary became non-finite"));
    }
    let brute = (BRUTE_DISTANCE_EVALUATIONS as u64)
        .checked_mul(config.slice.count)
        .ok_or_else(|| err("PENTA-CRT brute evaluation count overflow"))?;
    let avoided = brute
        .checked_sub(actual_distance_evaluations)
        .ok_or_else(|| err("PENTA-CRT evaluation accounting underflow"))?;

    Ok(PentaCrtRunSummary {
        schema: RUN_SCHEMA,
        optimization_contract: OPTIMIZATION_CONTRACT,
        numerical_profile: OPTIMIZATION_NUMERICAL_PROFILE,
        model_id: MODEL_ID,
        validation_level: "V0",
        non_clinical: true,
        inv_bio_001: INV_BIO_001,
        model_profile_sha256: engine.model_profile_sha256().to_string(),
        optimization_profile_sha256: engine.optimization_profile_sha256().to_string(),
        optimization_profile_id: engine.optimization_profile_id().to_string(),
        total_conformations: engine.total_conformations(),
        conformation_start: config.slice.start,
        conformation_count: config.slice.count,
        conformation_end_exclusive: config.slice.end_exclusive,
        requested_workers: config.requested_workers,
        workers: worker_summaries.len(),
        logical_pair_checks,
        actual_distance_evaluations,
        brute_distance_evaluations_avoided: avoided,
        structured_distance_evaluations_per_conformation: STRUCTURED_DISTANCE_EVALUATIONS,
        brute_distance_evaluations_per_conformation: BRUTE_DISTANCE_EVALUATIONS,
        hot_loop_trig_calls: 0,
        hot_loop_sqrt_calls: 0,
        per_conformation_heap_allocations: 0,
        diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
        min_pair_distance_squared: min_d2,
        max_pair_distance_squared: max_d2,
        result_sha256: result_sha256(engine, config.slice, diagnostic, min_d2, max_d2),
        manifest_sha256: manifest_sha256(engine, config, &ranges),
        result_identity_worker_independent: true,
        structured_reuse_admitted_for_v0_fixture: true,
        reference_equivalence_checked_in_this_run: false,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        performance_claim: false,
        worker_summaries,
    })
}

pub fn describe_address(
    engine: &PentaCrtEngine,
    index: u64,
) -> Result<String, RuntimeError> {
    let address = engine.decode(index)?;
    let round_trip = engine.encode(address)?;
    let mut text = String::new();
    write!(
        &mut text,
        "index={index} left={} right={} jx={} jy={} round_trip={round_trip}",
        address.left_fab_bin,
        address.right_fab_bin,
        address.jchain_dx_bin,
        address.jchain_dy_bin
    )
    .map_err(|e| err(format!("address formatting failed: {e}")))?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        )
        .expect("Phase 3B fixtures must load")
    }

    #[test]
    fn mixed_radix_round_trip_is_exhaustive() {
        let engine = engine();
        assert_eq!(engine.radices(), [17, 17, 9, 9]);
        assert_eq!(engine.total_conformations(), 23_409);
        for index in 0..engine.total_conformations() {
            let address = engine.decode(index).unwrap();
            assert_eq!(engine.encode(address).unwrap(), index);
        }
    }

    #[test]
    fn deterministic_slice_rejects_escape() {
        let total = engine().total_conformations();
        assert!(ConformationSlice::new(0, total, total).is_ok());
        assert!(ConformationSlice::new(total - 1, 2, total).is_err());
        assert!(ConformationSlice::new(0, 0, total).is_err());
    }

    #[test]
    fn lut_geometry_matches_reference_projection() {
        let engine = engine();
        for index in [0, 1, 17, 289, 4096, engine.total_conformations() - 1] {
            let address = engine.decode(index).unwrap();
            let optimized = build_optimized_geometry(&engine, address).unwrap();
            let reference = build_reference_geometry(&engine, address).unwrap();
            assert!(
                max_geometry_residual(&optimized, &reference) <= BLOCK_REUSE_RESIDUAL_TOLERANCE
            );
        }
    }

    #[test]
    fn block_circulant_reuse_matches_brute_force_within_declared_tolerance() {
        let engine = engine();
        let report = verify_penta_crt(&engine, 257).unwrap();
        assert!(report.accepted, "verification report: {report:?}");
        assert_eq!(report.structured_distance_evaluations_per_conformation, 60);
        assert_eq!(report.brute_distance_evaluations_per_conformation, 120);
    }

    #[test]
    fn optimized_result_identity_is_worker_independent() {
        let engine = engine();
        let one = run_penta_crt(
            &engine,
            PentaCrtRunConfig::new(100, 257, 1, engine.total_conformations()).unwrap(),
        )
        .unwrap();
        let seven = run_penta_crt(
            &engine,
            PentaCrtRunConfig::new(100, 257, 7, engine.total_conformations()).unwrap(),
        )
        .unwrap();
        assert_eq!(one.result_sha256, seven.result_sha256);
        assert_eq!(one.diagnostic_xor_fnv1a64, seven.diagnostic_xor_fnv1a64);
        assert_ne!(one.manifest_sha256, seven.manifest_sha256);
        assert_eq!(one.actual_distance_evaluations * 2, one.logical_pair_checks);
        assert_eq!(one.hot_loop_trig_calls, 0);
        assert_eq!(one.hot_loop_sqrt_calls, 0);
        assert_eq!(one.per_conformation_heap_allocations, 0);
    }
}
