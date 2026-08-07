use cell_types::{CellId, SheetPos};
use value_types::CellValue;

use crate::mirror::types::CellEntry;

use super::helpers::make_mirror;

#[test]
fn col_version_returns_zero_for_untracked() {
    let (mirror, sheet_id) = make_mirror();
    assert_eq!(mirror.col_version(&sheet_id, 0), 0);
    assert_eq!(mirror.col_version(&sheet_id, 99), 0);
}

#[test]
fn insert_cell_bumps_col_version() {
    let (mut mirror, sheet_id) = make_mirror();
    assert_eq!(mirror.col_version(&sheet_id, 3), 0);

    let cell_id = CellId::from_raw(10);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(0, 3),
        CellEntry {
            value: CellValue::number(1.0),
            formula: None,
        },
    );
    assert_eq!(mirror.col_version(&sheet_id, 3), 1);
}

#[test]
fn set_value_mut_bumps_col_version() {
    let (mut mirror, sheet_id) = make_mirror();
    let cell_id = CellId::from_raw(20);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(0, 5),
        CellEntry {
            value: CellValue::number(1.0),
            formula: None,
        },
    );
    let v_after_insert = mirror.col_version(&sheet_id, 5);

    mirror.set_value_mut(&cell_id, CellValue::number(2.0));
    assert_eq!(mirror.col_version(&sheet_id, 5), v_after_insert + 1);
}

#[test]
fn remove_cell_bumps_col_version() {
    let (mut mirror, sheet_id) = make_mirror();
    let cell_id = CellId::from_raw(30);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(0, 7),
        CellEntry {
            value: CellValue::number(1.0),
            formula: None,
        },
    );
    let v_after_insert = mirror.col_version(&sheet_id, 7);

    mirror.remove_cell(&cell_id);
    assert_eq!(mirror.col_version(&sheet_id, 7), v_after_insert + 1);
}

#[test]
fn apply_edit_bumps_col_version() {
    let (mut mirror, sheet_id) = make_mirror();
    assert_eq!(mirror.col_version(&sheet_id, 2), 0);

    let cell_id = CellId::from_raw(40);
    mirror.apply_edit(
        &sheet_id,
        cell_id,
        SheetPos::new(0, 2),
        CellValue::number(99.0),
        None,
    );
    assert_eq!(mirror.col_version(&sheet_id, 2), 1);
}

#[test]
fn insert_cell_creates_col_data_for_new_column() {
    let (mut mirror, sheet_id) = make_mirror();
    // Column 20 has no col_data entry initially
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert!(!sheet.col_data.contains_key(&20));

    let cell_id = CellId::from_raw(100);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(5, 20),
        CellEntry {
            value: CellValue::number(42.0),
            formula: None,
        },
    );

    // col_data should now exist for column 20
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert!(sheet.col_data.contains_key(&20));
    let col_vec = &sheet.col_data[&20];
    assert_eq!(col_vec[5], CellValue::number(42.0));
}

#[test]
fn set_value_mut_creates_col_data_for_new_column() {
    let (mut mirror, sheet_id) = make_mirror();
    // Insert a cell into a column that has no col_data
    let cell_id = CellId::from_raw(101);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(3, 25),
        CellEntry {
            value: CellValue::number(1.0),
            formula: None,
        },
    );
    // col_data should exist now (from insert_cell fix)
    // Verify set_value_mut also works on it
    mirror.set_value_mut(&cell_id, CellValue::number(99.0));
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert_eq!(sheet.col_data[&25][3], CellValue::number(99.0));
}

#[test]
fn apply_edit_creates_col_data_for_new_column() {
    let (mut mirror, sheet_id) = make_mirror();
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert!(!sheet.col_data.contains_key(&30));

    let cell_id = CellId::from_raw(102);
    mirror.apply_edit(
        &sheet_id,
        cell_id,
        SheetPos::new(2, 30),
        CellValue::number(77.0),
        None,
    );

    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert!(sheet.col_data.contains_key(&30));
    assert_eq!(sheet.col_data[&30][2], CellValue::number(77.0));
}

#[test]
fn writing_col_a_does_not_affect_col_b() {
    let (mut mirror, sheet_id) = make_mirror();
    let cell_id = CellId::from_raw(50);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        SheetPos::new(0, 0),
        CellEntry {
            value: CellValue::number(1.0),
            formula: None,
        },
    );
    assert_eq!(mirror.col_version(&sheet_id, 0), 1);
    assert_eq!(mirror.col_version(&sheet_id, 1), 0);
}

/// `apply_edit` must remove a CellId's old forward slot when an upsert moves
/// it from position A to position B. Otherwise two positions resolve to one
/// CellId and rendering position A returns position B's value.
#[test]
fn test_apply_edit_stale_pos_to_id_after_move() {
    use crate::projection::CellRender;

    let (mut mirror, sheet_id) = make_mirror();
    let cell_id = CellId::from_raw(500);

    // Step 1: Insert cell at position A (row=2, col=1) with value "hello"
    let pos_a = SheetPos::new(2, 1);
    mirror.insert_cell(
        &sheet_id,
        cell_id,
        pos_a,
        CellEntry {
            value: CellValue::from("hello"),
            formula: None,
        },
    );

    // Sanity: pos_a resolves to our cell_id
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert_eq!(sheet.cell_id_at(pos_a), Some(cell_id));
    assert_eq!(sheet.position_of(&cell_id), Some(pos_a));

    // Step 2: apply_edit with the SAME cell_id but at position B (row=15, col=12)
    let pos_b = SheetPos::new(15, 12);
    mirror.apply_edit(&sheet_id, cell_id, pos_b, CellValue::from("world"), None);

    // The derived reverse lookup points to the new position B.
    let sheet = mirror.get_sheet(&sheet_id).unwrap();
    assert_eq!(sheet.position_of(&cell_id), Some(pos_b));

    // The old forward slot and dense value are cleared.
    assert!(sheet.cell_id_at(pos_a).is_none());
    assert_eq!(
        sheet.col_data[&pos_a.col()][pos_a.row() as usize],
        CellValue::Null
    );

    assert!(matches!(
        mirror.cell_render_at(&sheet_id, pos_a.row(), pos_a.col()),
        CellRender::Empty
    ));
}
