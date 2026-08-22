# PENTA-CRT CPU Optimization Profile

Status: Phase 3B implementation for deterministic non-clinical research infrastructure.

> **INV-BIO-001: Perfect Mathematics Does Not Equal Perfect Biological Reality.**

`IGM-PENTA-CRT-CPU-V1` accelerates the schematic V0 runtime without changing the project's scientific authority boundary. It is an execution profile over `IGM-SCHEMATIC-PENTAMER-V0`, not a source-informed molecular model, molecular-dynamics engine, or clinical simulator.

## Contracts

- base runtime: `IGM-RUST-RUNTIME-V1`
- base model: `IGM-SCHEMATIC-PENTAMER-V0`
- execution profile: `IGM-PENTA-CRT-CPU-PROFILE-V1`
- optimization engine: `IGM-PENTA-CRT-CPU-V1`
- numerical profile: `IGM-PENTA-CRT-F64-LUT-BLOCK-CIRCULANT-ZRESIDUAL-V1`
- run schema: `IGM-PENTA-CRT-CPU-RUN-V1`
- verification schema: `IGM-PENTA-CRT-VERIFY-V1`

The profile file is `runtime/profiles/igm-penta-crt-cpu-v1.json`.

## Explicit discrete state space

Phase 3B does not hide synthetic hinge values inside runtime code. The execution profile declares four V0 computational degrees of freedom:

| DoF | Radix | Meaning |
| --- | ---: | --- |
| `left_fab_delta_deg` | 17 | shared synthetic left-arm angular offset, -8..+8 degrees |
| `right_fab_delta_deg` | 17 | shared synthetic right-arm angular offset, -8..+8 degrees |
| `jchain_dx` | 9 | sparse synthetic J-marker X offset, -0.04..+0.04 model-unit |
| `jchain_dy` | 9 | sparse synthetic J-marker Y offset, -0.04..+0.04 model-unit |

The exact mixed-radix state count is:

```text
17 x 17 x 9 x 9 = 23,409
```

These are **computational fixture states**. They are not asserted to be real IgM conformations or a measured hinge distribution.

## Mixed-radix addressing

Every conformation has one stable integer identity. With digits `(l, r, jx, jy)` and radices `(17, 17, 9, 9)`, the runtime uses little-endian mixed-radix ordering:

```text
index = l + 17 * (r + 17 * (jx + 9 * jy))
```

Decode/encode are exact integer operations. Unit tests exhaustively round-trip all 23,409 addresses.

Conformation identity is independent of CPU worker assignment. Partial campaigns use checked `[start, end)` slices and cannot escape the declared execution-profile domain.

## PENTAFOLD dynamic geometry

The base V0 runtime already generates the five-sector core with the fixed 72-degree C5 recurrence. Phase 3B extends that reuse to dynamic synthetic arm states.

For each conformation:

1. choose left/right articulation bins by exact mixed-radix decode;
2. load precomputed deterministic sine/cosine values from bounded lookup tables;
3. construct one sector's radial/arm directions;
4. advance the five sectors with the fixed C5 recurrence;
5. apply the J-marker offsets as an explicit sparse defect.

The hot path performs no trigonometric or square-root calls.

The C5 assumption is explicitly marked `assumed` and `biological_symmetry_claimed=false`. A future source-informed profile must not inherit this shortcut unless that profile independently satisfies the optimization admission rules.

## Deterministic lookup tables

Lookup tables are built once when the optimization engine is admitted. They use the same fixed-operation-order deterministic polynomial projection as the PR3 Rust reference.

The reference verifier deliberately recomputes those values instead of reading the LUT. This lets CI measure LUT-vs-reference geometry residuals rather than merely comparing a table with itself.

No platform `libm` trigonometry enters correctness identity.

## Structured C5 reuse with exact Z residuals

The first Phase 3B implementation deliberately tried full three-dimensional C5 block reuse. The residual gate rejected it. That failure was useful: the legacy Phase-2 V0 drawing has absolute-angle-dependent Z terms, so the complete 3D fixture is **not** block-circulant even though its XY construction is C5-structured.

The accepted implementation therefore makes the narrower mathematical claim that the **XY projection** of the 15 non-J nodes is C5-structured. It never silently upgrades that to a full-3D symmetry claim.

A brute-force 16-node pair set contains:

```text
16 choose 2 = 120 full 3D pair distances
```

The optimized evaluator computes:

- 45 planar XY squared-distance entries for the five `3 x 3` sector blocks from sector zero;
- 15 direct full 3D J-marker-to-symmetric-node distances;
- 105 exact local `dz^2` corrections for the non-J unordered pairs while reconstructing the canonical pair sequence.

For a non-J pair, the reconstruction is exactly:

```text
d2 = cached_xy_d2 + (z_i - z_j)^2
```

The reported `structured_distance_evaluations_per_conformation = 60` counts the expensive vector-distance evaluations replaced by structured reuse: 45 planar block evaluations plus 15 sparse J evaluations. The 105 scalar `dz^2` corrections are reported separately and are **not** hidden as free work.

This distinction matters. Phase 3B reduces repeated spatial-distance work for the declared V0 fixture, but it does not claim that every arithmetic operation has been halved, nor that the full 3D model is block-circulant.

## Sparse J-chain/asymmetry defect

The J marker is intentionally excluded from the C5 block reuse. Its synthetic `dx`/`dy` state is applied as a sparse correction, and all 15 J-to-symmetric-node distances are evaluated directly in 3D.

This is the implementation form of the design rule:

```text
structured execution core + explicit sparse asymmetry
```

rather than silently forcing an asymmetric model into exact C5 symmetry.

## SoA and allocation behavior

Each conformation is represented internally as fixed-size structure-of-arrays storage:

```text
x[16]
y[16]
z[16]
```

Per-conformation geometry and pair buffers are fixed stack arrays. The Phase 3B hot loop performs no heap allocation proportional to conformation count.

The larger GPU-shaped AoSoA/32-lane memory contract remains Phase 3C work.

## Reference verification

`igm-penta-crt verify` samples the mixed-radix domain deterministically and compares:

1. lookup-table dynamic geometry against a deterministic recomputed reference geometry;
2. XY block reuse plus exact Z residual reconstruction against brute-force full-3D evaluation of every unordered pair.

The current numerical acceptance tolerance is:

```text
1e-12
```

This is an implementation-equivalence threshold for the declared f64 V0 numerical profile. It is not a biological tolerance and must not be weakened solely to make a failing optimization pass.

The original full-3D reuse attempt produced a residual of about `4.95e-2` and was rejected rather than having the tolerance widened. The corrected representation preserves the legacy V0 Z terms explicitly.

## Result and manifest identity

Optimized execution keeps the PR3 identity split:

- `result_sha256` binds model profile, optimization profile, conformation slice, numerical profile, deterministic diagnostics, and distance extrema. It is independent of worker count.
- `manifest_sha256` additionally binds requested/effective worker partitioning.

Local elapsed time and conformations/second are emitted separately and are not identity-bearing.

## CLI

Inspect the explicit execution profile:

```bash
cargo run --locked --release --bin igm-penta-crt -- profile
```

Inspect one mixed-radix address:

```bash
cargo run --locked --release --bin igm-penta-crt -- address 17000
```

Run the independent residual gate:

```bash
cargo run --locked --release --bin igm-penta-crt -- verify --samples 257
```

Run a deterministic slice:

```bash
cargo run --locked --release --bin igm-penta-crt -- run --start 100 --count 4096 --workers 16
```

## Nonclaims

Phase 3B does **not** establish:

- a measured IgM hinge distribution;
- source-informed conformational probabilities;
- molecular dynamics;
- atomistic interactions;
- a biological C5 symmetry law;
- full-3D block-circulant IgM geometry;
- clinical significance of any computed state;
- patient-specific simulation;
- GPU execution or GPU speedup;
- superiority of structured reuse on every future profile.

A source-informed profile that violates the declared symmetry assumptions must reject this optimization rather than silently use it.
