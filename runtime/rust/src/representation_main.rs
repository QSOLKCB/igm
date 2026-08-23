// SPDX-License-Identifier: Apache-2.0

use igm_runtime::representation::Phase5Engine;
use igm_runtime::RuntimeError;
use std::env;
use std::path::Path;

fn usage() -> &'static str {
    "igm-represent\n\ncommands:\n  bundle MODEL.json CONFIG.json\n\nThe command emits a deterministic Phase 5 representation bundle. The bundle is\ncomputational/representational evidence only and does not promote biological or\nclinical validation.\n"
}

fn command_bundle(model: &Path, config: &Path) -> Result<(), RuntimeError> {
    let engine = Phase5Engine::load(model, config, Path::new("."))?;
    let bundle = engine.bundle()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&bundle).map_err(|e| RuntimeError(e.to_string()))?
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("bundle") if args.len() == 3 => {
            command_bundle(Path::new(&args[1]), Path::new(&args[2]))
        }
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
