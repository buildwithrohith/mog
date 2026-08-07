use super::support::*;
use super::*;
use value_types::CellValue;

#[test]
fn deferred_xlsx_full_calc_on_load_recalculates_empty_formula_caches_on_completion() {
    let bytes = deferred_calc_fixture_xlsx(DeferredCalcFixtureMode::FullCalcOnLoad);
    let (mut engine, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");
    let (first, _second) = sheet_ids(&engine);
    assert_eq!(engine.get_cell_value(&first, 0, 1), CellValue::Null);

    let (_, mutation) = engine
        .complete_deferred_hydration()
        .expect("full deferred hydration should honor fullCalcOnLoad");
    let (first, second) = sheet_ids(&engine);

    assert_eq!(engine.get_cell_value(&first, 0, 1), CellValue::number(6.0));
    assert_eq!(engine.get_cell_value(&second, 0, 1), CellValue::number(9.0));
    assert_changed_formula(&mutation, &first, 0, 1, 6.0);
    assert_changed_formula(&mutation, &second, 0, 1, 9.0);

    let settings = engine.get_calculation_settings();
    assert!(settings.full_calc_on_load);
    assert!(settings.enable_iterative_calculation);
    assert_eq!(settings.max_iterations, 12);
}

#[test]
fn deferred_xlsx_force_full_calc_recalculates_even_when_manual() {
    let bytes = deferred_calc_fixture_xlsx(DeferredCalcFixtureMode::ForceFullCalcManual);
    let (mut engine, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");

    let (_, mutation) = engine
        .complete_deferred_hydration()
        .expect("full deferred hydration should honor forceFullCalc");
    let (first, second) = sheet_ids(&engine);

    assert_eq!(engine.get_cell_value(&first, 0, 1), CellValue::number(6.0));
    assert_eq!(engine.get_cell_value(&second, 0, 1), CellValue::number(9.0));
    assert_changed_formula(&mutation, &first, 0, 1, 6.0);
    assert_changed_formula(&mutation, &second, 0, 1, 9.0);
    assert!(!engine.get_calculation_settings().full_calc_on_load);

    let exported = engine
        .export_to_xlsx_bytes()
        .expect("post-hydration export should preserve calc metadata");
    let archive = xlsx_parser::zip::XlsxArchive::new(&exported).expect("export should be a zip");
    let workbook_xml =
        String::from_utf8(archive.get_workbook().expect("workbook.xml should exist")).unwrap();
    assert!(
        workbook_xml.contains("forceFullCalc=\"1\""),
        "forceFullCalc must survive deferred completion/export: {workbook_xml}"
    );
}

#[test]
fn deferred_xlsx_without_force_calc_keeps_empty_formula_caches_until_explicit_recalc() {
    let bytes = deferred_calc_fixture_xlsx(DeferredCalcFixtureMode::Control);
    let (mut engine, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");

    let (_, mutation) = engine
        .complete_deferred_hydration()
        .expect("full deferred hydration should not force ordinary workbooks");
    let (first, second) = sheet_ids(&engine);

    assert!(
        mutation.recalc.changed_cells.is_empty(),
        "ordinary deferred completion should not recalc: {:?}",
        mutation.recalc.changed_cells
    );
    assert_eq!(engine.get_cell_value(&first, 0, 1), CellValue::Null);
    assert_eq!(engine.get_cell_value(&second, 0, 1), CellValue::Null);

    let recalc = engine
        .recalculate_with_options(&snapshot_types::RecalcOptions {
            iterative: Some(false),
            max_iterations: Some(100),
            max_change: Some(value_types::FiniteF64::must(0.001)),
        })
        .expect("explicit post-hydration full recalc should populate formula values");
    assert_eq!(engine.get_cell_value(&first, 0, 1), CellValue::number(6.0));
    assert_eq!(engine.get_cell_value(&second, 0, 1), CellValue::number(9.0));
    assert!(
        recalc.changed_cells.iter().any(|change| {
            change.sheet_id == first.to_uuid_string()
                && change
                    .position
                    .as_ref()
                    .is_some_and(|pos| pos.row == 0 && pos.col == 1)
        }),
        "explicit recalc should report first-sheet formula: {:?}",
        recalc.changed_cells
    );
    assert!(
        recalc.changed_cells.iter().any(|change| {
            change.sheet_id == second.to_uuid_string()
                && change
                    .position
                    .as_ref()
                    .is_some_and(|pos| pos.row == 0 && pos.col == 1)
        }),
        "explicit recalc should report second-sheet formula: {:?}",
        recalc.changed_cells
    );
}

#[test]
fn deferred_full_hydration_matches_direct_xlsx_open_content() {
    let bytes = three_sheet_deferred_fixture_xlsx();
    let (direct, _) =
        YrsComputeEngine::from_xlsx_bytes(&bytes).expect("direct XLSX open should succeed");
    let (mut deferred, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    deferred
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");
    deferred
        .complete_deferred_hydration()
        .expect("full deferred hydration should succeed");

    let direct_sheets = direct.build_parse_output_from_yrs().sheets;
    let deferred_sheets = deferred.build_parse_output_from_yrs().sheets;
    assert_eq!(deferred_sheets.len(), direct_sheets.len());

    for (direct_sheet, deferred_sheet) in direct_sheets.iter().zip(deferred_sheets.iter()) {
        assert_eq!(deferred_sheet.name, direct_sheet.name);
        assert_eq!(deferred_sheet.rows, direct_sheet.rows);
        assert_eq!(deferred_sheet.cols, direct_sheet.cols);

        let mut direct_cells: Vec<_> = direct_sheet
            .cells
            .iter()
            .map(|cell| (cell.row, cell.col, cell.value.clone(), cell.formula.clone()))
            .collect();
        let mut deferred_cells: Vec<_> = deferred_sheet
            .cells
            .iter()
            .map(|cell| (cell.row, cell.col, cell.value.clone(), cell.formula.clone()))
            .collect();
        direct_cells.sort_by_key(|cell| (cell.0, cell.1));
        deferred_cells.sort_by_key(|cell| (cell.0, cell.1));

        assert_eq!(
            deferred_cells, direct_cells,
            "cell content for {}",
            direct_sheet.name
        );
    }
}
