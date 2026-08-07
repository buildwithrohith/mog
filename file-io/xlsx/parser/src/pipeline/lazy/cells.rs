use crate::domain::cells::{CellData, parse_worksheet_fast_with_owned_strings};
use crate::zip::constants::MAX_WORKSHEET_CELLS;
use ooxml_types::worksheet::RowHeight;

use crate::domain::cells::FastParseDiagnostics;

use super::limits::{count_worksheet_cell_elements, ensure_lazy_limit};
use super::{ParseError, ParsedSheet, SheetMetadata};

pub(super) fn parse_materialized_cells(
    worksheet_xml: &[u8],
    metadata: &SheetMetadata,
    shared_string_refs: &[String],
) -> Result<ParsedSheet, ParseError> {
    let cell_capacity = count_worksheet_cell_elements(worksheet_xml);
    ensure_lazy_limit("worksheet cell", cell_capacity, MAX_WORKSHEET_CELLS)?;

    let estimated_strings = estimated_strings(metadata);
    let mut parsed = ParsedSheet::with_capacity(cell_capacity, estimated_strings);

    fill_materialized_cells(
        &mut parsed,
        worksheet_xml,
        cell_capacity,
        shared_string_refs,
    )?;

    Ok(parsed)
}

pub(super) fn estimated_cells(metadata: &SheetMetadata) -> usize {
    (metadata.uncompressed_size / 50)
        .max(1000)
        .min(MAX_WORKSHEET_CELLS)
}

pub(super) fn estimated_strings(metadata: &SheetMetadata) -> usize {
    metadata.uncompressed_size / 4
}

pub(super) fn fill_materialized_cells(
    parsed: &mut ParsedSheet,
    worksheet_xml: &[u8],
    cell_capacity: usize,
    shared_string_refs: &[String],
) -> Result<(), ParseError> {
    parsed.cells.resize(cell_capacity, CellData::default());

    let mut row_heights_buf: Vec<RowHeight> = Vec::new();
    let mut fast_parse_diagnostics = FastParseDiagnostics::default();
    let cell_count = parse_worksheet_fast_with_owned_strings(
        worksheet_xml,
        shared_string_refs,
        &mut parsed.cells,
        &mut parsed.strings,
        &mut row_heights_buf,
        &mut fast_parse_diagnostics,
        &[],
    );
    if fast_parse_diagnostics.total_count() > 0 {
        tracing::warn!(
            diagnostic_count = fast_parse_diagnostics.total_count(),
            "lazy worksheet materialization recovered from malformed cells"
        );
    }

    parsed.cells.truncate(cell_count);
    parsed.cell_count = cell_count;

    Ok(())
}
