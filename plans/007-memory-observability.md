# Plan 007: Memory observability — wasm heap sampling, update-source tags, type-size report

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 60b6ee3e..HEAD -- infra/transport/src/ compute/core/src/storage/engine/update_buffer.rs`
> On any change since 60b6ee3e, compare excerpts before proceeding.

## Status

- **Priority**: P2 (unblocks P1-tier decisions)
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: dx / perf-observability
- **Planned at**: commit `60b6ee3e`, 2026-08-06

## Why this matters

Three measurement gaps currently force guesswork on the largest memory
decisions in this engine:

1. **Wasm linear memory is only ever read AFTER a crash** — the sole
   consumer of `globalThis.__sapiexWasmMemory` is the trap handler's
   `console.error`. There is no way to watch a session approach the 4GiB
   wasm32 cliff.
2. **Every Yrs update payload is tagged `UserMutation`** — the
   `UpdateSource` enum (`ImportBootstrap`, `FullHydration`,
   `InternalRebuild`) is dead code, so the 64MB "bootstrap leak" guardrail
   reports the wrong subsystem when it fires.
3. **No measured struct sizes exist** for the per-cell types — the audit's
   ~500-600B/cell `CellData` estimate (→ ~700-840MB at 1.4M cells) is
   layout-derived, unmeasured. A `size_of` report test converts three L-tier
   "maybe" findings (CellData shrink, mirror double-store, Yrs per-cell cost)
   into numbers.

## Current state

Verified at `60b6ee3e`:

`infra/transport/src/wasm-loader.ts:117-120` sets
`globalThis.__sapiexWasmMemory = wasmExports.memory`. Only consumer:
`infra/transport/src/wasm-transport.ts` (~:70-115) reads
`mem.buffer.byteLength` inside the trap `catch` and `console.error`s it.

`compute/core/src/storage/engine/update_buffer.rs`:
```rust
// :96-98 — hardcoded source:
    pub(crate) fn push(&self, update: Vec<u8>) {
        self.push_with_source(UpdateSource::UserMutation, update);
    }
// :202-210 — the observer that feeds it:
pub(crate) fn install_observer(doc: &yrs::Doc, buffer: &Arc<UpdateBuffer>) -> ... {
    let buffer = Arc::clone(buffer);
    compute_collab::subscribe_update_v1(doc, move |bytes| { buffer.push(bytes.to_vec()); })
}
```
`UpdateSource` variants sit behind `#[allow(dead_code)]` (~:49-56).
`install_observer` call sites: engine construction (assemble), and
`construction/deferred.rs` (~:286 fast path, ~:921 commit) — grep
`install_observer(` to enumerate.

Existing size-measurement precedent:
`compute/core/tests/range_memory_budget.rs` measures `CellValue`/`CellEntry`
— model the new report test on its style.

Per-cell parse struct: `domain-types/src/parse_output.rs:1611-1686`
(`CellData`, 18 fields incl. embedded `FormulaCacheProvenance` at :98-130).

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Rust typecheck | `cargo check --workspace --locked` | exit 0 |
| Buffer tests | `cargo test -p compute-core --lib update_buffer` | all pass |
| New size test | `cargo test -p compute-core --test type_size_report -- --nocapture` | passes, prints table |
| TS install | `pnpm install` | exit 0 |
| Transport typecheck/build | `pnpm --filter @mog-sdk/transport typecheck 2>/dev/null \|\| pnpm --filter ./infra/transport typecheck` (find the actual package name in `infra/transport/package.json`) | exit 0 |

## Scope

**In scope**:
- `infra/transport/src/wasm-transport.ts` (+ a small export from
  `wasm-loader.ts` if cleaner)
- `compute/core/src/storage/engine/update_buffer.rs`
- The `install_observer` call sites (mechanical: pass a source)
- NEW `compute/core/tests/type_size_report.rs`

**Out of scope**:
- Acting on any measurement (no CellData refactor here)
- `construction/deferred.rs` beyond the one-argument observer change (plans
  001-004 own that file's logic — coordinate: your change there is limited
  to adding the source argument at existing `install_observer(` calls)
- kernel/, apps/

## Git workflow

Branch `opt/parse-intern-observability`, commit prefix `sapiex-patches:`,
do not push.

## Steps

### Step 1: Wasm memory sampling hook

In `wasm-transport.ts`, add and export:
```ts
export function wasmLinearMemoryBytes(): number | null {
  const mem = (globalThis as any).__sapiexWasmMemory;
  return mem?.buffer?.byteLength ?? null;
}
```
Then, in the transport's command dispatch path (the function that resolves
`wasm[command]` — around the existing trap handling): every 250 commands OR
whenever the reading has grown ≥256MB since the last log, emit
`console.info('[mog-wasm] linear memory', { bytes, deltaBytes, command })`.
Keep state in module-level `let` variables; zero overhead otherwise. Match
the file's existing logging style (it already console.errors with structured
objects).

**Verify**: transport package typecheck → exit 0.

### Step 2: Thread UpdateSource through the observer

Change `install_observer(doc, buffer)` to
`install_observer(doc, buffer, source: UpdateSource)`; the closure calls
`buffer.push_with_source(source, bytes.to_vec())`. Remove `#[allow(dead_code)]`
from the variants now used. Update call sites: engine construction →
`ImportBootstrap` at initial assembly IF that observer is installed before
first user edit and re-used after (read the assemble path; if one observer
serves the whole life, keep `UserMutation` there and instead tag the two
deferred.rs installs: fast-path install → `ImportBootstrap`, post-commit
reinstall → `FullHydration`). Keep `push` as a thin
`UserMutation` wrapper for any remaining non-observer callers (grep `\.push(`).

**Verify**: `cargo test -p compute-core --lib update_buffer` → all pass
(existing tests already use `push_with_source`); `cargo check` exit 0.

### Step 3: Type-size report test

New `compute/core/tests/type_size_report.rs`: with `--nocapture`, print a
table of `std::mem::size_of` for: `domain_types::parse_output::CellData`,
`domain_types::parse_output::FormulaCacheProvenance`, the mirror's
`CellEntry` and `CellValue` (import paths per `range_memory_budget.rs`), and
`yrs`-independent id types (`CellId`). Assert only GENEROUS ceilings
(e.g. `assert!(size_of::<CellData>() < 2048)`) so the test documents rather
than gates; the printed table is the deliverable. Add one line computing
`size_of::<CellData>() * 1_407_921` printed as MB, labeled "fixed parse
overhead at Cadence scale".

**Verify**: `cargo test -p compute-core --test type_size_report -- --nocapture`
→ passes; paste the printed table into your completion report.

## Done criteria

- [ ] `cargo check --workspace --locked` exits 0
- [ ] `cargo test -p compute-core --lib` exits 0
- [ ] `grep -n "dead_code" compute/core/src/storage/engine/update_buffer.rs` → no longer covers used variants
- [ ] Transport typecheck exits 0; sampling emits at the stated cadence (unit-testable by exporting the internal shouldLog fn or by threshold math test)
- [ ] Size report test prints the table (include it in your report)
- [ ] No files outside Scope modified; README row updated

## STOP conditions

- The transport has no single dispatch chokepoint for command counting.
- `install_observer`'s construction-time call serves the entire engine life
  AND the deferred installs are unreachable — report the actual topology.
- Persistent failure after two attempts.

## Maintenance notes

- The printed size table feeds three held plans (CellData shrink, mirror
  double-store, Yrs per-cell) — whoever picks those up starts from this
  test's output.
- Consider (follow-up, not now) surfacing `wasmLinearMemoryBytes()` through
  the SDK so the consuming app can show a memory badge.
