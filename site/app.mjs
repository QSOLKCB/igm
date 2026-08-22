// SPDX-License-Identifier: Apache-2.0
import {
  INV_BIO_001,
  buildCanonicalState,
  cloneForExport,
  stableStringify
} from "./igm-model.mjs";

const NS = "http://www.w3.org/2000/svg";
const viz = document.querySelector("#viz");
const telemetry = document.querySelector("#telemetry");
const inspector = document.querySelector("#inspector");
const parameterList = document.querySelector("#parameterList");
const graphControls = document.querySelector("#graphControls");
const fabricControls = document.querySelector("#fabricControls");
const rotateControl = document.querySelector("#rotateControl");
const zoomControl = document.querySelector("#zoomControl");
const references = document.querySelector("#references");

const ui = {
  view: "assembly",
  graphLayout: "structural",
  rotation: 0,
  zoom: 1,
  selected: null,
  fabricClasses: new Set(["structural", "constraint"])
};

let profile;
let sourceRegistry;
let state;

function svgEl(name, attrs = {}, text = null) {
  const el = document.createElementNS(NS, name);
  for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, String(value));
  if (text !== null) el.textContent = text;
  return el;
}

function clearSvg(title, subtitle) {
  viz.replaceChildren();
  viz.append(
    svgEl("text", {x: 34, y: 42, class: "viz-title"}, title),
    svgEl("text", {x: 34, y: 66, class: "viz-subtitle"}, subtitle),
    svgEl("text", {x: 34, y: 696, class: "viz-warning"}, `V0 · NOT CLINICAL · ${INV_BIO_001}`)
  );
}

function project(p) {
  const angle = ui.rotation * Math.PI / 180;
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  const x = p.x * c - p.y * s;
  const y = p.x * s + p.y * c;
  const depth = 1 + p.z * 0.08;
  return {
    x: 600 + x * 190 * ui.zoom * depth,
    y: 362 + y * 190 * ui.zoom * depth,
    z: p.z
  };
}

function statePoint(id) {
  return state.components.find((p) => p.id === id);
}

function select(kind, value) {
  ui.selected = {kind, value};
  renderInspector();
}

function clickable(el, kind, value) {
  el.classList.add("interactive");
  el.tabIndex = 0;
  el.addEventListener("click", () => select(kind, value));
  el.addEventListener("keydown", (event) => {
    if (event.key === "Enter" || event.key === " ") select(kind, value);
  });
  return el;
}

function nodeStyle(kind) {
  if (kind.includes("fab")) return {fill: "#c4b5fd", stroke: "#ede9fe", r: 16};
  if (kind.includes("j-chain")) return {fill: "#fbbf24", stroke: "#fef3c7", r: 18};
  return {fill: "#7dd3fc", stroke: "#e0f2fe", r: 23};
}

function renderAssembly() {
  clearSvg("Assembly / spatial schematic", "Fresh V0 projection. Positions are schematic model coordinates, not measured molecular geometry.");
  for (const edge of state.relationships) {
    const a = statePoint(edge.source);
    const b = statePoint(edge.target);
    if (!a || !b) continue;
    const pa = project(a); const pb = project(b);
    const line = svgEl("line", {
      x1: pa.x, y1: pa.y, x2: pb.x, y2: pb.y,
      stroke: edge.relationshipClass === "constraint" ? "#fbbf24" : "#52677c",
      "stroke-width": edge.relationshipClass === "constraint" ? 3 : 2,
      "stroke-dasharray": edge.relationshipClass === "constraint" ? "8 7" : "none"
    });
    clickable(line, "relationship", edge);
    viz.append(line);
  }
  const ordered = [...state.components].sort((a, b) => a.z - b.z);
  for (const node of ordered) {
    const p = project(node);
    const style = nodeStyle(node.kind);
    const circle = svgEl("circle", {cx: p.x, cy: p.y, r: style.r, fill: style.fill, stroke: style.stroke, "stroke-width": 2});
    clickable(circle, "component", node);
    viz.append(circle, svgEl("text", {x: p.x + style.r + 6, y: p.y + 4, class: "node-label"}, node.id));
  }
}

function matrixCellColor(value, max) {
  const ratio = max > 0 ? Math.min(1, value / max) : 0;
  const light = 18 + Math.round(ratio * 48);
  return `hsl(202 70% ${light}%)`;
}

function renderArray() {
  const data = state.numericalArrays.pairwiseDistance;
  clearSvg("Numerical array view", `${data.declaration}. INV-MATH-002 is enforced: this distance matrix is not presented as a tensor.`);
  const n = data.values.length;
  const left = 155, top = 105, size = Math.min(470 / n, 32);
  const max = Math.max(...data.values.flat());
  for (let i = 0; i < n; i += 1) {
    const short = data.componentOrder[i].replace("subunit:", "S:").replace("fab:", "F:").replace("jchain:", "J:");
    viz.append(svgEl("text", {x: left - 8, y: top + i * size + size * .68, class: "grid-label", "text-anchor": "end"}, short));
    for (let j = 0; j < n; j += 1) {
      const value = data.values[i][j];
      const rect = svgEl("rect", {x: left + j * size, y: top + i * size, width: size - 1, height: size - 1, fill: matrixCellColor(value, max)});
      rect.append(svgEl("title", {}, `${data.componentOrder[i]} ↔ ${data.componentOrder[j]} = ${value.toFixed(6)} model-unit`));
      clickable(rect, "observable", {kind: "pairwise-distance", a: data.componentOrder[i], b: data.componentOrder[j], value, units: data.units, declaration: data.declaration});
      viz.append(rect);
    }
  }
  viz.append(
    svgEl("text", {x: 760, y: 145, class: "viz-subtitle"}, "Array semantics"),
    svgEl("text", {x: 760, y: 176, class: "node-label"}, "• Euclidean pairwise distance"),
    svgEl("text", {x: 760, y: 198, class: "node-label"}, "• Coordinate-invariant under rigid transforms"),
    svgEl("text", {x: 760, y: 220, class: "node-label"}, "• No biological interpretation attached"),
    svgEl("text", {x: 760, y: 252, class: "viz-warning"}, "NOT A DECLARED TENSOR")
  );
}

function layoutNodes(mode) {
  if (mode === "structural") return new Map(state.components.map((n) => [n.id, project(n)]));
  if (mode === "circular") {
    const sorted = [...state.components].sort((a, b) => a.id.localeCompare(b.id));
    return new Map(sorted.map((n, i) => [n.id, {x: 600 + 245 * Math.cos(-Math.PI / 2 + i * Math.PI * 2 / sorted.length), y: 365 + 245 * Math.sin(-Math.PI / 2 + i * Math.PI * 2 / sorted.length)}]));
  }
  if (mode === "hierarchical") {
    const buckets = {"schematic-j-chain-constraint": 0, "schematic-igm-subunit": 1, "schematic-fab-arm": 2};
    const groups = new Map([[0, []], [1, []], [2, []]]);
    for (const node of [...state.components].sort((a, b) => a.id.localeCompare(b.id))) groups.get(buckets[node.kind] ?? 2).push(node);
    const out = new Map();
    for (const [level, nodes] of groups) {
      nodes.forEach((node, i) => out.set(node.id, {x: 130 + (940 * (i + 1) / (nodes.length + 1)), y: 150 + level * 205}));
    }
    return out;
  }
  return new Map();
}

function renderGraphMatrix() {
  clearSvg("Graph / adjacency matrix", "Rows and columns are stable component IDs. Cell presence means a declared V0 relationship, nothing more.");
  const nodes = [...state.components].sort((a, b) => a.id.localeCompare(b.id));
  const index = new Map(nodes.map((n, i) => [n.id, i]));
  const linked = new Map();
  for (const edge of state.relationships) {
    linked.set(`${index.get(edge.source)}:${index.get(edge.target)}`, edge);
    if (!edge.directed) linked.set(`${index.get(edge.target)}:${index.get(edge.source)}`, edge);
  }
  const size = Math.min(470 / nodes.length, 31), left = 165, top = 105;
  nodes.forEach((node, i) => {
    viz.append(svgEl("text", {x: left - 8, y: top + i * size + size * .68, class: "grid-label", "text-anchor": "end"}, node.id));
    nodes.forEach((_, j) => {
      const edge = linked.get(`${i}:${j}`);
      const rect = svgEl("rect", {x: left + j * size, y: top + i * size, width: size - 1, height: size - 1, fill: edge ? (edge.relationshipClass === "constraint" ? "#fbbf24" : "#7dd3fc") : "#18212c"});
      if (edge) clickable(rect, "relationship", edge);
      viz.append(rect);
    });
  });
}

function renderGraph() {
  if (ui.graphLayout === "matrix") return renderGraphMatrix();
  clearSvg(`Graph / ${ui.graphLayout}`, "Layout is presentation only. INV-VIZ-001 and INV-VIZ-002 prohibit promoting layout or proximity into model semantics.");
  const positions = layoutNodes(ui.graphLayout);
  for (const edge of state.relationships) {
    const a = positions.get(edge.source), b = positions.get(edge.target);
    if (!a || !b) continue;
    const line = svgEl("line", {x1: a.x, y1: a.y, x2: b.x, y2: b.y, stroke: edge.relationshipClass === "constraint" ? "#fbbf24" : "#52677c", "stroke-width": 2});
    clickable(line, "relationship", edge); viz.append(line);
  }
  for (const node of state.components) {
    const p = positions.get(node.id); if (!p) continue;
    const style = nodeStyle(node.kind);
    const circle = svgEl("circle", {cx: p.x, cy: p.y, r: 15, fill: style.fill, stroke: style.stroke});
    clickable(circle, "component", node);
    viz.append(circle, svgEl("text", {x: p.x + 20, y: p.y + 4, class: "node-label"}, node.id));
  }
  const m = state.graphMetrics;
  viz.append(svgEl("text", {x: 885, y: 132, class: "viz-subtitle"}, `nodes ${m.nodeCount} · relations ${m.relationCount}`));
  viz.append(svgEl("text", {x: 885, y: 156, class: "viz-subtitle"}, `density ${m.density.toFixed(4)} · max degree ${m.maxDegree}`));
  viz.append(svgEl("text", {x: 885, y: 180, class: "viz-warning"}, "metrics = computational observables"));
}

function renderFabric() {
  clearSvg("Fabric / relation view", "Original Apache-2.0 renderer: one component row, one visible relationship column. Row adjacency does not imply biological proximity.");
  const nodes = [...state.components].sort((a, b) => a.id.localeCompare(b.id));
  const row = new Map(nodes.map((n, i) => [n.id, 115 + i * 31]));
  const edges = [...state.relationships].filter((e) => ui.fabricClasses.has(e.relationshipClass)).sort((a, b) => `${a.relationshipClass}:${a.id}`.localeCompare(`${b.relationshipClass}:${b.id}`));
  const x0 = 300;
  const spacing = Math.min(32, 780 / Math.max(edges.length, 1));
  nodes.forEach((node) => {
    const y = row.get(node.id);
    const line = svgEl("line", {x1: 260, y1: y, x2: 1125, y2: y, stroke: "#263545", "stroke-width": 1});
    viz.append(line, svgEl("text", {x: 245, y: y + 4, class: "node-label", "text-anchor": "end"}, node.id));
  });
  edges.forEach((edge, i) => {
    const x = x0 + i * spacing;
    const y1 = row.get(edge.source), y2 = row.get(edge.target);
    const line = svgEl("line", {x1: x, y1: Math.min(y1, y2), x2: x, y2: Math.max(y1, y2), stroke: edge.relationshipClass === "constraint" ? "#fbbf24" : "#7dd3fc", "stroke-width": 3});
    clickable(line, "relationship", edge); viz.append(line);
    if (i % 2 === 0) viz.append(svgEl("text", {x: x + 3, y: 94, class: "grid-label", transform: `rotate(-55 ${x + 3} 94)`}, edge.relationshipClass));
  });
}

function renderVortex() {
  clearSvg("Vortex-inspired coordinate projection", "PARAMETERIZATION ONLY. The same canonical state remains available in non-vortex views. No claim that IgM is a vortex.");
  const nodes = [...state.components].sort((a, b) => a.id.localeCompare(b.id));
  const centerX = 600, centerY = 355;
  nodes.forEach((node, i) => {
    const theta = -Math.PI / 2 + i * 2.399963229728653;
    const radius = 70 + i * 15;
    const x = centerX + radius * Math.cos(theta);
    const y = centerY + radius * .62 * Math.sin(theta);
    const style = nodeStyle(node.kind);
    if (i > 0) {
      const prevTheta = -Math.PI / 2 + (i - 1) * 2.399963229728653;
      const prevR = 70 + (i - 1) * 15;
      viz.append(svgEl("line", {x1: centerX + prevR * Math.cos(prevTheta), y1: centerY + prevR * .62 * Math.sin(prevTheta), x2: x, y2: y, stroke: "#3a4c60", "stroke-width": 1.5}));
    }
    const circle = svgEl("circle", {cx: x, cy: y, r: 12, fill: style.fill, stroke: style.stroke});
    clickable(circle, "component", node); viz.append(circle, svgEl("text", {x: x + 16, y: y + 4, class: "node-label"}, node.id));
  });
  viz.append(svgEl("text", {x: 35, y: 100, class: "viz-warning"}, "REPRESENTATION IS NOT ONTOLOGY"));
}

function render() {
  graphControls.classList.toggle("hidden", ui.view !== "graph");
  fabricControls.classList.toggle("hidden", ui.view !== "fabric");
  document.querySelector(".camera-controls").classList.toggle("hidden", ui.view !== "assembly");
  if (ui.view === "assembly") renderAssembly();
  else if (ui.view === "array") renderArray();
  else if (ui.view === "graph") renderGraph();
  else if (ui.view === "fabric") renderFabric();
  else renderVortex();
}

function renderTelemetry() {
  const boundsStatus = state.validation?.finiteAndBoundsPassed === true
    ? `PASS (${state.validation.boundedParameterCount} bounded parameters checked)`
    : "FAIL";
  const rows = [
    ["Profile", state.modelId], ["Version", state.modelVersion], ["Validation", `${state.validationLevel} · SCHEMATIC`],
    ["Clinical status", "NOT CLINICAL"], ["Profile hash", state.profileFingerprint], ["State hash", state.stateFingerprint],
    ["Components", state.components.length], ["Relations", state.relationships.length], ["Hyperedges", state.hyperedges.length],
    ["Logical ensemble", state.sampling.logical.toLocaleString()], ["Evaluated", state.sampling.evaluated.toLocaleString()], ["Displayed", state.sampling.displayed.toLocaleString()],
    ["Finite/bounds", boundsStatus], ["Biological validity", "NOT CLAIMED"]
  ];
  telemetry.replaceChildren(...rows.flatMap(([k, v]) => [Object.assign(document.createElement("dt"), {textContent: k}), Object.assign(document.createElement("dd"), {textContent: String(v)})]));
}

function renderInspector() {
  if (!ui.selected) {
    inspector.textContent = "Select a component, relationship, observable, or parameter. V0 values remain assumed or unknown unless explicitly sourced.";
    return;
  }
  const {kind, value} = ui.selected;
  const payload = {selection_kind: kind, validation_level: state.validationLevel, non_clinical: true, ...value};
  inspector.innerHTML = `<strong>${escapeHtml(kind)}</strong><pre>${escapeHtml(JSON.stringify(payload, null, 2))}</pre>`;
}

function renderParameters() {
  parameterList.replaceChildren();
  for (const p of profile.parameters) {
    const button = document.createElement("button");
    button.textContent = `${p.name} · ${p.status}`;
    button.addEventListener("click", () => select("parameter", p));
    parameterList.append(button);
  }
}

function renderReferences() {
  const wanted = new Set([
    "visualization.biofabric-longabaugh-2012",
    "math.lim-2021-tensors-computations",
    "graph.koutrouli-2020-biological-network-era",
    "graph.pagliarini-chicco-2026-nine-tips"
  ]);
  const items = (sourceRegistry.sources ?? []).filter((s) => wanted.has(s.id));
  references.replaceChildren();
  for (const source of items) {
    const div = document.createElement("div");
    const a = document.createElement("a"); a.href = source.url; a.target = "_blank"; a.rel = "noreferrer"; a.textContent = source.title;
    div.append(a, document.createTextNode(` · ${source.class}`)); references.append(div);
  }
  const found = new Set(items.map((item) => item.id));
  const missing = [...wanted].filter((id) => !found.has(id));
  if (missing.length) {
    const warning = document.createElement("div");
    warning.textContent = `Missing registered method references: ${missing.join(", ")}`;
    references.append(warning);
  }
}

function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, (ch) => ({"&":"&amp;","<":"&lt;",">":"&gt;","\"":"&quot;","'":"&#39;"}[ch]));
}

function download(name, body, type) {
  const url = URL.createObjectURL(new Blob([body], {type}));
  const a = document.createElement("a"); a.href = url; a.download = name; document.body.append(a); a.click(); a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function canonicalStateExport() {
  return {...cloneForExport(state), export_notice: `V0 · NOT CLINICAL · ${INV_BIO_001}`};
}

function csvCell(value) {
  const text = String(value ?? "");
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text;
}

function exportCsv() {
  const matrix = state.numericalArrays.pairwiseDistance;
  const rows = ["record_type,source,target,distance_model_unit,notice"];
  matrix.componentOrder.forEach((source, i) => matrix.componentOrder.forEach((target, j) => {
    rows.push([
      "observation",
      source,
      target,
      matrix.values[i][j].toPrecision(12),
      ""
    ].map(csvCell).join(","));
  }));
  rows.push(["metadata", "", "", "", `V0 · NOT CLINICAL · ${INV_BIO_001}`].map(csvCell).join(","));
  download("igm-v0-observables.csv", rows.join("\n"), "text/csv");
}

function serializeSvg() {
  const clone = viz.cloneNode(true);
  clone.setAttribute("xmlns", NS);
  return new XMLSerializer().serializeToString(clone);
}

async function recordWebm() {
  if (!("MediaRecorder" in window) || !HTMLCanvasElement.prototype.captureStream) {
    alert("MediaRecorder/canvas captureStream is not available in this browser."); return;
  }
  const source = serializeSvg();
  const url = URL.createObjectURL(new Blob([source], {type: "image/svg+xml"}));
  const image = new Image();
  await new Promise((resolve, reject) => { image.onload = resolve; image.onerror = reject; image.src = url; });
  const canvas = document.createElement("canvas"); canvas.width = 1200; canvas.height = 720;
  const ctx = canvas.getContext("2d"); const stream = canvas.captureStream(10);
  const preferred = MediaRecorder.isTypeSupported("video/webm;codecs=vp9") ? "video/webm;codecs=vp9" : "video/webm";
  const recorder = new MediaRecorder(stream, {mimeType: preferred}); const chunks = [];
  recorder.ondataavailable = (e) => { if (e.data.size) chunks.push(e.data); };
  const done = new Promise((resolve) => { recorder.onstop = resolve; });
  recorder.start(); const start = performance.now();
  await new Promise((resolve) => {
    const frame = (now) => {
      ctx.fillStyle = "#0a0d12"; ctx.fillRect(0, 0, 1200, 720); ctx.drawImage(image, 0, 0, 1200, 720);
      ctx.fillStyle = "#fde68a"; ctx.font = "14px sans-serif"; ctx.fillText(`V0 · NOT CLINICAL · frame ${Math.floor((now - start) / 100)}`, 870, 26);
      if (now - start >= 8000) resolve(); else requestAnimationFrame(frame);
    }; requestAnimationFrame(frame);
  });
  recorder.stop(); await done; URL.revokeObjectURL(url);
  download("igm-v0-visual-lab.webm", new Blob(chunks, {type: "video/webm"}), "video/webm");
}

function bindControls() {
  document.querySelectorAll("[data-view]").forEach((button) => button.addEventListener("click", () => {
    const before = state.stateFingerprint; ui.view = button.dataset.view;
    document.querySelectorAll("[data-view]").forEach((b) => b.classList.toggle("active", b === button)); render();
    if (before !== state.stateFingerprint) throw new Error("view switch mutated canonical model identity");
  }));
  document.querySelectorAll("[data-layout]").forEach((button) => button.addEventListener("click", () => {
    ui.graphLayout = button.dataset.layout; document.querySelectorAll("[data-layout]").forEach((b) => b.classList.toggle("active", b === button)); render();
  }));
  fabricControls.querySelectorAll("input").forEach((input) => input.addEventListener("change", () => {
    if (input.checked) ui.fabricClasses.add(input.value); else ui.fabricClasses.delete(input.value); render();
  }));
  rotateControl.addEventListener("input", () => { ui.rotation = Number(rotateControl.value); render(); });
  zoomControl.addEventListener("input", () => { ui.zoom = Number(zoomControl.value) / 100; render(); });
  document.querySelector("#resetView").addEventListener("click", () => { ui.rotation = 0; ui.zoom = 1; rotateControl.value = "0"; zoomControl.value = "100"; render(); });
  document.querySelector("#exportState").addEventListener("click", () => download("igm-v0-state.json", `${stableStringify(canonicalStateExport())}\n`, "application/json"));
  document.querySelector("#exportCsv").addEventListener("click", exportCsv);
  document.querySelector("#exportProvenance").addEventListener("click", () => download("igm-v0-provenance.json", JSON.stringify({profile, selected: ui.selected, notice: `V0 · NOT CLINICAL · ${INV_BIO_001}`}, null, 2), "application/json"));
  document.querySelector("#exportSvg").addEventListener("click", () => download("igm-v0-visual.svg", serializeSvg(), "image/svg+xml"));
  document.querySelector("#recordWebm").addEventListener("click", () => recordWebm().catch((error) => alert(`Recording failed: ${error.message}`)));
}

async function boot() {
  const [profileResponse, sourceResponse] = await Promise.all([fetch("./data/profile.json"), fetch("./data/sources.json")]);
  if (!profileResponse.ok || !sourceResponse.ok) throw new Error("site data missing: run tools/build_site_data.py before serving locally");
  profile = await profileResponse.json(); sourceRegistry = await sourceResponse.json(); state = buildCanonicalState(profile);
  renderTelemetry(); renderParameters(); renderReferences(); bindControls(); render();
}

boot().catch((error) => {
  document.body.innerHTML = `<main class="card" style="margin:2rem;padding:2rem"><h1>IGM visual laboratory failed closed</h1><pre>${escapeHtml(error.stack || error.message)}</pre><p>Nothing is rendered when the profile or generated site data fails validation.</p></main>`;
});
