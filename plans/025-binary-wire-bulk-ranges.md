# Plan 025: Binary wire format for bulk range reads

> **Drift check**: base `c7f9f39c`;
> `git diff --stat c7f9f39c..HEAD -- compute/core/src/storage/engine/queries.rs infra/rust-bridge/ kernel/src/bridges/` empty in YOUR worktree.

## Status
P2 / L / Risk MED. Planned at `c7f9f39c`.

## Why (verified evidence)
The bridge codegen has a bytes fast path ONLY for `Vec<u8>` and
`(Vec<u8>, T)` return shapes (`infra/rust-bridge/bridge-napi/macros/src/classify.rs:42-46`);
everything else serde-serializes per call. Bulk reads —
`query_range` (queries.rs:874), `query_ranges` (queries.rs:900), and the 2d
value/format getters the viewer calls per viewport — pay JSON encode/decode
on potentially hundreds of thousands of cells per scroll.

## The change
1. Map the wire surface first: from kernel/src/bridges/compute usage, list
   which bulk read methods the spreadsheet-app/viewer actually calls per
   viewport render (grep the app dist is allowed read-only; the kernel bridge
   callers are the truth). Target the top 2-3 by data volume ONLY.
2. For each target add a sibling method returning `(Vec<u8>, MetaT)` using
   the EXISTING bytes-tuple fast path (classify.rs already supports it — no
   macro changes unless proven necessary): a compact little-endian layout
   (document it in a module doc comment; reuse the PayloadEncoding style
   from snapshot RangeData where types align). Keep the JSON siblings —
   external callers may use them (this is a fork; upstream diff surface
   matters — additive only).
3. Decode on the TS side in the kernel bridge (typed-array views, no per-cell
   object churn where the consumer allows), regenerate the bridge
   (`pnpm run generate:bridge`), and switch the viewer's calls to the binary
   siblings. If generation isn't reproducible, STOP (hand-editing .gen.ts is
   forbidden).

## Verify
`cargo test -p compute-core --lib` 0 failures; new Rust round-trip test per
encoding (encode → decode reference impl → equality with the JSON sibling's
output on a mixed-type fixture: numbers, strings, errors, empties, formats);
`pnpm --filter @mog-sdk/kernel typecheck`; kernel bridge tests. Perf proof:
a #[cfg(test)] micro-benchmark is NOT required — instead assert byte size:
binary payload for a 10k-cell numeric range must be <25% of the JSON
sibling's serialized size, printed in the test output and quoted in the
commit message.

## Scope
**In**: queries.rs (+ its module files), kernel/src/bridges/compute (+ gen),
spreadsheet-app call sites if needed for the viewer switch. **Out**: macro
crates (unless step 2's assumption fails — then STOP and report), wasm glue
internals, snapshot formats.

## Git workflow
Worktree /Users/vish/Repos/analyst/mog-opt-r, branch opt/binary-wire,
commit per step, prefix `sapiex-patches:`, no push. Only external writable
file: plans/README.md (row 025). Never run git in the main repo.

## STOP conditions
- The bytes-tuple fast path doesn't actually exist end-to-end for wasm (it
  is napi-named: verify the wasm bridge macro has the equivalent BEFORE
  writing encoders; if wasm lacks it, report — the viewer is the wasm
  consumer and this plan's value collapses).
- Two failed attempts per step.
