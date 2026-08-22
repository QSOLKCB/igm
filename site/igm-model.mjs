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

function finiteNumber(value, label) {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${label} must be an actual finite JSON number`);
  }
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

export function validateParameterBounds(profile) {
  let checked = 0;
  for (const item of profile.parameters ?? []) {
    const hasLower = item.lower_bound !== undefined && item.lower_bound !== null;
    const hasUpper = item.upper_bound !== undefined && item.upper_bound !== null;
    if ("value" in item && typeof item.value === "number") {
      finiteNumber(item.value, `${item.name}.value`);
    }
    if (!hasLower && !hasUpper) continue;
    checked += 1;
    if (!("value" in item)) throw new Error(`${item.name} declares bounds but has no value`);
    const value = finiteNumber(item.value, `${item.name}.value`);
    const lower = hasLower ? finiteNumber(item.lower_bound, `${item.name}.lower_bound`) : null;
    const upper = hasUpper ? finiteNumber(item.upper_bound, `${item.name}.upper_bound`) : null;
    if (lower !== null && upper !== null && lower > upper) {
      throw new Error(`${item.name} lower_bound exceeds upper_bound`);
    }
    if (lower !== null && value < lower) throw new Error(`${item.name} is below lower_bound`);
    if (upper !== null && value > upper) throw new Error(`${item.name} is above upper_bound`);
  }
  return Object.freeze({passed: true, checked});
}

function parameterValue(params, name) {
  const item = params.get(name);
  if (!item || !("value" in item)) throw new Error(`missing value-bearing parameter: ${name}`);
  const value = finiteNumber(item.value, name);
  const hasLower = item.lower_bound !== undefined && item.lower_bound !== null;
  const hasUpper = item.upper_bound !== undefined && item.upper_bound !== null;
  if (hasLower) {
    const lower = finiteNumber(item.lower_bound, `${name}.lower_bound`);
    if (value < lower) throw new Error(`${name} is below lower_bound`);
  }
  if (hasUpper) {
    const upper = finiteNumber(item.upper_bound, `${name}.upper_bound`);
    if (value > upper) throw new Error(`${name} is above upper_bound`);
  }
  return value;
}

function integerParameterValue(params, name) {
  const value = parameterValue(params, name);
  if (!Number.isSafeInteger(value) || value < 0) {
    throw new Error(`${name} must be a non-negative safe integer`);
  }
  return value;
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
  validateParameterBounds(profile);
}

function componentIndex(profile) {
  return new Map((profile.components ?? []).map((component) => [component.id, component]));
}

function constraintIndex(profile) {
  return new Map((profile.constraints ?? []).map((constraint) => [constraint.id, constraint]));
}

function requireComponent(index, id, expectedKind = null) {
  const component = index.get(id);
  if (!component) throw new Error(`profile component missing for adapter id: ${id}`);
  if (expectedKind !== null && component.kind !== expectedKind) {
    throw new Error(`${id} must have kind ${expectedKind}, got ${component.kind}`);
  }
  return component;
}

function requireConstraint(index, id) {
  const constraint = index.get(id);
  if (!constraint) throw new Error(`profile constraint missing for adapter id: ${id}`);
  return constraint;
}

function componentMetadata(component) {
  return {
    source_status: component.source_status,
    source_ids: Object.freeze([...(component.source_ids ?? [])]),
    notes: component.notes ?? ""
  };
}

function point(id, kind, x, y, z = 0, metadata = {}) {
  return Object.freeze({
    id,
    kind,
    x: finite(x, `${id}.x`),
    y: finite(y, `${id}.y`),
    z: finite(z, `${id}.z`),
    ...metadata
  });
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
    source_ids: Object.freeze([...(options.source_ids ?? [])]),
    constraint_id: options.constraint_id ?? null,
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

function participantsOf(constraint) {
  const participants = constraint.definition?.participants;
  if (!Array.isArray(participants) || !participants.length || !participants.every((id) => typeof id === "string")) {
    throw new Error(`${constraint.id} requires a non-empty string participants array`);
  }
  if (new Set(participants).size !== participants.length) {
    throw new Error(`${constraint.id} contains duplicate participants`);
  }
  return participants;
}

function buildGeometry(profile) {
  const params = parameterMap(profile);
  const components = componentIndex(profile);
  const constraints = constraintIndex(profile);
  const sectors = integerParameterValue(params, "assembly_sector_count");
  if (sectors !== 5) throw new Error("V0 fixture requires exactly five schematic sectors");
  const radius = parameterValue(params, "core_radius");
  const fabLength = parameterValue(params, "fab_length");
  const spread = parameterValue(params, "fab_spread_deg") * Math.PI / 180;
  const jOffset = parameterValue(params, "jchain_offset");

  const ringConstraint = requireConstraint(constraints, "constraint:five-sector-ring");
  const ringParticipants = participantsOf(ringConstraint);
  if (ringParticipants.length !== sectors) {
    throw new Error("five-sector ring participant count must equal assembly_sector_count");
  }
  if (ringConstraint.definition?.closed !== true) {
    throw new Error("V0 five-sector ring constraint must remain closed");
  }

  const armsConstraint = requireConstraint(constraints, "constraint:two-arms-per-sector");
  if (armsConstraint.definition?.arms_per_subunit !== 2) {
    throw new Error("V0 adapter requires exactly two schematic arms per subunit");
  }

  const points = [];
  const relations = [];

  for (let i = 0; i < ringParticipants.length; i += 1) {
    const subunitId = ringParticipants[i];
    const subunit = requireComponent(components, subunitId, "schematic-igm-subunit");
    if (!subunitId.startsWith("subunit:")) throw new Error(`unsupported V0 subunit id: ${subunitId}`);
    const key = subunitId.slice("subunit:".length);
    const theta = -Math.PI / 2 + (TAU * i / sectors);
    const sx = radius * Math.cos(theta);
    const sy = radius * Math.sin(theta);
    const sz = 0.08 * Math.sin(theta * 2);
    points.push(point(subunitId, subunit.kind, sx, sy, sz, {
      sector: i,
      theta,
      ...componentMetadata(subunit)
    }));

    for (const [side, sign] of [["l", -1], ["r", 1]]) {
      const armId = `fab:${key}:${side}`;
      const arm = requireComponent(components, armId, "schematic-fab-arm");
      const armTheta = theta + sign * spread;
      const fx = sx + fabLength * Math.cos(armTheta);
      const fy = sy + fabLength * Math.sin(armTheta);
      const fz = sz + sign * 0.06 * Math.cos(theta);
      points.push(point(armId, arm.kind, fx, fy, fz, {
        sector: i,
        side,
        theta: armTheta,
        ...componentMetadata(arm)
      }));
      relations.push(relation(`edge:${key}:${side}`, subunitId, armId, "structural", {
        status: armsConstraint.status,
        source_ids: armsConstraint.source_ids,
        constraint_id: armsConstraint.id,
        semantics: "synthetic V0 ownership/articulation relation"
      }));
    }
  }

  for (let i = 0; i < ringParticipants.length; i += 1) {
    const source = ringParticipants[i];
    const target = ringParticipants[(i + 1) % ringParticipants.length];
    relations.push(relation(`edge:ring:${i}`, source, target, "structural", {
      status: ringConstraint.status,
      source_ids: ringConstraint.source_ids,
      constraint_id: ringConstraint.id,
      semantics: "synthetic V0 ring adjacency derived from profile participant order"
    }));
  }

  const jConstraint = requireConstraint(constraints, "constraint:jchain-marker");
  const jParticipants = participantsOf(jConstraint);
  const jMarkers = jParticipants.filter(
    (id) => requireComponent(components, id).kind === "schematic-j-chain-constraint"
  );
  if (jMarkers.length !== 1) {
    throw new Error("constraint:jchain-marker must contain exactly one schematic J-chain marker");
  }
  const jMarkerId = jMarkers[0];
  const jMarker = requireComponent(components, jMarkerId, "schematic-j-chain-constraint");
  const jTargets = jParticipants.filter((id) => id !== jMarkerId);
  if (!jTargets.length) throw new Error("constraint:jchain-marker requires at least one target");
  for (const target of jTargets) requireComponent(components, target, "schematic-igm-subunit");

  points.push(point(jMarkerId, jMarker.kind, -jOffset, -jOffset * 0.35, 0, {
    constraintOnly: true,
    ...componentMetadata(jMarker)
  }));
  jTargets.forEach((target, index) => {
    relations.push(relation(`edge:j:${index}`, jMarkerId, target, "constraint", {
      status: jConstraint.status,
      source_ids: jConstraint.source_ids,
      constraint_id: jConstraint.id,
      semantics: "synthetic V0 asymmetry marker derived from profile participants"
    }));
  });

  const producedIds = new Set(points.map((p) => p.id));
  const profileIds = new Set(profile.components.map((component) => component.id));
  if (producedIds.size !== profileIds.size || [...profileIds].some((id) => !producedIds.has(id))) {
    throw new Error("V0 adapter must consume every declared profile component exactly once");
  }

  const hyperedges = Object.freeze([
    Object.freeze({
      id: "hyperedge:pentamer-core",
      relationshipClass: "constraint",
      participants: Object.freeze([...ringParticipants]),
      status: ringConstraint.status,
      source_ids: Object.freeze([...(ringConstraint.source_ids ?? [])]),
      constraint_id: ringConstraint.id,
      semantics: "synthetic V0 grouped core constraint derived from profile participants"
    })
  ]);

  return {points: Object.freeze(points), relations: Object.freeze(relations), hyperedges};
}

export function buildCanonicalState(profile) {
  assertProfileBoundary(profile);
  const bounds = validateParameterBounds(profile);
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
      logical: integerParameterValue(params, "logical_ensemble_size"),
      evaluated: integerParameterValue(params, "evaluated_sample_count"),
      displayed: integerParameterValue(params, "displayed_sample_count")
    },
    validation: {
      finiteAndBoundsPassed: bounds.passed,
      boundedParameterCount: bounds.checked
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
  finite(angle, "angle");
  finite(tx, "tx");
  finite(ty, "ty");
  finite(tz, "tz");
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return points.map((p) => {
    const {id, kind, x, y, z, ...metadata} = p;
    return point(
      id,
      kind,
      x * c - y * s + tx,
      x * s + y * c + ty,
      z + tz,
      {...metadata, transformed: true}
    );
  });
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
