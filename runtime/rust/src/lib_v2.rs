// SPDX-License-Identifier: Apache-2.0
//! IGM deterministic structural research runtime.
//!
//! This runtime is a computational reference for a schematic V0 profile. It is
//! not a biological authority, medical device, diagnostic tool, treatment
//! system, or patient-specific model.
//!
//! INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: &str = "0.1.0";
pub const RUNTIME_CONTRACT: &str = "IGM-RUST-RUNTIME-V1";
pub const EXECUTION_CONTRACT: &str = "IGM-CRT-PENTAFOLD-30-V1";
pub const PROFILE_CONTRACT: &str = "IGM-MODEL-PROFILE-V1";
pub const SUPPORTED_MODEL_ID: &str = "IGM-SCHEMATIC-PENTAMER-V0";
pub const NUMERICAL_PROFILE: &str = "IGM-RUST-F64-DETERMINISTIC-POLY-V1";
pub const INV_BIO_001: &str = "Perfect Mathematics Does Not Equal Perfect Biological Reality";

pub const MAX_PROFILE_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_COMPONENTS: usize = 4_096;
pub const MAX_WORKERS: usize = 256;
pub const MAX_WORK_ITEMS: u64 = 100_000_000;
pub const EXECUTION_CELL_STATES: u8 = 30;
pub const CUDA_WARP_WIDTH: u8 = 32;

// Exact algebraic values are cos(2π/5)=(sqrt(5)-1)/4 and
// sin(2π/5)=sqrt(10+2sqrt(5))/4. The f64 literals are the declared runtime
// projection of those constants. Sector generation uses recurrence, not trig.
pub const C72: f64 = 0.309_016_994_374_947_45;
pub const S72: f64 = 0.951_056_516_295_153_5;

// Legacy V0 schematic drawing constants preserved exactly from the Phase-2
// browser fixture for cross-runtime parity. They are explicitly computational
// constants, not biological measurements or calibration values.
const V0_SUBUNIT_Z_AMPLITUDE: f64 = 0.08;
const V0_FAB_Z_OFFSET: f64 = 0.06;
const V0_JCHAIN_Y_RATIO: f64 = 0.35;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const RESULT_DOMAIN: &[u8] = b"IGM-RUST-RESULT-V1\0";
const MANIFEST_DOMAIN: &[u8] = b"IGM-RUST-MANIFEST-V1\0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError(pub String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuntimeError {}

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: String,
    pub model_id: String,
    pub version: String,
    pub validation_level: String,
    pub representation: Representation,
    pub components: Vec<Component>,
    pub parameters: Vec<Parameter>,
    #[serde(default)]
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    pub source_ids: Vec<String>,
    pub claims: Claims,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub primary: String,
    pub coordinate_adapter: Option<String>,
    pub coordinate_adapter_is_biological_ontology: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    pub kind: String,
    pub source_status: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub value: Option<Value>,
    pub unit: Option<String>,
    pub status: String,
    pub source_id: Option<String>,
    pub derivation: Option<String>,
    pub uncertainty: Option<Value>,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub id: String,
    pub kind: String,
    pub status: String,
    #[serde(default)]
    pub source_ids: Vec<String>,
    pub definition: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Claims {
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub medical_device_claimed: bool,
    pub diagnostic_use_claimed: bool,
    pub treatment_use_claimed: bool,
}

#[derive(Debug, Clone)]
pub struct LoadedProfile {
    profile: Profile,
    raw: Value,
    canonical_json: String,
    profile_sha256: String,
    source_registry_sha256: Option<String>,
}

impl LoadedProfile {
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    pub fn profile_sha256(&self) -> &str {
        &self.profile_sha256
    }

    pub fn source_registry_sha256(&self) -> Option<&str> {
        self.source_registry_sha256.as_deref()
    }

    pub fn canonical_json(&self) -> &str {
        &self.canonical_json
    }
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

fn valid_evidence_status(value: &str) -> bool {
    matches!(
        value,
        "observed" | "source-derived" | "calibrated" | "inferred" | "assumed" | "unknown"
    )
}

fn valid_derivation(value: &str) -> bool {
    matches!(
        value,
        "direct" | "transformed" | "calibrated" | "inferred" | "assumed" | "unknown"
    )
}

fn valid_primary(value: &str) -> bool {
    matches!(value, "articulated-geometry" | "tensor" | "graph" | "hybrid")
}

fn valid_coordinate_adapter(value: &str) -> bool {
    matches!(value, "cartesian" | "cyclic" | "vortex-inspired" | "custom")
}

fn is_simple_semver(value: &str) -> bool {
    let core = value
        .split_once('+')
        .map_or(value, |(left, _)| left)
        .split_once('-')
        .map_or_else(|| value.split_once('+').map_or(value, |(left, _)| left), |(left, _)| left);
    let mut parts = core.split('.');
    let valid_part = |part: &str| {
        !part.is_empty()
            && part.bytes().all(|b| b.is_ascii_digit())
            && (part == "0" || !part.starts_with('0'))
    };
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(a), Some(b), Some(c), None) if valid_part(a) && valid_part(b) && valid_part(c)
    )
}

fn unique_strings(values: &[String], label: &str) -> Result<BTreeSet<String>, RuntimeError> {
    let mut set = BTreeSet::new();
    for value in values {
        if value.is_empty() {
            return Err(err(format!("{label} may not contain an empty identifier")));
        }
        if !set.insert(value.clone()) {
            return Err(err(format!("{label} contains duplicate identifier: {value}")));
        }
    }
    Ok(set)
}

fn validate_raw_semantics(raw: &Value) -> Result<(), RuntimeError> {
    let root = raw
        .as_object()
        .ok_or_else(|| err("profile root must be a JSON object"))?;
    let parameters = root
        .get("parameters")
        .and_then(Value::as_array)
        .ok_or_else(|| err("parameters must be an array"))?;
    for parameter in parameters {
        let object = parameter
            .as_object()
            .ok_or_else(|| err("parameter entry must be an object"))?;
        let status = object
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| err("parameter status must be a string"))?;
        if status == "unknown" && object.contains_key("value") {
            return Err(err(format!(
                "unknown parameter {} may not carry a value",
                object.get("name").and_then(Value::as_str).unwrap_or("<unnamed>")
            )));
        }
    }
    Ok(())
}

fn find_source_registry(profile_path: &Path) -> Option<PathBuf> {
    let canonical = profile_path.canonicalize().ok()?;
    let parent = canonical.parent()?;
    for ancestor in parent.ancestors() {
        let candidate = ancestor.join("research/sources.json");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn validate_registry_sources(
    profile: &Profile,
    profile_path: &Path,
) -> Result<Option<String>, RuntimeError> {
    if profile.source_ids.is_empty() {
        return Ok(None);
    }
    let registry_path = find_source_registry(profile_path).ok_or_else(|| {
        err("profile declares source_ids but research/sources.json could not be located; evidence provenance fails closed")
    })?;
    let bytes = fs::read(&registry_path)
        .map_err(|e| err(format!("cannot read source registry {}: {e}", registry_path.display())))?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("source registry is not strict JSON: {e}")))?;
    let sources = raw
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| err("source registry requires sources array"))?;
    let mut registry_ids = BTreeSet::new();
    for source in sources {
        let id = source
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| err("source registry entry requires string id"))?;
        if !registry_ids.insert(id.to_string()) {
            return Err(err(format!("duplicate source registry id: {id}")));
        }
    }
    for id in &profile.source_ids {
        if !registry_ids.contains(id) {
            return Err(err(format!("profile source_id does not resolve in research/sources.json: {id}")));
        }
    }
    Ok(Some(sha256_hex(&bytes)))
}

fn expected_component_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for sector in ['a', 'b', 'c', 'd', 'e'] {
        ids.insert(format!("subunit:{sector}"));
        ids.insert(format!("fab:{sector}:l"));
        ids.insert(format!("fab:{sector}:r"));
    }
    ids.insert("jchain:0".to_string());
    ids
}

fn participants(constraint: &Constraint) -> Result<Vec<&str>, RuntimeError> {
    constraint
        .definition
        .get("participants")
        .and_then(Value::as_array)
        .ok_or_else(|| err(format!("constraint {} requires participants array", constraint.id)))?
        .iter()
        .map(|v| {
            v.as_str()
                .ok_or_else(|| err(format!("constraint {} participant must be string", constraint.id)))
        })
        .collect()
}

fn validate_constraints(profile: &Profile) -> Result<(), RuntimeError> {
    let component_by_id: BTreeMap<_, _> = profile
        .components
        .iter()
        .map(|component| (component.id.as_str(), component))
        .collect();

    let mut constraint_ids = BTreeSet::new();
    for constraint in &profile.constraints {
        if constraint.id.is_empty() || constraint.kind.is_empty() {
            return Err(err("constraint id and kind must be non-empty"));
        }
        if !constraint_ids.insert(constraint.id.as_str()) {
            return Err(err(format!("duplicate constraint id: {}", constraint.id)));
        }
    }
    let by_id: BTreeMap<_, _> = profile.constraints.iter().map(|c| (c.id.as_str(), c)).collect();

    let ring = by_id
        .get("constraint:five-sector-ring")
        .ok_or_else(|| err("missing constraint:five-sector-ring"))?;
    let ring_vec = participants(ring)?;
    let ring_set: BTreeSet<_> = ring_vec.iter().copied().collect();
    let expected_ring: BTreeSet<_> = [
        "subunit:a",
        "subunit:b",
        "subunit:c",
        "subunit:d",
        "subunit:e",
    ]
    .into_iter()
    .collect();
    if ring_vec.len() != 5 || ring_set.len() != 5 || ring_set != expected_ring {
        return Err(err("five-sector ring participants must be five distinct V0 subunits"));
    }
    if ring.definition.get("closed").and_then(Value::as_bool) != Some(true) {
        return Err(err("five-sector ring must declare closed=true"));
    }

    let j = by_id
        .get("constraint:jchain-marker")
        .ok_or_else(|| err("missing constraint:jchain-marker"))?;
    let j_participants = participants(j)?;
    if j_participants.len() != 3 || j_participants[0] != "jchain:0" {
        return Err(err("J-chain marker requires jchain:0 plus exactly two subunit participants"));
    }
    let j_unique: BTreeSet<_> = j_participants.iter().copied().collect();
    if j_unique.len() != 3 {
        return Err(err("J-chain marker participants must be distinct"));
    }
    for participant in &j_participants[1..] {
        let component = component_by_id
            .get(participant)
            .ok_or_else(|| err(format!("J-chain marker references missing component: {participant}")))?;
        if component.kind != "schematic-igm-subunit" {
            return Err(err(format!("J-chain marker target must be a schematic subunit: {participant}")));
        }
    }

    let arms = by_id
        .get("constraint:two-arms-per-sector")
        .ok_or_else(|| err("missing constraint:two-arms-per-sector"))?;
    if arms.definition.get("arms_per_subunit").and_then(Value::as_u64) != Some(2) {
        return Err(err("V0 adapter requires arms_per_subunit=2"));
    }
    Ok(())
}

fn validate_profile_adapter(profile: &Profile, raw: &Value) -> Result<(), RuntimeError> {
    if profile.schema != PROFILE_CONTRACT {
        return Err(err(format!("unsupported profile schema: {}", profile.schema)));
    }
    if profile.model_id != SUPPORTED_MODEL_ID {
        return Err(err(format!(
            "PR3 runtime adapter supports only {SUPPORTED_MODEL_ID}, got {}",
            profile.model_id
        )));
    }
    if !is_simple_semver(&profile.version) {
        return Err(err("profile version must be SemVer-compatible x.y.z"));
    }
    if profile.validation_level != "V0" {
        return Err(err("PR3 schematic runtime adapter must remain V0"));
    }
    if !valid_primary(&profile.representation.primary) || profile.representation.primary != "hybrid" {
        return Err(err("PR3 V0 adapter requires representation.primary=hybrid"));
    }
    match profile.representation.coordinate_adapter.as_deref() {
        Some(value) if valid_coordinate_adapter(value) && value == "cartesian" => {}
        _ => return Err(err("PR3 V0 adapter requires coordinate_adapter=cartesian")),
    }
    if profile.representation.coordinate_adapter_is_biological_ontology != Some(false) {
        return Err(err("coordinate adapter must explicitly declare biological ontology=false"));
    }
    if profile.components.is_empty() || profile.components.len() > MAX_COMPONENTS {
        return Err(err("component count outside runtime bound"));
    }

    let declared_sources = unique_strings(&profile.source_ids, "profile.source_ids")?;
    let mut component_ids = BTreeSet::new();
    for component in &profile.components {
        if component.id.is_empty()
            || !component
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b':' | b'-'))
        {
            return Err(err(format!("invalid component id: {}", component.id)));
        }
        if component.kind.is_empty() || !valid_evidence_status(&component.source_status) {
            return Err(err(format!("invalid component metadata for {}", component.id)));
        }
        if !component_ids.insert(component.id.clone()) {
            return Err(err(format!("duplicate component id: {}", component.id)));
        }
        let ids = unique_strings(&component.source_ids, &format!("{}.source_ids", component.id))?;
        for id in ids {
            if !declared_sources.contains(&id) {
                return Err(err(format!("component source_id is not declared by profile: {id}")));
            }
        }
    }
    if component_ids != expected_component_ids() {
        return Err(err("V0 adapter requires exactly 5 subunits, 10 Fab placeholders, and jchain:0"));
    }

    if profile.claims.biological_validity_claimed
        || profile.claims.clinical_validity_claimed
        || profile.claims.medical_device_claimed
        || profile.claims.diagnostic_use_claimed
        || profile.claims.treatment_use_claimed
    {
        return Err(err("V0 runtime refuses biological/clinical/medical claims"));
    }

    let mut names = BTreeSet::new();
    for parameter in &profile.parameters {
        if parameter.name.is_empty() || !names.insert(parameter.name.as_str()) {
            return Err(err(format!("duplicate/empty parameter name: {}", parameter.name)));
        }
        if !valid_evidence_status(&parameter.status) {
            return Err(err(format!("invalid parameter evidence status: {}", parameter.status)));
        }
        if let Some(derivation) = parameter.derivation.as_deref() {
            if !valid_derivation(derivation) {
                return Err(err(format!("invalid parameter derivation: {derivation}")));
            }
        }
        if let (Some(lower), Some(upper)) = (parameter.lower_bound, parameter.upper_bound) {
            if !lower.is_finite() || !upper.is_finite() || lower > upper {
                return Err(err(format!("invalid bounds for parameter {}", parameter.name)));
            }
        }
        if let Some(source_id) = parameter.source_id.as_deref() {
            if !declared_sources.contains(source_id) {
                return Err(err(format!(
                    "parameter {} source_id is not declared by profile: {source_id}",
                    parameter.name
                )));
            }
        }
        if matches!(parameter.status.as_str(), "observed" | "source-derived" | "calibrated") {
            let source_id = parameter
                .source_id
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| err(format!("evidence-backed parameter {} lacks source_id", parameter.name)))?;
            if !declared_sources.contains(source_id) {
                return Err(err(format!("evidence-backed parameter {} has unresolved source_id", parameter.name)));
            }
            if parameter.derivation.as_deref().unwrap_or("").is_empty() {
                return Err(err(format!("evidence-backed parameter {} lacks derivation", parameter.name)));
            }
        }
        if parameter.status == "observed" && parameter.derivation.as_deref() != Some("direct") {
            return Err(err(format!("observed parameter {} requires direct derivation", parameter.name)));
        }
        if parameter.status == "calibrated" && parameter.derivation.as_deref() != Some("calibrated") {
            return Err(err(format!("calibrated parameter {} requires calibrated derivation", parameter.name)));
        }
        if parameter.status == "unknown" {
            if parameter.source_id.is_some() || parameter.derivation.as_deref() != Some("unknown") {
                return Err(err(format!("unknown parameter {} must keep source null and derivation unknown", parameter.name)));
            }
        }
    }

    for constraint in &profile.constraints {
        if !valid_evidence_status(&constraint.status) {
            return Err(err(format!("invalid constraint evidence status: {}", constraint.status)));
        }
        let ids = unique_strings(&constraint.source_ids, &format!("{}.source_ids", constraint.id))?;
        for id in ids {
            if !declared_sources.contains(&id) {
                return Err(err(format!("constraint source_id is not declared by profile: {id}")));
            }
        }
    }

    validate_raw_semantics(raw)?;
    validate_constraints(profile)?;
    Ok(())
}

fn load_profile_value(raw: Value, path: &Path) -> Result<LoadedProfile, RuntimeError> {
    validate_raw_semantics(&raw)?;
    let profile: Profile = serde_json::from_value(raw.clone())
        .map_err(|e| err(format!("profile does not match runtime structural contract: {e}")))?;
    validate_profile_adapter(&profile, &raw)?;
    let source_registry_sha256 = validate_registry_sources(&profile, path)?;
    let canonical_json = canonical_json(&raw)?;
    let profile_sha256 = sha256_hex(canonical_json.as_bytes());
    Ok(LoadedProfile {
        profile,
        raw,
        canonical_json,
        profile_sha256,
        source_registry_sha256,
    })
}

/// Public loader. This is a checked API: callers cannot obtain a LoadedProfile
/// without native structural, semantic, provenance, and adapter validation.
pub fn load_profile(path: &Path) -> Result<LoadedProfile, RuntimeError> {
    let metadata = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat profile {}: {e}", path.display())))?;
    if !metadata.is_file() {
        return Err(err(format!("profile is not a regular file: {}", path.display())));
    }
    if metadata.len() > MAX_PROFILE_BYTES {
        return Err(err(format!(
            "profile is {} bytes; runtime limit is {MAX_PROFILE_BYTES}",
            metadata.len()
        )));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read profile {}: {e}", path.display())))?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("profile is not strict JSON: {e}")))?;
    load_profile_value(raw, path)
}

fn parameter_map(profile: &Profile) -> BTreeMap<&str, &Parameter> {
    profile.parameters.iter().map(|p| (p.name.as_str(), p)).collect()
}

fn numeric_parameter(params: &BTreeMap<&str, &Parameter>, name: &str) -> Result<f64, RuntimeError> {
    let parameter = params
        .get(name)
        .ok_or_else(|| err(format!("missing required runtime parameter: {name}")))?;
    let value = parameter
        .value
        .as_ref()
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("runtime parameter {name} must be an actual JSON number")))?;
    if !value.is_finite() {
        return Err(err(format!("runtime parameter {name} must be finite")));
    }
    if let Some(lower) = parameter.lower_bound {
        if value < lower {
            return Err(err(format!("runtime parameter {name} is below lower_bound")));
        }
    }
    if let Some(upper) = parameter.upper_bound {
        if value > upper {
            return Err(err(format!("runtime parameter {name} exceeds upper_bound")));
        }
    }
    Ok(value)
}

fn integer_parameter(params: &BTreeMap<&str, &Parameter>, name: &str) -> Result<u64, RuntimeError> {
    let parameter = params
        .get(name)
        .ok_or_else(|| err(format!("missing required runtime parameter: {name}")))?;
    let value = parameter
        .value
        .as_ref()
        .and_then(Value::as_u64)
        .ok_or_else(|| err(format!("runtime parameter {name} must be a nonnegative JSON integer")))?;
    let value_f64 = value as f64;
    if let Some(lower) = parameter.lower_bound {
        if value_f64 < lower {
            return Err(err(format!("runtime parameter {name} is below lower_bound")));
        }
    }
    if let Some(upper) = parameter.upper_bound {
        if value_f64 > upper {
            return Err(err(format!("runtime parameter {name} exceeds upper_bound")));
        }
    }
    Ok(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn checked_squared_distance(self, other: Self) -> Result<f64, RuntimeError> {
        if !self.is_finite() || !other.is_finite() {
            return Err(err("squared-distance input must be finite"));
        }
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
            return Err(err("squared-distance subtraction overflowed"));
        }
        let d2 = dx * dx + dy * dy + dz * dz;
        if !d2.is_finite() {
            return Err(err("derived squared distance is non-finite"));
        }
        Ok(d2)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GeometryNode {
    pub id: String,
    pub kind: String,
    pub source_status: String,
    pub source_ids: Vec<String>,
    pub notes: Option<String>,
    pub position: Vec3,
}

#[derive(Debug, Clone)]
pub struct GeometryState {
    pub nodes: Vec<GeometryNode>,
    pub jchain_participants: [String; 2],
}

impl GeometryState {
    pub fn positions(&self) -> impl Iterator<Item = Vec3> + '_ {
        self.nodes.iter().map(|node| node.position)
    }

    pub fn max_pairwise_distance_residual(&self, reference: &[(String, Vec3)]) -> Result<f64, RuntimeError> {
        if reference.len() != self.nodes.len() {
            return Err(err("reference geometry length mismatch"));
        }
        let mut max = 0.0_f64;
        for (node, (id, expected)) in self.nodes.iter().zip(reference) {
            if &node.id != id {
                return Err(err(format!("reference geometry id mismatch: {} != {id}", node.id)));
            }
            max = max.max((node.position.x - expected.x).abs());
            max = max.max((node.position.y - expected.y).abs());
            max = max.max((node.position.z - expected.z).abs());
        }
        if !max.is_finite() {
            return Err(err("geometry residual became non-finite"));
        }
        Ok(max)
    }
}

fn component<'a>(profile: &'a Profile, id: &str) -> Result<&'a Component, RuntimeError> {
    profile
        .components
        .iter()
        .find(|component| component.id == id)
        .ok_or_else(|| err(format!("missing component {id}")))
}

fn geometry_node(profile: &Profile, id: &str, position: Vec3) -> Result<GeometryNode, RuntimeError> {
    if !position.is_finite() {
        return Err(err(format!("component {id} produced non-finite geometry")));
    }
    let source = component(profile, id)?;
    Ok(GeometryNode {
        id: source.id.clone(),
        kind: source.kind.clone(),
        source_status: source.source_status.clone(),
        source_ids: source.source_ids.clone(),
        notes: source.notes.clone(),
        position,
    })
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

/// Deterministic f64 sine/cosine projection with fixed operation order.
///
/// This avoids platform/libm-dependent transcendental calls in identity-bearing
/// geometry. Fourteen Taylor terms after deterministic range reduction provide
/// ample accuracy for the bounded PR3 schematic domain.
fn deterministic_sin_cos(angle: f64) -> Result<(f64, f64), RuntimeError> {
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
    Ok((sin_sum, cos_sum))
}

/// Build the exact PR3 f64 projection of the Pages V0 schematic.
///
/// Pentamer sectors use a fixed C5 recurrence. The one declared Fab spread
/// angle uses the runtime's deterministic polynomial projection once during
/// model construction. No platform libm trig or sqrt occurs in the per-work-item
/// hot loop.
pub fn build_geometry(profile: &Profile) -> Result<GeometryState, RuntimeError> {
    let params = parameter_map(profile);
    let sectors = integer_parameter(&params, "assembly_sector_count")?;
    if sectors != 5 {
        return Err(err("V0 runtime requires exactly five schematic sectors"));
    }
    let core_radius = numeric_parameter(&params, "core_radius")?;
    let fab_length = numeric_parameter(&params, "fab_length")?;
    let spread_deg = numeric_parameter(&params, "fab_spread_deg")?;
    let jchain_offset = numeric_parameter(&params, "jchain_offset")?;

    let spread = spread_deg * std::f64::consts::PI / 180.0;
    let (spread_sin, spread_cos) = deterministic_sin_cos(spread)?;
    let ring_constraint = profile
        .constraints
        .iter()
        .find(|c| c.id == "constraint:five-sector-ring")
        .ok_or_else(|| err("missing constraint:five-sector-ring"))?;
    let ring_participants = participants(ring_constraint)?;
    let mut nodes = Vec::with_capacity(16);

    // theta_0=-pi/2 => unit radial vector (0,-1).
    let mut ux = 0.0_f64;
    let mut uy = -1.0_f64;
    for (sector_index, subunit_id) in ring_participants.iter().enumerate() {
        let key = subunit_id
            .strip_prefix("subunit:")
            .ok_or_else(|| err(format!("unsupported V0 subunit id: {subunit_id}")))?;
        let sx = core_radius * ux;
        let sy = core_radius * uy;
        let sz = V0_SUBUNIT_Z_AMPLITUDE * (2.0 * ux * uy); // sin(2 theta)
        nodes.push(geometry_node(profile, subunit_id, Vec3::new(sx, sy, sz))?);

        let left_dx = ux * spread_cos + uy * spread_sin;
        let left_dy = uy * spread_cos - ux * spread_sin;
        let right_dx = ux * spread_cos - uy * spread_sin;
        let right_dy = uy * spread_cos + ux * spread_sin;

        nodes.push(geometry_node(
            profile,
            &format!("fab:{key}:l"),
            Vec3::new(
                sx + fab_length * left_dx,
                sy + fab_length * left_dy,
                sz - V0_FAB_Z_OFFSET * ux,
            ),
        )?);
        nodes.push(geometry_node(
            profile,
            &format!("fab:{key}:r"),
            Vec3::new(
                sx + fab_length * right_dx,
                sy + fab_length * right_dy,
                sz + V0_FAB_Z_OFFSET * ux,
            ),
        )?);

        if sector_index != 4 {
            let next_x = C72 * ux - S72 * uy;
            let next_y = S72 * ux + C72 * uy;
            ux = next_x;
            uy = next_y;
        }
    }

    nodes.push(geometry_node(
        profile,
        "jchain:0",
        Vec3::new(-jchain_offset, -jchain_offset * V0_JCHAIN_Y_RATIO, 0.0),
    )?);

    let j_constraint = profile
        .constraints
        .iter()
        .find(|c| c.id == "constraint:jchain-marker")
        .ok_or_else(|| err("missing constraint:jchain-marker"))?;
    let j = participants(j_constraint)?;
    let jchain_participants = [j[1].to_string(), j[2].to_string()];

    Ok(GeometryState {
        nodes,
        jchain_participants,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ExecutionAddress {
    pub sector: u8,
    pub arm: u8,
    pub lane: u8,
}

impl ExecutionAddress {
    pub fn from_sequence(sequence: u8) -> Result<Self, RuntimeError> {
        if sequence >= EXECUTION_CELL_STATES {
            return Err(err("CRT execution sequence must be in [0,29]"));
        }
        Ok(Self {
            sector: sequence % 5,
            arm: sequence % 2,
            lane: sequence % 3,
        })
    }

    /// Chinese-remainder inverse for Z5 × Z2 × Z3.
    /// n = 6*sector + 15*arm + 10*lane (mod 30).
    pub fn sequence(self) -> Result<u8, RuntimeError> {
        if self.sector >= 5 || self.arm >= 2 || self.lane >= 3 {
            return Err(err("CRT execution address outside Z5 x Z2 x Z3"));
        }
        Ok((6 * self.sector + 15 * self.arm + 10 * self.lane) % EXECUTION_CELL_STATES)
    }

    /// Contiguous sector-major storage index, deliberately distinct from
    /// traversal order: idx = 6*sector + 3*arm + lane.
    pub fn storage_index(self) -> Result<u8, RuntimeError> {
        if self.sector >= 5 || self.arm >= 2 || self.lane >= 3 {
            return Err(err("storage address outside Z5 x Z2 x Z3"));
        }
        Ok(6 * self.sector + 3 * self.arm + self.lane)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct WorkRange {
    pub worker: usize,
    pub start: u64,
    pub end: u64,
    pub length: u64,
}

pub fn partition_ranges(items: u64, requested_workers: usize) -> Result<Vec<WorkRange>, RuntimeError> {
    if items == 0 || items > MAX_WORK_ITEMS {
        return Err(err("work item count outside runtime bound"));
    }
    if requested_workers == 0 || requested_workers > MAX_WORKERS {
        return Err(err("worker count outside runtime bound"));
    }
    let workers = requested_workers.min(usize::try_from(items).unwrap_or(usize::MAX));
    let base = items / workers as u64;
    let remainder = items % workers as u64;
    let mut cursor = 0_u64;
    let mut ranges = Vec::with_capacity(workers);
    for worker in 0..workers {
        let length = base + if (worker as u64) < remainder { 1 } else { 0 };
        let start = cursor;
        let end = start
            .checked_add(length)
            .ok_or_else(|| err("work range overflow"))?;
        ranges.push(WorkRange {
            worker,
            start,
            end,
            length,
        });
        cursor = end;
    }
    if cursor != items {
        return Err(err("deterministic partition coverage invariant failed"));
    }
    Ok(ranges)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquaredDistanceGate {
    cutoff_squared: f64,
}

impl SquaredDistanceGate {
    pub fn new(cutoff: f64) -> Result<Self, RuntimeError> {
        if !cutoff.is_finite() || cutoff <= 0.0 {
            return Err(err("distance cutoff must be finite and > 0"));
        }
        let cutoff_squared = cutoff * cutoff;
        if !cutoff_squared.is_finite() {
            return Err(err("distance cutoff squared overflowed"));
        }
        Ok(Self { cutoff_squared })
    }

    pub fn below(self, left: Vec3, right: Vec3) -> Result<bool, RuntimeError> {
        Ok(left.checked_squared_distance(right)? < self.cutoff_squared)
    }
}

/// Generic bounded Z-axis articulation primitive. It has no biological meaning
/// until a model profile/source adapter assigns one.
pub fn bounded_rotate_z(
    point: Vec3,
    pivot: Vec3,
    angle: f64,
    lower: f64,
    upper: f64,
) -> Result<Vec3, RuntimeError> {
    if !point.is_finite()
        || !pivot.is_finite()
        || !angle.is_finite()
        || !lower.is_finite()
        || !upper.is_finite()
        || lower > upper
        || angle < lower
        || angle > upper
    {
        return Err(err("bounded articulation input outside declared finite domain"));
    }
    let (s, c) = deterministic_sin_cos(angle)?;
    let x = point.x - pivot.x;
    let y = point.y - pivot.y;
    let result = Vec3::new(
        pivot.x + c * x - s * y,
        pivot.y + s * x + c * y,
        point.z,
    );
    if !result.is_finite() {
        return Err(err("bounded articulation produced non-finite geometry"));
    }
    Ok(result)
}

/// Validated generic execution request. The profile logical domain is not a
/// caller-supplied constructor argument; it is re-derived at execution time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunConfig {
    work_items: u64,
    requested_workers: usize,
}

impl RunConfig {
    pub fn new(work_items: u64, workers: usize) -> Result<Self, RuntimeError> {
        if work_items == 0 || work_items > MAX_WORK_ITEMS {
            return Err(err("work_items outside runtime bound"));
        }
        if workers == 0 || workers > MAX_WORKERS {
            return Err(err("workers outside runtime bound"));
        }
        Ok(Self {
            work_items,
            requested_workers: workers,
        })
    }

    pub fn work_items(self) -> u64 {
        self.work_items
    }

    pub fn requested_workers(self) -> usize {
        self.requested_workers
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkerSummary {
    pub worker: usize,
    pub start: u64,
    pub end: u64,
    pub work_items: u64,
    pub pair_checks: u64,
    pub crt_microstates: u64,
    pub diagnostic_xor_fnv1a64: String,
    pub min_pair_distance_squared: f64,
    pub max_pair_distance_squared: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RunSummary {
    pub schema: &'static str,
    pub runtime_contract: &'static str,
    pub runtime_version: &'static str,
    pub execution_contract: &'static str,
    pub numerical_profile: &'static str,
    pub model_id: String,
    pub model_version: String,
    pub validation_level: String,
    pub non_clinical: bool,
    pub inv_bio_001: &'static str,
    pub profile_sha256: String,
    pub source_registry_sha256: Option<String>,
    pub work_items: u64,
    pub requested_workers: usize,
    pub workers: usize,
    pub component_count: usize,
    pub pair_checks: u64,
    pub crt_microstates: u64,
    pub warp_width_target: u8,
    pub meaningful_warp_lanes: u8,
    pub padded_warp_lanes: u8,
    pub hot_loop_trig_calls: u64,
    pub hot_loop_sqrt_calls: u64,
    pub diagnostic_xor_fnv1a64: String,
    pub min_pair_distance_squared: f64,
    pub max_pair_distance_squared: f64,
    pub result_sha256: String,
    pub manifest_sha256: String,
    pub result_identity_worker_independent: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub performance_claim: bool,
    pub worker_summaries: Vec<WorkerSummary>,
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

fn evaluate_work_item(index: u64, geometry: &GeometryState) -> Result<(u64, f64, f64), RuntimeError> {
    let mut hash = fnv_update(FNV_OFFSET, b"IGM-STRUCTURAL-FIXTURE-WORK-V1\0");
    hash = fnv_u64(hash, index);
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    for i in 0..geometry.nodes.len() {
        for j in i + 1..geometry.nodes.len() {
            let d2 = geometry.nodes[i]
                .position
                .checked_squared_distance(geometry.nodes[j].position)?;
            min_d2 = min_d2.min(d2);
            max_d2 = max_d2.max(d2);
            hash = fnv_update(hash, &d2.to_bits().to_le_bytes());
        }
    }
    if !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("derived pair-distance summary is non-finite"));
    }

    // ETQ-inspired execution traversal. It is scheduling metadata only, not a
    // biological graph walk. Rotate the traversal start by work-item index so
    // each work item exercises address conversion without allocation.
    let start = (index % u64::from(EXECUTION_CELL_STATES)) as u8;
    for offset in 0..EXECUTION_CELL_STATES {
        let sequence = (start + offset) % EXECUTION_CELL_STATES;
        let address = ExecutionAddress::from_sequence(sequence)?;
        hash = fnv_update(hash, &[address.sector, address.arm, address.lane]);
        hash = fnv_update(hash, &[address.storage_index()?]);
    }
    Ok((hash, min_d2, max_d2))
}

fn result_sha256(
    profile_sha256: &str,
    work_items: u64,
    component_count: usize,
    pair_checks: u64,
    crt_microstates: u64,
    diagnostic: u64,
    min_d2: f64,
    max_d2: f64,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(RESULT_DOMAIN);
    hasher.update(NUMERICAL_PROFILE.as_bytes());
    hasher.update(profile_sha256.as_bytes());
    hasher.update(work_items.to_le_bytes());
    hasher.update((component_count as u64).to_le_bytes());
    hasher.update(pair_checks.to_le_bytes());
    hasher.update(crt_microstates.to_le_bytes());
    hasher.update(diagnostic.to_le_bytes());
    hasher.update(min_d2.to_bits().to_le_bytes());
    hasher.update(max_d2.to_bits().to_le_bytes());
    format!("{:x}", hasher.finalize())
}

fn manifest_sha256(
    profile_sha256: &str,
    work_items: u64,
    requested_workers: usize,
    ranges: &[WorkRange],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(MANIFEST_DOMAIN);
    hasher.update(NUMERICAL_PROFILE.as_bytes());
    hasher.update(profile_sha256.as_bytes());
    hasher.update(work_items.to_le_bytes());
    hasher.update((requested_workers as u64).to_le_bytes());
    hasher.update((ranges.len() as u64).to_le_bytes());
    for range in ranges {
        hasher.update((range.worker as u64).to_le_bytes());
        hasher.update(range.start.to_le_bytes());
        hasher.update(range.end.to_le_bytes());
        hasher.update(range.length.to_le_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub fn run_structural_fixture(
    loaded: &LoadedProfile,
    config: RunConfig,
) -> Result<RunSummary, RuntimeError> {
    // Recheck the loaded profile at the execution boundary. LoadedProfile fields
    // are private, but defense in depth keeps the boundary explicit.
    validate_profile_adapter(&loaded.profile, &loaded.raw)?;
    let logical_limit = logical_ensemble_size(&loaded.profile)?;
    if config.work_items > logical_limit {
        return Err(err(format!(
            "work_items {} exceeds profile logical_ensemble_size {logical_limit}",
            config.work_items
        )));
    }

    let geometry = build_geometry(&loaded.profile)?;
    let ranges = partition_ranges(config.work_items, config.requested_workers)?;
    let pair_count = geometry
        .nodes
        .len()
        .checked_mul(geometry.nodes.len().saturating_sub(1))
        .and_then(|v| v.checked_div(2))
        .ok_or_else(|| err("pair count overflow"))? as u64;
    let pair_checks = pair_count
        .checked_mul(config.work_items)
        .ok_or_else(|| err("pair check count overflow"))?;
    let crt_microstates = u64::from(EXECUTION_CELL_STATES)
        .checked_mul(config.work_items)
        .ok_or_else(|| err("CRT microstate count overflow"))?;

    let worker_summaries = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(ranges.len());
        for range in ranges.iter().copied() {
            let geometry = &geometry;
            handles.push(scope.spawn(move || -> Result<WorkerSummary, RuntimeError> {
                let mut diagnostic = 0_u64;
                let mut min_d2 = f64::INFINITY;
                let mut max_d2 = 0.0_f64;
                for index in range.start..range.end {
                    let (record, record_min, record_max) = evaluate_work_item(index, geometry)?;
                    diagnostic ^= record;
                    min_d2 = min_d2.min(record_min);
                    max_d2 = max_d2.max(record_max);
                }
                if !min_d2.is_finite() || !max_d2.is_finite() {
                    return Err(err("worker derived distance summary is non-finite"));
                }
                Ok(WorkerSummary {
                    worker: range.worker,
                    start: range.start,
                    end: range.end,
                    work_items: range.length,
                    pair_checks: pair_count
                        .checked_mul(range.length)
                        .ok_or_else(|| err("worker pair count overflow"))?,
                    crt_microstates: u64::from(EXECUTION_CELL_STATES)
                        .checked_mul(range.length)
                        .ok_or_else(|| err("worker CRT count overflow"))?,
                    diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
                    min_pair_distance_squared: min_d2,
                    max_pair_distance_squared: max_d2,
                })
            }));
        }
        let mut summaries = Vec::with_capacity(handles.len());
        for handle in handles {
            let summary = handle
                .join()
                .map_err(|_| err("worker thread panicked"))??;
            summaries.push(summary);
        }
        Ok::<_, RuntimeError>(summaries)
    })?;

    let mut diagnostic = 0_u64;
    let mut min_d2 = f64::INFINITY;
    let mut max_d2 = 0.0_f64;
    for worker in &worker_summaries {
        let value = u64::from_str_radix(&worker.diagnostic_xor_fnv1a64, 16)
            .map_err(|e| err(format!("internal diagnostic parse failed: {e}")))?;
        diagnostic ^= value;
        min_d2 = min_d2.min(worker.min_pair_distance_squared);
        max_d2 = max_d2.max(worker.max_pair_distance_squared);
    }
    if !min_d2.is_finite() || !max_d2.is_finite() {
        return Err(err("global derived distance summary is non-finite"));
    }

    let result_sha256 = result_sha256(
        &loaded.profile_sha256,
        config.work_items,
        geometry.nodes.len(),
        pair_checks,
        crt_microstates,
        diagnostic,
        min_d2,
        max_d2,
    );
    let manifest_sha256 = manifest_sha256(
        &loaded.profile_sha256,
        config.work_items,
        config.requested_workers,
        &ranges,
    );

    Ok(RunSummary {
        schema: "IGM-RUST-STRUCTURAL-RUN-V1",
        runtime_contract: RUNTIME_CONTRACT,
        runtime_version: VERSION,
        execution_contract: EXECUTION_CONTRACT,
        numerical_profile: NUMERICAL_PROFILE,
        model_id: loaded.profile.model_id.clone(),
        model_version: loaded.profile.version.clone(),
        validation_level: loaded.profile.validation_level.clone(),
        non_clinical: true,
        inv_bio_001: INV_BIO_001,
        profile_sha256: loaded.profile_sha256.clone(),
        source_registry_sha256: loaded.source_registry_sha256.clone(),
        work_items: config.work_items,
        requested_workers: config.requested_workers,
        workers: worker_summaries.len(),
        component_count: geometry.nodes.len(),
        pair_checks,
        crt_microstates,
        warp_width_target: CUDA_WARP_WIDTH,
        meaningful_warp_lanes: EXECUTION_CELL_STATES,
        padded_warp_lanes: CUDA_WARP_WIDTH - EXECUTION_CELL_STATES,
        hot_loop_trig_calls: 0,
        hot_loop_sqrt_calls: 0,
        diagnostic_xor_fnv1a64: format!("{diagnostic:016x}"),
        min_pair_distance_squared: min_d2,
        max_pair_distance_squared: max_d2,
        result_sha256,
        manifest_sha256,
        result_identity_worker_independent: true,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        performance_claim: false,
        worker_summaries,
    })
}

pub fn logical_ensemble_size(profile: &Profile) -> Result<u64, RuntimeError> {
    integer_parameter(&parameter_map(profile), "logical_ensemble_size")
}

pub fn default_evaluated_count(profile: &Profile) -> Result<u64, RuntimeError> {
    integer_parameter(&parameter_map(profile), "evaluated_sample_count")
}

pub fn browser_v0_reference() -> Vec<(String, Vec3)> {
    vec![
        ("subunit:a".into(), Vec3::new(6.123233995736766e-17, -1.0, -9.797174393178826e-18)),
        ("fab:a:l".into(), Vec3::new(-0.3594643403670435, -1.737011117965317, -1.3471114790620885e-17)),
        ("fab:a:r".into(), Vec3::new(0.35946434036704356, -1.737011117965317, -6.123233995736766e-18)),
        ("subunit:b".into(), Vec3::new(0.9510565162951535, -0.3090169943749474, -0.04702282018339785)),
        ("fab:b:l".into(), Vec3::new(1.5409151525728475, -0.8786368581513251, -0.10408621116110706)),
        ("fab:b:r".into(), Vec3::new(1.7630763326632413, -0.1948950515876937, 0.010040570794311358)),
        ("subunit:c".into(), Vec3::new(0.5877852522924731, 0.8090169943749475, 0.07608452130361229)),
        ("fab:c:l".into(), Vec3::new(1.3118022784367933, 1.1939836758593778, 0.040817406166063906)),
        ("fab:c:r".into(), Vec3::new(0.7301767579793561, 1.6165593518449581, 0.11135163644116067)),
        ("subunit:d".into(), Vec3::new(-0.587785252292473, 0.8090169943749475, -0.07608452130361229)),
        ("fab:d:l".into(), Vec3::new(-0.7301767579793559, 1.6165593518449581, -0.04081740616606391)),
        ("fab:d:r".into(), Vec3::new(-1.311802278436793, 1.1939836758593778, -0.11135163644116067)),
        ("subunit:e".into(), Vec3::new(-0.9510565162951536, -0.3090169943749473, 0.04702282018339783)),
        ("fab:e:l".into(), Vec3::new(-1.7630763326632413, -0.19489505158769338, 0.10408621116110706)),
        ("fab:e:r".into(), Vec3::new(-1.5409151525728475, -0.8786368581513251, -0.010040570794311386)),
        ("jchain:0".into(), Vec3::new(-0.32, -0.11199999999999999, 0.0)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture_path() -> &'static Path {
        Path::new("profiles/igm-schematic-pentamer-v0.json")
    }

    fn fixture_profile() -> LoadedProfile {
        load_profile(fixture_path()).expect("repository V0 profile must load")
    }

    fn fixture_raw() -> Value {
        serde_json::from_slice(&fs::read(fixture_path()).unwrap()).unwrap()
    }

    #[test]
    fn crt30_is_bijective_and_invertible() {
        let mut storage = BTreeSet::new();
        let mut addresses = BTreeSet::new();
        for sequence in 0..EXECUTION_CELL_STATES {
            let address = ExecutionAddress::from_sequence(sequence).unwrap();
            assert_eq!(address.sequence().unwrap(), sequence);
            assert!(storage.insert(address.storage_index().unwrap()));
            assert!(addresses.insert((address.sector, address.arm, address.lane)));
        }
        assert_eq!(storage.len(), 30);
        assert_eq!(addresses.len(), 30);
    }

    #[test]
    fn deterministic_partition_has_exact_coverage() {
        for workers in [1, 2, 3, 7, 16, 64] {
            let ranges = partition_ranges(4096, workers).unwrap();
            assert_eq!(ranges.first().unwrap().start, 0);
            assert_eq!(ranges.last().unwrap().end, 4096);
            for pair in ranges.windows(2) {
                assert_eq!(pair[0].end, pair[1].start);
            }
            assert_eq!(ranges.iter().map(|r| r.length).sum::<u64>(), 4096);
        }
    }

    #[test]
    fn pentafold_geometry_matches_pages_reference() {
        let loaded = fixture_profile();
        let geometry = build_geometry(loaded.profile()).unwrap();
        let residual = geometry
            .max_pairwise_distance_residual(&browser_v0_reference())
            .unwrap();
        assert!(residual < 2.0e-15, "browser/Rust coordinate residual {residual:e}");
    }

    #[test]
    fn result_identity_is_worker_independent() {
        let loaded = fixture_profile();
        let one = run_structural_fixture(&loaded, RunConfig::new(257, 1).unwrap()).unwrap();
        let seven = run_structural_fixture(&loaded, RunConfig::new(257, 7).unwrap()).unwrap();
        assert_eq!(one.result_sha256, seven.result_sha256);
        assert_eq!(one.diagnostic_xor_fnv1a64, seven.diagnostic_xor_fnv1a64);
        assert_eq!(one.min_pair_distance_squared.to_bits(), seven.min_pair_distance_squared.to_bits());
        assert_eq!(one.max_pair_distance_squared.to_bits(), seven.max_pair_distance_squared.to_bits());
        assert_ne!(one.manifest_sha256, seven.manifest_sha256);
    }

    #[test]
    fn manifest_records_requested_and_effective_workers() {
        let loaded = fixture_profile();
        let two = run_structural_fixture(&loaded, RunConfig::new(1, 2).unwrap()).unwrap();
        let seven = run_structural_fixture(&loaded, RunConfig::new(1, 7).unwrap()).unwrap();
        assert_eq!(two.workers, 1);
        assert_eq!(seven.workers, 1);
        assert_eq!(two.requested_workers, 2);
        assert_eq!(seven.requested_workers, 7);
        assert_eq!(two.result_sha256, seven.result_sha256);
        assert_ne!(two.manifest_sha256, seven.manifest_sha256);
    }

    #[test]
    fn hot_loop_uses_checked_squared_distance_gate() {
        let gate = SquaredDistanceGate::new(2.0).unwrap();
        assert!(gate
            .below(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0))
            .unwrap());
        assert!(!gate
            .below(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0))
            .unwrap());
        assert!(SquaredDistanceGate::new(1.0e200).is_err());
    }

    #[test]
    fn articulation_rejects_out_of_bounds_angles() {
        let point = Vec3::new(1.0, 0.0, 0.0);
        let pivot = Vec3::new(0.0, 0.0, 0.0);
        assert!(bounded_rotate_z(point, pivot, 0.5, -1.0, 1.0).is_ok());
        assert!(bounded_rotate_z(point, pivot, 2.0, -1.0, 1.0).is_err());
    }

    #[test]
    fn public_loader_rejects_schema_invalid_representation() {
        let mut raw = fixture_raw();
        raw["representation"]["primary"] = json!("not-a-schema-value");
        assert!(load_profile_value(raw, fixture_path()).is_err());
    }

    #[test]
    fn jchain_targets_must_exist_and_be_distinct() {
        let mut raw = fixture_raw();
        let constraints = raw["constraints"].as_array_mut().unwrap();
        let j = constraints
            .iter_mut()
            .find(|item| item["id"] == "constraint:jchain-marker")
            .unwrap();
        j["definition"]["participants"] = json!(["jchain:0", "subunit:a", "subunit:a"]);
        assert!(load_profile_value(raw, fixture_path()).is_err());
    }

    #[test]
    fn evidence_source_id_must_resolve_through_profile() {
        let mut raw = fixture_raw();
        let parameters = raw["parameters"].as_array_mut().unwrap();
        let p = parameters.iter_mut().find(|item| item["name"] == "core_radius").unwrap();
        p["status"] = json!("observed");
        p["source_id"] = json!("missing.source");
        p["derivation"] = json!("direct");
        assert!(load_profile_value(raw, fixture_path()).is_err());
    }

    #[test]
    fn run_rechecks_profile_logical_limit() {
        let loaded = fixture_profile();
        let logical = logical_ensemble_size(loaded.profile()).unwrap();
        let config = RunConfig::new(logical + 1, 1).unwrap();
        assert!(run_structural_fixture(&loaded, config).is_err());
    }

    #[test]
    fn derived_nonfinite_distance_is_rejected_before_hashing() {
        let mut raw = fixture_raw();
        let parameters = raw["parameters"].as_array_mut().unwrap();
        let p = parameters.iter_mut().find(|item| item["name"] == "core_radius").unwrap();
        p["value"] = json!(1.0e200);
        p.as_object_mut().unwrap().remove("lower_bound");
        p.as_object_mut().unwrap().remove("upper_bound");
        let loaded = load_profile_value(raw, fixture_path()).unwrap();
        assert!(run_structural_fixture(&loaded, RunConfig::new(1, 1).unwrap()).is_err());
    }

    #[test]
    fn deterministic_projection_is_identity_bearing() {
        let loaded = fixture_profile();
        let summary = run_structural_fixture(&loaded, RunConfig::new(1, 1).unwrap()).unwrap();
        assert_eq!(summary.numerical_profile, NUMERICAL_PROFILE);
    }
}
