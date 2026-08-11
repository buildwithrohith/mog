/**
 * Plan 027 — import-peak memory attribution.
 *
 * Drives a full XLSX import through the real napi addon call sequence used
 * in production (kernel/src/document/document-lifecycle-system.ts
 * executeHydrateXlsx: `computeBridge.importFromXlsxBytesDeferred(bytes)`,
 * which maps 1:1 onto the native instance method
 * `compute_import_from_xlsx_bytes_deferred`), then the deferred-hydration
 * completion call, then measures RSS on a 25ms background sampler across
 * labelled phase boundaries:
 *
 *   addon-loaded -> baseline (blank engine constructed) -> file-read ->
 *   parse+construct (deferred import — parse + non-Yrs index build, FUSED
 *   into one opaque native call, no JS-visible split) -> hydrate (Yrs CRDT
 *   write via complete_deferred_hydration, a SEPARATE native call) ->
 *   encode (encode_diff against the pre-import state vector, closest napi
 *   analog to a full-state durable snapshot encode) -> drop (null refs +
 *   forced GC) -> settle+3s -> settle+30s.
 *
 * API surface was discovered from the compiled addon's own
 * Object.getOwnPropertyNames(ComputeEngine.prototype) (index.d.ts ships
 * empty in this build) and cross-checked against
 * compute/core/src/storage/engine/bridge_imports.rs and
 * kernel/src/document/document-lifecycle-system.ts. Not guessed.
 *
 * Usage: node --expose-gc mem-attrib.mjs [runLabel]
 */

import { createRequire } from 'module';
import { readFileSync } from 'fs';
import { Worker, isMainThread, workerData, parentPort } from 'worker_threads';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);

const FIXTURE =
  '/Users/vish/Repos/GeniorLabs/sapiex/apps/web/public/dev-fixtures/repro194.xlsx';

const SAMPLE_INTERVAL_MS = 25;
const MAX_SAMPLES = 400_000; // ~166 minutes of headroom at 25ms; run is seconds

// ===========================================================================
// Worker-thread RSS sampler (runs on a separate OS thread so it keeps
// sampling process.memoryUsage().rss while the main thread is blocked
// inside a synchronous native FFI call).
// ===========================================================================
if (!isMainThread) {
  const { sab, ctrlSab } = workerData;
  const times = new Float64Array(sab, 0, MAX_SAMPLES);
  const rss = new Float64Array(sab, MAX_SAMPLES * 8, MAX_SAMPLES);
  const ctrl = new Int32Array(ctrlSab); // ctrl[0] = write index, ctrl[1] = stop flag

  const t0 = performance.now();
  while (Atomics.load(ctrl, 1) === 0) {
    const idx = Atomics.load(ctrl, 0);
    if (idx < MAX_SAMPLES) {
      times[idx] = performance.now() - t0;
      rss[idx] = process.memoryUsage().rss;
      Atomics.store(ctrl, 0, idx + 1);
    }
    Atomics.wait(ctrl, 1, 0, SAMPLE_INTERVAL_MS);
  }
  // Final sample after stop signal.
  const idx = Atomics.load(ctrl, 0);
  if (idx < MAX_SAMPLES) {
    times[idx] = performance.now() - t0;
    rss[idx] = process.memoryUsage().rss;
    Atomics.store(ctrl, 0, idx + 1);
  }
  parentPort.postMessage('done');
} else {
  await mainRun();
}

async function mainRun() {
  const runLabel = process.argv[2] || 'run';
  const t0 = performance.now();

  const sab = new SharedArrayBuffer(MAX_SAMPLES * 8 * 2);
  const ctrlSab = new SharedArrayBuffer(8);
  const ctrl = new Int32Array(ctrlSab);
  const times = new Float64Array(sab, 0, MAX_SAMPLES);
  const rssArr = new Float64Array(sab, MAX_SAMPLES * 8, MAX_SAMPLES);

  const worker = new Worker(__filename, { workerData: { sab, ctrlSab } });
  const workerDone = new Promise((resolve) => worker.once('message', resolve));

  const boundaries = []; // { label, tMs, rss }
  function mark(label) {
    // process.memoryUsage().rss on the MAIN thread — same process, same
    // metric as the worker samples; used for exact phase-boundary RSS.
    boundaries.push({ label, tMs: performance.now() - t0, rss: process.memoryUsage().rss });
  }

  // Give the worker a moment to start sampling before we begin.
  await new Promise((r) => setTimeout(r, 50));

  mark('addon-loaded');

  const require = createRequire(import.meta.url);
  const addon = require('./compute-core-napi.node');
  mark('addon-required');

  // ---- Construct a blank engine (mirrors kernel's STARTED bridge state
  // before hydrateXlsx runs; this is the real pre-import baseline). ----
  const SHEET_ID = '00000000-0000-0000-0000-000000000001';
  const minimalSnapshot = JSON.stringify({
    sheets: [{ id: SHEET_ID, name: 'Sheet1', rows: 100, cols: 26, cells: [] }],
  });
  const DEFAULT_LAYOUT_METRICS = JSON.stringify(null);

  let engine = new addon.ComputeEngine(minimalSnapshot, DEFAULT_LAYOUT_METRICS);
  addon.computeSetCurrentTime(45292.0);
  mark('baseline');

  // Pre-import state vector — used as the diff baseline for the "encode"
  // phase (closest napi-callable analog to a full-state snapshot encode;
  // encode_full_state itself is not exposed to napi, only encode_diff /
  // encode_state_vector are).
  let preImportSv = engine.compute_encode_state_vector();

  // ---- file-read ----
  let xlsxBytes = readFileSync(FIXTURE);
  mark('file-read');

  // ---- parse+construct (FUSED: XlsxParser parse + non-Yrs index build in
  // one opaque native call — no JS-visible split exists for this call). ----
  let importResult = engine.compute_import_from_xlsx_bytes_deferred(xlsxBytes);
  mark('parse+construct(deferred)');

  // ---- hydrate (separate native call: Yrs CRDT write + full-fidelity
  // index rebuild from the deferred snapshot data). ----
  let hydrateResult = engine.compute_complete_deferred_hydration();
  mark('hydrate(complete_deferred)');

  // ---- encode (durable snapshot equivalent) ----
  let encoded = engine.compute_encode_diff(preImportSv);
  mark('encode');

  // ---- drop ----
  const importResultSize = Buffer.isBuffer(importResult) ? importResult.length : -1;
  const hydrateResultSize = Buffer.isBuffer(hydrateResult) ? hydrateResult.length : -1;
  const encodedSize = Buffer.isBuffer(encoded) ? encoded.length : -1;
  engine = null;
  xlsxBytes = null;
  importResult = null;
  hydrateResult = null;
  encoded = null;
  preImportSv = null;
  if (globalThis.gc) {
    globalThis.gc();
    globalThis.gc();
  }
  mark('drop');

  await new Promise((r) => setTimeout(r, 3000));
  if (globalThis.gc) globalThis.gc();
  mark('settle+3s');

  await new Promise((r) => setTimeout(r, 27000));
  if (globalThis.gc) globalThis.gc();
  mark('settle+30s');

  // Stop the sampler.
  Atomics.store(ctrl, 1, 1);
  Atomics.notify(ctrl, 1);
  await workerDone;
  await worker.terminate();

  const nSamples = Atomics.load(ctrl, 0);
  const samples = [];
  for (let i = 0; i < nSamples; i++) samples.push({ tMs: times[i], rss: rssArr[i] });

  // ---- Build phase table ----
  console.log(`\n=== mem-attrib :: ${runLabel} ===`);
  console.log(`pid=${process.pid} node=${process.version} samples=${nSamples}`);
  console.log(
    `importResult bytes=${importResultSize} hydrateResult bytes=${hydrateResultSize} encoded diff bytes=${encodedSize}`,
  );

  const rows = [];
  for (let i = 0; i < boundaries.length; i++) {
    const b = boundaries[i];
    const prev = i > 0 ? boundaries[i - 1] : null;
    const deltaMb = prev ? (b.rss - prev.rss) / 1024 / 1024 : 0;
    rows.push({
      label: b.label,
      tMs: b.tMs.toFixed(1),
      rssMb: (b.rss / 1024 / 1024).toFixed(1),
      deltaMb: deltaMb.toFixed(1),
    });
  }

  console.log('\nPhase boundary table:');
  console.log(
    rows
      .map((r) => `  ${r.label.padEnd(26)} t=${r.tMs.padStart(9)}ms  rss=${r.rssMb.padStart(9)}MB  delta=${r.deltaMb.padStart(9)}MB`)
      .join('\n'),
  );

  // ---- Global peak + which phase interval contains it ----
  let peak = { tMs: -1, rss: -1 };
  for (const s of samples) if (s.rss > peak.rss) peak = s;

  let peakPhase = 'unknown';
  for (let i = 0; i < boundaries.length - 1; i++) {
    if (peak.tMs >= boundaries[i].tMs && peak.tMs <= boundaries[i + 1].tMs) {
      peakPhase = `${boundaries[i].label} -> ${boundaries[i + 1].label}`;
      break;
    }
  }
  if (peak.tMs > boundaries[boundaries.length - 1].tMs) {
    peakPhase = `after ${boundaries[boundaries.length - 1].label}`;
  }

  console.log(
    `\nGlobal peak: rss=${(peak.rss / 1024 / 1024).toFixed(1)}MB at t=${peak.tMs.toFixed(1)}ms  (interval: ${peakPhase})`,
  );

  // ---- Per-phase peak (max sample within each interval) + share of total peak ----
  console.log('\nPer-phase peak RSS (max sample within interval):');
  for (let i = 0; i < boundaries.length - 1; i++) {
    const lo = boundaries[i].tMs;
    const hi = boundaries[i + 1].tMs;
    let maxRss = -Infinity;
    for (const s of samples) if (s.tMs >= lo && s.tMs <= hi && s.rss > maxRss) maxRss = s.rss;
    const share = peak.rss > 0 ? ((maxRss / peak.rss) * 100).toFixed(1) : '0.0';
    console.log(
      `  ${(boundaries[i].label + ' -> ' + boundaries[i + 1].label).padEnd(46)} peak=${(maxRss / 1024 / 1024).toFixed(1).padStart(9)}MB  (${share}% of global peak)`,
    );
  }

  // ---- Samples immediately around the peak (+/- 5 samples) ----
  const peakIdx = samples.findIndex((s) => s.tMs === peak.tMs);
  const around = samples.slice(Math.max(0, peakIdx - 5), peakIdx + 6);
  console.log('\nSamples around peak:');
  console.log(
    around
      .map((s) => `  t=${s.tMs.toFixed(1).padStart(9)}ms  rss=${(s.rss / 1024 / 1024).toFixed(1).padStart(9)}MB`)
      .join('\n'),
  );

  console.log(`\n=== end ${runLabel} ===\n`);
}
