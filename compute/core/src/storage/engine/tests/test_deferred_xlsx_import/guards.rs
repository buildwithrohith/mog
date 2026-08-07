use super::support::*;
use super::*;
use cell_types::SheetId;
use formula_types::StructureChange;

#[test]
fn empty_yrs_storage_with_nonempty_mirror_rejects_export_and_sync_diff() {
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine.with_storage_and_mirror_for_test(|_, mirror| {
        *mirror = crate::mirror::CellMirror::from_snapshot(simple_snapshot())
            .expect("mirror fixture should build");
    });

    assert!(engine.storage().sheet_order().is_empty());
    assert_eq!(engine.mirror().sheet_count(), 1);

    let export_err = engine
        .export_to_parse_output()
        .expect_err("export must reject an empty Yrs document with mirrored sheets");
    assert!(
        export_err.to_string().contains("empty"),
        "export guard should name the empty Yrs state: {export_err}"
    );

    let remote_state_vector = engine.encode_state_vector();
    let diff_err = engine
        .encode_diff(&remote_state_vector)
        .expect_err("sync diff must reject an empty Yrs document with mirrored sheets");
    assert!(
        diff_err.to_string().contains("empty"),
        "sync guard should name the empty Yrs state: {diff_err}"
    );
}

#[test]
fn deferred_structure_change_completes_hydration_before_mutating_and_exporting() {
    let bytes = basic_import_fixture_xlsx();
    let (mut engine, _) = YrsComputeEngine::from_snapshot(WorkbookSnapshot::default()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");

    let sheet_id = SheetId::from_uuid_str(
        &engine
            .get_all_sheet_ids()
            .first()
            .cloned()
            .expect("deferred fixture should have a sheet"),
    )
    .expect("deferred fixture sheet id should be valid");
    engine
        .structure_change(
            &sheet_id,
            &StructureChange::InsertRows {
                at: 1,
                count: 1,
                new_row_ids: Vec::new(),
            },
        )
        .expect("structural deferred path should complete hydration and insert a row");

    assert!(
        !engine.storage().sheet_order().is_empty(),
        "structural completion must not commit an empty Yrs document"
    );
    let exported = engine
        .export_to_xlsx_bytes()
        .expect("export should succeed after structural deferred completion");
    let parsed = xlsx_api::parse(&exported).expect("structural export should parse");
    assert_eq!(parsed.output.sheets.len(), 1);
}
