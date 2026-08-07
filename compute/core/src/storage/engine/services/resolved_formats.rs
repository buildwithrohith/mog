use crate::mirror::CellMirror;
use crate::storage::engine::settings::EngineSettings;
use crate::storage::engine::stores::EngineStores;
use crate::storage::engine::viewport;
use cell_types::{SheetId, SheetPos};
use compute_document::hex::id_to_hex;
use domain_types::CellFormat;
use std::collections::HashMap;
use yrs::Transact;

pub(in crate::storage::engine) fn get_resolved_cell_format(
    stores: &EngineStores,
    mirror: &CellMirror,
    settings: &EngineSettings,
    sheet_id: &SheetId,
    row: u32,
    col: u32,
) -> CellFormat {
    let grid_index = stores.grid_indexes.get(sheet_id);
    let cell_id = grid_index
        .and_then(|grid| grid.cell_id_at(row, col))
        .or_else(|| mirror.resolve_cell_id(sheet_id, SheetPos::new(row, col)));

    let mut format = if let Some(cell_id) = cell_id {
        let cell_hex = id_to_hex(cell_id.as_u128());
        let table_format = super::resolve_structured_format_at_cell(mirror, sheet_id, row, col);
        crate::storage::properties::get_effective_format(
            &stores.storage,
            sheet_id,
            &cell_hex,
            row,
            col,
            table_format.as_ref(),
            grid_index,
            mirror.get_sheet(sheet_id),
        )
    } else {
        crate::storage::properties::get_positional_format(
            &stores.storage,
            sheet_id,
            row,
            col,
            grid_index,
            mirror.get_sheet(sheet_id),
        )
    };

    domain_types::theme_color::resolve_theme_refs(&mut format, &settings.theme_palette);

    if let Some(cache_entry) = stores.cf_cache.get(sheet_id)
        && let Some(cf_result) = cache_entry.results.get(&(row, col))
    {
        viewport::merge_cf_into_format(&mut format, cf_result);
    }

    format
}

/// Resolve formats for selected positions using one set of preloaded format
/// layers per sheet. The returned formats are owned so the caller can retain
/// them for the duration of an enrichment pass without holding Yrs data.
pub(in crate::storage::engine) fn get_resolved_cell_formats(
    stores: &EngineStores,
    mirror: &CellMirror,
    settings: &EngineSettings,
    sheet_id: &SheetId,
    positions: &[(u32, u32)],
) -> HashMap<(u32, u32), CellFormat> {
    let grid_index = stores.grid_indexes.get(sheet_id);
    let mut cell_ids = positions
        .iter()
        .filter_map(|&(row, col)| {
            grid_index
                .and_then(|grid| grid.cell_id_at(row, col))
                .or_else(|| mirror.resolve_cell_id(sheet_id, SheetPos::new(row, col)))
        })
        .collect::<Vec<_>>();
    cell_ids.sort_unstable_by_key(|cell_id| cell_id.as_u128());
    cell_ids.dedup();

    let txn = stores.storage.doc().transact();
    let cell_formats = crate::storage::properties::get_cell_format_layers_for_ids_with_txn(
        stores.storage.workbook_map(),
        stores.storage.sheets(),
        sheet_id,
        &cell_ids,
        &txn,
    );
    let row_formats = crate::storage::properties::get_all_row_formats_with_txn(
        stores.storage.sheets(),
        sheet_id,
        grid_index,
        &txn,
    )
    .into_iter()
    .filter_map(|entry| entry.format.map(|format| (entry.row, format)))
    .collect::<HashMap<_, _>>();
    let col_formats = crate::storage::properties::get_all_col_formats_with_txn(
        stores.storage.sheets(),
        sheet_id,
        grid_index,
        &txn,
    )
    .into_iter()
    .filter_map(|entry| entry.format.map(|format| (entry.col, format)))
    .collect::<HashMap<_, _>>();
    let base_format =
        crate::storage::properties::get_workbook_base_format_with_txn(&stores.storage, &txn);
    drop(txn);

    positions
        .iter()
        .copied()
        .map(|(row, col)| {
            let cell_id = grid_index
                .and_then(|grid| grid.cell_id_at(row, col))
                .or_else(|| mirror.resolve_cell_id(sheet_id, SheetPos::new(row, col)));
            let structured_format = cell_id
                .and_then(|_| super::resolve_structured_format_at_cell(mirror, sheet_id, row, col));
            let range_format = mirror.get_sheet(sheet_id).and_then(|sheet| {
                let mut merged: Option<CellFormat> = None;
                for (_id, format) in sheet.format_ranges_at(row, col) {
                    merged = Some(match merged.take() {
                        Some(previous) => {
                            crate::storage::properties::merge_formats(&previous, format)
                        }
                        None => format.clone(),
                    });
                }
                merged
            });
            let mut format =
                crate::storage::properties::get_effective_format_from_preloaded_layers_with_range(
                    &base_format,
                    col_formats.get(&col),
                    row_formats.get(&row),
                    col,
                    range_format.as_ref(),
                    structured_format.as_ref(),
                    cell_id.and_then(|id| cell_formats.get(&id)),
                    mirror.get_sheet(sheet_id),
                    cell_id.is_none(),
                );
            domain_types::theme_color::resolve_theme_refs(&mut format, &settings.theme_palette);
            if let Some(cache_entry) = stores.cf_cache.get(sheet_id)
                && let Some(cf_result) = cache_entry.results.get(&(row, col))
            {
                viewport::merge_cf_into_format(&mut format, cf_result);
            }
            ((row, col), format)
        })
        .collect()
}
