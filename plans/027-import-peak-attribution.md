# Plan 027: Import-peak attribution — where does the ~10GB live?

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. This plan is MEASUREMENT-ONLY: no
> optimization changes ship from it. Never run git in the main repo; you are
> in worktree `.worktrees/opt-027-mem-attrib` (branch `opt/027-mem-attrib`)
> and may commit THERE only.

## Status

- **Priority**: P1 (gates the decision between streaming parse vs process isolation)
- **Effort**: S-M
- **Risk**: LOW (diagnostic script + optional env-gated instrumentation)
- **Depends on**: none (007's size-report exists; this is the native phase view)
- **Planned at**: `ec0a3229`, 2026-08-10

## Why

Prod metrics (2026-08-10, HANDOFF post-effort section) show import spikes to
~10GB that jemalloc returns within minutes — capacity-safe, but the peak
itself is unattributed. Before green-lighting either "streaming sheet-by-sheet
parse" (L, MED-HIGH risk, parser hot path) or "process isolation for big
parses" (infra), we need to know which phase owns the peak:

1. raw zip/XML buffers held by `XlsxParser` (bridge in
   `compute/napi/src/lib.rs:65-122`, parser crate `xlsx_api`),
2. engine construction (`compute/core/src/storage/engine/construction/xlsx.rs`
   + `deferred.rs` + `assembly.rs`),
3. durable snapshot encode (`encode_full_state`),
4. or the kernel-side extra byte copy (`executeHydrateXlsx`, held-list item).

## The measurement

Write `compute/napi/mem-attrib.mjs` (committed to this branch) that:

1. Loads the addon exactly like `compute/napi/smoke-test.mjs` does. Discover
   the real import call sequence from `compute/napi/index.d.ts` — do NOT
   guess; read the `.d.ts` and mirror how Sapiex's
   `packages/xlsx-core` drives a full import (read-only reference:
   `/Users/vish/Repos/GeniorLabs/sapiex/packages/xlsx-core/src/`).
2. Samples `process.memoryUsage().rss` on a 25ms interval for the whole run
   into an array (timestamped), and records labelled phase boundaries:
   `baseline` → `file-read` → `parse` (XlsxParser) → `construct/hydrate`
   (engine build; per-sheet if the API is staged) → `encode` (full-state
   snapshot, if callable) → `drop` (engine + parser out of scope) →
   `settle+3s` → `settle+30s`.
3. Fixture: `/Users/vish/Repos/GeniorLabs/sapiex/apps/web/public/dev-fixtures/repro194.xlsx`
   (19.4MB, 1.4M cells; READ-ONLY — never modify the sapiex repo).
4. Prints a phase table: RSS at each boundary, delta per phase, global peak
   and which phase interval contains it, plus samples around the peak.

**Fallback if phases are fused** (one opaque native call does parse+build):
add TEMPORARY, env-gated (`MOG_MEM_PHASES=1`) instrumentation to the native
path — jemalloc `stats.allocated`/`stats.resident` printed at internal phase
boundaries in the construction path — rebuild darwin
(`pnpm --dir compute/napi run build:release`), and re-run. Keep the
instrumentation commits separate from the script commit so they can be
dropped or kept behind the flag by the advisor.

## Verification / deliverable

- The phase table from THREE consecutive runs (same process fresh each run),
  peak attribution consistent across runs.
- A one-paragraph verdict in the report: which phase owns ≥60% of the peak,
  and whether the peak is additive (parser buffers still live during engine
  build — i.e. overlap) or sequential (parser freed before build).
- The overlap question is decision-critical: if parser output is freed
  before construction peaks, streaming parse buys little; if both are live
  simultaneously, freeing parser buffers per-sheet is the direct win.
- macOS RSS caveat (HANDOFF standing rule): RSS lies about MADV_FREE pages —
  report `/usr/bin/footprint` for any retention claim; peak-during-run is
  unaffected.

## STOP conditions

- The addon API cannot drive a full import from Node at all → report the
  exact missing surface; do not modify the API.
- Instrumented build fails to compile → report; do not fight the build
  beyond one honest attempt.
- Peak attribution differs wildly across the three runs (>25% swing) →
  report the three tables; the advisor decides.

## Done criteria

- [ ] `mem-attrib.mjs` committed on `opt/027-mem-attrib`
- [ ] Three-run phase tables + verdict paragraph reported back
- [ ] Any temporary instrumentation isolated in its own commit(s)

## RESULT (2026-08-10, executed same day)

Three consecutive runs, fresh process each, real fixture, 25ms RSS sampling on
a worker thread (survives synchronous native calls). Script:
`compute/napi/mem-attrib.mjs` (commit b33a2b71). Production-representative
path confirmed: `import_from_xlsx_bytes_deferred` then
`complete_deferred_hydration` (mirrors kernel `executeHydrateXlsx`).

| phase | run1 Δ | run2 Δ | run3 Δ |
|---|---|---|---|
| parse+construct (fused deferred import) | +48MB | +63MB | +48MB |
| hydrate (complete_deferred_hydration) | **+8,334MB** | **+7,804MB** | **+7,676MB** |
| encode (encode_diff) | +768MB | +2,603MB | +1,312MB |
| peak | 9,396MB | 10,581MB | 9,487MB |
| settle+30s | 5,017MB | 5,206MB | 5,013MB |

Peaks swing 12.1% (< 25% STOP bar); peak always inside hydrate→encode.

**VERDICT: the parser is a non-factor (0.5-0.6% of peak). The peak is the
Yrs CRDT materialization** — hydrate owns 75-90% of the climb in every run,
encode a secondary 10-25%. Deferred-import staging stays live as the SOURCE
for the entire ~18-19s hydrate while the Yrs structures grow as the
DESTINATION (RSS climbs smoothly across the whole call → both resident
together). Consequences:

- **Streaming sheet-by-sheet parse is ruled OUT as a peak reducer** — at most
  ~63MB at stake.
- The only real peak-shrinker would be **staged per-sheet hydration with
  per-sheet staging release** (extends the 008/022 machinery into the import
  path) — L, MED-HIGH risk, in the path 019 just hardened.
- **Decision: NOT NOW.** The spike is transient (prod returns it within ~2min,
  HANDOFF post-effort section), capacity-safe at BIG_PARSE_CONCURRENCY=2 on
  24GB replicas, and contributes negligibly to spend (bill is baseline-driven).
  Revisit only if replica OOMs appear or concurrency needs to rise; process
  isolation (worker-service parses) is the cheaper de-risk if that day comes.

Side finding (non-blocking): without `MALLOC_CONF=dirty_decay_ms:0,...`
(which prod sets, Dockerfile:103) jemalloc retains ~50% of peak past 30s on
macOS; `/usr/bin/footprint` confirmed the retained ~5GB is genuinely dirty,
0B reclaimable — the macOS-vs-prod decay difference is the env var, not
measurement error.
