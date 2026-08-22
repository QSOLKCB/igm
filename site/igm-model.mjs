// SPDX-License-Identifier: Apache-2.0
// Fresh IGM implementation. No BioFabric source code or assets are used here.

export const INV_BIO_001 = "Perfect Mathematics Does Not Equal Perfect Biological Reality";
export const INV_MATH_002 = "A Multidimensional Array Is Not Automatically a Tensor";
export const INV_MATH_003 = "Coordinate Presentation Must Not Alter Coordinate-Invariant Observables";
export const INV_GRAPH_001 = "Graph Representation Must Match Declared Relationship Semantics";
export const INV_GRAPH_002 = "Topology Is Measured or Sourced, Never Assumed";
export const INV_VIZ_001 = "Visualization Layout Must Not Alter Model Semantics";
export const INV_VIZ_002 = "Visual Proximity Does Not Imply Biological Proximity";

const TAU = Math.PI * 2;
const MASK_64 = (1n << 64n) - 1n;
const FNV_OFFSET = 0xcbf29ce484222325n;
const FNV_PRIME = 0x100000001b3n;

function finite(value, label) {
  if (!Number.isFinite(value)) throw new Error(`${label} must be finite`);
  return value;
}

export function stableStringify(value) {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) throw new Error("non-finite numbers are forbidden");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(stableStringify).join(",")}]`;
  if (typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableStringify(value[key])}`).join(",")}}`;
  }
  throw new Error(`unsupported canonical value type: ${typeof value}`);
}

export function fnv1a64(text) {
  let state = FNV_OFFSET;
  for (const byte of new TextEncoder().encode(text)) {
    state ^= BigInt(byte);
    state = (state * FNV_PRIME) & MASK_64;
  }
  return state.toString(16).padStart(16, "0");
}

export function profileFingerprint(profile) {
  return `fnv1a64:${fnv1a64(stableStringify(profile))}`;
}

export function parameterMap(profile) {
  return new Map((profile.parameters ?? []).map((p) => [p.name, p]));
}

function parameterValue(params, name) {
  const item = params.get(name);
  if (!item || !("value" in item)) throw new Error(`missing value-bearing parameter: ${name}`);
  return finite(Number(item.value), name);
}

function assertProfileBoundary(profile) {
  if (profile.schema !== "IGM-MODEL-PROFILE-V1") throw new Error("unexpected profile schema");
  if (profile.validation_level !== "V0") throw new Error("PR2 browser fixture must remain V0");
  if (profile.claims?.biological_validity_claimed !== false) throw new Error("V0 biological validity claim forbidden");
  for (const [key, value] of Object.entries(profile.claims ?? {})) {
    if (key !== "biological_validity_claimed" && value !== false) throw new Error(`forbidden upstream claim: ${key}`);
  }
  const ids = new Set();
  for (const component of profile.components ?? []) {
    if (ids.has(component.id)) throw new Error(`duplicate component id: ${component.id}`);
    ids.add(component.id);
  }
  for (const parameter of profile.parameters ?? []) {
    if (parameter.status === "unknown" && Object.hasOwn(parameter, "value")) {
      throw new Error(`unknown parameter may not carry value: ${parameter.name}`);
    }
  }
}

function point(id, kind, x, y, z = 0, metadata = {}) {
  return Object.freeze({id, kind, x: finite(x, `${id}.x`), y: finite(y, `${id}.y`), z: finite(z, `${id}.z`), ...metadata});
}

function relation(id, source, target, relationshipClass, options = {}) {
  return Object.freeze({
    id,
    source,
    target,
    relationshipClass,
    directed: Boolean(options.directed),
    weight: options.weight ?? null,
    status: options.status ?? "assumed",
    semantics: options.semantics ?? "synthetic V0 relationship"
  });
}

export function computeDistanceMatrix(points) {
  return points.map((a) => points.map((b) => {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    const dz = a.z - b.z;
    return Math.hypot(dx, dy, dz);
  }));
}

export function graphMetrics(nodes, relations) {
  const degree = new Map(nodes.map((node) => [node.id, 0]));
  for (const edge of relations) {
    if (!degree.has(edge.source) || !degree.has(edge.target)) continue;
    degree.set(edge.source, degree.get(edge.source) + 1);
    degree.set(edge.target, degree.get(edge.target) + 1);
  }
  const values = [...degree.values()];
  const n = nodes.length;
  const e = relations.length;
  const maxEdges = n > 1 ? (n * (n - 1)) / 2 : 0;
  return Object.freeze({
    nodeCount: n,
    relationCount: e,
    meanDegree: n ? values.reduce((a, b) => a + b, 0) / n : 0,
    maxDegree: values.length ? Math.max(...values) : 0,
    density: maxEdges ? e / maxEdges : 0,
    degree: Object.freeze(Object.fromEntries([...degree.entries()].sort(([a], [b]) => a.localeCompare(b))))
  });
}

function buildGeometry(profile) {
  const params = parameterMap(profile);
  const sectors = parameterValue(params, "assembly_sector_count");
  if (sectors !== 5) throw new Error("V0 fixture requires exactly five schematic sectors");
  const radius = parameterValue(params, "core_radius");
  const fabLength = parameterValue(params, "fab_length");
  const spread = parameterValue(params, "fab_spread_deg") * Math.PI / 180;
  const jOffset = parameterValue(params, "jchain_offset");
  const labels = ["a", "b", "c", "d", "e"];
  const points = [];
  const relations = [];

  for (let i = 0; i < sectors; i += 1) {
    const key = labels[i];
    const theta = -Math.PI / 2 + (TAU * i / sectors);
    const sx = radius * Math.cos(theta);
    const sy = radius * Math.sin(theta);
    const sz = 0.08 * Math.sin(theta * 2);
    points.push(point(`subunit:${key}`, "schematic-igm-subunit", sx, sy, sz, {sector: i, theta}));

    for (const [side, sign] of [["l", -1], ["r", 1]]) {
      const armTheta = theta + sign * spread;
      const fx = sx + fabLength * Math.cos(armTheta);
      const fy = sy + fabLength * Math.sin(armTheta);
      const fz = sz + sign * 0.06 * Math.cos(theta);
      points.push(point(`fab:${key}:${side}`, "schematic-fab-arm", fx, fy, fz, {sector: i, side, theta: armTheta}));
      relations.push(relation(`edge:${key}:${side}`, `subunit:${key}`, `fab:${key}:${side}`, "structural", {
        semantics: "synthetic V0 ownership/articulation relation"
      }));
    }
  }

  for (let i = 0; i < sectors; i += 1) {
    const a = labels[i];
    const b = labels[(i + 1) % sectors];
    relations.push(relation(`edge:ring:${a}:${b}`, `subunit:${a}`, `subunit:${b}`, "structural", {
      semantics: "synthetic V0 ring adjacency"
    }));
  }

  points.push(point("jchain:0", "schematic-j-chain-constraint", -jOffset, -jOffset * 0.35, 0, {constraintOnly: true}));
  relations.push(relation("edge:j:a", "jchain:0", "subunit:a", "constraint", {semantics: "synthetic V0 asymmetry marker"}));
  relations.push(relation("edge:j:e", "jchain:0", "subunit:e", "constraint", {semantics: "synthetic V0 asymmetry marker"}));

  const hyperedges = Object.freeze([
    Object.freeze({
      id: "hyperedge:pentamer-core",
      relationshipClass: "constraint",
      participants: Object.freeze(labels.map((x) => `subunit:${x}`)),
      status: "assumed",
      semantics: "synthetic V0 grouped core constraint"
    })
  ]);

  return {points: Object.freeze(points), relations: Object.freeze(relations), hyperedges};
}

export function buildCanonicalState(profile) {
  assertProfileBoundary(profile);
  const geometry = buildGeometry(profile);
  const distances = computeDistanceMatrix(geometry.points);
  const params = parameterMap(profile);
  const canonical = {
    contract: "IGM-PAGES-CANONICAL-STATE-V1",
    modelId: profile.model_id,
    modelVersion: profile.version,
    validationLevel: profile.validation_level,
    nonClinical: true,
    profileFingerprint: profileFingerprint(profile),
    components: geometry.points,
    relationships: geometry.relations,
    hyperedges: geometry.hyperedges,
    numericalArrays: {
      pairwiseDistance: {
        declaration: "numerical-array-not-declared-tensor",
        units: "model-unit",
        componentOrder: geometry.points.map((p) => p.id),
        values: distances
      }
    },
    graphMetrics: graphMetrics(geometry.points, geometry.relations),
    sampling: {
      logical: parameterValue(params, "logical_ensemble_size"),
      evaluated: parameterValue(params, "evaluated_sample_count"),
      displayed: parameterValue(params, "displayed_sample_count")
    },
    claims: Object.freeze({...profile.claims}),
    invariants: Object.freeze([
      INV_BIO_001,
      INV_MATH_002,
      INV_MATH_003,
      INV_GRAPH_001,
      INV_GRAPH_002,
      INV_VIZ_001,
      INV_VIZ_002
    ])
  };
  canonical.stateFingerprint = `fnv1a64:${fnv1a64(stableStringify(canonical))}`;
  return deepFreeze(canonical);
}

export function deepFreeze(value) {
  if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
  for (const child of Object.values(value)) deepFreeze(child);
  return Object.freeze(value);
}

export function rigidTransformPoints(points, angle, tx, ty, tz = 0) {
  finite(angle, "angle"); finite(tx, "tx"); finite(ty, "ty"); finite(tz, "tz");
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return points.map((p) => point(
    p.id,
    p.kind,
    p.x * c - p.y * s + tx,
    p.x * s + p.y * c + ty,
    p.z + tz,
    {transformed: true}
  ));
}

export function maxDistanceResidual(a, b) {
  if (a.length !== b.length) throw new Error("distance matrix shape mismatch");
  let max = 0;
  for (let i = 0; i < a.length; i += 1) {
    if (a[i].length !== b[i].length) throw new Error("distance matrix row mismatch");
    for (let j = 0; j < a[i].length; j += 1) max = Math.max(max, Math.abs(a[i][j] - b[i][j]));
  }
  return max;
}

export function cloneForExport(state) {
  return JSON.parse(JSON.stringify(state));
}
