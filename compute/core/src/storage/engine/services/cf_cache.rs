//! Extracted CF cache service functions.
//!
//! Handles re-evaluation of conditional formatting rules and cache management.
//! The original methods on `YrsComputeEngine` delegate to these free functions.

use std::collections::HashMap;

use crate::mirror::CellMirror;
use crate::snapshot::RecalcResult;
use crate::storage::engine::CFCacheEntry;
use crate::storage::engine::cf_cache::convert_cf_formats_to_rules;
use crate::storage::engine::stores::EngineStores;
use crate::storage::sheet::cf_store;
use cell_types::{CellId, SheetId};
use rustc_hash::{FxHashMap, FxHashSet};

#[cfg(test)]
use std::cell::Cell;

#[cfg(test)]
thread_local! {
    static REFRESH_CF_CACHE_CALLS: Cell<usize> = Cell::new(0);
}

#[cfg(test)]
pub(crate) fn reset_refresh_cf_cache_calls_for_tests() {
    REFRESH_CF_CACHE_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
pub(crate) fn refresh_cf_cache_calls_for_tests() -> usize {
    REFRESH_CF_CACHE_CALLS.with(Cell::get)
}

/// After a recalculation pass, refresh the CF cache for every sheet that
/// both (a) has conditional formatting rules and (b) had at least one cell
/// change in the recalc result.
///
/// Returns, per sheet, the `(row, col)` pairs of cells whose CF result
/// changed but were **not** already in `recalc.changed_cells`. These are
/// "sibling" cells — e.g. the other member of a Duplicate-Values pair, or
/// the previous Top-N entry that got displaced — whose viewport entries
/// must be patched even though their cell value didn't change.
pub(in crate::storage::engine) fn refresh_cf_caches_after_recalc(
    stores: &mut EngineStores,
    mirror: &CellMirror,
    theme_palette: &HashMap<String, String>,
    recalc: &RecalcResult,
) -> FxHashMap<SheetId, Vec<(u32, u32)>> {
    if stores.cf_cache.is_empty() {
        return FxHashMap::default();
    }

    // Collect unique sheet IDs from changed cells that have CF rules
    let mut affected_sheets: FxHashSet<SheetId> = FxHashSet::default();
    for change in &recalc.changed_cells {
        if let Ok(sid) = SheetId::from_uuid_str(&change.sheet_id)
            && stores.cf_cache.contains_key(&sid)
        {
            affected_sheets.insert(sid);
        }
    }

    // Also check projection changes (dynamic array spills)
    for proj in &recalc.projection_changes {
        if let Ok(sid) = SheetId::from_uuid_str(&proj.sheet_id)
            && stores.cf_cache.contains_key(&sid)
        {
            affected_sheets.insert(sid);
        }
    }

    if affected_sheets.is_empty() {
        return FxHashMap::default();
    }

    // Build a set of positions that are already covered by recalc.changed_cells
    // so we don't double-patch them.
    let mut already_changed: FxHashSet<(SheetId, u32, u32)> = FxHashSet::default();
    for change in &recalc.changed_cells {
        if let (Ok(sid), Some(pos)) = (SheetId::from_uuid_str(&change.sheet_id), &change.position) {
            already_changed.insert((sid, pos.row, pos.col));
        }
    }

    let mut cf_only_changes: FxHashMap<SheetId, Vec<(u32, u32)>> = FxHashMap::default();

    for sheet_id in &affected_sheets {
        // Take ownership of the old map before refresh rebuilds the cache.
        // This avoids cloning every CellCFResult while also releasing the
        // mutable cache borrow before the evaluator runs.
        let old_results: FxHashMap<(u32, u32), crate::cf::types::CellCFResult> = stores
            .cf_cache
            .get_mut(sheet_id)
            .map(|e| std::mem::take(&mut e.results))
            .unwrap_or_default();

        refresh_cf_cache(stores, mirror, theme_palette, sheet_id);

        // Borrow the rebuilt map for comparison; the old map is owned above,
        // so there is no borrow overlap to work around with a clone.
        let new_results = stores.cf_cache.get(sheet_id).map(|e| &e.results);

        let changed = diff_cf_results(sheet_id, &old_results, new_results, &already_changed);

        if !changed.is_empty() {
            cf_only_changes.insert(*sheet_id, changed);
        }
    }

    cf_only_changes
}

/// Return CF-result positions whose rendered result changed, disappeared, or
/// appeared, excluding positions already covered by the value recalc.
///
/// The old map is owned by the caller and the new map is borrowed from the
/// rebuilt cache. Keeping this comparison separate makes the ownership change
/// easy to test without constructing a full engine or evaluator.
fn diff_cf_results(
    sheet_id: &SheetId,
    old_results: &FxHashMap<(u32, u32), crate::cf::types::CellCFResult>,
    new_results: Option<&FxHashMap<(u32, u32), crate::cf::types::CellCFResult>>,
    already_changed: &FxHashSet<(SheetId, u32, u32)>,
) -> Vec<(u32, u32)> {
    let mut changed = Vec::new();

    // Cells that were in old CF but their result changed or they left CF.
    for (&pos, old_result) in old_results {
        if already_changed.contains(&(*sheet_id, pos.0, pos.1)) {
            continue;
        }
        match new_results.and_then(|results| results.get(&pos)) {
            Some(new_result) if new_result == old_result => {} // unchanged
            _ => changed.push(pos),                            // lost or changed
        }
    }

    // Cells that are newly in the CF results (gained CF coloring).
    for &pos in new_results.into_iter().flat_map(|results| results.keys()) {
        if already_changed.contains(&(*sheet_id, pos.0, pos.1)) {
            continue;
        }
        if !old_results.contains_key(&pos) {
            changed.push(pos);
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cf::types::{CellCFResult, CfRenderStyle};

    fn result(row: u32, col: u32, bold: Option<bool>) -> CellCFResult {
        CellCFResult {
            row,
            col,
            style: bold.map(|bold| CfRenderStyle {
                bold: Some(bold),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn diff_cf_results_covers_changed_left_and_entered_cells() {
        let sheet_id =
            SheetId::from_uuid_str("71000000-0000-4000-8000-000000000001").expect("valid sheet id");
        let mut old_results = FxHashMap::default();
        old_results.insert((0, 0), result(0, 0, Some(true)));
        old_results.insert((0, 1), result(0, 1, Some(false)));
        old_results.insert((0, 2), result(0, 2, Some(true)));
        old_results.insert((0, 4), result(0, 4, Some(false)));

        let mut new_results = FxHashMap::default();
        new_results.insert((0, 0), result(0, 0, Some(true))); // unchanged
        new_results.insert((0, 1), result(0, 1, Some(true))); // changed
        new_results.insert((0, 3), result(0, 3, Some(true))); // entered
        new_results.insert((0, 4), result(0, 4, Some(true))); // value-changed cell

        let mut already_changed = FxHashSet::default();
        already_changed.insert((sheet_id, 0, 4));

        let mut changed = diff_cf_results(
            &sheet_id,
            &old_results,
            Some(&new_results),
            &already_changed,
        );
        changed.sort_unstable();

        assert_eq!(changed, vec![(0, 1), (0, 2), (0, 3)]);
    }
}

/// Re-evaluate all conditional formatting rules for a sheet and update the cache.
///
/// Pipeline: read CF formats from Yrs storage -> convert domain types to
/// compute-cf rules -> evaluate via `ComputeCore::eval_cf` -> store results
/// in `cf_cache` keyed by `(row, col)`.
pub(in crate::storage::engine) fn refresh_cf_cache(
    stores: &mut EngineStores,
    mirror: &CellMirror,
    theme_palette: &HashMap<String, String>,
    sheet_id: &SheetId,
) {
    #[cfg(test)]
    REFRESH_CF_CACHE_CALLS.with(|calls| calls.set(calls.get() + 1));

    // 1. Read CF formats from Yrs storage
    let formats = cf_store::get_formats_for_sheet(
        stores.storage.doc(),
        &stores.storage.sheets_ref(),
        sheet_id,
    );

    // 2. Convert domain types to evaluation types.
    //    Pass a resolver closure that resolves CellId UUID strings to (row, col)
    //    positions via the CellMirror.
    let rules = convert_cf_formats_to_rules(
        &formats,
        |sheet_id_str, cell_id_str| {
            let sid = SheetId::from_uuid_str(sheet_id_str).ok()?;
            let cid = CellId::from_uuid_str(cell_id_str).ok()?;
            let sheet = mirror.get_sheet(&sid)?;
            let pos = sheet.position_of(&cid)?;
            Some((pos.row(), pos.col()))
        },
        Some(*sheet_id),
        theme_palette,
    );

    // 3. If no rules, remove cache entry and return
    if rules.is_empty() {
        stores.cf_cache.remove(sheet_id);
        return;
    }

    // 4. Evaluate CF rules
    let results = stores.compute.eval_cf(mirror, sheet_id, &rules);

    // 5. Convert Vec<CellCFResult> to HashMap keyed by (row, col)
    let mut result_map = FxHashMap::default();
    for result in results {
        result_map.insert((result.row, result.col), result);
    }

    // 6. Store in cache
    stores.cf_cache.insert(
        *sheet_id,
        CFCacheEntry {
            results: result_map,
            dirty: false,
        },
    );
}
