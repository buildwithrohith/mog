# Plan 018: Native allocator — return transient import memory to the OS

> **Executor instructions**: steps in order. Update row 018 in
> `/Users/vish/Repos/analyst/mog/plans/README.md` when done.
>
> **Drift check**: base `984efd41`;
> `git diff --stat 984efd41..HEAD -- compute/napi/` empty in your worktree.

## Status
P1 (server capacity) / S-M / Risk LOW-MED / Planned at `984efd41`. NATIVE
BUILD ONLY — must not affect the wasm target.

## Why: measured evidence

Native full import of the 19.4MB / 1.4M-cell repro: RSS peaks ~13.2GB.
Dropping the entire engine returns RSS only to 10.4GB; a second import in
the same process adds just +2.9GB. Conclusion: live data ≈ 2.9GB; ~10GB is
transient parse churn RETAINED by the system allocator as free pages never
returned to the OS. On a 24GB server replica this makes one workbook look
like 13GB instead of ~3GB — capacity is allocator behavior, not data.
(Plan 016 shrinks the churn itself; this plan makes whatever churn remains
returnable.)

## The change

Adopt **mimalloc** as the global allocator for the NAPI crate only, tuned
to return memory:

1. Check workspace deps first (`grep -rn "mimalloc\|jemalloc" Cargo.toml
   compute/*/Cargo.toml`). If a preferred allocator is already present
   anywhere, use that one.
2. In `compute/napi/Cargo.toml`: add `mimalloc` (default features; no
   `secure`) as a NON-wasm dependency of the napi crate only.
3. In `compute/napi/src/lib.rs`:
   ```rust
   #[global_allocator]
   static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
   ```
4. Configure eager page return: mimalloc respects env/option
   `mi_option_purge_delay` (v2 naming; check the crate's docs.rs for the
   exposed API — if the crate exposes no options API, set
   `MIMALLOC_PURGE_DELAY=0` via an env default documented in the commit and
   README row; verify the default behavior empirically in step V2 — recent
   mimalloc purges eagerly enough that defaults may already pass).
5. Confirm `compute/wasm` does NOT pull the napi crate (it doesn't — wasm
   uses its own crate; verify with `cargo tree -p compute-core-wasm | grep -i mimalloc`
   → empty).

## Verification

- V1: `cargo check --workspace --locked` exit 0;
  `pnpm --dir compute/napi run build:release` succeeds;
  `node compute/napi/smoke-test.mjs` all pass.
- V2 (the acceptance measurement): re-run the double-import diagnostic
  (script shape: import repro194 → drop engine → gc + settle 3s → measure
  RSS). PASS = post-drop RSS ≤ 4GB (vs 10.4GB today). Record the numbers in
  your report. The fixture lives at
  `/Users/vish/Repos/analyst/sapiex/apps/web/public/dev-fixtures/repro194.xlsx`
  (READ-ONLY — do not modify anything in that repo).
- V3: wasm unaffected: `cargo tree -p compute-core-wasm 2>/dev/null | grep -ci mimalloc` → 0,
  and `bash compute/wasm/build.sh --profile release` still succeeds.
- V4: full `cargo test -p compute-core --lib` → 0 failures (allocator swap
  must not surface latent bugs; if it DOES surface one, that's a real
  pre-existing bug — STOP and report it precisely).

## Scope
**In**: `compute/napi/Cargo.toml`, `compute/napi/src/lib.rs`, workspace
`Cargo.lock`. **Out**: wasm build, any engine code, any Sapiex repo file.

## Git workflow
Worktree `/Users/vish/Repos/analyst/mog-opt-k`, branch `opt/native-allocator`,
prefix `sapiex-patches:`, no push.

## Done criteria
- [ ] V1-V4 all pass; post-drop RSS measured ≤4GB and reported
- [ ] wasm build proven allocator-unchanged
- [ ] README row 018 updated with the measured before/after

## STOP conditions
- mimalloc (or chosen allocator) fails to build for any workspace target —
  report, do not force-feature-gate beyond the napi crate.
- V2 improves by less than 3GB — report numbers; the advisor decides
  whether to keep it.
