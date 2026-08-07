# Plan 012: CellValue shrink — box the Image variant (56B → ~24B everywhere)

> **Executor instructions**: Follow step by step; verify each step. On any
> STOP condition, stop and report. Update your row in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check (run first)**: base `80f56679`.
> `git diff --stat 80f56679..HEAD -- compute/core/crates/types/value-types/`
> in YOUR worktree must be empty before starting.

## Status

- **Priority**: P1 (multiplies across every store in the engine)
- **Effort**: S-M
- **Risk**: LOW (rare variant; compiler-enumerated call sites)
- **Depends on**: none (independent of 009/010/011 — different files)
- **Category**: perf/memory
- **Planned at**: commit `80f56679`, 2026-08-06

## Why this matters

`type_size_report` measures `value_types::CellValue` at **56 bytes**. The
enum's own doc comments show the design intent is cheap-to-clone, small
values (`Text(Arc<str>)`, `Array(Arc<CellArray>)` both Arc'd for O(1)
clone). The outlier is:

`compute/core/crates/types/value-types/src/cell_value/mod.rs` (~:62-88):
```rust
pub enum CellValue {
    Number(FiniteF64),
    Text(Arc<str>),
    Boolean(bool),
    Error(CellError, Option<Arc<str>>),
    Null,
    Array(Arc<CellArray>),
    Control(CellControl),      // 3 bytes — fine
    Image(CellImage),          // <-- INLINE ~48 bytes, sets the enum size
}
```
`CellImage` (`value-types/src/cell_image.rs:24`): `source: Arc<str>` (16) +
`alt_text: Option<Arc<str>>` (8, niche) + `sizing` + `height/width:
Option<u32>` (2×8) ≈ 48 bytes, paid by EVERY CellValue slot in the engine
even though image cells only exist as `IMAGE()` formula results.

After `Image(Arc<CellImage>)`, the largest payload is `Text`/`Error` at 16B
→ whole enum ~24 bytes. This multiplies:

- Mirror `cells` map: `CellEntry` measured 64B → ~32B (value + boxed formula)
- Mirror `col_data`: `Vec<CellValue>` DENSE per column, padded with
  `CellValue::Null` to the used row extent — every slot including padding
  drops 56→24 (the padding win costs nothing anywhere else)
- Retained parse output: `CellData.value` (1.4M × 32B saved at repro scale)
- Every evaluator clone: the parallel demand evaluator clones cell values on
  read (the enum's own doc comment); smaller enum = less memcpy per clone

Use `Arc<CellImage>` (not `Box`) to match the enum's O(1)-clone convention —
the evaluator clones values constantly and a Box would deep-copy.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Typecheck | `cargo check --workspace --locked` | exit 0 |
| Size proof | `cargo test -p compute-core --test type_size_report -- --nocapture` | CellValue printed ≤ 32 (expect 24); CellEntry ≤ 40; update ceiling asserts |
| Value-types unit tests | `cargo test -p value-types` (confirm package name in its Cargo.toml) | all pass |
| Image/control features | `grep -rn "CellValue::Image" compute/core/src --include=*.rs -l` then run the owning test modules (expect an `image` fn/feature module; e.g. `cargo test -p compute-core --lib image`) | all pass |
| Range/memory budget | `cargo test -p compute-core --test range_memory_budget` | passes (its budget asserts may now be loose — tighten only if the test itself instructs) |
| Full engine + parser | `cargo test -p compute-core --lib` and `cargo test -p xlsx-parser -- --test-threads=1` | zero failures (post-009) or documented baselines only |

## Scope

**In scope**: `value-types/src/cell_value/mod.rs` (the variant),
`cell_image.rs` if helpers need it, every compile-error site
(constructions `CellValue::Image(img)` → `CellValue::Image(Arc::new(img))`
or a `CellValue::image(...)` helper; matches `Image(img)` → deref patterns),
`type_size_report.rs` ceilings, serde attributes if the enum derives them
(check: wire format must stay IDENTICAL — `Arc<T>` serializes as `T`, so
derived serde output is unchanged; verify with an existing serde test or add
a one-liner).

**Out of scope**: `Control` (already 3B); `Error`'s message field;
`CellArray` internals; any TS/wasm bridge type (the bridge serializes
values, layout is Rust-internal — but VERIFY the bridge doesn't do raw
transmute/size assumptions: grep `size_of` usages of CellValue outside the
report test; expect none).

## Git workflow

Worktree `/Users/vish/Repos/analyst/mog-opt-f`, branch `opt/cellvalue-shrink`
(created for you), commit prefix `sapiex-patches:`, do not push.

## Steps

1. Change the variant to `Arc<CellImage>`; add a `CellValue::image(img: CellImage) -> Self`
   constructor helper; fix all compile errors (constructions and matches).
   **Verify**: cargo check exit 0.
2. Serde-format proof: find or write one test serializing a
   `CellValue::Image` and assert the JSON is identical pre/post (Arc is
   transparent in serde) — if the enum has no serde derives, note that and
   skip.
3. Update `type_size_report` ceilings; run the full command table.
   **Verify**: CellValue ≤ 32 printed; all suites green.

## Done criteria

- [ ] `size_of::<CellValue>()` ≤ 32 (expect 24), printed in report
- [ ] `size_of::<CellEntry>()` correspondingly down, printed
- [ ] Serde wire format proven unchanged (or proven absent)
- [ ] No `size_of::<CellValue>()` assumptions elsewhere (grep clean)
- [ ] Full suites at expected outcomes; README row updated

## STOP conditions

- Any code transmutes/assumes CellValue layout (FFI, bridge, unsafe) —
  report the site, do not work around.
- The niche optimization interacts badly (e.g. some wrapper's size balloons
  instead of shrinking — check `Option<CellValue>` in the report) — report
  numbers.
- Persistent failure after two attempts.

## Maintenance notes

- Reviewers: confirm no hot path constructs images in a loop (Arc::new per
  construction is fine for IMAGE() results, catastrophic in a per-cell loop
  — expect none).
- Follow-up candidate if this lands well: `Error(CellError, Option<Arc<str>>)`
  could collapse to a single `Arc` payload, but 24B is already at the
  Text floor, so likely not worth it — note only.
