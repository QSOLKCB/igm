// SPDX-License-Identifier: Apache-2.0
//! Phase 4 replaceable evidence adapters.
//!
//! Source ingestion is a provenance/normalization boundary, not biological
//! authority. Adapters may normalize representation, units, indexing, and
//! explicitly declared transformations, but they may not silently strengthen
//! the claim supported by the source.

use crate::{RuntimeError, INV_BIO_001};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const SOURCE_ADAPTER_CONTRACT: &str = "IGM-SOURCE-ADAPTER-V1";
pub const CRYO_EM_ADAPTER_ID: &str = "IGM-CRYO-EM-PARAMETER-ADAPTER-V1";
pub const MD_ADAPTER_ID: &str = "IGM-MD-TRAJECTORY-ADAPTER-V1";
pub const BIOCHEMICAL_ADAPTER_ID: &str = "IGM-BIOCHEMICAL-CALIBRATION-ADAPTER-V1";
pub const EVIDENCE_INPUT_SCHEMA: &str = "IGM-EVIDENCE-INPUT-V1";
pub const EVIDENCE_CANDIDATE_SCHEMA: &str = "IGM-EVIDENCE-CANDIDATE-V1";
pub const EVIDENCE_BUNDLE_SCHEMA: &str = "IGM-EVIDENCE-BUNDLE-V1";
pub const EVIDENCE_BUNDLE_CONTRACT: &str = "IGM-PHASE4-EVIDENCE-BUNDLE-V1";
pub const SOURCE_REGISTRY_SCHEMA: &str = "igm-source-registry/1";
pub const SNAPSHOT_POLICY_SCHEMA: &str = "IGM-SOURCE-SNAPSHOT-POLICY-V1";
pub const MAX_EVIDENCE_INPUT_BYTES: u64 = 1024 * 1024;
pub const MAX_EVIDENCE_CANDIDATES: usize = 1024;

const CANDIDATE_DOMAIN: &[u8] = b"IGM-EVIDENCE-CANDIDATE-V1\0";
const BUNDLE_DOMAIN: &[u8] = b"IGM-EVIDENCE-BUNDLE-V1\0";

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
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
                out.push_str(&canonical_json(
                    map.get(key).expect("canonical key must exist"),
                )?);
            }
            out.push('}');
            Ok(out)
        }
    }
}

fn load_bounded_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RuntimeError> {
    let metadata = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat evidence file {}: {e}", path.display())))?;
    if !metadata.is_file() || metadata.len() > MAX_EVIDENCE_INPUT_BYTES {
        return Err(err(format!(
            "evidence file {} is not a bounded regular file",
            path.display()
        )));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read evidence file {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("{} is not valid strict JSON: {e}", path.display())))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceAccess {
    pub status: String,
    pub redistribution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub id: String,
    #[serde(rename = "class")]
    pub source_class: String,
    pub title: String,
    pub authority: String,
    pub url: String,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub pubmed: Option<String>,
    #[serde(default)]
    pub pdb: Option<String>,
    #[serde(default)]
    pub emdb: Option<String>,
    #[serde(default)]
    pub access: Option<SourceAccess>,
    #[serde(default)]
    pub supports: Vec<String>,
    #[serde(default)]
    pub does_not_support: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceRegistry {
    pub schema: String,
    pub project: String,
    pub sources: Vec<SourceRecord>,
}

impl SourceRegistry {
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let registry: Self = load_bounded_json(path)?;
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> Result<(), RuntimeError> {
        if self.schema != SOURCE_REGISTRY_SCHEMA || self.project != "QSOLKCB/igm" {
            return Err(err("unsupported Phase 4 source registry identity"));
        }
        if self.sources.is_empty() {
            return Err(err("source registry must not be empty"));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut identifiers = std::collections::BTreeSet::new();
        for source in &self.sources {
            if source.id.is_empty() || !ids.insert(source.id.as_str()) {
                return Err(err(format!("duplicate/empty source id: {}", source.id)));
            }
            if source.title.is_empty()
                || source.authority.is_empty()
                || !source.url.starts_with("https://")
            {
                return Err(err(format!("invalid source metadata: {}", source.id)));
            }
            for (kind, value) in [
                ("doi", source.doi.as_deref()),
                ("pdb", source.pdb.as_deref()),
                ("emdb", source.emdb.as_deref()),
            ] {
                if let Some(value) = value {
                    if !identifiers.insert((kind, value.to_ascii_lowercase())) {
                        return Err(err(format!(
                            "duplicate external source identifier: {kind}={value}"
                        )));
                    }
                }
            }
            if source.authority == "structural-source"
                && source.doi.is_none()
                && source.pdb.is_none()
                && source.emdb.is_none()
            {
                return Err(err(format!(
                    "structural source {} requires DOI, PDB, or EMDB identifier",
                    source.id
                )));
            }
        }
        Ok(())
    }

    pub fn source(&self, id: &str) -> Result<&SourceRecord, RuntimeError> {
        self.sources
            .iter()
            .find(|source| source.id == id)
            .ok_or_else(|| err(format!("evidence source is not registered: {id}")))
    }

    pub fn structural_source_count(&self) -> usize {
        self.sources
            .iter()
            .filter(|source| source.authority == "structural-source")
            .count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SnapshotMode {
    ReferenceOnly,
    HashOnly,
    Packaged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRef {
    pub mode: SnapshotMode,
    #[serde(default)]
    pub external_payload_sha256: Option<String>,
    pub external_payload_committed: bool,
    pub redistribution_permission_verified: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct SnapshotPolicyRecord {
    source_id: String,
    mode: SnapshotMode,
    redistribution_permission_verified: bool,
    external_payload_committed: bool,
    #[serde(default)]
    external_payload_sha256: Option<String>,
    reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotPolicy {
    schema: String,
    default_mode: SnapshotMode,
    records: Vec<SnapshotPolicyRecord>,
}

impl SnapshotPolicy {
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let policy: Self = load_bounded_json(path)?;
        if policy.schema != SNAPSHOT_POLICY_SCHEMA || policy.default_mode != SnapshotMode::ReferenceOnly {
            return Err(err("unsupported source snapshot policy"));
        }
        let mut seen = std::collections::BTreeSet::new();
        for record in &policy.records {
            if record.source_id.is_empty() || !seen.insert(record.source_id.as_str()) {
                return Err(err("duplicate/empty source snapshot policy record"));
            }
            if record.reason.is_empty() {
                return Err(err(format!(
                    "snapshot policy record {} requires reason",
                    record.source_id
                )));
            }
            validate_snapshot_shape(
                &SnapshotRef {
                    mode: record.mode.clone(),
                    external_payload_sha256: record.external_payload_sha256.clone(),
                    external_payload_committed: record.external_payload_committed,
                    redistribution_permission_verified: record.redistribution_permission_verified,
                    notes: None,
                },
                &record.source_id,
            )?;
        }
        Ok(policy)
    }

    fn validate_for_source(&self, source_id: &str, snapshot: &SnapshotRef) -> Result<(), RuntimeError> {
        validate_snapshot_shape(snapshot, source_id)?;
        if let Some(record) = self.records.iter().find(|record| record.source_id == source_id) {
            if snapshot.mode != record.mode
                || snapshot.external_payload_committed != record.external_payload_committed
                || snapshot.redistribution_permission_verified
                    != record.redistribution_permission_verified
                || snapshot.external_payload_sha256 != record.external_payload_sha256
            {
                return Err(err(format!(
                    "evidence snapshot does not match policy for source {source_id}"
                )));
            }
        } else if snapshot.mode != SnapshotMode::ReferenceOnly
            || snapshot.external_payload_committed
            || snapshot.redistribution_permission_verified
            || snapshot.external_payload_sha256.is_some()
        {
            return Err(err(format!(
                "source {source_id} has no explicit snapshot permission; only reference-only ingestion is allowed"
            )));
        }
        Ok(())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_snapshot_shape(snapshot: &SnapshotRef, source_id: &str) -> Result<(), RuntimeError> {
    match snapshot.mode {
        SnapshotMode::ReferenceOnly => {
            if snapshot.external_payload_committed
                || snapshot.redistribution_permission_verified
                || snapshot.external_payload_sha256.is_some()
            {
                return Err(err(format!(
                    "reference-only snapshot for {source_id} cannot carry payload/permission/hash"
                )));
            }
        }
        SnapshotMode::HashOnly => {
            let digest = snapshot
                .external_payload_sha256
                .as_deref()
                .ok_or_else(|| err(format!("hash-only snapshot for {source_id} requires SHA-256")))?;
            if snapshot.external_payload_committed || !valid_sha256(digest) {
                return Err(err(format!(
                    "hash-only snapshot for {source_id} requires lowercase SHA-256 and no committed payload"
                )));
            }
        }
        SnapshotMode::Packaged => {
            let digest = snapshot
                .external_payload_sha256
                .as_deref()
                .ok_or_else(|| err(format!("packaged snapshot for {source_id} requires SHA-256")))?;
            if !snapshot.external_payload_committed
                || !snapshot.redistribution_permission_verified
                || !valid_sha256(digest)
            {
                return Err(err(format!(
                    "packaged snapshot for {source_id} requires verified permission, committed payload, and lowercase SHA-256"
                )));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceTargetKind {
    Parameter,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceTarget {
    pub kind: EvidenceTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceUncertainty {
    pub kind: String,
    #[serde(default)]
    pub lower: Option<f64>,
    #[serde(default)]
    pub upper: Option<f64>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub level: Option<f64>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl EvidenceUncertainty {
    fn validate(&self) -> Result<(), RuntimeError> {
        for value in [self.lower, self.upper, self.value, self.level]
            .into_iter()
            .flatten()
        {
            if !value.is_finite() {
                return Err(err("evidence uncertainty must remain finite"));
            }
        }
        match self.kind.as_str() {
            "unknown" | "source-reported" => {
                if self.notes.as_deref().unwrap_or("").is_empty() {
                    return Err(err(format!(
                        "uncertainty kind {} requires explanatory notes",
                        self.kind
                    )));
                }
            }
            "interval" | "confidence-interval" => {
                let lower = self
                    .lower
                    .ok_or_else(|| err(format!("uncertainty kind {} requires lower", self.kind)))?;
                let upper = self
                    .upper
                    .ok_or_else(|| err(format!("uncertainty kind {} requires upper", self.kind)))?;
                if lower > upper {
                    return Err(err("uncertainty lower bound exceeds upper bound"));
                }
            }
            "standard-deviation" => {
                let value = self
                    .value
                    .ok_or_else(|| err("standard-deviation uncertainty requires value"))?;
                if value < 0.0 {
                    return Err(err("standard-deviation uncertainty must be non-negative"));
                }
            }
            other => return Err(err(format!("unsupported evidence uncertainty kind: {other}"))),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceInput {
    pub schema: String,
    pub adapter_id: String,
    pub observation_id: String,
    pub source_id: String,
    pub support_statement: String,
    pub target: EvidenceTarget,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub unit: Option<String>,
    pub derivation: String,
    pub uncertainty: EvidenceUncertainty,
    pub snapshot: SnapshotRef,
    #[serde(default)]
    pub notes: Option<String>,
}

impl EvidenceInput {
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let input: Self = load_bounded_json(path)?;
        input.validate_common()?;
        Ok(input)
    }

    fn validate_common(&self) -> Result<(), RuntimeError> {
        if self.schema != EVIDENCE_INPUT_SCHEMA
            || self.observation_id.is_empty()
            || self.source_id.is_empty()
            || self.support_statement.is_empty()
            || self.target.id.is_empty()
        {
            return Err(err("malformed Phase 4 evidence input identity"));
        }
        self.uncertainty.validate()?;
        match self.derivation.as_str() {
            "unknown" => {
                if self.value.is_some() {
                    return Err(err("unknown evidence input may not carry a value"));
                }
            }
            "direct" | "transformed" | "calibrated" | "inferred" | "assumed" => {
                if self.value.is_none() {
                    return Err(err(format!(
                        "evidence derivation {} requires an explicit value/definition",
                        self.derivation
                    )));
                }
            }
            other => return Err(err(format!("unsupported evidence derivation: {other}"))),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceLocator {
    pub url: String,
    pub doi: Option<String>,
    pub pubmed: Option<String>,
    pub pdb: Option<String>,
    pub emdb: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceCandidate {
    pub schema: &'static str,
    pub source_adapter_contract: &'static str,
    pub adapter_id: &'static str,
    pub observation_id: String,
    pub source_id: String,
    pub source_class: String,
    pub source_authority: String,
    pub source_locator: SourceLocator,
    pub source_access: SourceAccess,
    pub source_support_statement: String,
    pub source_does_not_support: Vec<String>,
    pub target: EvidenceTarget,
    pub value: Option<Value>,
    pub unit: Option<String>,
    pub status: String,
    pub derivation: String,
    pub uncertainty: EvidenceUncertainty,
    pub snapshot: SnapshotRef,
    pub claim_strengthening_detected: bool,
    pub validation_level_promoted_by_adapter: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub inv_bio_001: &'static str,
    pub candidate_sha256: String,
}

fn status_from_derivation(derivation: &str) -> Result<&'static str, RuntimeError> {
    match derivation {
        "direct" => Ok("observed"),
        "transformed" => Ok("source-derived"),
        "calibrated" => Ok("calibrated"),
        "inferred" => Ok("inferred"),
        "assumed" => Ok("assumed"),
        "unknown" => Ok("unknown"),
        other => Err(err(format!("unsupported evidence derivation: {other}"))),
    }
}

fn candidate_identity(candidate: &EvidenceCandidate) -> Result<String, RuntimeError> {
    let mut hasher = Sha256::new();
    hasher.update(CANDIDATE_DOMAIN);
    for value in [
        candidate.source_adapter_contract,
        candidate.adapter_id,
        candidate.observation_id.as_str(),
        candidate.source_id.as_str(),
        candidate.source_class.as_str(),
        candidate.source_authority.as_str(),
        candidate.source_support_statement.as_str(),
        candidate.target.id.as_str(),
        candidate.status.as_str(),
        candidate.derivation.as_str(),
        candidate.inv_bio_001,
    ] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(match candidate.target.kind {
        EvidenceTargetKind::Parameter => [0_u8],
        EvidenceTargetKind::Constraint => [1_u8],
    });
    hasher.update(canonical_json(&candidate.value.clone().unwrap_or(Value::Null))?.as_bytes());
    hasher.update(serde_json::to_vec(&candidate.unit).map_err(|e| err(e.to_string()))?);
    hasher.update(serde_json::to_vec(&candidate.uncertainty).map_err(|e| err(e.to_string()))?);
    hasher.update(serde_json::to_vec(&candidate.snapshot).map_err(|e| err(e.to_string()))?);
    hasher.update(serde_json::to_vec(&candidate.source_access).map_err(|e| err(e.to_string()))?);
    hasher.update(serde_json::to_vec(&candidate.source_locator).map_err(|e| err(e.to_string()))?);
    hasher.update(serde_json::to_vec(&candidate.source_does_not_support).map_err(|e| err(e.to_string()))?);
    Ok(format!("{:x}", hasher.finalize()))
}

pub trait SourceAdapter {
    fn adapter_id(&self) -> &'static str;
    fn target_kind(&self) -> EvidenceTargetKind;
    fn accepts_source_class(&self, source_class: &str) -> bool;
    fn accepts_derivation(&self, derivation: &str) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct CryoEmParameterAdapter;

impl SourceAdapter for CryoEmParameterAdapter {
    fn adapter_id(&self) -> &'static str {
        CRYO_EM_ADAPTER_ID
    }

    fn target_kind(&self) -> EvidenceTargetKind {
        EvidenceTargetKind::Parameter
    }

    fn accepts_source_class(&self, source_class: &str) -> bool {
        matches!(source_class, "public-structure" | "peer-reviewed-literature")
    }

    fn accepts_derivation(&self, derivation: &str) -> bool {
        matches!(derivation, "direct" | "transformed" | "inferred" | "unknown")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MolecularDynamicsTrajectoryAdapter;

impl SourceAdapter for MolecularDynamicsTrajectoryAdapter {
    fn adapter_id(&self) -> &'static str {
        MD_ADAPTER_ID
    }

    fn target_kind(&self) -> EvidenceTargetKind {
        EvidenceTargetKind::Parameter
    }

    fn accepts_source_class(&self, source_class: &str) -> bool {
        source_class == "molecular-dynamics"
    }

    fn accepts_derivation(&self, derivation: &str) -> bool {
        matches!(derivation, "transformed" | "inferred" | "unknown")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BiochemicalCalibrationConstraintAdapter;

impl SourceAdapter for BiochemicalCalibrationConstraintAdapter {
    fn adapter_id(&self) -> &'static str {
        BIOCHEMICAL_ADAPTER_ID
    }

    fn target_kind(&self) -> EvidenceTargetKind {
        EvidenceTargetKind::Constraint
    }

    fn accepts_source_class(&self, source_class: &str) -> bool {
        source_class == "biochemical-measurement"
    }

    fn accepts_derivation(&self, derivation: &str) -> bool {
        matches!(
            derivation,
            "direct" | "transformed" | "calibrated" | "inferred" | "unknown"
        )
    }
}

fn adapt_with<A: SourceAdapter>(
    adapter: A,
    registry: &SourceRegistry,
    snapshot_policy: &SnapshotPolicy,
    input: &EvidenceInput,
) -> Result<EvidenceCandidate, RuntimeError> {
    input.validate_common()?;
    if input.adapter_id != adapter.adapter_id() {
        return Err(err(format!(
            "evidence input adapter mismatch: expected {}, got {}",
            adapter.adapter_id(),
            input.adapter_id
        )));
    }
    if input.target.kind != adapter.target_kind() {
        return Err(err(format!(
            "adapter {} cannot emit target kind {:?}",
            adapter.adapter_id(),
            input.target.kind
        )));
    }
    if !adapter.accepts_derivation(&input.derivation) {
        return Err(err(format!(
            "adapter {} does not admit derivation {}",
            adapter.adapter_id(),
            input.derivation
        )));
    }

    let source = registry.source(&input.source_id)?;
    if !adapter.accepts_source_class(&source.source_class) {
        return Err(err(format!(
            "adapter {} does not accept source class {}",
            adapter.adapter_id(),
            source.source_class
        )));
    }
    let access = source
        .access
        .clone()
        .ok_or_else(|| err(format!("source {} lacks access/licence metadata", source.id)))?;
    if !source
        .supports
        .iter()
        .any(|statement| statement == &input.support_statement)
    {
        return Err(err(format!(
            "support_statement for {} must exactly match a registered supports statement",
            source.id
        )));
    }
    snapshot_policy.validate_for_source(&source.id, &input.snapshot)?;

    let mut candidate = EvidenceCandidate {
        schema: EVIDENCE_CANDIDATE_SCHEMA,
        source_adapter_contract: SOURCE_ADAPTER_CONTRACT,
        adapter_id: adapter.adapter_id(),
        observation_id: input.observation_id.clone(),
        source_id: source.id.clone(),
        source_class: source.source_class.clone(),
        source_authority: source.authority.clone(),
        source_locator: SourceLocator {
            url: source.url.clone(),
            doi: source.doi.clone(),
            pubmed: source.pubmed.clone(),
            pdb: source.pdb.clone(),
            emdb: source.emdb.clone(),
        },
        source_access: access,
        source_support_statement: input.support_statement.clone(),
        source_does_not_support: source.does_not_support.clone(),
        target: input.target.clone(),
        value: input.value.clone(),
        unit: input.unit.clone(),
        status: status_from_derivation(&input.derivation)?.to_string(),
        derivation: input.derivation.clone(),
        uncertainty: input.uncertainty.clone(),
        snapshot: input.snapshot.clone(),
        claim_strengthening_detected: false,
        validation_level_promoted_by_adapter: false,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
        candidate_sha256: String::new(),
    };
    candidate.candidate_sha256 = candidate_identity(&candidate)?;
    Ok(candidate)
}

pub fn adapt_evidence(
    registry: &SourceRegistry,
    snapshot_policy: &SnapshotPolicy,
    input: &EvidenceInput,
) -> Result<EvidenceCandidate, RuntimeError> {
    match input.adapter_id.as_str() {
        CRYO_EM_ADAPTER_ID => adapt_with(CryoEmParameterAdapter, registry, snapshot_policy, input),
        MD_ADAPTER_ID => adapt_with(
            MolecularDynamicsTrajectoryAdapter,
            registry,
            snapshot_policy,
            input,
        ),
        BIOCHEMICAL_ADAPTER_ID => adapt_with(
            BiochemicalCalibrationConstraintAdapter,
            registry,
            snapshot_policy,
            input,
        ),
        other => Err(err(format!("unsupported Phase 4 adapter id: {other}"))),
    }
}

pub fn adapt_evidence_files(
    registry_path: &Path,
    snapshot_policy_path: &Path,
    input_path: &Path,
) -> Result<EvidenceCandidate, RuntimeError> {
    let registry = SourceRegistry::load(registry_path)?;
    let policy = SnapshotPolicy::load(snapshot_policy_path)?;
    let input = EvidenceInput::load(input_path)?;
    adapt_evidence(&registry, &policy, &input)
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceBundleState {
    Single,
    Concordant,
    Conflict,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceBundle {
    pub schema: &'static str,
    pub bundle_contract: &'static str,
    pub target: EvidenceTarget,
    pub state: EvidenceBundleState,
    pub candidates: Vec<EvidenceCandidate>,
    pub resolved_value: Option<Value>,
    pub reconciliation_performed: bool,
    pub claim_strengthening_detected: bool,
    pub validation_level_promoted_by_adapter: bool,
    pub biological_validity_claimed: bool,
    pub clinical_validity_claimed: bool,
    pub inv_bio_001: &'static str,
    pub bundle_sha256: String,
}

fn bundle_identity(bundle: &EvidenceBundle) -> String {
    let mut hasher = Sha256::new();
    hasher.update(BUNDLE_DOMAIN);
    hasher.update(bundle.target.id.as_bytes());
    hasher.update([match bundle.target.kind {
        EvidenceTargetKind::Parameter => 0,
        EvidenceTargetKind::Constraint => 1,
    }]);
    hasher.update([match bundle.state {
        EvidenceBundleState::Single => 0,
        EvidenceBundleState::Concordant => 1,
        EvidenceBundleState::Conflict => 2,
        EvidenceBundleState::Unknown => 3,
    }]);
    for candidate in &bundle.candidates {
        hasher.update(candidate.candidate_sha256.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

pub fn bundle_candidates(
    candidates: Vec<EvidenceCandidate>,
) -> Result<EvidenceBundle, RuntimeError> {
    if candidates.is_empty() || candidates.len() > MAX_EVIDENCE_CANDIDATES {
        return Err(err("evidence bundle candidate count outside bounded domain"));
    }
    let target = candidates[0].target.clone();
    if candidates.iter().any(|candidate| candidate.target != target) {
        return Err(err("evidence bundle candidates must target the same parameter/constraint"));
    }
    for candidate in &candidates {
        if candidate.claim_strengthening_detected
            || candidate.validation_level_promoted_by_adapter
            || candidate.biological_validity_claimed
            || candidate.clinical_validity_claimed
            || candidate.candidate_sha256 != candidate_identity(candidate)?
        {
            return Err(err("evidence candidate failed Phase 4 integrity/non-promotion gate"));
        }
    }

    let state = if candidates.iter().all(|candidate| candidate.value.is_none()) {
        EvidenceBundleState::Unknown
    } else if candidates.len() == 1 {
        EvidenceBundleState::Single
    } else {
        let first_value = canonical_json(&candidates[0].value.clone().unwrap_or(Value::Null))?;
        let first_unit = &candidates[0].unit;
        if candidates.iter().all(|candidate| {
            canonical_json(&candidate.value.clone().unwrap_or(Value::Null))
                .map(|value| value == first_value && &candidate.unit == first_unit)
                .unwrap_or(false)
        }) {
            EvidenceBundleState::Concordant
        } else {
            EvidenceBundleState::Conflict
        }
    };

    // Only a single candidate is carried through as a resolved value. Multiple
    // sources are always preserved as separate candidates, even when concordant;
    // the adapter layer does not average, vote, or silently reconcile them.
    let resolved_value = if state == EvidenceBundleState::Single {
        candidates[0].value.clone()
    } else {
        None
    };

    let mut bundle = EvidenceBundle {
        schema: EVIDENCE_BUNDLE_SCHEMA,
        bundle_contract: EVIDENCE_BUNDLE_CONTRACT,
        target,
        state,
        candidates,
        resolved_value,
        reconciliation_performed: false,
        claim_strengthening_detected: false,
        validation_level_promoted_by_adapter: false,
        biological_validity_claimed: false,
        clinical_validity_claimed: false,
        inv_bio_001: INV_BIO_001,
        bundle_sha256: String::new(),
    };
    bundle.bundle_sha256 = bundle_identity(&bundle);
    Ok(bundle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn access() -> SourceAccess {
        SourceAccess {
            status: "open-access".into(),
            redistribution: "test metadata only".into(),
        }
    }

    fn reference_snapshot() -> SnapshotRef {
        SnapshotRef {
            mode: SnapshotMode::ReferenceOnly,
            external_payload_sha256: None,
            external_payload_committed: false,
            redistribution_permission_verified: false,
            notes: Some("test reference only".into()),
        }
    }

    fn policy() -> SnapshotPolicy {
        SnapshotPolicy {
            schema: SNAPSHOT_POLICY_SCHEMA.into(),
            default_mode: SnapshotMode::ReferenceOnly,
            records: Vec::new(),
        }
    }

    fn registry(source: SourceRecord) -> SourceRegistry {
        SourceRegistry {
            schema: SOURCE_REGISTRY_SCHEMA.into(),
            project: "QSOLKCB/igm".into(),
            sources: vec![source],
        }
    }

    fn input(adapter_id: &str, derivation: &str, kind: EvidenceTargetKind) -> EvidenceInput {
        EvidenceInput {
            schema: EVIDENCE_INPUT_SCHEMA.into(),
            adapter_id: adapter_id.into(),
            observation_id: "test.obs".into(),
            source_id: "source.test".into(),
            support_statement: "bounded test support".into(),
            target: EvidenceTarget {
                kind,
                id: "test.target".into(),
            },
            value: Some(json!(1.25)),
            unit: Some("test-unit".into()),
            derivation: derivation.into(),
            uncertainty: EvidenceUncertainty {
                kind: "source-reported".into(),
                lower: None,
                upper: None,
                value: None,
                level: None,
                unit: Some("test-unit".into()),
                notes: Some("synthetic adapter unit test uncertainty".into()),
            },
            snapshot: reference_snapshot(),
            notes: None,
        }
    }

    #[test]
    fn repository_cryo_fixture_is_claim_preserving() {
        let registry = SourceRegistry::load(Path::new("research/sources.json")).unwrap();
        let policy = SnapshotPolicy::load(Path::new("research/source-snapshot-policy.json")).unwrap();
        let input = EvidenceInput::load(Path::new("research/evidence/cryo-em-pentamer-count.json")).unwrap();
        let candidate = adapt_evidence(&registry, &policy, &input).unwrap();
        assert_eq!(candidate.adapter_id, CRYO_EM_ADAPTER_ID);
        assert_eq!(candidate.status, "observed");
        assert_eq!(candidate.derivation, "direct");
        assert_eq!(candidate.value, Some(json!(5)));
        assert!(!candidate.claim_strengthening_detected);
        assert!(!candidate.validation_level_promoted_by_adapter);
        assert!(!candidate.biological_validity_claimed);
        assert!(!candidate.clinical_validity_claimed);
        assert!(valid_sha256(&candidate.candidate_sha256));
    }

    #[test]
    fn md_adapter_maps_transformation_to_source_derived_not_observed() {
        let source = SourceRecord {
            id: "source.test".into(),
            source_class: "molecular-dynamics".into(),
            title: "synthetic MD adapter test".into(),
            authority: "simulation-source".into(),
            url: "https://example.invalid/md".into(),
            doi: None,
            pubmed: None,
            pdb: None,
            emdb: None,
            access: Some(access()),
            supports: vec!["bounded test support".into()],
            does_not_support: vec!["biological validation".into()],
        };
        let candidate = adapt_evidence(
            &registry(source),
            &policy(),
            &input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter),
        )
        .unwrap();
        assert_eq!(candidate.status, "source-derived");
        assert_ne!(candidate.status, "observed");
    }

    #[test]
    fn biochemical_adapter_preserves_calibrated_constraint_status() {
        let source = SourceRecord {
            id: "source.test".into(),
            source_class: "biochemical-measurement".into(),
            title: "synthetic biochemical adapter test".into(),
            authority: "measurement-source".into(),
            url: "https://example.invalid/biochem".into(),
            doi: None,
            pubmed: None,
            pdb: None,
            emdb: None,
            access: Some(access()),
            supports: vec!["bounded test support".into()],
            does_not_support: vec!["clinical validity".into()],
        };
        let candidate = adapt_evidence(
            &registry(source),
            &policy(),
            &input(
                BIOCHEMICAL_ADAPTER_ID,
                "calibrated",
                EvidenceTargetKind::Constraint,
            ),
        )
        .unwrap();
        assert_eq!(candidate.status, "calibrated");
        assert_eq!(candidate.target.kind, EvidenceTargetKind::Constraint);
    }

    #[test]
    fn conflict_bundle_preserves_candidates_without_reconciliation() {
        let source = SourceRecord {
            id: "source.test".into(),
            source_class: "molecular-dynamics".into(),
            title: "synthetic MD adapter test".into(),
            authority: "simulation-source".into(),
            url: "https://example.invalid/md".into(),
            doi: None,
            pubmed: None,
            pdb: None,
            emdb: None,
            access: Some(access()),
            supports: vec!["bounded test support".into()],
            does_not_support: vec!["biological validation".into()],
        };
        let registry = registry(source);
        let policy = policy();
        let left = adapt_evidence(
            &registry,
            &policy,
            &input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter),
        )
        .unwrap();
        let mut right_input = input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter);
        right_input.observation_id = "test.obs.2".into();
        right_input.value = Some(json!(1.5));
        let right = adapt_evidence(&registry, &policy, &right_input).unwrap();
        let bundle = bundle_candidates(vec![left, right]).unwrap();
        assert_eq!(bundle.state, EvidenceBundleState::Conflict);
        assert_eq!(bundle.candidates.len(), 2);
        assert!(bundle.resolved_value.is_none());
        assert!(!bundle.reconciliation_performed);
        assert!(!bundle.claim_strengthening_detected);
    }

    #[test]
    fn packaged_payload_fails_without_explicit_snapshot_permission() {
        let source = SourceRecord {
            id: "source.test".into(),
            source_class: "molecular-dynamics".into(),
            title: "synthetic MD adapter test".into(),
            authority: "simulation-source".into(),
            url: "https://example.invalid/md".into(),
            doi: None,
            pubmed: None,
            pdb: None,
            emdb: None,
            access: Some(access()),
            supports: vec!["bounded test support".into()],
            does_not_support: vec!["biological validation".into()],
        };
        let mut input = input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter);
        input.snapshot = SnapshotRef {
            mode: SnapshotMode::Packaged,
            external_payload_sha256: Some("0".repeat(64)),
            external_payload_committed: true,
            redistribution_permission_verified: true,
            notes: None,
        };
        assert!(adapt_evidence(&registry(source), &policy(), &input).is_err());
    }
}
