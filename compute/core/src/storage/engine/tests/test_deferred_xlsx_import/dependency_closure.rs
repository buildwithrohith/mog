use super::*;

use domain_types::{DocumentFormat, NamedRange, SheetData};
use value_types::CellValue;

fn formula_cell(value: f64, formula: Option<&str>) -> domain_types::CellData {
    let mut cell = domain_types::CellData {
        row: 0,
        col: 0,
        value: CellValue::number(value),
        ..Default::default()
    };
    cell.extras_mut().formula = formula.map(str::to_string);
    cell
}

fn dependency_fixture(
    sheets: &[(&str, f64, Option<&str>)],
    named_ranges: Vec<NamedRange>,
) -> Vec<u8> {
    let output = domain_types::ParseOutput {
        sheets: sheets
            .iter()
            .map(|(name, value, formula)| SheetData {
                name: (*name).to_string(),
                rows: 1,
                cols: 1,
                cells: vec![formula_cell(*value, *formula)],
                ..Default::default()
            })
            .collect(),
        named_ranges,
        ..Default::default()
    };
    xlsx_parser::write::write_xlsx_from_parse_output(&output)
        .expect("dependency-closure fixture should be writable")
}

fn imported_sheet_ids(engine: &YrsComputeEngine) -> Vec<SheetId> {
    engine
        .get_all_sheet_ids()
        .iter()
        .map(|id| SheetId::from_uuid_str(id).expect("imported sheet id should parse"))
        .collect()
}

fn assert_hydrated(engine: &YrsComputeEngine, sheet_id: SheetId, expected: bool) {
    let state = engine
        .deferred_hydration
        .as_ref()
        .expect("workbook should remain incrementally deferred");
    assert_eq!(
        state.yrs_hydrated_sheets.contains(&sheet_id),
        expected,
        "unexpected Yrs hydration state for {sheet_id}"
    );
    assert_eq!(
        state.mirror_materialized_sheets.contains(&sheet_id),
        expected,
        "unexpected mirror materialization state for {sheet_id}"
    );
}

#[test]
fn edit_hydrates_downstream_and_all_of_its_precedents_only() {
    let xlsx = dependency_fixture(
        &[
            ("Inputs", 2.0, None),
            ("Revenue", 10.0, None),
            ("DCF", 20.0, Some("Inputs!A1*Revenue!A1")),
            ("Notes", 99.0, None),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);

    let input_cell_id = engine.deferred_hydration.as_ref().unwrap().allocations[0].cell_ids[0];
    engine
        .batch_set_cells(
            vec![(
                ids[0],
                input_cell_id,
                0,
                0,
                mutation::CellInput::Parse { text: "3".into() },
            )],
            false,
        )
        .expect("input edit should be admitted after closure hydration");

    assert_eq!(
        engine.get_cell_value(&ids[2], 0, 0),
        CellValue::number(30.0)
    );
    assert_hydrated(&engine, ids[0], true);
    assert_hydrated(&engine, ids[1], true);
    assert_hydrated(&engine, ids[2], true);
    assert_hydrated(&engine, ids[3], false);
    assert!(engine.get_cell_id_at_yrs(&ids[3], 0, 0).is_none());

    engine
        .complete_deferred_hydration()
        .expect("legacy full completion must preserve admitted edits");
    assert_eq!(engine.get_cell_value(&ids[0], 0, 0), CellValue::number(3.0));
    assert_eq!(
        engine.get_cell_value(&ids[2], 0, 0),
        CellValue::number(30.0)
    );
    assert!(engine.deferred_hydration.is_none());
    assert!(
        engine.can_undo(),
        "admitted edit must survive as user history"
    );
    engine
        .undo()
        .expect("completion-preserved edit should undo");
    assert_eq!(engine.get_cell_value(&ids[0], 0, 0), CellValue::number(2.0));
    assert_eq!(
        engine.get_cell_value(&ids[2], 0, 0),
        CellValue::number(20.0)
    );
}

#[test]
fn edit_in_strongly_connected_component_hydrates_the_whole_component() {
    let xlsx = dependency_fixture(
        &[
            ("A", 1.0, Some("B!A1+1")),
            ("B", 2.0, Some("C!A1+1")),
            ("C", 3.0, Some("A!A1+1")),
            ("Unrelated", 4.0, None),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);

    engine
        .batch_set_cells_by_position(
            vec![(
                ids[0],
                0,
                0,
                mutation::CellInput::Parse { text: "7".into() },
            )],
            false,
        )
        .expect("SCC edit should hydrate and admit its full component");

    assert_hydrated(&engine, ids[0], true);
    assert_hydrated(&engine, ids[1], true);
    assert_hydrated(&engine, ids[2], true);
    assert_hydrated(&engine, ids[3], false);
    assert_eq!(engine.get_cell_value(&ids[0], 0, 0), CellValue::number(7.0));
}

#[test]
fn dynamic_named_range_dependency_fails_before_hydration_or_edit() {
    let xlsx = dependency_fixture(
        &[("Inputs", 2.0, None), ("Output", 2.0, Some("DynamicInput"))],
        vec![NamedRange {
            name: "DynamicInput".into(),
            refers_to: "INDIRECT(\"Inputs!A1\")".into(),
            ..Default::default()
        }],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);
    let yrs_before = compute_collab::encode_full_state(engine.storage().doc());
    let sets_before = {
        let state = engine.deferred_hydration.as_ref().unwrap();
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone(),
        )
    };

    let err = engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                0,
                mutation::CellInput::Parse { text: "5".into() },
            )],
            false,
        )
        .expect_err("dynamic named dependency must fail closed");

    let error_text = err.to_string().to_ascii_lowercase();
    assert!(
        error_text.contains("named") || error_text.contains("indirect"),
        "unexpected fail-closed error: {err}"
    );
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_before
    );
    let state = engine.deferred_hydration.as_ref().unwrap();
    assert_eq!(
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone()
        ),
        sets_before
    );
}

#[test]
fn external_reference_dependency_fails_before_hydration_or_edit() {
    let xlsx = dependency_fixture(
        &[
            ("Inputs", 2.0, None),
            ("Output", 2.0, Some("'[Book.xlsx]Inputs'!A1")),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);
    let yrs_before = compute_collab::encode_full_state(engine.storage().doc());
    let sets_before = {
        let state = engine.deferred_hydration.as_ref().unwrap();
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone(),
        )
    };

    let err = engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                0,
                mutation::CellInput::Parse { text: "5".into() },
            )],
            false,
        )
        .expect_err("external dependency must fail closed");

    assert!(
        err.to_string().to_ascii_lowercase().contains("external"),
        "unexpected fail-closed error: {err}"
    );
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_before
    );
    let state = engine.deferred_hydration.as_ref().unwrap();
    assert_eq!(
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone()
        ),
        sets_before
    );
}

#[test]
fn public_set_cell_fails_closed_before_any_external_dependency_write() {
    let xlsx = dependency_fixture(
        &[
            ("Inputs", 2.0, None),
            ("Output", 2.0, Some("'[Book.xlsx]Inputs'!A1")),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);
    // The parse-only sheet has not allocated cell identities yet. Dependency
    // preflight must reject before this placeholder can be observed.
    let output_cell_id = CellId::from_raw(u128::MAX);
    let yrs_before = compute_collab::encode_full_state(engine.storage().doc());
    let sets_before = {
        let state = engine.deferred_hydration.as_ref().unwrap();
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone(),
        )
    };

    let err = engine
        .set_cell(
            &ids[1],
            output_cell_id,
            0,
            0,
            mutation::CellInput::Parse { text: "5".into() },
        )
        .expect_err("public set_cell must run dependency preflight before writing");

    assert!(err.to_string().to_ascii_lowercase().contains("external"));
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_before
    );
    let state = engine.deferred_hydration.as_ref().unwrap();
    assert_eq!(
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone()
        ),
        sets_before
    );
}

#[test]
fn prospective_external_formula_fails_before_hydration_or_edit() {
    let xlsx = dependency_fixture(&[("Landing", 1.0, None), ("Target", 0.0, None)], vec![]);
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);
    let yrs_before = compute_collab::encode_full_state(engine.storage().doc());
    let sets_before = {
        let state = engine.deferred_hydration.as_ref().unwrap();
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone(),
        )
    };

    let err = engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                0,
                mutation::CellInput::Parse {
                    text: "='[Book.xlsx]Landing'!A1".into(),
                },
            )],
            false,
        )
        .expect_err("prospective external reference must fail closed");

    assert!(err.to_string().to_ascii_lowercase().contains("external"));
    assert_eq!(
        compute_collab::encode_full_state(engine.storage().doc()),
        yrs_before
    );
    let state = engine.deferred_hydration.as_ref().unwrap();
    assert_eq!(
        (
            state.mirror_materialized_sheets.clone(),
            state.yrs_hydrated_sheets.clone()
        ),
        sets_before
    );
}

#[test]
fn prospective_formula_hydrates_its_precedent_before_write() {
    let xlsx = dependency_fixture(
        &[
            ("Landing", 1.0, None),
            ("Target", 0.0, None),
            ("Revenue", 11.0, None),
            ("Unrelated", 99.0, None),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);

    engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                0,
                mutation::CellInput::Parse {
                    text: "=Revenue!A1*2".into(),
                },
            )],
            false,
        )
        .expect("prospective formula dependencies should hydrate before write");

    assert_eq!(
        engine.get_cell_value(&ids[1], 0, 0),
        CellValue::number(22.0)
    );
    assert_hydrated(&engine, ids[0], true);
    assert_hydrated(&engine, ids[1], true);
    assert_hydrated(&engine, ids[2], true);
    assert_hydrated(&engine, ids[3], false);
}

#[test]
fn completion_preserves_formula_inherited_number_format() {
    let output = domain_types::ParseOutput {
        style_palette: vec![
            DocumentFormat::default(),
            DocumentFormat {
                number_format: Some("$#,##0.00".into()),
                ..Default::default()
            },
        ],
        sheets: vec![
            SheetData {
                name: "Landing".into(),
                rows: 1,
                cols: 1,
                cells: vec![domain_types::CellData {
                    row: 0,
                    col: 0,
                    value: CellValue::number(11.0),
                    style_id: Some(1),
                    ..Default::default()
                }],
                ..Default::default()
            },
            SheetData {
                name: "Target".into(),
                rows: 1,
                cols: 1,
                cells: vec![formula_cell(0.0, None)],
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    let xlsx = xlsx_parser::write::write_xlsx_from_parse_output(&output).unwrap();
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine.import_from_xlsx_bytes_deferred(&xlsx).unwrap();
    let ids = imported_sheet_ids(&engine);

    engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                0,
                mutation::CellInput::Parse {
                    text: "=Landing!A1*2".into(),
                },
            )],
            false,
        )
        .unwrap();
    assert_eq!(
        engine
            .get_resolved_format(&ids[1], 0, 0)
            .number_format
            .as_deref(),
        Some("$#,##0.00")
    );

    engine.complete_deferred_hydration().unwrap();
    assert_eq!(
        engine.get_cell_value(&ids[1], 0, 0),
        CellValue::number(22.0)
    );
    assert_eq!(
        engine
            .get_resolved_format(&ids[1], 0, 0)
            .number_format
            .as_deref(),
        Some("$#,##0.00")
    );
}

#[test]
fn formula_free_edit_hydrates_exactly_the_edited_nonlanding_sheet() {
    let xlsx = dependency_fixture(
        &[
            ("Landing", 1.0, None),
            ("Edited", 2.0, None),
            ("Unrelated", 3.0, None),
        ],
        vec![],
    );
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&xlsx)
        .expect("deferred XLSX import should succeed");
    let ids = imported_sheet_ids(&engine);

    engine
        .batch_set_cells_by_position(
            vec![(
                ids[1],
                0,
                1,
                mutation::CellInput::Parse { text: "8".into() },
            )],
            false,
        )
        .expect("formula-free edit should admit one target sheet");

    assert_eq!(engine.get_cell_value(&ids[1], 0, 1), CellValue::number(8.0));
    assert_hydrated(&engine, ids[0], true);
    assert_hydrated(&engine, ids[1], true);
    assert_hydrated(&engine, ids[2], false);

    let cell_id_before = engine
        .get_cell_id_at_yrs(&ids[1], 0, 1)
        .expect("newly written cell should have a durable identity");
    engine
        .complete_deferred_hydration()
        .expect("completion should replay a newly allocated cell");
    assert_eq!(engine.get_cell_value(&ids[1], 0, 1), CellValue::number(8.0));
    assert_eq!(
        engine.get_cell_id_at_yrs(&ids[1], 0, 1).as_deref(),
        Some(cell_id_before.as_str())
    );
}
