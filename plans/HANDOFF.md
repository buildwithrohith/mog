# HANDOFF — mog optimization effort (updated 2026-08-07, during round 4 dispatch)

Read this + `plans/README.md` (status table) to resume. Advisor session handle
for lane mailbox: `term_a888fa0d-f6eb-4a5e-bcd8-ce30e3176b01`.

## Where everything lives
- Fork: `~/Repos/analyst/mog`, branch `sapiex-patches`, remote `fork` =
  github.com/buildwithrohith/mog (all fork commits authored+committed Rohith
  K Rangan; history rewritten 2026-08-06, backup branch
  `backup-pre-author-rewrite` local only). HEAD at round-4 dispatch: `02403d9b`.
- Sapiex: vendored engine tarballs `vendor/mog/*-02403d9.tgz` (wasm +
  spreadsheet-app), refs in root+apps/web package.json. Rounds 1-3 all MERGED
  to develop (PRs #2699, #2702 fix, #2716) and pushed; staging auto-deploys.
- Plans 001-019: ALL DONE (see README table for per-plan acceptance evidence).
- Measurement scripts: session scratchpad `mog-ab-measure.mjs`,
  `mog-double-import.mjs` (usage: node --expose-gc <script> <path-to-.node>
  <xlsx> <label>). Stock binary for A/B:
  `sapiex/node_modules/.pnpm/@mog-sdk+darwin-arm64@0.10.4/node_modules/@mog-sdk/darwin-arm64/compute-core-napi.node`.
  Fixture: `sapiex/apps/web/public/dev-fixtures/repro194.xlsx` (1,407,921
  cells, 28 sheets). Browser harness:
  `localhost:<port>/dev/mog-app-test?file=repro194&engine=adapter` (standalone
  `next dev` in apps/web ONLY — root `pnpm dev` hardcodes port 3001 + kills
  ports; never use it for verification).

## Measured state (same-session A/B vs stock, repro194)
- Import wall −11%; peak import RSS 13.4→10.7GB; live per-open-workbook
  2.9→1.63GB; post-drop physical footprint 10.4→~5.2GB (jemalloc,
  MALLOC_CONF deploy-tunable; eager decay:0 costs ~20% import time — prod
  recommendation: time-boxed decay e.g. dirty_decay_ms:10000).
- Browser: full 28-sheet browse zero traps, peak 4,017MB (round-1 engine
  DIED at 4,086MB on same workload). Normal browsing ~199MB.
- Suites at 02403d9b: compute-core 3,023/0; xlsx-parser 3,582/0. Only fmt
  drift: upstream bridge-wasm macro (deliberate).

## Round 4 (dispatched this session — supervise, review, integrate)
Lanes (codex TUI panes; recipe: orca terminal create --command "codex
--dangerously-bypass-approvals-and-sandbox --cd <worktree> -m gpt-5.6-luna
-c model_reasoning_effort=max" ... sol medium for M; send task text with
--enter; bare --enter re-submits a queued line; verify pickup; monitor
pattern: text-parse `status:` line, 3-strike exits, bash-3.2-safe plain vars):
- Lane M (SOL medium): 008-DESIGN-OUTPUT.md §5 row 4 — SheetDependencyManifest
  + closure admission. Worktree mog-opt-m, branch opt/dependency-closure.
- Lane N (luna max): plan 020 incremental snapshot lowering. Worktree
  mog-opt-n, branch opt/incremental-lowering.
- Lane O (luna max): plan 021 small-wins bundle. Worktree mog-opt-o, branch
  opt/round4-small-wins.
Dispatch state 2026-08-07: tasks SENT to all three panes and pickup VERIFIED
(M exploring parser AST for the manifest; N working sequentially in its
worktree after noticing the orca registry's stale worktree field — the field
is cosmetic, codex was started with --cd so cwd is correct; O started 021).
Persistent monitor armed (script: session scratchpad r4-monitor.sh; signals:
MAILBOX-REPORT / LANE-IDLE / PANE-EXITED / PROMPT-STUCK / LANE-ERROR;
liveness = `orca terminal show` connected: field, NOT `terminal info` which
doesn't exist; mailbox = `orca orchestration check --peek`, legacy read-only).
Handles: M=term_bc76f6fe-6fea-46e8-af26-70ed836d2894,
N=term_b19ef4a2-8182-4b43-b779-7ab62578bce1,
O=term_33ab97e1-0a9f-4539-b40c-ab82ce02ae17 (also /tmp/r4-lanes.txt).
Integration: review each diff vs plan (verify claims yourself — every round
had a false lane claim caught by re-running on base), merge to sapiex-patches,
full battery (lib + parser via FILE capture, awk on pipes is flaky), cargo
fmt -p on touched crates, push fork, rebuild wasm
(`bash compute/wasm/build.sh --profile release`) + app
(`pnpm --filter @mog-sdk/spreadsheet-app build`; dist at
runtime/spreadsheet-app/dist embeds wasm — verify sha match) + napi
(`pnpm --dir compute/napi run build:release`), re-run A/B vs stock IN THE
SAME SESSION (cross-session wall times lie under load), pack tarballs from
scratchpad/pack/{wasm,app} skeletons (manifests already there; content-addressed
-<shorthash> names), wire sapiex package.jsons, browser matrix (repro194
full-browse peak + formats, c446k no update-depth, c648k formats), then PR to
develop + SERVER-SIDE `gh pr merge` (local develop is OWNED by another
session's worktree `.claude/worktrees/merge-wave` with unpushed model-registry
commits — NEVER branch from local develop; branch from origin/develop and
cherry-pick, delete stale .next if typecheck fails on phantom routes).

## Round 5+ backlog (documented, not planned)
- idToPos schema-v20 cutover (008-DESIGN-OUTPUT §6; after dependency-closure
  lands; 40-50% per-cell Yrs cost).
- Binary wire format for bulk range reads (audit A1: classify.rs only
  fast-paths Vec<u8>; query_range et al serde_json per call). L.
- Async/worker NAPI (audit B1) — L/HIGH risk.
- Mirror pos_to_id/id_to_pos bimap merge (audit C1) — L, correctness-critical.
- Excel session capacity config: BIG_PARSE_CONCURRENCY sized to measured peak
  (2/replica at 24GB), global admission lease in Postgres ONLY if staging
  shows placement imbalance (no Redis — none exists in the stack).
- Server adoption of fork native build: napi .node built+measured on darwin;
  Railway needs linux build in the Docker image; vendoring pattern TBD
  (platform-package override like @mog-sdk/darwin-arm64).

## Standing rules (bite you if forgotten)
- Sapiex git: `gub` identity, gh = buildwithrohith. Never push mog work to
  origin (fundamental-research-labs) — only `fork` remote.
- plans/ IS tracked in the fork (001-019 + README are in the tree; earlier
  "plans/ is untracked" note was stale). Lanes may EDIT plans/README.md but
  never run git in the main repo (round 4: lane O committed to sapiex-patches
  directly; reset and recommitted by advisor).
- Known-baseline test failures: NONE anymore (fill-color pair fixed by 009).
- macOS RSS lies about MADV_FREE pages: use /usr/bin/footprint for retention
  claims. jemalloc, not system allocator, in compute/napi (native only —
  verify wasm unaffected via cargo tree).
- Codex lanes: luna max = workhorse; sol medium = judgment/hard; never raise
  sol effort. Panes need ~20s boot before send; "Sent N bytes" ≠ submitted —
  verify pickup, bare --enter if text sits in the input line.
