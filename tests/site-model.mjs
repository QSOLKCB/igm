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

const a = buildCanonicalState(profile);
const b = buildCanonicalState(JSON.parse(JSON.stringify(profile)));
assert.equal(a.stateFingerprint, b.stateFingerprint, "same profile must produce same canonical state identity");
assert.equal(a.profileFingerprint, profileFingerprint(profile));
assert.equal(a.validationLevel, "V0");
assert.equal(a.nonClinical, true);
assert.equal(a.components.length, 16);
assert.equal(a.relationships.length, 17);
assert.equal(a.hyperedges.length, 1);
assert.equal(a.claims.biological_validity_claimed, false);

const before = computeDistanceMatrix(a.components);
const transformed = rigidTransformPoints(a.components, Math.PI * 0.371, 17.25, -9.5, 3.125);
const after = computeDistanceMatrix(transformed);
assert.ok(maxDistanceResidual(before, after) < 1e-12, "rigid transform must preserve pairwise distances");

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

console.log("OK: deterministic IGM Pages model tests passed");
