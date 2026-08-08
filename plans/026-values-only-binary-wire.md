# Plan 026: Identity-free binary wire variant for value/format reads

> **Drift check**: base `b95d85be`;
> `git diff --stat b95d85be..HEAD -- compute/core/src/storage/engine/range_binary.rs kernel/src/bridges/` empty in YOUR worktree.

## Status
P2 / M / Risk LOW-MED. Planned at `b95d85be`. Round-6 wrap-up item.

## Why (measured evidence from round 5)
Lane R's byte breakdown of a pure-numeric 10k-cell `query_range_binary`
payload: 260,047 bytes total, of which **UUIDs 170,000 (65%)**, f64 values
80,000, masks 10,000, header+counts+type-runs 39. The viewer's dominant read
paths (`getRangeValues2d`, `getRangeFormats2d` — see the round-5 overrides in
`kernel/src/bridges/compute/compute-bridge.ts:568-630`) do NOT consume cell
identities; they reshape values/formats only. Dropping identities from THOSE
paths lands ~90,000/899,407 ≈ 10% of the JSON baseline.

## The change
1. Read `compute/core/src/storage/engine/range_binary.rs` (the round-5 codec:
   rectangle header, RLE type stream, varint inline formats — keep its
   conventions and its version-byte discipline) and the existing bridged
   methods behind `getRangeValues2d`/`getRangeFormats2d` (they already have
   binary siblings from round 5 — find them in compute-bridge.gen.ts).
2. Determine whether those two siblings ALREADY omit identities. If they do,
   measure their byte size on the 10k-numeric fixture and check whether the
   65%-UUID overhead applies only to `query_range_binary`. If values/formats
   paths are already identity-free, the remaining work is ONLY switching any
   remaining identity-carrying viewer read that doesn't need identity — list
   them from compute-bridge.ts and the spreadsheet-app dist (read-only grep)
   and STOP with a report if none exist (the plan may be already satisfied;
   do not invent work).
3. If `query_range_binary` (used by validation/metadata paths) is the only
   identity-carrying bulk read actually hit per viewport, add a sibling
   `query_range_values_binary` that omits the UUID section (bump the codec
   version byte or add a flags bit in the header — self-describing), decode
   in `kernel/src/bridges/wire/range-binary.ts`, regenerate the bridge
   (`pnpm run generate:bridge`), and route the callers that don't need
   identity (cell-metadata-cache validation population is the known one:
   `kernel/src/bridges/wire/cell-metadata-cache.ts:454-478` consumes only
   row/col).
4. Byte-size proof in a Rust test (print sizes; quote in the commit message):
   values-only 10k-numeric payload vs JSON sibling — target <=12%.

## Verify
`cargo test -p compute-core --lib` 0 failures (baseline at b95d85be: 3,060 +
1 ignored); wire Jest suite (`cd kernel && NODE_OPTIONS="--experimental-vm-modules --disable-warning=ExperimentalWarning" npx jest src/bridges/wire`)
all passing (baseline 321); bridge regen byte-stable after commit;
round-trip test on a mixed fixture INCLUDING the round-5 regression payload
(fixtures dir in kernel/src/bridges/wire/__tests__/fixtures/).

## Scope
**In**: range_binary.rs, kernel/src/bridges/wire/**, compute-bridge.ts (+gen),
cell-metadata-cache.ts caller routing, tests. **Out**: snapshot formats,
dense viewport buffer paths, macro crates, anything touching identities'
SEMANTICS (this is a wire-shape change only).

## Git workflow
Worktree /Users/vish/Repos/analyst/mog-opt-t, branch opt/values-only-wire,
commit per step, prefix `sapiex-patches:`, no push, never run git in the main
repo; only external writable file: plans/README.md (add row 026).

## STOP conditions
- Step 2 finds the values/formats paths already identity-free AND no
  identity-carrying caller that can drop identity — report; done is done.
- Two failed attempts per step.
