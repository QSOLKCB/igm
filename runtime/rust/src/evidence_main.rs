// SPDX-License-Identifier: Apache-2.0

use igm_runtime::evidence::{
    adapt_evidence, bundle_candidates, EvidenceInput, SnapshotPolicy, SourceRegistry,
    BIOCHEMICAL_ADAPTER_ID, CRYO_EM_ADAPTER_ID, MD_ADAPTER_ID, SOURCE_ADAPTER_CONTRACT,
};
use igm_runtime::{RuntimeError, INV_BIO_001};
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};

fn registry_path() -> PathBuf {
    PathBuf::from("research/sources.json")
}

fn policy_path() -> PathBuf {
    PathBuf::from("research/source-snapshot-policy.json")
}

fn usage() -> &'static str {
    "igm-evidence\n\n\
commands:\n\
  registry\n\
  adapt INPUT.json\n\
  bundle INPUT1.json INPUT2.json [...]\n\n\
Phase 4 evidence adapters normalize registered source observations while preserving\n\
provenance, uncertainty, access/reuse metadata, conflicts, and claim strength.\n\
Adapters do not promote validation level or biological/clinical validity.\n"
}

fn load_registry_policy() -> Result<(SourceRegistry, SnapshotPolicy), RuntimeError> {
    Ok((
        SourceRegistry::load(&registry_path())?,
        SnapshotPolicy::load(&policy_path())?,
    ))
}

fn command_registry() -> Result<(), RuntimeError> {
    let registry = SourceRegistry::load(&registry_path())?;
    let structural = registry
        .sources
        .iter()
        .filter(|source| source.authority == "structural-source")
        .collect::<Vec<_>>();
    let doi_count = structural.iter().filter(|source| source.doi.is_some()).count();
    let pdb_count = structural.iter().filter(|source| source.pdb.is_some()).count();
    let emdb_count = structural.iter().filter(|source| source.emdb.is_some()).count();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema": "IGM-PHASE4-SOURCE-REGISTRY-REPORT-V1",
            "source_adapter_contract": SOURCE_ADAPTER_CONTRACT,
            "source_count": registry.sources.len(),
            "structural_source_count": registry.structural_source_count(),
            "structural_identifier_counts": {
                "doi": doi_count,
                "pdb": pdb_count,
                "emdb": emdb_count
            },
            "adapters": [CRYO_EM_ADAPTER_ID, MD_ADAPTER_ID, BIOCHEMICAL_ADAPTER_ID],
            "validation_level_promoted_by_adapter": false,
            "biological_validity_claimed": false,
            "clinical_validity_claimed": false,
            "inv_bio_001": INV_BIO_001
        }))
        .map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_adapt(input_path: &Path) -> Result<(), RuntimeError> {
    let (registry, policy) = load_registry_policy()?;
    let input = EvidenceInput::load(input_path)?;
    let candidate = adapt_evidence(&registry, &policy, &input)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&candidate).map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn command_bundle(paths: &[String]) -> Result<(), RuntimeError> {
    if paths.is_empty() {
        return Err(RuntimeError("bundle requires at least one evidence input path".into()));
    }
    let (registry, policy) = load_registry_policy()?;
    let mut candidates = Vec::with_capacity(paths.len());
    for path in paths {
        let input = EvidenceInput::load(Path::new(path))?;
        candidates.push(adapt_evidence(&registry, &policy, &input)?);
    }
    let bundle = bundle_candidates(candidates)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&bundle).map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("registry") if args.len() == 1 => command_registry(),
        Some("adapt") if args.len() == 2 => command_adapt(Path::new(&args[1])),
        Some("bundle") if args.len() >= 2 => command_bundle(&args[1..]),
        Some("-h") | Some("--help") | None => {
            print!("{}", usage());
            Ok(())
        }
        Some(other) => Err(RuntimeError(format!("unknown/invalid command: {other}\n\n{}", usage()))),
    };
    if let Err(error) = result {
        eprintln!("FAIL: {error}");
        std::process::exit(1);
    }
}
