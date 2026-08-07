use cell_types::{CellId, SheetId, SheetPos};
use value_types::CellValue;

use crate::mirror::CellMirror;
use crate::snapshot::RecalcResult;
use crate::storage::engine::services::cell_editing::NO_OLD_FORMULA_SENTINEL;
use crate::storage::engine::services::resolved_formats;
use crate::storage::engine::settings::EngineSettings;
use crate::storage::engine::stores::EngineStores;
use crate::storage::sheet::{comments, hyperlinks, sparklines};
use compute_document::hex::hex_to_id;
use compute_wire::flags as render_flags;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// enrich_display_text
// ---------------------------------------------------------------------------

/// Populate `display_text` on each `CellChange` using the canonical
/// `format_value_at_cell` method.
pub(in crate::storage::engine) fn enrich_display_text(
    stores: &EngineStores,
    mirror: &CellMirror,
    settings: &EngineSettings,
    result: &mut RecalcResult,
    format_value_fn: &dyn Fn(&CellValue, &SheetId, u32, u32) -> String,
) {
    let mut format_cache: HashMap<SheetId, HashMap<(u32, u32), domain_types::CellFormat>> =
        HashMap::new();
    let positions_by_sheet = result
        .changed_cells
        .iter()
        .filter_map(|change| {
            let pos = change.position.as_ref()?;
            let sheet_id = SheetId::from_uuid_str(&change.sheet_id).ok()?;
            Some((sheet_id, (pos.row, pos.col)))
        })
        .fold(
            HashMap::<SheetId, Vec<(u32, u32)>>::new(),
            |mut acc, (sheet, pos)| {
                acc.entry(sheet).or_default().push(pos);
                acc
            },
        );
    for (sheet_id, positions) in positions_by_sheet {
        format_cache.insert(
            sheet_id.clone(),
            resolved_formats::get_resolved_cell_formats(
                stores, mirror, settings, &sheet_id, &positions,
            ),
        );
    }

    for change in &mut result.changed_cells {
        let Some(pos) = change.position.clone() else {
            continue;
        };
        let sheet_id = match SheetId::from_uuid_str(&change.sheet_id) {
            Ok(id) => id,
            Err(_) => continue,
        };
        let old_formula_known_absent =
            change.old_formula.as_deref() == Some(NO_OLD_FORMULA_SENTINEL);
        let had_before_formula_snapshot = change.old_formula.is_some();
        if change.old_display_text.is_none()
            && let Some(old_value) = &change.old_value
        {
            change.old_display_text = Some(format_value_fn(old_value, &sheet_id, pos.row, pos.col));
        }

        change.display_text = Some(format_value_fn(&change.value, &sheet_id, pos.row, pos.col));

        let effective_format = format_cache
            .get(&sheet_id)
            .and_then(|formats| formats.get(&(pos.row, pos.col)))
            .cloned()
            .unwrap_or_default();
        change.number_format = Some(
            effective_format
                .number_format
                .unwrap_or_else(|| "General".to_string()),
        );

        if let Ok(cell_id) = CellId::from_uuid_str(&change.cell_id) {
            change.new_formula = stores.compute.get_formula(&cell_id).map(str::to_owned);
            if old_formula_known_absent {
                change.old_formula = None;
            } else if change.old_formula.is_none()
                && !had_before_formula_snapshot
                && change.old_value.is_some()
                && change.new_formula.is_some()
            {
                change.old_formula = change.new_formula.clone();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// enrich_metadata_flags
// ---------------------------------------------------------------------------

/// Populate `extra_flags` on each `CellChange` with metadata flags
/// (HAS_FORMULA, HAS_COMMENT, HAS_SPARKLINE, HAS_HYPERLINK).
pub(in crate::storage::engine) fn enrich_metadata_flags(
    stores: &EngineStores,
    mirror: &CellMirror,
    recalc: &mut RecalcResult,
) {
    let mut comment_cache: std::collections::HashMap<SheetId, std::collections::HashSet<u128>> =
        std::collections::HashMap::new();
    let mut hyperlink_cache: HashMap<SheetId, HashMap<(u32, u32), Option<String>>> = HashMap::new();
    let positions_by_sheet = recalc
        .changed_cells
        .iter()
        .filter_map(|change| {
            let pos = change.position.as_ref()?;
            let sheet_id = SheetId::from_uuid_str(&change.sheet_id).ok()?;
            Some((sheet_id, (pos.row, pos.col)))
        })
        .fold(
            HashMap::<SheetId, Vec<(u32, u32)>>::new(),
            |mut acc, (sheet, pos)| {
                acc.entry(sheet).or_default().push(pos);
                acc
            },
        );
    for (sheet_id, positions) in positions_by_sheet {
        let cache = stores
            .grid_indexes
            .get(&sheet_id)
            .map(|grid| {
                hyperlinks::get_hyperlinks_for_positions(
                    stores.storage.doc(),
                    stores.storage.sheets(),
                    &sheet_id,
                    grid,
                    &positions,
                )
            })
            .unwrap_or_else(|| positions.into_iter().map(|pos| (pos, None)).collect());
        hyperlink_cache.insert(sheet_id, cache);
    }

    for change in &mut recalc.changed_cells {
        let Some(pos) = change.position.clone() else {
            continue;
        };

        let sheet_id = match SheetId::from_uuid_str(&change.sheet_id) {
            Ok(id) => id,
            Err(_) => continue,
        };
        let cell_id = match CellId::from_uuid_str(&change.cell_id) {
            Ok(id) => id,
            Err(_) => continue,
        };

        // --- HAS_FORMULA ---
        // Check three sources:
        // 1. Compute store has a formula for this cell's CellId (anchor cells).
        // 2. Mirror has a formula for the cell at this position (legacy CellId path).
        // 3. Projection registry: CSE members own the legacy array formula.
        //    Dynamic-array spill members are associated projected values, not
        //    formula owners, and are marked through IS_SPILL_MEMBER patches.
        let has_formula = stores.compute.get_formula(&cell_id).is_some()
            || mirror
                .get_sheet(&sheet_id)
                .and_then(|sheet| {
                    sheet
                        .cell_id_at(SheetPos::new(pos.row, pos.col))
                        .and_then(|cid| sheet.get_cell(&cid))
                })
                .is_some_and(|entry| entry.formula.is_some())
            || mirror
                .projection_registry
                .resolve(&sheet_id, pos.row, pos.col)
                .is_some_and(|(anchor_id, _, _)| mirror.is_cse_anchor(&anchor_id));

        if has_formula {
            change.extra_flags |= render_flags::HAS_FORMULA;
        }

        // --- HAS_COMMENT ---
        let comment_ids = comment_cache.entry(sheet_id).or_insert_with(|| {
            let cell_id_hexes = comments::get_cell_ids_with_comments(
                stores.storage.doc(),
                stores.storage.sheets(),
                &sheet_id,
            );
            cell_id_hexes
                .iter()
                .filter_map(|hex| hex_to_id(hex))
                .collect()
        });
        if comment_ids.contains(&cell_id.as_u128()) {
            change.extra_flags |= render_flags::HAS_COMMENT;
        }

        // --- HAS_SPARKLINE ---
        if sparklines::has_sparkline(
            stores.storage.doc(),
            &stores.storage.sheets_ref(),
            &sheet_id,
            pos.row,
            pos.col,
        ) {
            change.extra_flags |= render_flags::HAS_SPARKLINE;
        }

        // --- HAS_HYPERLINK ---
        if hyperlink_cache
            .get(&sheet_id)
            .and_then(|cache| cache.get(&(pos.row, pos.col)))
            .and_then(|url| url.as_ref())
            .is_some()
        {
            change.extra_flags |= render_flags::HAS_HYPERLINK;
        }
    }
}
