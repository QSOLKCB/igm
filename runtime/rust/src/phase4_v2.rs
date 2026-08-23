// SPDX-License-Identifier: Apache-2.0
//! Hardened public Phase 4 evidence-adapter boundary.
//!
//! The original Phase 4 implementation remains private below this wrapper. This
//! boundary adds strict structural parsing, claim-to-target/value bindings,
//! packaged-payload byte verification, and duplicate-evidence rejection before
//! any candidate can enter an evidence bundle.

#[path = "phase4.rs"]
mod inner;

pub use inner::{
    BiochemicalCalibrationConstraintAdapter, CryoEmParameterAdapter, EvidenceBundle,
    EvidenceBundleState, EvidenceCandidate, MolecularDynamicsTrajectoryAdapter, SourceAdapter,
    BIOCHEMICAL_ADAPTER_ID, CRYO_EM_ADAPTER_ID, EVIDENCE_BUNDLE_CONTRACT,
    EVIDENCE_BUNDLE_SCHEMA, EVIDENCE_CANDIDATE_SCHEMA, EVIDENCE_INPUT_SCHEMA,
    MAX_EVIDENCE_CANDIDATES, MAX_EVIDENCE_INPUT_BYTES, MD_ADAPTER_ID,
    SNAPSHOT_POLICY_SCHEMA, SOURCE_ADAPTER_CONTRACT, SOURCE_REGISTRY_SCHEMA,
};

use crate::RuntimeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn canonical_json(value: &Value) -> Result<String, RuntimeError> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).map_err(|e| err(e.to_string()))
        }
        Value::Array(values) => {
            let mut out = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(',');
                }
                out.push_str(&canonical_json(value)?);
            }
            out.push(']');
            Ok(out)
        }
        Value::Object(map) => {
            let mut keys = map.keys().collect::<Vec<_>>();
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn load_bounded_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RuntimeError> {
    let metadata = fs::metadata(path)
        .map_err(|e| err(format!("cannot stat Phase 4 JSON {}: {e}", path.display())))?;
    if !metadata.is_file() || metadata.len() > MAX_EVIDENCE_INPUT_BYTES {
        return Err(err(format!(
            "Phase 4 JSON {} is not a bounded regular file",
            path.display()
        )));
    }
    let bytes = fs::read(path)
        .map_err(|e| err(format!("cannot read Phase 4 JSON {}: {e}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("{} violates the strict Phase 4 structural contract: {e}", path.display())))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceAccess {
    pub status: String,
    pub redistribution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceTargetKind {
    Parameter,
    Constraint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceTarget {
    pub kind: EvidenceTargetKind,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceMapping {
    pub id: String,
    pub support_statement: String,
    pub target: EvidenceTarget,
    pub derivation: String,
    pub value: Value,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub id: String,
    #[serde(rename = "class")]
    pub source_class: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub year: Option<u64>,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub pubmed: Option<String>,
    #[serde(default)]
    pub pdb: Option<String>,
    #[serde(default)]
    pub emdb: Option<String>,
    pub url: String,
    #[serde(default)]
    pub effective_date: Option<String>,
    pub authority: String,
    #[serde(default)]
    pub access: Option<SourceAccess>,
    #[serde(default)]
    pub supports: Vec<String>,
    #[serde(default)]
    pub does_not_support: Vec<String>,
    #[serde(default)]
    pub evidence_mappings: Vec<EvidenceMapping>,
    #[serde(default)]
    pub notes: Option<String>,
}

impl SourceRecord {
    fn to_inner(&self) -> inner::SourceRecord {
        inner::SourceRecord {
            id: self.id.clone(),
            source_class: self.source_class.clone(),
            title: self.title.clone(),
            authority: self.authority.clone(),
            url: self.url.clone(),
            doi: self.doi.clone(),
            pubmed: self.pubmed.clone(),
            pdb: self.pdb.clone(),
            emdb: self.emdb.clone(),
            access: self.access.as_ref().map(|access| inner::SourceAccess {
                status: access.status.clone(),
                redistribution: access.redistribution.clone(),
            }),
            supports: self.supports.clone(),
            does_not_support: self.does_not_support.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceRegistryDocument {
    schema: String,
    project: String,
    sources: Vec<SourceRecord>,
}

#[derive(Debug)]
pub struct SourceRegistry {
    pub schema: String,
    pub project: String,
    pub sources: Vec<SourceRecord>,
    inner: inner::SourceRegistry,
}

impl SourceRegistry {
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let document: SourceRegistryDocument = load_bounded_json(path)?;
        let mut registry = Self::from_records(document.schema, document.project, document.sources)?;
        registry.validate()?;
        Ok(registry)
    }

    pub fn from_records(
        schema: String,
        project: String,
        sources: Vec<SourceRecord>,
    ) -> Result<Self, RuntimeError> {
        let inner_registry = inner::SourceRegistry {
            schema: schema.clone(),
            project: project.clone(),
            sources: sources.iter().map(SourceRecord::to_inner).collect(),
        };
        Ok(Self {
            schema,
            project,
            sources,
            inner: inner_registry,
        })
    }

    pub fn validate(&mut self) -> Result<(), RuntimeError> {
        self.inner.validate()?;
        let mut mapping_ids = BTreeSet::new();
        for source in &self.sources {
            for mapping in &source.evidence_mappings {
                if mapping.id.is_empty() || !mapping_ids.insert((source.id.clone(), mapping.id.clone())) {
                    return Err(err(format!("duplicate/empty evidence mapping id for {}", source.id)));
                }
                if mapping.target.id.is_empty()
                    || mapping.support_statement.is_empty()
                    || !source.supports.iter().any(|statement| statement == &mapping.support_statement)
                {
                    return Err(err(format!(
                        "evidence mapping {} is not bound to a registered support statement/target",
                        mapping.id
                    )));
                }
                match mapping.derivation.as_str() {
                    "direct" | "transformed" | "calibrated" | "inferred" | "assumed" | "unknown" => {}
                    other => return Err(err(format!("unsupported mapping derivation: {other}"))),
                }
            }
        }
        Ok(())
    }

    pub fn structural_source_count(&self) -> usize {
        self.sources
            .iter()
            .filter(|source| source.authority == "structural-source")
            .count()
    }

    fn source(&self, id: &str) -> Result<&SourceRecord, RuntimeError> {
        self.sources
            .iter()
            .find(|source| source.id == id)
            .ok_or_else(|| err(format!("evidence source is not registered: {id}")))
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
#[serde(deny_unknown_fields)]
pub struct SnapshotRef {
    pub mode: SnapshotMode,
    #[serde(default)]
    pub external_payload_sha256: Option<String>,
    #[serde(default)]
    pub external_payload_path: Option<String>,
    pub external_payload_committed: bool,
    pub redistribution_permission_verified: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

impl SnapshotRef {
    fn to_inner(&self) -> inner::SnapshotRef {
        inner::SnapshotRef {
            mode: match self.mode {
                SnapshotMode::ReferenceOnly => inner::SnapshotMode::ReferenceOnly,
                SnapshotMode::HashOnly => inner::SnapshotMode::HashOnly,
                SnapshotMode::Packaged => inner::SnapshotMode::Packaged,
            },
            external_payload_sha256: self.external_payload_sha256.clone(),
            external_payload_committed: self.external_payload_committed,
            redistribution_permission_verified: self.redistribution_permission_verified,
            notes: self.notes.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotPolicyRecord {
    source_id: String,
    mode: SnapshotMode,
    redistribution_permission_verified: bool,
    external_payload_committed: bool,
    #[serde(default)]
    external_payload_sha256: Option<String>,
    #[serde(default)]
    external_payload_path: Option<String>,
    reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SnapshotPolicyDocument {
    schema: String,
    default_mode: SnapshotMode,
    records: Vec<SnapshotPolicyRecord>,
}

#[derive(Debug)]
pub struct SnapshotPolicy {
    inner: inner::SnapshotPolicy,
    records: Vec<SnapshotPolicyRecord>,
    repository_root: PathBuf,
}

fn find_repository_root(path: &Path) -> Result<PathBuf, RuntimeError> {
    let canonical = path
        .canonicalize()
        .map_err(|e| err(format!("cannot canonicalize {}: {e}", path.display())))?;
    let start = canonical
        .parent()
        .ok_or_else(|| err("snapshot policy path has no parent"))?;
    for ancestor in start.ancestors() {
        if ancestor.join("Cargo.toml").is_file() && ancestor.join("research/sources.json").is_file() {
            return ancestor
                .canonicalize()
                .map_err(|e| err(format!("cannot canonicalize repository root: {e}")));
        }
    }
    Err(err("could not locate repository root for source snapshot policy"))
}

fn validate_relative_payload_path(path: &str) -> Result<(), RuntimeError> {
    let candidate = Path::new(path);
    if path.is_empty()
        || candidate.is_absolute()
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(err("packaged payload path must be a plain repository-relative path"));
    }
    Ok(())
}

fn validate_snapshot_shape(snapshot: &SnapshotRef, source_id: &str) -> Result<(), RuntimeError> {
    match snapshot.mode {
        SnapshotMode::ReferenceOnly => {
            if snapshot.external_payload_committed
                || snapshot.redistribution_permission_verified
                || snapshot.external_payload_sha256.is_some()
                || snapshot.external_payload_path.is_some()
            {
                return Err(err(format!(
                    "reference-only snapshot for {source_id} cannot carry payload/permission/hash/path"
                )));
            }
        }
        SnapshotMode::HashOnly => {
            let digest = snapshot
                .external_payload_sha256
                .as_deref()
                .ok_or_else(|| err(format!("hash-only snapshot for {source_id} requires SHA-256")))?;
            if snapshot.external_payload_committed
                || snapshot.external_payload_path.is_some()
                || !valid_sha256(digest)
            {
                return Err(err(format!(
                    "hash-only snapshot for {source_id} requires lowercase SHA-256 and no committed payload/path"
                )));
            }
        }
        SnapshotMode::Packaged => {
            let digest = snapshot
                .external_payload_sha256
                .as_deref()
                .ok_or_else(|| err(format!("packaged snapshot for {source_id} requires SHA-256")))?;
            let path = snapshot
                .external_payload_path
                .as_deref()
                .ok_or_else(|| err(format!("packaged snapshot for {source_id} requires repository-relative payload path")))?;
            validate_relative_payload_path(path)?;
            if !snapshot.external_payload_committed
                || !snapshot.redistribution_permission_verified
                || !valid_sha256(digest)
            {
                return Err(err(format!(
                    "packaged snapshot for {source_id} requires verified permission, committed payload path, and lowercase SHA-256"
                )));
            }
        }
    }
    Ok(())
}

impl SnapshotPolicy {
    pub fn load(path: &Path) -> Result<Self, RuntimeError> {
        let document: SnapshotPolicyDocument = load_bounded_json(path)?;
        if document.schema != SNAPSHOT_POLICY_SCHEMA || document.default_mode != SnapshotMode::ReferenceOnly {
            return Err(err("unsupported source snapshot policy"));
        }
        let mut seen = BTreeSet::new();
        for record in &document.records {
            if record.source_id.is_empty() || !seen.insert(record.source_id.as_str()) || record.reason.is_empty() {
                return Err(err("duplicate/empty source snapshot policy record"));
            }
            validate_snapshot_shape(
                &SnapshotRef {
                    mode: record.mode.clone(),
                    external_payload_sha256: record.external_payload_sha256.clone(),
                    external_payload_path: record.external_payload_path.clone(),
                    external_payload_committed: record.external_payload_committed,
                    redistribution_permission_verified: record.redistribution_permission_verified,
                    notes: None,
                },
                &record.source_id,
            )?;
        }
        let inner_policy = inner::SnapshotPolicy::load(path)?;
        Ok(Self {
            inner: inner_policy,
            records: document.records,
            repository_root: find_repository_root(path)?,
        })
    }

    fn verify_packaged_payload(&self, snapshot: &SnapshotRef) -> Result<(), RuntimeError> {
        if snapshot.mode != SnapshotMode::Packaged {
            return Ok(());
        }
        let relative = snapshot
            .external_payload_path
            .as_deref()
            .ok_or_else(|| err("packaged snapshot missing payload path"))?;
        validate_relative_payload_path(relative)?;
        let root = self.repository_root.canonicalize().map_err(|e| err(e.to_string()))?;
        let payload = root.join(relative);
        let metadata = fs::metadata(&payload)
            .map_err(|e| err(format!("packaged payload {} is not present: {e}", payload.display())))?;
        if !metadata.is_file() {
            return Err(err("packaged payload path is not a regular file"));
        }
        let canonical_payload = payload
            .canonicalize()
            .map_err(|e| err(format!("cannot canonicalize packaged payload: {e}")))?;
        if !canonical_payload.starts_with(&root) {
            return Err(err("packaged payload escaped repository root"));
        }
        let bytes = fs::read(&canonical_payload)
            .map_err(|e| err(format!("cannot read packaged payload: {e}")))?;
        let actual = sha256_hex(&bytes);
        let expected = snapshot
            .external_payload_sha256
            .as_deref()
            .ok_or_else(|| err("packaged snapshot missing SHA-256"))?;
        if actual != expected {
            return Err(err(format!(
                "packaged payload SHA-256 mismatch: expected {expected}, got {actual}"
            )));
        }
        Ok(())
    }

    fn validate_for_source(&self, source_id: &str, snapshot: &SnapshotRef) -> Result<(), RuntimeError> {
        validate_snapshot_shape(snapshot, source_id)?;
        if let Some(record) = self.records.iter().find(|record| record.source_id == source_id) {
            if snapshot.mode != record.mode
                || snapshot.external_payload_committed != record.external_payload_committed
                || snapshot.redistribution_permission_verified != record.redistribution_permission_verified
                || snapshot.external_payload_sha256 != record.external_payload_sha256
                || snapshot.external_payload_path != record.external_payload_path
            {
                return Err(err(format!("evidence snapshot does not match policy for source {source_id}")));
            }
        } else if snapshot.mode != SnapshotMode::ReferenceOnly
            || snapshot.external_payload_committed
            || snapshot.redistribution_permission_verified
            || snapshot.external_payload_sha256.is_some()
            || snapshot.external_payload_path.is_some()
        {
            return Err(err(format!(
                "source {source_id} has no explicit snapshot permission; only reference-only ingestion is allowed"
            )));
        }
        self.verify_packaged_payload(snapshot)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
        for value in [self.lower, self.upper, self.value, self.level].into_iter().flatten() {
            if !value.is_finite() {
                return Err(err("evidence uncertainty must remain finite"));
            }
        }
        if let Some(level) = self.level {
            if !(0.0..=1.0).contains(&level) {
                return Err(err("evidence uncertainty level must be in [0,1]"));
            }
        }
        match self.kind.as_str() {
            "unknown" | "source-reported" => {
                if self.notes.as_deref().unwrap_or("").is_empty() {
                    return Err(err(format!("uncertainty kind {} requires explanatory notes", self.kind)));
                }
            }
            "interval" | "confidence-interval" => {
                let lower = self.lower.ok_or_else(|| err(format!("uncertainty kind {} requires lower", self.kind)))?;
                let upper = self.upper.ok_or_else(|| err(format!("uncertainty kind {} requires upper", self.kind)))?;
                if lower > upper {
                    return Err(err("uncertainty lower bound exceeds upper bound"));
                }
            }
            "standard-deviation" => {
                let value = self.value.ok_or_else(|| err("standard-deviation uncertainty requires value"))?;
                if value < 0.0 {
                    return Err(err("standard-deviation uncertainty must be non-negative"));
                }
            }
            other => return Err(err(format!("unsupported evidence uncertainty kind: {other}"))),
        }
        Ok(())
    }

    fn to_inner(&self) -> inner::EvidenceUncertainty {
        inner::EvidenceUncertainty {
            kind: self.kind.clone(),
            lower: self.lower,
            upper: self.upper,
            value: self.value,
            level: self.level,
            unit: self.unit.clone(),
            notes: self.notes.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
        validate_snapshot_shape(&self.snapshot, &self.source_id)
    }

    fn to_inner(&self) -> inner::EvidenceInput {
        inner::EvidenceInput {
            schema: self.schema.clone(),
            adapter_id: self.adapter_id.clone(),
            observation_id: self.observation_id.clone(),
            source_id: self.source_id.clone(),
            support_statement: self.support_statement.clone(),
            target: inner::EvidenceTarget {
                kind: match self.target.kind {
                    EvidenceTargetKind::Parameter => inner::EvidenceTargetKind::Parameter,
                    EvidenceTargetKind::Constraint => inner::EvidenceTargetKind::Constraint,
                },
                id: self.target.id.clone(),
            },
            value: self.value.clone(),
            unit: self.unit.clone(),
            derivation: self.derivation.clone(),
            uncertainty: self.uncertainty.to_inner(),
            snapshot: self.snapshot.to_inner(),
            notes: self.notes.clone(),
        }
    }
}

fn mapping_matches(mapping: &EvidenceMapping, input: &EvidenceInput) -> Result<bool, RuntimeError> {
    let input_value = input.value.clone().unwrap_or(Value::Null);
    Ok(mapping.support_statement == input.support_statement
        && mapping.target == input.target
        && mapping.derivation == input.derivation
        && mapping.unit == input.unit
        && canonical_json(&mapping.value)? == canonical_json(&input_value)?)
}

pub fn adapt_evidence(
    registry: &SourceRegistry,
    snapshot_policy: &SnapshotPolicy,
    input: &EvidenceInput,
) -> Result<EvidenceCandidate, RuntimeError> {
    input.validate_common()?;
    let source = registry.source(&input.source_id)?;
    if !source.evidence_mappings.iter().any(|mapping| mapping_matches(mapping, input).unwrap_or(false)) {
        return Err(err(format!(
            "registered source support does not authorize target/value/derivation for observation {}",
            input.observation_id
        )));
    }
    snapshot_policy.validate_for_source(&source.id, &input.snapshot)?;
    let inner_input = input.to_inner();
    inner::adapt_evidence(&registry.inner, &snapshot_policy.inner, &inner_input)
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

pub fn bundle_candidates(candidates: Vec<EvidenceCandidate>) -> Result<EvidenceBundle, RuntimeError> {
    let mut identities = BTreeSet::new();
    for candidate in &candidates {
        if !identities.insert(candidate.candidate_sha256.clone()) {
            return Err(err(format!(
                "duplicate evidence candidate identity cannot manufacture corroboration: {}",
                candidate.candidate_sha256
            )));
        }
    }
    inner::bundle_candidates(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn synthetic_source(class: &str, adapter_id: &str, derivation: &str, kind: EvidenceTargetKind) -> SourceRecord {
        let target = EvidenceTarget { kind, id: "test.target".into() };
        SourceRecord {
            id: "source.test".into(),
            source_class: class.into(),
            title: "synthetic Phase 4 wrapper test".into(),
            authors: vec![],
            publisher: None,
            year: None,
            doi: None,
            pubmed: None,
            pdb: None,
            emdb: None,
            url: "https://example.invalid/source".into(),
            effective_date: None,
            authority: if adapter_id == CRYO_EM_ADAPTER_ID { "structural-source".into() } else { "test-source".into() },
            access: Some(SourceAccess { status: "open-access".into(), redistribution: "test metadata only".into() }),
            supports: vec!["bounded test support".into()],
            does_not_support: vec!["biological validation".into()],
            evidence_mappings: vec![EvidenceMapping {
                id: "test.mapping".into(),
                support_statement: "bounded test support".into(),
                target,
                derivation: derivation.into(),
                value: json!(1.25),
                unit: Some("test-unit".into()),
                notes: None,
            }],
            notes: None,
        }
    }

    fn input(adapter_id: &str, derivation: &str, kind: EvidenceTargetKind) -> EvidenceInput {
        EvidenceInput {
            schema: EVIDENCE_INPUT_SCHEMA.into(),
            adapter_id: adapter_id.into(),
            observation_id: "test.obs".into(),
            source_id: "source.test".into(),
            support_statement: "bounded test support".into(),
            target: EvidenceTarget { kind, id: "test.target".into() },
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
                notes: Some("synthetic uncertainty".into()),
            },
            snapshot: SnapshotRef {
                mode: SnapshotMode::ReferenceOnly,
                external_payload_sha256: None,
                external_payload_path: None,
                external_payload_committed: false,
                redistribution_permission_verified: false,
                notes: None,
            },
            notes: None,
        }
    }

    #[test]
    fn repository_fixture_is_bound_to_registered_target_value_and_derivation() {
        let registry = SourceRegistry::load(Path::new("research/sources.json")).unwrap();
        let policy = SnapshotPolicy::load(Path::new("research/source-snapshot-policy.json")).unwrap();
        let mut evidence = EvidenceInput::load(Path::new("research/evidence/cryo-em-pentamer-count.json")).unwrap();
        assert!(adapt_evidence(&registry, &policy, &evidence).is_ok());
        evidence.target.id = "totally_unrelated_parameter".into();
        assert!(adapt_evidence(&registry, &policy, &evidence).is_err());
    }

    #[test]
    fn duplicate_candidate_identity_is_rejected() {
        let registry = SourceRegistry::from_records(
            SOURCE_REGISTRY_SCHEMA.into(),
            "QSOLKCB/igm".into(),
            vec![synthetic_source("molecular-dynamics", MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter)],
        )
        .unwrap();
        let policy = SnapshotPolicy::load(Path::new("research/source-snapshot-policy.json")).unwrap();
        let candidate = adapt_evidence(
            &registry,
            &policy,
            &input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter),
        )
        .unwrap();
        let duplicate = serde_json::to_value(&candidate).unwrap();
        assert!(duplicate.get("candidate_sha256").is_some());
        assert!(bundle_candidates(vec![candidate.clone(), candidate]).is_err());
    }

    #[test]
    fn malformed_uncertainty_is_rejected_at_public_boundary() {
        let mut evidence = input(MD_ADAPTER_ID, "transformed", EvidenceTargetKind::Parameter);
        evidence.uncertainty = EvidenceUncertainty {
            kind: "standard-deviation".into(),
            lower: None,
            upper: None,
            value: Some(-1.0),
            level: None,
            unit: None,
            notes: None,
        };
        assert!(evidence.validate_common().is_err());
    }
}
