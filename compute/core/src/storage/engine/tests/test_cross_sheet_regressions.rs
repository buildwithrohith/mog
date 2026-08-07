use super::super::*;
use super::helpers::*;
use crate::snapshot::{CellData, SheetSnapshot, WorkbookSnapshot};
use cell_types::{CellId, SheetId};
use compute_document::hex::hex_to_id;
use formula_types::StructureChange;
use value_types::{CellValue, FiniteF64};

const SHEET1_ID: &str = "00000000-0000-0000-0000-000000010001";
const SHEET2_ID: &str = "00000000-0000-0000-0000-000000010002";
const SHEET1_A1_ID: &str = "00000000-0000-0000-0000-000000010101";
const SHEET1_A2_ID: &str = "00000000-0000-0000-0000-000000010102";
const SHEET2_A1_ID: &str = "00000000-0000-0000-0000-000000020101";
const SHEET2_A2_ID: &str = "00000000-0000-0000-0000-000000020102";

fn sid(raw: &str) -> SheetId {
    SheetId::from_uuid_str(raw).unwrap()
}

fn cid(raw: &str) -> CellId {
    CellId::from_uuid_str(raw).unwrap()
}

fn text(value: &str) -> CellValue {
    CellValue::Text(value.into())
}

fn cross_sheet_snapshot() -> WorkbookSnapshot {
    WorkbookSnapshot {
        sheets: vec![
            SheetSnapshot {
                id: SHEET1_ID.to_string(),
                name: "Sheet1".to_string(),
                rows: 100,
                cols: 26,
                cells: vec![
                    CellData {
                        cell_id: SHEET1_A1_ID.to_string(),
                        row: 0,
                        col: 0,
                        value: CellValue::Null,
                        formula: Some("=Sheet2!A2".to_string()),
                        identity_formula: None,
                        array_ref: None,
                    },
                    CellData {
                        cell_id: SHEET1_A2_ID.to_string(),
                        row: 1,
                        col: 0,
                        value: CellValue::Null,
                        formula: Some("=Sheet2!A1".to_string()),
                        identity_formula: None,
                        array_ref: None,
                    },
                ],
                ranges: vec![],
            },
            SheetSnapshot {
                id: SHEET2_ID.to_string(),
                name: "Sheet2".to_string(),
                rows: 100,
                cols: 26,
                cells: vec![
                    CellData {
                        cell_id: SHEET2_A1_ID.to_string(),
                        row: 0,
                        col: 0,
                        value: text("Sheet2Data"),
                        formula: None,
                        identity_formula: None,
                        array_ref: None,
                    },
                    CellData {
                        cell_id: SHEET2_A2_ID.to_string(),
                        row: 1,
                        col: 0,
                        value: CellValue::Number(FiniteF64::must(99.0)),
                        formula: None,
                        identity_formula: None,
                        array_ref: None,
                    },
                ],
                ranges: vec![],
            },
        ],
        named_ranges: vec![],
        tables: vec![],
        pivot_tables: vec![],
        data_table_regions: vec![],
        iterative_calc: false,
        max_iterations: 100,
        max_change: FiniteF64::must(0.001),
        calculation_settings: None,
    }
}

#[test]
fn cross_sheet_structural_formula_writeback_survives_undo_redo() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(cross_sheet_snapshot()).unwrap();
    let sheet1 = sid(SHEET1_ID);
    let sheet2 = sid(SHEET2_ID);
    let sheet1_a1 = cid(SHEET1_A1_ID);

    assert_eq!(
        engine.get_formula(&sheet1_a1).as_deref(),
        Some("=Sheet2!A2")
    );
    assert_eq!(
        cell_value_at(&engine, &sheet1, 0, 0),
        CellValue::Number(FiniteF64::must(99.0))
    );

    let insert = StructureChange::InsertRows {
        at: 1,
        count: 1,
        new_row_ids: vec![],
    };
    engine
        .structure_change(&sheet2, &insert)
        .expect("insert row on referenced sheet");

    assert_eq!(
        engine.get_formula(&sheet1_a1).as_deref(),
        Some("=Sheet2!A3")
    );
    assert_eq!(
        cell_value_at(&engine, &sheet1, 0, 0),
        CellValue::Number(FiniteF64::must(99.0))
    );

    engine.undo().expect("undo insert row");
    assert_eq!(
        engine.get_formula(&sheet1_a1).as_deref(),
        Some("=Sheet2!A2")
    );
    assert_eq!(
        cell_value_at(&engine, &sheet1, 0, 0),
        CellValue::Number(FiniteF64::must(99.0))
    );

    engine.redo().expect("redo insert row");
    assert_eq!(
        engine.get_formula(&sheet1_a1).as_deref(),
        Some("=Sheet2!A3")
    );
    assert_eq!(
        cell_value_at(&engine, &sheet1, 0, 0),
        CellValue::Number(FiniteF64::must(99.0))
    );
}

#[test]
fn copy_sheet_preserves_existing_cross_sheet_dependency_edges() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(cross_sheet_snapshot()).unwrap();
    let sheet1 = sid(SHEET1_ID);
    let sheet2 = sid(SHEET2_ID);

    let (copy_hex, _) = engine.copy_sheet(&sheet1, "Sheet1 (2)").unwrap();
    let copy_sheet = SheetId::from_raw(hex_to_id(&copy_hex).expect("copy sheet hex"));
    engine
        .recalculate()
        .expect("bridge recalc after copied sheet registration");

    let copy_a2 = engine.query_range(&copy_sheet, 1, 0, 1, 0);
    let copy_a2 = copy_a2.cells.first().expect("copied A2 formula cell");
    assert_eq!(copy_a2.formula.as_deref(), Some("=Sheet2!A1"));
    assert_eq!(copy_a2.value, text("Sheet2Data"));

    engine
        .set_cell_value_parsed(&sheet2, 0, 0, "UpdatedSheet2Data")
        .expect("edit referenced Sheet2 A1");

    assert_eq!(
        cell_value_at(&engine, &sheet1, 1, 0),
        text("UpdatedSheet2Data"),
        "original Sheet1 formula must remain a dependent after copy_sheet"
    );
    assert_eq!(
        cell_value_at(&engine, &copy_sheet, 1, 0),
        text("UpdatedSheet2Data"),
        "copied sheet formula must also depend on Sheet2 A1"
    );
}

#[test]
fn cross_sheet_relocation_persists_identity_and_removes_displaced_binding() {
    let source_sheet = sid(SHEET1_ID);
    let target_sheet = sid(SHEET2_ID);
    let moved_cell = cid(SHEET1_A1_ID);
    let displaced_cell = cid(SHEET2_A1_ID);
    let snapshot = WorkbookSnapshot {
        sheets: vec![
            SheetSnapshot {
                id: SHEET1_ID.to_string(),
                name: "Source".to_string(),
                rows: 10,
                cols: 10,
                cells: vec![CellData {
                    cell_id: SHEET1_A1_ID.to_string(),
                    row: 0,
                    col: 0,
                    value: text("moved payload"),
                    formula: None,
                    identity_formula: None,
                    array_ref: None,
                }],
                ranges: vec![],
            },
            SheetSnapshot {
                id: SHEET2_ID.to_string(),
                name: "Target".to_string(),
                rows: 10,
                cols: 10,
                cells: vec![CellData {
                    cell_id: SHEET2_A1_ID.to_string(),
                    row: 2,
                    col: 2,
                    value: text("displaced payload"),
                    formula: None,
                    identity_formula: None,
                    array_ref: None,
                }],
                ranges: vec![],
            },
        ],
        named_ranges: vec![],
        tables: vec![],
        pivot_tables: vec![],
        data_table_regions: vec![],
        iterative_calc: false,
        max_iterations: 100,
        max_change: FiniteF64::must(0.001),
        calculation_settings: None,
    };
    let (mut engine, _) = YrsComputeEngine::from_snapshot(snapshot).unwrap();

    engine
        .relocate_cells_yrs(&source_sheet, 0, 0, 0, 0, &target_sheet, 2, 2)
        .expect("cross-sheet relocate");
    assert_eq!(
        engine
            .grid_index(&source_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        None
    );
    assert_eq!(
        engine
            .grid_index(&target_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        Some((2, 2))
    );
    assert_eq!(
        engine
            .grid_index(&target_sheet)
            .and_then(|grid| grid.cell_position(&displaced_cell)),
        None
    );
    assert_eq!(
        cell_value_at(&engine, &target_sheet, 2, 2),
        text("moved payload")
    );

    let state = compute_collab::encode_full_state(engine.storage().doc());
    let (reloaded, _) = YrsComputeEngine::from_yrs_state(&state).expect("reload relocated state");
    assert_eq!(
        reloaded
            .grid_index(&source_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        None
    );
    assert_eq!(
        reloaded
            .grid_index(&target_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        Some((2, 2))
    );
    assert_eq!(
        reloaded
            .grid_index(&target_sheet)
            .and_then(|grid| grid.cell_position(&displaced_cell)),
        None
    );
    assert_eq!(
        cell_value_at(&reloaded, &target_sheet, 2, 2),
        text("moved payload")
    );
    assert_eq!(
        cell_value_at(&reloaded, &source_sheet, 0, 0),
        CellValue::Null
    );
}

#[test]
fn relocation_beyond_current_axes_persists_same_and_cross_sheet_identity() {
    fn compact_snapshot() -> WorkbookSnapshot {
        WorkbookSnapshot {
            sheets: vec![
                SheetSnapshot {
                    id: SHEET1_ID.to_string(),
                    name: "Source".to_string(),
                    rows: 2,
                    cols: 2,
                    cells: vec![CellData {
                        cell_id: SHEET1_A1_ID.to_string(),
                        row: 0,
                        col: 0,
                        value: text("payload"),
                        formula: None,
                        identity_formula: None,
                        array_ref: None,
                    }],
                    ranges: vec![],
                },
                SheetSnapshot {
                    id: SHEET2_ID.to_string(),
                    name: "Target".to_string(),
                    rows: 2,
                    cols: 2,
                    cells: vec![],
                    ranges: vec![],
                },
            ],
            ..Default::default()
        }
    }

    let source_sheet = sid(SHEET1_ID);
    let target_sheet = sid(SHEET2_ID);
    let moved_cell = cid(SHEET1_A1_ID);

    let (mut same_sheet, _) = YrsComputeEngine::from_snapshot(compact_snapshot()).unwrap();
    same_sheet
        .relocate_cells_yrs(&source_sheet, 0, 0, 0, 0, &source_sheet, 19, 4)
        .expect("same-sheet relocate beyond axes");
    let state = compute_collab::encode_full_state(same_sheet.storage().doc());
    let (same_reloaded, _) = YrsComputeEngine::from_yrs_state(&state).unwrap();
    assert_eq!(
        same_reloaded
            .grid_index(&source_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        Some((19, 4))
    );
    assert_eq!(
        cell_value_at(&same_reloaded, &source_sheet, 19, 4),
        text("payload")
    );

    let (mut cross_sheet, _) = YrsComputeEngine::from_snapshot(compact_snapshot()).unwrap();
    cross_sheet
        .relocate_cells_yrs(&source_sheet, 0, 0, 0, 0, &target_sheet, 19, 4)
        .expect("cross-sheet relocate beyond axes");
    let state = compute_collab::encode_full_state(cross_sheet.storage().doc());
    let (cross_reloaded, _) = YrsComputeEngine::from_yrs_state(&state).unwrap();
    assert_eq!(
        cross_reloaded
            .grid_index(&target_sheet)
            .and_then(|grid| grid.cell_position(&moved_cell)),
        Some((19, 4))
    );
    assert_eq!(
        cell_value_at(&cross_reloaded, &target_sheet, 19, 4),
        text("payload")
    );

    let mut empty_snapshot = compact_snapshot();
    empty_snapshot.sheets[0].cells.clear();
    let (mut empty_source, _) = YrsComputeEngine::from_snapshot(empty_snapshot).unwrap();
    let before_state = compute_collab::encode_full_state(empty_source.storage().doc());
    let before_dims = empty_source
        .grid_index(&target_sheet)
        .map(|grid| (grid.row_count(), grid.col_count()))
        .unwrap();
    empty_source
        .relocate_cells_yrs(&source_sheet, 0, 0, 0, 0, &target_sheet, 9_999, 99)
        .expect("empty relocation remains a no-op");
    assert_eq!(
        empty_source
            .grid_index(&target_sheet)
            .map(|grid| (grid.row_count(), grid.col_count())),
        Some(before_dims)
    );
    assert_eq!(
        compute_collab::encode_full_state(empty_source.storage().doc()),
        before_state,
        "empty relocation must not create a durable dimension mutation"
    );
}
