#!/usr/bin/env python3
"""Build deterministic local data for the static IGM Pages visual laboratory."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SITE = ROOT / "site"
DATA = SITE / "data"
PROFILE = ROOT / "profiles" / "igm-schematic-pentamer-v0.json"
SOURCES = ROOT / "research" / "sources.json"


def reject_constant(value: str) -> None:
    raise ValueError(f"non-standard JSON constant forbidden: {value}")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"), parse_constant=reject_constant)


def canonical_bytes(value) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n").encode("utf-8")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def main() -> int:
    DATA.mkdir(parents=True, exist_ok=True)
    profile = load(PROFILE)
    sources = load(SOURCES)
    (DATA / "profile.json").write_bytes(canonical_bytes(profile))
    (DATA / "sources.json").write_bytes(canonical_bytes(sources))

    manifest_entries = []
    for path in sorted(SITE.rglob("*")):
        if not path.is_file() or path.name == "manifest.json":
            continue
        rel = path.relative_to(SITE).as_posix()
        manifest_entries.append({"path": rel, "sha256": sha256(path), "bytes": path.stat().st_size})
    manifest = {
        "schema": "igm-pages-manifest/1",
        "profile": profile["model_id"],
        "validation_level": profile["validation_level"],
        "non_clinical": True,
        "entries": manifest_entries,
    }
    (SITE / "manifest.json").write_bytes(canonical_bytes(manifest))
    print(f"OK: built IGM Pages data with {len(manifest_entries)} hashed files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
