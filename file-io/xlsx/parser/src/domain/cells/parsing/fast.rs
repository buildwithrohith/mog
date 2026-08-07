use super::super::adapters::find_byte;
use super::super::helpers::{
    ScanResult, find_closing_tag_span, find_sheet_data_bounds, find_start_tag,
    looks_like_cell_start, parse_row_number, scan_cell, start_tag_at,
};
use super::super::types::{CellData, FastParseDiagnostics, ParseExtras};
use super::cell_extras::{CellExtrasInput, collect_cell_extras};
use super::formula_extras::collect_formula_extras;
use super::rows::apply_fast_row_attrs;
use ooxml_types::worksheet::RowHeight;

#[inline]
fn resync_after_malformed_cell(xml: &[u8], search_from: usize, limit: usize) -> Option<usize> {
    let mut next_cell = None;
    let mut candidate_from = search_from;
    while let Some(tag) = find_start_tag(xml, b"c", candidate_from) {
        if tag.lt >= limit {
            break;
        }
        if tag.tag_end < limit && !xml[tag.name_end..tag.tag_end].contains(&b'<') {
            next_cell = Some(tag.lt);
            break;
        }
        candidate_from = tag.lt.saturating_add(1);
    }
    let next_row = find_closing_tag_span(xml, b"row", search_from)
        .and_then(|tag| (tag.lt < limit && tag.end <= limit).then_some(tag.lt));

    match (next_cell, next_row) {
        (Some(cell), Some(row)) => Some(cell.min(row)),
        (Some(cell), None) => Some(cell),
        (None, Some(row)) => Some(row),
        (None, None) => None,
    }
}

pub(super) fn parse_worksheet_core(
    xml: &[u8],
    shared_strings: &[&str],
    cells: &mut [CellData],
    strings: &mut Vec<u8>,
    row_heights: &mut Vec<RowHeight>,
    mut extras: Option<&mut ParseExtras>,
    diagnostics: &mut FastParseDiagnostics,
    col_styles: &[Option<u32>],
) -> usize {
    let mut cell_idx = 0;
    let mut pos = 0;

    let sheet_data_bounds = match find_sheet_data_bounds(xml, pos) {
        Some(bounds) => bounds,
        None => return 0,
    };
    pos = sheet_data_bounds.content_start;

    let sheet_data_end = sheet_data_bounds.content_end;
    let mut current_row: u32 = 0;
    let mut current_row_style: Option<u32> = None;

    while pos < sheet_data_end && cell_idx < cells.len() {
        if let Some(tag_start) = find_byte(xml, b'<', pos) {
            if tag_start >= sheet_data_end {
                break;
            }
            pos = tag_start + 1;

            if let Some(row_tag) = start_tag_at(xml, tag_start, b"row") {
                if let Some(row_num) = parse_row_number(xml, row_tag.name_end) {
                    current_row = row_num.saturating_sub(1);
                }
                let applied = apply_fast_row_attrs(
                    &xml[tag_start..row_tag.tag_end],
                    current_row,
                    row_tag.is_self_closing,
                    row_heights,
                    extras.as_deref_mut(),
                );
                current_row_style = applied.row_style;
                pos = row_tag.content_start;
            } else if start_tag_at(xml, tag_start, b"c").is_some()
                || looks_like_cell_start(xml, tag_start)
            {
                let cell_start = tag_start;
                let ScanResult {
                    cell: cell_opt,
                    end: cell_end,
                    is_self_closing,
                    cm_val,
                    vm_val,
                    has_ph,
                    has_explicit_s,
                    has_xml_space_v,
                    sst_raw_idx,
                    authored_style_only,
                } = match scan_cell(
                    xml,
                    cell_start,
                    current_row,
                    shared_strings,
                    strings,
                    diagnostics,
                    current_row_style,
                    col_styles,
                ) {
                    Some(sr) => sr,
                    None => {
                        let next = resync_after_malformed_cell(
                            xml,
                            cell_start.saturating_add(1),
                            sheet_data_end,
                        );
                        match next {
                            Some(next_pos) => {
                                pos = next_pos;
                                continue;
                            }
                            None => break,
                        }
                    }
                };

                let cell_parsed = if let Some(cell_data) = cell_opt {
                    cells[cell_idx] = cell_data;
                    cell_idx += 1;
                    true
                } else {
                    false
                };

                if let Some(ext) = extras.as_deref_mut() {
                    if let Some(style_only) = authored_style_only {
                        ext.authored_style_only_cells.push(style_only);
                    }
                    if cell_parsed {
                        let last_idx = cell_idx - 1;
                        let cell_data = cells[last_idx];
                        collect_cell_extras(
                            ext,
                            last_idx,
                            cell_data,
                            strings,
                            CellExtrasInput {
                                cm_val,
                                vm_val,
                                has_ph,
                                has_explicit_s,
                                has_xml_space_v,
                                sst_raw_idx,
                            },
                        );
                        if !is_self_closing {
                            let cell_xml = &xml[cell_start..cell_end];
                            collect_formula_extras(
                                ext,
                                last_idx,
                                cell_data,
                                cell_xml,
                                strings,
                                has_xml_space_v,
                            );
                        }
                    }
                }

                pos = cell_end;
            } else if xml[pos] == b'/' {
                if let Some(gt) = find_byte(xml, b'>', pos) {
                    pos = gt + 1;
                }
            } else if let Some(gt) = find_byte(xml, b'>', pos) {
                pos = gt + 1;
            }
        } else {
            break;
        }
    }

    cell_idx
}
