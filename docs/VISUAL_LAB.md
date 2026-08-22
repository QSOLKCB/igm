# IGM Visual Laboratory

Status: Phase 2 implementation for the deterministic GitHub Pages research visualizer.

The visual laboratory is a transparent browser interface over one canonical V0 model state. It is not a clinical tool and it is not biological authority.

> **Perfect Mathematics Does Not Equal Perfect Biological Reality.**

## Purpose

The browser application demonstrates how one versioned IGM profile can be projected into multiple computational views without changing the underlying model identity:

```text
IGM-MODEL-PROFILE-V1
        |
        v
canonical browser state
        |
  +-----+------+-------+--------+--------+
  |            |       |        |        |
assembly     array    graph    fabric   cyclic
  |            |       |        |        |
  +------------+-------+--------+--------+
               |
        derived observables
```

The current profile, `IGM-SCHEMATIC-PENTAMER-V0`, is deliberately synthetic/schematic. It is a software fixture for testing replaceable modelling infrastructure.

## Views

### Assembly / spatial schematic

Displays five schematic subunit sectors, ten schematic Fab-arm placeholders, and one J-chain constraint marker. Coordinates are dimensionless model coordinates. They are not presented as atomistic positions or measured cryo-EM coordinates.

### Numerical array

Displays the pairwise Euclidean distance matrix for the canonical component points.

The application deliberately labels this object:

```text
numerical-array-not-declared-tensor
```

This enforces `INV-MATH-002`: a multidimensional array is not automatically a tensor.

### Graph

Displays the same relationships with deterministic structural, circular, hierarchical, or adjacency-matrix layouts. Layout is presentation only. Relationship direction, weight, multiplicity and type belong to model semantics rather than renderer convenience.

### Fabric / relation view

The IGM fabric renderer is an original Apache-2.0 implementation using a simple published visualization concept:

- one component per horizontal row;
- one relationship per vertical column;
- multiple relationships remain separately visible;
- columns can be deterministically grouped/filtered by relationship class.

This is useful for avoiding conventional dense graph "hairballs" and for making multi-edge semantics inspectable.

### Vortex-inspired coordinate projection

This is an optional cyclic presentation of the exact same canonical state. It is permanently labelled as a coordinate projection/parameterization only. The project makes no claim that IgM is a vortex or that vortex physics explains IgM biology.

## Determinism

Canonical model generation uses no `Math.random()`.

The model profile is serialized canonically and receives a deterministic FNV-1a-64 diagnostic fingerprint. The same canonical state is shared by all views and deep-frozen in browser memory.

Camera and view state are deliberately excluded from canonical model identity.

The test suite verifies that rigid rotations/translations preserve the pairwise-distance observable within a tight floating-point tolerance.

## Logical versus displayed scale

The V0 profile deliberately separates:

- logical ensemble size;
- evaluated sample count;
- displayed sample count.

These are demonstration values, not claims about the number of real IgM conformations. The browser remains a bounded visualization surface even when future runtimes evaluate much larger ensembles.

## Provenance inspector

Components, relations, parameters and numerical observables can be selected for inspection. Status such as `assumed` or `unknown` is displayed textually rather than encoded only by colour.

Future V1+ profiles can replace schematic values with source-derived or calibrated parameters while retaining the same UI and runtime boundary.

## Exports

The Pages application can export:

- canonical state JSON;
- pairwise-observable CSV;
- provenance JSON;
- SVG snapshots;
- a bounded WebM recording when browser MediaRecorder support is available.

Visual exports retain `V0`, `NOT CLINICAL`, and `INV-BIO-001` labelling.

## Local use

Build deterministic site data:

```bash
python3 tools/build_site_data.py
```

Serve the repository root or `site/` with a local HTTP server, for example:

```bash
python3 -m http.server 8000 -d site
```

Then open `http://localhost:8000/`.

Do not open `site/index.html` directly with `file://`; browser module/fetch rules may block the generated data files.

## Validation

Phase 2 CI runs:

```bash
python3 tools/validate_docs.py
python3 tools/validate_profile.py --self-test
python3 tools/validate_profile.py profiles/igm-schematic-pentamer-v0.json
python3 tools/validate_site.py
node tests/site-model.mjs
python3 tools/build_site_data.py
```

The site fails closed if the V0 profile violates upstream claim boundaries or if required generated data is unavailable.
