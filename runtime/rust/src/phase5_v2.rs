// SPDX-License-Identifier: Apache-2.0
//! Hardened public Phase 5 representation boundary.
//!
//! The Phase 5 implementation remains private below this wrapper. The public
//! loader additionally runs the repository JSON-Schema gates for both the model
//! profile and the Phase 5 configuration before projection, retains the exact
//! Phase 3B execution-profile identity used for ensemble statistics, and
//! recomputes the final bundle identity after that provenance is attached.

#[path = "phase5.rs"]
mod inner;

use crate::RuntimeError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUNDLE_DOMAIN: &[u8] = b"IGM-PHASE5-REPRESENTATION-V1\0";
const MAX_EXECUTION_PROFILE_BYTES: u64 = 64 * 1024;

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

fn resolve_repository_input(repository_root: &Path, input: &Path) -> Result<PathBuf, RuntimeError> {
    let root = fs::canonicalize(repository_root).map_err(|e| {
        err(format!(
            "cannot resolve Phase 5 repository root {}: {e}",
            repository_root.display()
        ))
    })?;
    let candidate = if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    };
    fs::canonicalize(&candidate).map_err(|e| {
        err(format!(
            "cannot resolve Phase 5 input {}: {e}",
            candidate.display()
        ))
    })
}

fn run_schema_gate(
    repository_root: &Path,
    schema_relative: &str,
    instance: &Path,
) -> Result<(), RuntimeError> {
    let root = fs::canonicalize(repository_root).map_err(|e| {
        err(format!(
            "cannot resolve Phase 5 repository root {}: {e}",
            repository_root.display()
        ))
    })?;
    let validator = root.join("tools/validate_json_schema.py");
    let schema = root.join(schema_relative);
    if !validator.is_file() || !schema.is_file() {
        return Err(err(format!(
            "Phase 5 structural gate requires {} and {}",
            validator.display(),
            schema.display()
        )));
    }
    let status = Command::new("python3")
        .arg(&validator)
        .arg("--schema")
        .arg(&schema)
        .arg(instance)
        .current_dir(&root)
        .status()
        .map_err(|e| err(format!("cannot execute Phase 5 JSON-Schema gate: {e}")))?;
    if !status.success() {
        return Err(err(format!(
            "Phase 5 JSON-Schema gate rejected {} against {}",
            instance.display(),
            schema.display()
        )));
    }
    Ok(())
}

fn execution_profile_identity(
    repository_root: &Path,
    config_path: &Path,
) -> Result<(String, String), RuntimeError> {
    let config_bytes = fs::read(config_path).map_err(|e| {
        err(format!(
            "cannot read schema-admitted Phase 5 config {}: {e}",
            config_path.display()
        ))
    })?;
    let config: Value = serde_json::from_slice(&config_bytes)
        .map_err(|e| err(format!("Phase 5 config is not strict JSON after schema admission: {e}")))?;
    let relative = config
        .get("ensemble")
        .and_then(|value| value.get("execution_profile_path"))
        .and_then(Value::as_str)
        .ok_or_else(|| err("Phase 5 config lacks ensemble.execution_profile_path"))?;
    let root = fs::canonicalize(repository_root)
        .map_err(|e| err(format!("cannot resolve Phase 5 repository root: {e}")))?;
    let execution_path = root.join(relative);
    let metadata = fs::metadata(&execution_path).map_err(|e| {
        err(format!(
            "cannot stat Phase 3B execution profile {}: {e}",
            execution_path.display()
        ))
    })?;
    if !metadata.is_file() || metadata.len() > MAX_EXECUTION_PROFILE_BYTES {
        return Err(err("Phase 3B execution profile is not a bounded regular file"));
    }
    let bytes = fs::read(&execution_path).map_err(|e| {
        err(format!(
            "cannot read Phase 3B execution profile {}: {e}",
            execution_path.display()
        ))
    })?;
    let raw: Value = serde_json::from_slice(&bytes)
        .map_err(|e| err(format!("Phase 3B execution profile is not strict JSON: {e}")))?;
    let profile_id = raw
        .get("profile_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| err("Phase 3B execution profile lacks profile_id"))?
        .to_string();
    let digest = sha256_hex(canonical_json(&raw)?.as_bytes());
    Ok((profile_id, digest))
}

fn bundle_identity(value: &Value) -> Result<String, RuntimeError> {
    let mut blanked = value.clone();
    let root = blanked
        .as_object_mut()
        .ok_or_else(|| err("Phase 5 bundle must serialize as an object"))?;
    root.insert("bundle_sha256".into(), Value::String(String::new()));
    let canonical = canonical_json(&blanked)?;
    let mut hasher = Sha256::new();
    hasher.update(BUNDLE_DOMAIN);
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Debug)]
pub struct Phase5Engine {
    inner: inner::Phase5Engine,
    source_execution_profile_id: String,
    source_execution_profile_sha256: String,
}

impl Phase5Engine {
    pub fn load(
        model_path: &Path,
        config_path: &Path,
        repository_root: &Path,
    ) -> Result<Self, RuntimeError> {
        let model_path = resolve_repository_input(repository_root, model_path)?;
        let config_path = resolve_repository_input(repository_root, config_path)?;

        // These are the same portable structural gates required by repository CI.
        // The public CLI/library boundary therefore cannot project an input that
        // the declared schemas reject merely because Serde/native checks are
        // narrower than the schema.
        run_schema_gate(
            repository_root,
            "schemas/model-profile.schema.json",
            &model_path,
        )?;
        run_schema_gate(
            repository_root,
            "schemas/phase5-representation-config.schema.json",
            &config_path,
        )?;

        let (source_execution_profile_id, source_execution_profile_sha256) =
            execution_profile_identity(repository_root, &config_path)?;
        let inner = inner::Phase5Engine::load(&model_path, &config_path, repository_root)?;
        Ok(Self {
            inner,
            source_execution_profile_id,
            source_execution_profile_sha256,
        })
    }

    pub fn bundle(&self) -> Result<Value, RuntimeError> {
        let inner_bundle = self.inner.bundle()?;
        let mut value = serde_json::to_value(inner_bundle).map_err(|e| err(e.to_string()))?;
        let ensemble = value
            .get_mut("ensemble_statistics")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| err("Phase 5 bundle lacks ensemble_statistics object"))?;
        ensemble.insert(
            "source_execution_profile_id".into(),
            Value::String(self.source_execution_profile_id.clone()),
        );
        ensemble.insert(
            "source_execution_profile_sha256".into(),
            Value::String(self.source_execution_profile_sha256.clone()),
        );
        let identity = bundle_identity(&value)?;
        value
            .as_object_mut()
            .expect("bundle object checked above")
            .insert("bundle_sha256".into(), Value::String(identity));
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_boundary_retains_execution_profile_identity() {
        let engine = Phase5Engine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-phase5-v0.json"),
            Path::new("."),
        )
        .expect("schema-admitted Phase 5 engine");
        let bundle = engine.bundle().expect("hardened bundle");
        let ensemble = bundle["ensemble_statistics"].as_object().unwrap();
        assert_eq!(
            ensemble["source_execution_profile_id"],
            Value::String("IGM-PENTA-CRT-SYMMETRIC-V0".into())
        );
        assert_eq!(
            ensemble["source_execution_profile_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
    }

    #[test]
    fn public_bundle_identity_is_recomputed_after_provenance_attachment() {
        let engine = Phase5Engine::load(
            Path::new("profiles/igm-schematic-pentamer-v0.json"),
            Path::new("runtime/profiles/igm-phase5-v0.json"),
            Path::new("."),
        )
        .expect("schema-admitted Phase 5 engine");
        let bundle = engine.bundle().expect("hardened bundle");
        assert_eq!(
            bundle["bundle_sha256"].as_str().unwrap(),
            bundle_identity(&bundle).unwrap()
        );
    }
}
