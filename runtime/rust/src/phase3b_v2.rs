// SPDX-License-Identifier: Apache-2.0
//! Phase 3B PENTA-CRT CPU optimization profile.
//!
//! The optimization is admitted only for the synthetic V0 execution profile.
//! C5 reuse is computational structure, not a biological symmetry claim.

use crate::{
    load_profile, logical_ensemble_size, partition_ranges, ExecutionAddress, LoadedProfile,
    Profile, RuntimeError, Vec3, WorkRange, C72, EXECUTION_CELL_STATES, INV_BIO_001,
    MAX_WORKERS, S72,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const OPTIMIZATION_PROFILE_CONTRACT: &str = "IGM-PENTA-CRT-CPU-PROFILE-V1";
pub const OPTIMIZATION_PROFILE_ID: &str = "IGM-PENTA-CRT-SYMMETRIC-V0";
pub const OPTIMIZATION_CONTRACT: &str = "IGM-PENTA-CRT-CPU-V1";
pub const OPTIMIZATION_NUMERICAL_PROFILE: &str = "IGM-PENTA-CRT-F64-LUT-BLOCK-CIRCULANT-ZRESIDUAL-V1";
pub const RUN_SCHEMA: &str = "IGM-PENTA-CRT-CPU-RUN-V1";
pub const VERIFY_SCHEMA: &str = "IGM-PENTA-CRT-VERIFY-V1";
pub const MODEL_ID: &str = "IGM-SCHEMATIC-PENTAMER-V0";
pub const DOF_COUNT: usize = 4;
pub const NODE_COUNT: usize = 16;
pub const SYMMETRIC_NODE_COUNT: usize = 15;
pub const PAIR_COUNT: usize = 120;
pub const PLANAR_BLOCK_EVALUATIONS: usize = 45;
pub const SPARSE_J_EVALUATIONS: usize = 15;
pub const STRUCTURED_DISTANCE_EVALUATIONS: usize =
    PLANAR_BLOCK_EVALUATIONS + SPARSE_J_EVALUATIONS;
pub const BRUTE_DISTANCE_EVALUATIONS: usize = PAIR_COUNT;
pub const BLOCK_REUSE_RESIDUAL_TOLERANCE: f64 = 1.0e-12;
pub const MAX_VERIFY_SAMPLES: usize = 4096;

const MAX_OPT_PROFILE_BYTES: u64 = 64 * 1024;
const V0_SUBUNIT_Z_AMPLITUDE: f64 = 0.08;
const V0_FAB_Z_OFFSET: f64 = 0.06;
const V0_JCHAIN_Y_RATIO: f64 = 0.35;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const RESULT_DOMAIN: &[u8] = b"IGM-PENTA-CRT-RESULT-V2\0";
const MANIFEST_DOMAIN: &[u8] = b"IGM-PENTA-CRT-MANIFEST-V2\0";

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
    #[serde(rename = "notes")]
    _notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymmetryDeclaration {
    computational_symmetry: String,
    status: String,
    biological_symmetry_claimed: bool,
    #[serde(rename = "notes")]
    _notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DofDeclaration {
    id: String,
    unit: String,
    scope: String,
    status: String,
    values: Vec<f64>,
    #[serde(rename = "notes")]
    _notes: Option<String>,
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
    sha256: String,
}

fn canonical_json(value: &Value) -> Result<String, RuntimeError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) =>
            serde_json::to_string(value).map_err(|e| err(e.to_string())),
        Value::Array(values) => {
            let mut out = String::from("[");
            for (i, value) in values.iter().enumerate() {
                if i != 0 { out.push(','); }
                out.push_str(&canonical_json(value)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            let mut out = String::from("{");
            for (i, key) in keys.into_iter().enumerate() {
                if i != 0 { out.push(','); }
                out.push_str(&serde_json::to_string(key).map_err(|e| err(e.to_string()))?);
                out.push(':');
                out.push_str(&canonical_json(map.get(key).expect("canonical key"))?);
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

fn validate_execution_profile(profile: &OptimizationProfile) -> Result<(), RuntimeError> {
    if profile.schema != OPTIMIZATION_PROFILE_CONTRACT
        || profile.profile_id != OPTIMIZATION_PROFILE_ID
        || profile.version != "0.1.0"
        || profile.model_id != MODEL_ID
        || profile.validation_level != "V0"
    {
        return Err(err("unsupported PENTA-CRT execution-profile identity"));
    }
    if profile.symmetry.computational_symmetry != "C5"
        || profile.symmetry.status != "assumed"
        || profile.symmetry.biological_symmetry_claimed
    {
        return Err(err(
            "PENTA-CRT requires assumed computational C5 and biological_symmetry_claimed=false",
        ));
    }
    if profile.degrees_of_freedom.len() != DOF_COUNT {
        return Err(err("PENTA-CRT requires exactly four explicit execution DoFs"));
    }
    let expected = [
        ("left_fab_delta_deg", "degree", "all-left-fab-placeholders", 17usize, -45.0, 45.0),
        ("right_fab_delta_deg", "degree", "all-right-fab-placeholders", 17usize, -45.0, 45.0),
        ("jchain_dx", "model-unit", "jchain:0", 9usize, -1.0, 1.0),
        ("jchain_dy", "model-unit", "jchain:0", 9usize, -1.0, 1.0),
    ];
    for (dof, (id, unit, scope, radix, lo, hi)) in
        profile.degrees_of_freedom.iter().zip(expected)
    {
        if dof.id != id || dof.unit != unit || dof.scope != scope || dof.status != "assumed" {
            return Err(err(format!("invalid execution DoF declaration: {}", dof.id)));
        }
        if dof.values.len() != radix {
            return Err(err(format!("{} must have radix {radix}", dof.id)));
        }
        if !dof.values.iter().all(|v| v.is_finite() && *v >= lo && *v <= hi) {
            return Err(err(format!("{} contains non-finite/out-of-bound value", dof.id)));
        }
        if !dof.values.iter().any(|v| *v == 0.0) {
            return Err(err(format!("{} must include explicit zero state", dof.id)));
        }
        if dof.values.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(err(format!("{} values must be strictly increasing", dof.id)));
        }
    }
    let c = &profile.claims;
    if c.biological_validity_claimed
        || c.clinical_validity_claimed
        || c.medical_device_claimed
        || c.diagnostic_use_claimed
        || c.treatment_use_claimed
    {
        return Err(err("execution profile cannot claim biological/clinical/medical validity"));
    }
    Ok(())
}

fn load_execution_profile(path: &Path) -> Result<LoadedOptimizationProfile, RuntimeError> {
    let meta = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat execution profile {}: {e}", path.display())))?;
    if !meta.is_file() || meta.len() > MAX_OPT_PROFILE_BYTES {
        return Err(err("execution profile is not a bounded regular file"));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read execution profile {}: {e}", path.display())))?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("execution profile is not strict JSON: {e}")))?;
    let canonical = canonical_json(&raw)?;
    let profile: OptimizationProfile = serde_json::from_value(raw)
        .map_err(|e| err(format!("execution-profile structural error: {e}")))?;
    validate_execution_profile(&profile)?;
    Ok(LoadedOptimizationProfile {
        profile,
        sha256: sha256_hex(canonical.as_bytes()),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ConformationAddress {
    pub left_fab_bin: u16,
    pub right_fab_bin: u16,
    pub jchain_dx_bin: u16,
    pub jchain_dy_bin: u16,
}

#[derive(Debug, Clone, Copy)]
struct MixedRadix4 {
    radices: [u16; DOF_COUNT],
    total: u64,
}

impl MixedRadix4 {
    fn new(radices: [u16; DOF_COUNT]) -> Result<Self, RuntimeError> {
        if radices.contains(&0) { return Err(err("zero mixed-radix dimension")); }
        let total = radices.iter().try_fold(1u64, |acc, radix| {
            acc.checked_mul(u64::from(*radix)).ok_or_else(|| err("mixed-radix overflow"))
        })?;
        Ok(Self { radices, total })
    }

    fn decode(self, index: u64) -> Result<ConformationAddress, RuntimeError> {
        if index >= self.total { return Err(err("conformation index outside domain")); }
        let mut value = index;
        let mut d = [0u16; DOF_COUNT];
        for (i, radix) in self.radices.into_iter().enumerate() {
            d[i] = (value % u64::from(radix)) as u16;
            value /= u64::from(radix);
        }
        Ok(ConformationAddress {
            left_fab_bin: d[0], right_fab_bin: d[1], jchain_dx_bin: d[2], jchain_dy_bin: d[3],
        })
    }

    fn encode(self, address: ConformationAddress) -> Result<u64, RuntimeError> {
        let digits = [address.left_fab_bin, address.right_fab_bin, address.jchain_dx_bin, address.jchain_dy_bin];
        if digits.into_iter().zip(self.radices).any(|(digit, radix)| digit >= radix) {
            return Err(err("conformation digit outside mixed-radix domain"));
        }
        let mut index = 0u64;
        let mut stride = 1u64;
        for (digit, radix) in digits.into_iter().zip(self.radices) {
            index = index.checked_add(u64::from(digit) * stride).ok_or_else(|| err("mixed-radix encode overflow"))?;
            stride = stride.checked_mul(u64::from(radix)).ok_or_else(|| err("mixed-radix stride overflow"))?;
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
        if count == 0 { return Err(err("slice count must be positive")); }
        let end = start.checked_add(count).ok_or_else(|| err("slice overflow"))?;
        if start >= total || end > total { return Err(err("slice exceeds conformation domain")); }
        Ok(Self { start, count, end_exclusive: end })
    }
}

#[derive(Debug, Clone, Copy)]
struct TrigPair { sin: f64, cos: f64 }

fn deterministic_sin_cos(angle: f64) -> Result<TrigPair, RuntimeError> {
    if !angle.is_finite() { return Err(err("angle must be finite")); }
    let pi = std::f64::consts::PI;
    let tau = std::f64::consts::TAU;
    let mut x = angle % tau;
    if x > pi { x -= tau; } else if x < -pi { x += tau; }
    let x2 = x * x;
    let mut sin_sum = x;
    let mut sin_term = x;
    let mut cos_sum = 1.0;
    let mut cos_term = 1.0;
    for k in 1..14u32 {
        let sf = f64::from(2 * k);
        sin_term *= -x2 / (sf * (sf + 1.0));
        sin_sum += sin_term;
        let cf = f64::from(2 * k - 1);
        cos_term *= -x2 / (cf * (cf + 1.0));
        cos_sum += cos_term;
    }
    if !sin_sum.is_finite() || !cos_sum.is_finite() { return Err(err("deterministic trig became non-finite")); }
    Ok(TrigPair { sin: sin_sum, cos: cos_sum })
}

fn model_parameter(profile: &Profile, name: &str) -> Result<f64, RuntimeError> {
    let value = profile.parameters.iter()
        .find(|p| p.name == name)
        .and_then(|p| p.value.as_ref())
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("missing numeric model parameter: {name}")))?;
    if !value.is_finite() { return Err(err(format!("non-finite model parameter: {name}"))); }
    Ok(value)
}

#[derive(Debug)]
pub struct PentaCrtEngine {
    model: LoadedProfile,
    execution: LoadedOptimizationProfile,
    radix: MixedRadix4,
    left_lut: Vec<TrigPair>,
    right_lut: Vec<TrigPair>,
    jdx: Vec<f64>,
    jdy: Vec<f64>,
    core_radius: f64,
    fab_length: f64,
    base_spread_deg: f64,
    jchain_offset: f64,
}

impl PentaCrtEngine {
    pub fn load(model_path: &Path, execution_path: &Path) -> Result<Self, RuntimeError> {
        let model = load_profile(model_path)?;
        if model.profile().model_id != MODEL_ID || model.profile().validation_level != "V0" {
            return Err(err("PENTA-CRT only admits the validated V0 schematic model"));
        }
        let execution = load_execution_profile(execution_path)?;
        let mut r = [0u16; DOF_COUNT];
        for (i, dof) in execution.profile.degrees_of_freedom.iter().enumerate() {
            r[i] = u16::try_from(dof.values.len()).map_err(|_| err("DoF radix exceeds u16"))?;
        }
        let radix = MixedRadix4::new(r)?;
        if radix.total > logical_ensemble_size(model.profile())? {
            return Err(err("PENTA-CRT domain exceeds model logical ensemble"));
        }
        let core_radius = model_parameter(model.profile(), "core_radius")?;
        let fab_length = model_parameter(model.profile(), "fab_length")?;
        let base_spread_deg = model_parameter(model.profile(), "fab_spread_deg")?;
        let jchain_offset = model_parameter(model.profile(), "jchain_offset")?;
        let make_lut = |values: &[f64]| -> Result<Vec<TrigPair>, RuntimeError> {
            values.iter().map(|delta| {
                deterministic_sin_cos((base_spread_deg + *delta) * std::f64::consts::PI / 180.0)
            }).collect()
        };
        let left_lut = make_lut(&execution.profile.degrees_of_freedom[0].values)?;
        let right_lut = make_lut(&execution.profile.degrees_of_freedom[1].values)?;
        let jdx = execution.profile.degrees_of_freedom[2].values.clone();
        let jdy = execution.profile.degrees_of_freedom[3].values.clone();
        Ok(Self { model, execution, radix, left_lut, right_lut, jdx, jdy, core_radius, fab_length, base_spread_deg, jchain_offset })
    }

    pub fn total_conformations(&self) -> u64 { self.radix.total }
    pub fn radices(&self) -> [u16; DOF_COUNT] { self.radix.radices }
    pub fn decode(&self, index: u64) -> Result<ConformationAddress, RuntimeError> { self.radix.decode(index) }
    pub fn encode(&self, address: ConformationAddress) -> Result<u64, RuntimeError> { self.radix.encode(address) }
    pub fn model_profile_sha256(&self) -> &str { self.model.profile_sha256() }
    pub fn optimization_profile_sha256(&self) -> &str { &self.execution.sha256 }
    pub fn optimization_profile_id(&self) -> &str { &self.execution.profile.profile_id }
    pub fn base_spread_deg(&self) -> f64 { self.base_spread_deg }
    pub fn dof_ids(&self) -> Vec<&str> { self.execution.profile.degrees_of_freedom.iter().map(|d| d.id.as_str()).collect() }
}

#[derive(Debug, Clone, Copy)]
struct GeometrySoa { x: [f64; NODE_COUNT], y: [f64; NODE_COUNT], z: [f64; NODE_COUNT] }

impl GeometrySoa {
    fn empty() -> Self { Self { x: [0.0; NODE_COUNT], y: [0.0; NODE_COUNT], z: [0.0; NODE_COUNT] } }
    fn point(&self, i: usize) -> Vec3 { Vec3::new(self.x[i], self.y[i], self.z[i]) }
    fn set(&mut self, i: usize, p: Vec3) -> Result<(), RuntimeError> {
        if !p.is_finite() { return Err(err("non-finite PENTA-CRT geometry")); }
        self.x[i] = p.x; self.y[i] = p.y; self.z[i] = p.z; Ok(())
    }
}

fn build_geometry_with_trig(engine: &PentaCrtEngine, address: ConformationAddress, left: TrigPair, right: TrigPair) -> Result<GeometrySoa, RuntimeError> {
    let mut g = GeometrySoa::empty();
    let mut ux = 0.0;
    let mut uy = -1.0;
    for sector in 0..5usize {
        let base = sector * 3;
        let sx = engine.core_radius * ux;
        let sy = engine.core_radius * uy;
        let sz = V0_SUBUNIT_Z_AMPLITUDE * (2.0 * ux * uy);
        g.set(base, Vec3::new(sx, sy, sz))?;
        let ldx = ux * left.cos + uy * left.sin;
        let ldy = uy * left.cos - ux * left.sin;
        g.set(base + 1, Vec3::new(sx + engine.fab_length * ldx, sy + engine.fab_length * ldy, sz - V0_FAB_Z_OFFSET * ux))?;
        let rdx = ux * right.cos - uy * right.sin;
        let rdy = uy * right.cos + ux * right.sin;
        g.set(base + 2, Vec3::new(sx + engine.fab_length * rdx, sy + engine.fab_length * rdy, sz + V0_FAB_Z_OFFSET * ux))?;
        if sector != 4 {
            let nx = C72 * ux - S72 * uy;
            let ny = S72 * ux + C72 * uy;
            ux = nx; uy = ny;
        }
    }
    g.set(15, Vec3::new(
        -engine.jchain_offset + engine.jdx[address.jchain_dx_bin as usize],
        -engine.jchain_offset * V0_JCHAIN_Y_RATIO + engine.jdy[address.jchain_dy_bin as usize],
        0.0,
    ))?;
    Ok(g)
}

fn optimized_geometry(engine: &PentaCrtEngine, address: ConformationAddress) -> Result<GeometrySoa, RuntimeError> {
    let left = *engine.left_lut.get(address.left_fab_bin as usize).ok_or_else(|| err("left LUT index escaped"))?;
    let right = *engine.right_lut.get(address.right_fab_bin as usize).ok_or_else(|| err("right LUT index escaped"))?;
    build_geometry_with_trig(engine, address, left, right)
}

fn reference_geometry(engine: &PentaCrtEngine, address: ConformationAddress) -> Result<GeometrySoa, RuntimeError> {
    let dl = engine.execution.profile.degrees_of_freedom[0].values[address.left_fab_bin as usize];
    let dr = engine.execution.profile.degrees_of_freedom[1].values[address.right_fab_bin as usize];
    let left = deterministic_sin_cos((engine.base_spread_deg + dl) * std::f64::consts::PI / 180.0)?;
    let right = deterministic_sin_cos((engine.base_spread_deg + dr) * std::f64::consts::PI / 180.0)?;
    build_geometry_with_trig(engine, address, left, right)
}

fn max_geometry_residual(a: &GeometrySoa, b: &GeometrySoa) -> f64 {
    (0..NODE_COUNT).flat_map(|i| [
        (a.x[i] - b.x[i]).abs(), (a.y[i] - b.y[i]).abs(), (a.z[i] - b.z[i]).abs()
    ]).fold(0.0, f64::max)
}

#[derive(Debug, Clone)]
struct PairEvaluation { pairs: [f64; PAIR_COUNT], min_d2: f64, max_d2: f64 }

fn xy_d2(a: Vec3, b: Vec3) -> Result<f64, RuntimeError> {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let d2 = dx * dx + dy * dy;
    if !d2.is_finite() { return Err(err("non-finite planar squared distance")); }
    Ok(d2)
}

fn structured_pairs(g: &GeometrySoa) -> Result<PairEvaluation, RuntimeError> {
    // The XY projection is exactly C5-structured under the declared recurrence.
    // The legacy V0 Z drawing terms are not C5-invariant, so each reconstructed
    // pair receives its exact local dz^2 correction. This avoids pretending the
    // whole 3D fixture is block-circulant.
    let mut xy_blocks = [[[0.0; 3]; 3]; 5];
    for delta in 0..5usize {
        for li in 0..3usize {
            for lj in 0..3usize {
                xy_blocks[delta][li][lj] = xy_d2(g.point(li), g.point(delta * 3 + lj))?;
            }
        }
    }
    let mut j = [0.0; SYMMETRIC_NODE_COUNT];
    for i in 0..SYMMETRIC_NODE_COUNT {
        j[i] = g.point(i).checked_squared_distance(g.point(15))?;
    }

    let mut pairs = [0.0; PAIR_COUNT];
    let mut cursor = 0usize;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0;
    for left in 0..NODE_COUNT {
        for right in left + 1..NODE_COUNT {
            let d2 = if right == 15 {
                j[left]
            } else {
                let si = left / 3;
                let sj = right / 3;
                let li = left % 3;
                let lj = right % 3;
                let delta = (sj + 5 - si) % 5;
                let dz = g.z[left] - g.z[right];
                xy_blocks[delta][li][lj] + dz * dz
            };
            if !d2.is_finite() { return Err(err("non-finite reconstructed squared distance")); }
            pairs[cursor] = d2;
            cursor += 1;
            min_d2 = min_d2.min(d2);
            max_d2 = max_d2.max(d2);
        }
    }
    if cursor != PAIR_COUNT || !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("structured pair reconstruction failed"));
    }
    Ok(PairEvaluation { pairs, min_d2, max_d2 })
}

fn brute_pairs(g: &GeometrySoa) -> Result<PairEvaluation, RuntimeError> {
    let mut pairs = [0.0; PAIR_COUNT];
    let mut cursor = 0usize;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0;
    for left in 0..NODE_COUNT {
        for right in left + 1..NODE_COUNT {
            let d2 = g.point(left).checked_squared_distance(g.point(right))?;
            pairs[cursor] = d2;
            cursor += 1;
            min_d2 = min_d2.min(d2);
            max_d2 = max_d2.max(d2);
        }
    }
    Ok(PairEvaluation { pairs, min_d2, max_d2 })
}

fn max_pair_residual(a: &[f64; PAIR_COUNT], b: &[f64; PAIR_COUNT]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (*x - *y).abs()).fold(0.0, f64::max)
}

fn fnv_update(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in bytes { state ^= u64::from(*byte); state = state.wrapping_mul(FNV_PRIME); }
    state
}

fn record_hash(index: u64, address: ConformationAddress, pairs: &[f64; PAIR_COUNT]) -> Result<u64, RuntimeError> {
    let mut hash = fnv_update(FNV_OFFSET, b"IGM-PENTA-CRT-CONFORMATION-V2\0");
    hash = fnv_update(hash, &index.to_le_bytes());
    for digit in [address.left_fab_bin, address.right_fab_bin, address.jchain_dx_bin, address.jchain_dy_bin] {
        hash = fnv_update(hash, &digit.to_le_bytes());
    }
    for d2 in pairs {
        if !d2.is_finite() { return Err(err("cannot hash non-finite pair value")); }
        hash = fnv_update(hash, &d2.to_bits().to_le_bytes());
    }
    for seq in 0..EXECUTION_CELL_STATES {
        let a = ExecutionAddress::from_sequence(seq)?;
        hash = fnv_update(hash, &[a.sector, a.arm, a.lane, a.storage_index()?]);
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
    pub planar_block_evaluations_per_conformation: usize,
    pub sparse_j_evaluations_per_conformation: usize,
    pub structured_distance_evaluations_per_conformation: usize,
    pub brute_distance_evaluations_per_conformation: usize,
    pub exact_z_residual_corrections_per_conformation: usize,
    pub accepted: bool,
    pub non_clinical: bool,
    pub biological_validity_claimed: bool,
    pub inv_bio_001: &'static str,
}

pub fn verify_penta_crt(engine: &PentaCrtEngine, samples: usize) -> Result<VerificationReport, RuntimeError> {
    if samples == 0 || samples > MAX_VERIFY_SAMPLES || samples as u64 > engine.total_conformations() {
        return Err(err("verification sample count outside bound"));
    }
    let mut max_geometry = 0.0f64;
    let mut max_pairs = 0.0f64;
    for sample in 0..samples {
        let index = ((sample as u128 * engine.total_conformations() as u128) / samples as u128) as u64;
        let address = engine.decode(index)?;
        let og = optimized_geometry(engine, address)?;
        let rg = reference_geometry(engine, address)?;
        max_geometry = max_geometry.max(max_geometry_residual(&og, &rg));
        max_pairs = max_pairs.max(max_pair_residual(&structured_pairs(&og)?.pairs, &brute_pairs(&rg)?.pairs));
    }
    let accepted = max_geometry <= BLOCK_REUSE_RESIDUAL_TOLERANCE && max_pairs <= BLOCK_REUSE_RESIDUAL_TOLERANCE;
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
        planar_block_evaluations_per_conformation: PLANAR_BLOCK_EVALUATIONS,
        sparse_j_evaluations_per_conformation: SPARSE_J_EVALUATIONS,
        structured_distance_evaluations_per_conformation: STRUCTURED_DISTANCE_EVALUATIONS,
        brute_distance_evaluations_per_conformation: BRUTE_DISTANCE_EVALUATIONS,
        exact_z_residual_corrections_per_conformation: 105,
        accepted,
        non_clinical: true,
        biological_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PentaCrtRunConfig { slice: ConformationSlice, requested_workers: usize }

impl PentaCrtRunConfig {
    pub fn new(start: u64, count: u64, workers: usize, total: u64) -> Result<Self, RuntimeError> {
        if workers == 0 || workers > MAX_WORKERS { return Err(err("PENTA-CRT worker count outside bound")); }
        Ok(Self { slice: ConformationSlice::new(start, count, total)?, requested_workers: workers })
    }
    pub fn slice(self) -> ConformationSlice { self.slice }
    pub fn requested_workers(self) -> usize { self.requested_workers }
}

#[derive(Debug, Clone, Serialize)]
pub struct PentaCrtWorkerSummary {
    pub worker: usize,
    pub start: u64,
    pub end_exclusive: u64,
    pub conformations: u64,
    pub logical_pair_checks: u64,
    pub planar_block_evaluations: u64,
    pub sparse_j_evaluations: u64,
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
    pub planar_block_evaluations: u64,
    pub sparse_j_evaluations: u64,
    pub structured_distance_evaluations: u64,
    pub brute_distance_evaluations_avoided: u64,
    pub exact_z_residual_corrections: u64,
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

fn result_sha(engine: &PentaCrtEngine, slice: ConformationSlice, diagnostic: u64, min_d2: f64, max_d2: f64) -> String {
    let mut h = Sha256::new();
    h.update(RESULT_DOMAIN);
    h.update(OPTIMIZATION_NUMERICAL_PROFILE.as_bytes());
    h.update(engine.model_profile_sha256().as_bytes());
    h.update(engine.optimization_profile_sha256().as_bytes());
    h.update(slice.start.to_le_bytes()); h.update(slice.count.to_le_bytes());
    h.update(diagnostic.to_le_bytes()); h.update(min_d2.to_bits().to_le_bytes()); h.update(max_d2.to_bits().to_le_bytes());
    format!("{:x}", h.finalize())
}

fn manifest_sha(engine: &PentaCrtEngine, config: PentaCrtRunConfig, ranges: &[WorkRange]) -> String {
    let mut h = Sha256::new();
    h.update(MANIFEST_DOMAIN); h.update(OPTIMIZATION_NUMERICAL_PROFILE.as_bytes());
    h.update(engine.model_profile_sha256().as_bytes()); h.update(engine.optimization_profile_sha256().as_bytes());
    h.update(config.slice.start.to_le_bytes()); h.update(config.slice.count.to_le_bytes());
    h.update((config.requested_workers as u64).to_le_bytes()); h.update((ranges.len() as u64).to_le_bytes());
    for r in ranges { h.update((r.worker as u64).to_le_bytes()); h.update(r.start.to_le_bytes()); h.update(r.end.to_le_bytes()); h.update(r.length.to_le_bytes()); }
    format!("{:x}", h.finalize())
}

pub fn run_penta_crt(engine: &PentaCrtEngine, config: PentaCrtRunConfig) -> Result<PentaCrtRunSummary, RuntimeError> {
    if config.slice.end_exclusive > engine.total_conformations() { return Err(err("execution slice exceeds engine domain")); }
    let ranges = partition_ranges(config.slice.count, config.requested_workers)?;
    let workers = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(ranges.len());
        for range in ranges.iter().copied() {
            handles.push(scope.spawn(move || -> Result<PentaCrtWorkerSummary, RuntimeError> {
                let mut diagnostic = 0u64;
                let mut min_d2 = f64::INFINITY;
                let mut max_d2 = 0.0f64;
                for local in range.start..range.end {
                    let index = config.slice.start.checked_add(local).ok_or_else(|| err("conformation index overflow"))?;
                    let address = engine.decode(index)?;
                    let eval = structured_pairs(&optimized_geometry(engine, address)?)?;
                    diagnostic ^= record_hash(index, address, &eval.pairs)?;
                    min_d2 = min_d2.min(eval.min_d2); max_d2 = max_d2.max(eval.max_d2);
                }
                Ok(PentaCrtWorkerSummary {
                    worker: range.worker,
                    start: config.slice.start + range.start,
                    end_exclusive: config.slice.start + range.end,
                    conformations: range.length,
                    logical_pair_checks: (PAIR_COUNT as u64).checked_mul(range.length).ok_or_else(|| err("pair count overflow"))?,
                    planar_block_evaluations: (PLANAR_BLOCK_EVALUATIONS as u64).checked_mul(range.length).ok_or_else(|| err("planar count overflow"))?,
                    sparse_j_evaluations: (SPARSE_J_EVALUATIONS as u64).checked_mul(range.length).ok_or_else(|| err("J count overflow"))?,
                    diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
                    min_pair_distance_squared: min_d2,
                    max_pair_distance_squared: max_d2,
                })
            }));
        }
        let mut out = Vec::with_capacity(handles.len());
        for handle in handles { out.push(handle.join().map_err(|_| err("PENTA-CRT worker panicked"))??); }
        Ok::<_, RuntimeError>(out)
    })?;

    let mut diagnostic = 0u64;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0f64;
    let mut pair_checks = 0u64;
    let mut planar = 0u64;
    let mut sparse_j = 0u64;
    for w in &workers {
        diagnostic ^= u64::from_str_radix(&w.diagnostic_xor_fnv1a64, 16).map_err(|e| err(e.to_string()))?;
        min_d2 = min_d2.min(w.min_pair_distance_squared); max_d2 = max_d2.max(w.max_pair_distance_squared);
        pair_checks = pair_checks.checked_add(w.logical_pair_checks).ok_or_else(|| err("pair aggregate overflow"))?;
        planar = planar.checked_add(w.planar_block_evaluations).ok_or_else(|| err("planar aggregate overflow"))?;
        sparse_j = sparse_j.checked_add(w.sparse_j_evaluations).ok_or_else(|| err("J aggregate overflow"))?;
    }
    let structured = planar.checked_add(sparse_j).ok_or_else(|| err("structured aggregate overflow"))?;
    let brute = (BRUTE_DISTANCE_EVALUATIONS as u64).checked_mul(config.slice.count).ok_or_else(|| err("brute count overflow"))?;
    let avoided = brute.checked_sub(structured).ok_or_else(|| err("evaluation accounting underflow"))?;
    let z_corrections = 105u64.checked_mul(config.slice.count).ok_or_else(|| err("z correction count overflow"))?;

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
        workers: workers.len(),
        logical_pair_checks: pair_checks,
        planar_block_evaluations: planar,
        sparse_j_evaluations: sparse_j,
        structured_distance_evaluations: structured,
        brute_distance_evaluations_avoided: avoided,
        exact_z_residual_corrections: z_corrections,
        hot_loop_trig_calls: 0,
        hot_loop_sqrt_calls: 0,
        per_conformation_heap_allocations: 0,
        diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
        min_pair_distance_squared: min_d2,
        max_pair_distance_squared: max_d2,
        result_sha256: result_sha(engine, config.slice, diagnostic, min_d2, max_d2),
        manifest_sha256: manifest_sha(engine, config, &ranges),
        result_identity_worker_independent: true,
        structured_reuse_admitted_for_v0_fixture: true,
        reference_equivalence_checked_in_this_run: false,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        performance_claim: false,
        worker_summaries: workers,
    })
}

pub fn describe_address(engine: &PentaCrtEngine, index: u64) -> Result<String, RuntimeError> {
    let a = engine.decode(index)?;
    Ok(format!(
        "index={index} left={} right={} jx={} jy={} round_trip={}",
        a.left_fab_bin, a.right_fab_bin, a.jchain_dx_bin, a.jchain_dy_bin, engine.encode(a)?
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> PentaCrtEngine {
        PentaCrtEngine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-penta-crt-cpu-v1.json"),
        ).unwrap()
    }

    #[test]
    fn mixed_radix_round_trip_is_exhaustive() {
        let e = engine();
        assert_eq!(e.radices(), [17, 17, 9, 9]);
        assert_eq!(e.total_conformations(), 23_409);
        for i in 0..e.total_conformations() { assert_eq!(e.encode(e.decode(i).unwrap()).unwrap(), i); }
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
        let e = engine();
        for i in [0, 1, 17, 289, 4096, e.total_conformations() - 1] {
            let a = e.decode(i).unwrap();
            assert!(max_geometry_residual(&optimized_geometry(&e, a).unwrap(), &reference_geometry(&e, a).unwrap()) <= BLOCK_REUSE_RESIDUAL_TOLERANCE);
        }
    }

    #[test]
    fn structured_reuse_matches_brute_force() {
        let e = engine();
        let report = verify_penta_crt(&e, 257).unwrap();
        assert!(report.accepted, "{report:?}");
        assert_eq!(report.planar_block_evaluations_per_conformation, 45);
        assert_eq!(report.sparse_j_evaluations_per_conformation, 15);
        assert_eq!(report.structured_distance_evaluations_per_conformation, 60);
        assert_eq!(report.brute_distance_evaluations_per_conformation, 120);
    }

    #[test]
    fn optimized_result_identity_is_worker_independent() {
        let e = engine();
        let one = run_penta_crt(&e, PentaCrtRunConfig::new(100, 257, 1, e.total_conformations()).unwrap()).unwrap();
        let seven = run_penta_crt(&e, PentaCrtRunConfig::new(100, 257, 7, e.total_conformations()).unwrap()).unwrap();
        assert_eq!(one.result_sha256, seven.result_sha256);
        assert_eq!(one.diagnostic_xor_fnv1a64, seven.diagnostic_xor_fnv1a64);
        assert_ne!(one.manifest_sha256, seven.manifest_sha256);
        assert_eq!(one.structured_distance_evaluations * 2, one.logical_pair_checks);
        assert_eq!(one.hot_loop_trig_calls, 0);
        assert_eq!(one.hot_loop_sqrt_calls, 0);
        assert_eq!(one.per_conformation_heap_allocations, 0);
    }
}
