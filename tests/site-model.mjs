// SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import fs from "node:fs";
import {
  buildCanonicalState,
  computeDistanceMatrix,
  maxDistanceResidual,
  profileFingerprint,
  rigidTransformPoints,
  stableStringify
} from "../site/igm-model.mjs";

const profile = JSON.parse(fs.readFileSync(new URL("../profiles/igm-schematic-pentamer-v0.json", import.meta.url), "utf8"));
const clone = (value) => JSON.parse(JSON.stringify(value));

const a = buildCanonicalState(profile);
const b = buildCanonicalState(clone(profile));
assert.equal(a.stateFingerprint, b.stateFingerprint, "same profile must produce same canonical state identity");
assert.equal(a.profileFingerprint, profileFingerprint(profile));
assert.equal(a.validationLevel, "V0");
assert.equal(a.nonClinical, true);
assert.equal(a.components.length, 16);
assert.equal(a.relationships.length, 17);
assert.equal(a.hyperedges.length, 1);
assert.equal(a.claims.biological_validity_claimed, false);
assert.equal(a.validation.finiteAndBoundsPassed, true);
assert.equal(a.validation.boundedParameterCount, 4);

const subunitA = a.components.find((component) => component.id === "subunit:a");
assert.equal(subunitA.source_status, "assumed", "canonical components must preserve provenance status");
assert.match(subunitA.notes, /V0 visualization component/, "canonical components must preserve provenance notes");
assert.deepEqual(subunitA.source_ids, [], "canonical components must preserve source id list");

const before = computeDistanceMatrix(a.components);
const transformed = rigidTransformPoints(a.components, Math.PI * 0.371, 17.25, -9.5, 3.125);
const after = computeDistanceMatrix(transformed);
assert.ok(maxDistanceResidual(before, after) < 1e-12, "rigid transform must preserve pairwise distances");
assert.equal(transformed.find((component) => component.id === "subunit:a").source_status, "assumed", "rigid presentation transforms must retain provenance");

const originalIdentity = a.stateFingerprint;
for (const presentation of ["assembly", "array", "graph", "fabric", "vortex"]) {
  const viewState = {presentation, camera: {rotation: presentation.length, zoom: 1}};
  assert.ok(stableStringify(viewState).includes(presentation));
  assert.equal(a.stateFingerprint, originalIdentity, "presentation state must not alter canonical model identity");
}

const unknown = profile.parameters.find((p) => p.status === "unknown");
assert.ok(unknown);
assert.equal(Object.hasOwn(unknown, "value"), false, "unknown parameters must not carry invented values");

const reordered = Object.fromEntries(Object.entries(profile).reverse());
assert.equal(profileFingerprint(profile), profileFingerprint(reordered), "profile fingerprint must be key-order independent");

const falseRadius = clone(profile);
falseRadius.parameters.find((p) => p.name === "core_radius").value = false;
assert.throws(() => buildCanonicalState(falseRadius), /actual finite JSON number/, "runtime adapter must not coerce booleans into geometry numbers");

const nullRadius = clone(profile);
nullRadius.parameters.find((p) => p.name === "core_radius").value = null;
assert.throws(() => buildCanonicalState(nullRadius), /actual finite JSON number/, "runtime adapter must not coerce null into geometry numbers");

const outOfBounds = clone(profile);
outOfBounds.parameters.find((p) => p.name === "core_radius").value = 99;
assert.throws(() => buildCanonicalState(outOfBounds), /above upper_bound/, "out-of-bounds profiles must fail before telemetry can report PASS");

const changedJ = clone(profile);
changedJ.constraints.find((constraint) => constraint.id === "constraint:jchain-marker").definition.participants = [
  "jchain:0",
  "subunit:b",
  "subunit:d"
];
const changedJState = buildCanonicalState(changedJ);
const jTargets = changedJState.relationships
  .filter((edge) => edge.constraint_id === "constraint:jchain-marker")
  .map((edge) => edge.target)
  .sort();
assert.deepEqual(jTargets, ["subunit:b", "subunit:d"], "J-chain relation edges must come from profile participants, not hardcoded endpoints");
assert.equal(changedJState.relationships.some((edge) => edge.constraint_id === "constraint:jchain-marker" && edge.target === "subunit:a"), false);

console.log("OK: deterministic IGM Pages model tests passed");
