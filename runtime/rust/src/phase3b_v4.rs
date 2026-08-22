// SPDX-License-Identifier: Apache-2.0
//! Hardened public Phase 3B boundary.
//!
//! This wrapper keeps the Phase 3B implementation private and performs the
//! model-profile admission checks that are required specifically by the fixed
//! five-sector PENTA-CRT engine before exposing an executable engine handle.

use crate::{load_profile, Profile, RuntimeError};
use serde_json::Value;
use std::path::Path;

#[path = "phase3b_v3.rs"]
mod inner;

pub use inner::{
    ConformationAddress, ConformationSlice, PentaCrtRunConfig, PentaCrtRunSummary,
    PentaCrtWorkerSummary, VerificationReport, BLOCK_REUSE_RESIDUAL_TOLERANCE,
    BRUTE_DISTANCE_EVALUATIONS, DOF_COUNT, EXACT_Z_RESIDUAL_CORRECTIONS,
    MAX_VERIFY_SAMPLES, MODEL_ID, NODE_COUNT, OPTIMIZATION_CONTRACT,
    OPTIMIZATION_NUMERICAL_PROFILE, OPTIMIZATION_PROFILE_CONTRACT,
    OPTIMIZATION_PROFILE_ID, PAIR_COUNT, PLANAR_BLOCK_EVALUATIONS, RUN_SCHEMA,
    SPARSE_J_EVALUATIONS, STRUCTURED_DISTANCE_EVALUATIONS, VERIFY_SCHEMA,
};

fn err(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

fn bounded_numeric_parameter(profile: &Profile, name: &str) -> Result<f64, RuntimeError> {
    let parameter = profile
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .ok_or_else(|| err(format!("missing Phase 3B model parameter: {name}")))?;
    let value = parameter
        .value
        .as_ref()
        .and_then(Value::as_f64)
        .ok_or_else(|| err(format!("Phase 3B model parameter {name} must be numeric")))?;
    if !value.is_finite() {
        return Err(err(format!("Phase 3B model parameter {name} must be finite")));
    }
    if let Some(lower) = parameter.lower_bound {
        if !lower.is_finite() || value < lower {
            return Err(err(format!(
                "Phase 3B model parameter {name} is below its declared lower bound"
            )));
        }
    }
    if let Some(upper) = parameter.upper_bound {
        if !upper.is_finite() || value > upper {
            return Err(err(format!(
                "Phase 3B model parameter {name} exceeds its declared upper bound"
            )));
        }
    }
    Ok(value)
}

fn validate_all_declared_numeric_bounds(profile: &Profile) -> Result<(), RuntimeError> {
    for parameter in &profile.parameters {
        let has_lower = parameter.lower_bound.is_some();
        let has_upper = parameter.upper_bound.is_some();
        if !has_lower && !has_upper {
            continue;
        }
        let value = parameter
            .value
            .as_ref()
            .and_then(Value::as_f64)
            .ok_or_else(|| {
                err(format!(
                    "bounded model parameter {} must carry a numeric value",
                    parameter.name
                ))
            })?;
        if !value.is_finite() {
            return Err(err(format!(
                "bounded model parameter {} must be finite",
                parameter.name
            )));
        }
        if let Some(lower) = parameter.lower_bound {
            if !lower.is_finite() || value < lower {
                return Err(err(format!(
                    "model parameter {} is below lower_bound",
                    parameter.name
                )));
            }
        }
        if let Some(upper) = parameter.upper_bound {
            if !upper.is_finite() || value > upper {
                return Err(err(format!(
                    "model parameter {} exceeds upper_bound",
                    parameter.name
                )));
            }
        }
        if let (Some(lower), Some(upper)) = (parameter.lower_bound, parameter.upper_bound) {
            if lower > upper {
                return Err(err(format!(
                    "model parameter {} has lower_bound greater than upper_bound",
                    parameter.name
                )));
            }
        }
    }
    Ok(())
}

fn validate_phase3b_model(profile: &Profile) -> Result<(), RuntimeError> {
    validate_all_declared_numeric_bounds(profile)?;

    let sector_count = profile
        .parameters
        .iter()
        .find(|parameter| parameter.name == "assembly_sector_count")
        .and_then(|parameter| parameter.value.as_ref())
        .and_then(Value::as_u64)
        .ok_or_else(|| err("Phase 3B requires integer assembly_sector_count"))?;
    if sector_count != 5 {
        return Err(err(
            "Phase 3B fixed PENTA-CRT engine requires assembly_sector_count=5",
        ));
    }

    // Explicitly re-read every value consumed by the optimized geometry through
    // the bounded path so the public engine cannot diverge from the mandatory
    // semantic profile gate.
    for name in ["core_radius", "fab_length", "fab_spread_deg", "jchain_offset"] {
        bounded_numeric_parameter(profile, name)?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct PentaCrtEngine {
    inner: inner::PentaCrtEngine,
}

impl PentaCrtEngine {
    pub fn load(model_path: &Path, execution_path: &Path) -> Result<Self, RuntimeError> {
        let checked_model = load_profile(model_path)?;
        validate_phase3b_model(checked_model.profile())?;
        let inner = inner::PentaCrtEngine::load(model_path, execution_path)?;
        Ok(Self { inner })
    }

    pub fn total_conformations(&self) -> u64 {
        self.inner.total_conformations()
    }

    pub fn radices(&self) -> [u16; DOF_COUNT] {
        self.inner.radices()
    }

    pub fn decode(&self, index: u64) -> Result<ConformationAddress, RuntimeError> {
        self.inner.decode(index)
    }

    pub fn encode(&self, address: ConformationAddress) -> Result<u64, RuntimeError> {
        self.inner.encode(address)
    }

    pub fn model_profile_sha256(&self) -> &str {
        self.inner.model_profile_sha256()
    }

    pub fn optimization_profile_sha256(&self) -> &str {
        self.inner.optimization_profile_sha256()
    }

    pub fn optimization_profile_id(&self) -> &str {
        self.inner.optimization_profile_id()
    }

    pub fn base_spread_deg(&self) -> f64 {
        self.inner.base_spread_deg()
    }

    pub fn dof_ids(&self) -> Vec<&str> {
        self.inner.dof_ids()
    }
}

pub fn verify_penta_crt(
    engine: &PentaCrtEngine,
    samples: usize,
) -> Result<VerificationReport, RuntimeError> {
    inner::verify_penta_crt(&engine.inner, samples)
}

pub fn run_penta_crt(
    engine: &PentaCrtEngine,
    config: PentaCrtRunConfig,
) -> Result<PentaCrtRunSummary, RuntimeError> {
    inner::run_penta_crt(&engine.inner, config)
}

pub fn describe_address(engine: &PentaCrtEngine, index: u64) -> Result<String, RuntimeError> {
    inner::describe_address(&engine.inner, index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> Profile {
        load_profile(Path::new("profiles/igm-schematic-pentamer-v0.json"))
            .expect("repository V0 profile")
            .profile()
            .clone()
    }

    #[test]
    fn phase3b_rejects_out_of_bounds_model_parameter() {
        let mut profile = model();
        let radius = profile
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name == "core_radius")
            .unwrap();
        radius.value = Some(Value::from(5.0));
        radius.upper_bound = Some(4.0);
        assert!(validate_phase3b_model(&profile).is_err());
    }

    #[test]
    fn phase3b_rejects_nonfive_sector_count() {
        let mut profile = model();
        let sectors = profile
            .parameters
            .iter_mut()
            .find(|parameter| parameter.name == "assembly_sector_count")
            .unwrap();
        sectors.value = Some(Value::from(4_u64));
        assert!(validate_phase3b_model(&profile).is_err());
    }
}
