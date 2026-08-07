use super::*;
use cell_types::{CellId, SheetId};
use std::collections::HashSet;
use value_types::CellValue;
use xlsx_parser::write::ZipWriter;

fn active_second_sheet_identity_collision_fixture_xlsx() -> Vec<u8> {
    let workbook = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <bookViews>
    <workbookView activeTab="1"/>
  </bookViews>
  <sheets>
    <sheet name="Earlier" sheetId="1" r:id="rId1"/>
    <sheet name="ActiveModel" sheetId="2" r:id="rId2"/>
    <sheet name="Later" sheetId="3" r:id="rId3"/>
  </sheets>
</workbook>"#;
    let sheet1 = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:D1"/>
  <sheetData>
    <row r="1">
      <c r="A1"><v>11</v></c>
      <c r="B1"><v>12</v></c>
      <c r="C1"><v>13</v></c>
      <c r="D1"><v>14</v></c>
    </row>
  </sheetData>
</worksheet>"#;
    let sheet2 = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:B1"/>
  <sheetData>
    <row r="1">
      <c r="A1"><f>1+2</f><v>3</v></c>
    </row>
  </sheetData>
</worksheet>"#;
    let sheet3 = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:B1"/>
  <sheetData>
    <row r="1">
      <c r="A1"><v>21</v></c>
      <c r="B1"><v>22</v></c>
    </row>
  </sheetData>
</worksheet>"#;

    let mut zip = ZipWriter::new();
    zip.add_file(
        "[Content_Types].xml",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/worksheets/sheet3.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#
            .to_vec(),
    )
    .add_file(
        "_rels/.rels",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#
            .to_vec(),
    )
    .add_file("xl/workbook.xml", workbook.as_bytes().to_vec())
    .add_file(
        "xl/_rels/workbook.xml.rels",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet3.xml"/>
</Relationships>"#
            .to_vec(),
    )
    .add_file("xl/worksheets/sheet1.xml", sheet1.as_bytes().to_vec())
    .add_file("xl/worksheets/sheet2.xml", sheet2.as_bytes().to_vec())
    .add_file("xl/worksheets/sheet3.xml", sheet3.as_bytes().to_vec());
    zip.finish()
        .expect("write active-second identity collision fixture")
}

#[test]
fn deferred_full_hydration_does_not_reuse_active_sheet_cell_ids_for_earlier_sheets() {
    let bytes = active_second_sheet_identity_collision_fixture_xlsx();

    let (mut engine, _) = YrsComputeEngine::from_snapshot(simple_snapshot()).unwrap();
    engine
        .import_from_xlsx_bytes_deferred(&bytes)
        .expect("deferred XLSX import should succeed");

    let sheet_ids = engine.get_all_sheet_ids();
    assert_eq!(sheet_ids.len(), 3);
    let earlier = SheetId::from_uuid_str(&sheet_ids[0]).expect("earlier sheet id");
    let active_model = SheetId::from_uuid_str(&sheet_ids[1]).expect("active model sheet id");
    let later = SheetId::from_uuid_str(&sheet_ids[2]).expect("later sheet id");
    assert_eq!(engine.get_sheet_name(&earlier).as_deref(), Some("Earlier"));
    assert_eq!(
        engine.get_sheet_name(&active_model).as_deref(),
        Some("ActiveModel")
    );
    assert_eq!(engine.get_sheet_name(&later).as_deref(), Some("Later"));

    let active_a1_before = engine
        .get_cell_id_at(&active_model, 0, 0)
        .expect("ActiveModel!A1 should have a first-paint cell id");
    let active_grid_before = engine
        .stores
        .grid_indexes
        .get(&active_model)
        .expect("active sheet grid should exist");
    let active_rows_before = active_grid_before.row_ids_ordered();
    let active_cols_before = active_grid_before.col_ids_ordered();

    // Simulate an edit allocating a new position identity from the live
    // runtime authority before the remaining sheets are hydrated. The source
    // XLSX does not contain B1, so completion must not reuse this ID elsewhere.
    let edited_active_cell = engine
        .stores
        .grid_indexes
        .get_mut(&active_model)
        .expect("active sheet grid should exist")
        .ensure_cell_id(0, 1);

    engine
        .complete_deferred_hydration()
        .expect("full deferred hydration should succeed");

    let earlier_d1 = engine
        .get_cell_id_at(&earlier, 0, 3)
        .expect("Earlier!D1 should have a cell id");
    let active_a1 = engine
        .get_cell_id_at(&active_model, 0, 0)
        .expect("ActiveModel!A1 should have a cell id");
    let active_grid_after = engine
        .stores
        .grid_indexes
        .get(&active_model)
        .expect("active sheet grid should exist after completion");
    assert_eq!(active_grid_after.row_ids_ordered(), active_rows_before);
    assert_eq!(active_grid_after.col_ids_ordered(), active_cols_before);

    assert_ne!(
        earlier_d1, active_a1,
        "full deferred hydration must reserve IDs allocated to the active sheet during first paint"
    );
    assert_eq!(
        engine.get_cell_value(&earlier, 0, 3),
        CellValue::number(14.0)
    );
    assert_eq!(
        engine.get_formula(&CellId::from_uuid_str(&earlier_d1).expect("Earlier!D1 cell id")),
        None,
        "Earlier!D1 must not inherit ActiveModel!A1 formula text through CellId reuse"
    );
    assert_eq!(
        engine.get_formula(&CellId::from_uuid_str(&active_a1).expect("ActiveModel!A1 cell id")),
        Some("=1+2".to_string())
    );

    let mut all_identity_ids = HashSet::new();
    for sheet_id in &sheet_ids {
        let sheet_id = SheetId::from_uuid_str(sheet_id).expect("sheet id should parse");
        assert!(all_identity_ids.insert(sheet_id.as_u128()));
        let grid = engine
            .stores
            .grid_indexes
            .get(&sheet_id)
            .expect("completed sheet grid should exist");
        for row_id in grid.row_ids_ordered() {
            assert!(all_identity_ids.insert(row_id.as_u128()));
        }
        for col_id in grid.col_ids_ordered() {
            assert!(all_identity_ids.insert(col_id.as_u128()));
        }
        for (cell_id, _, _) in grid.cells() {
            assert!(all_identity_ids.insert(cell_id.as_u128()));
        }
    }
    assert!(
        !all_identity_ids.contains(&edited_active_cell.as_u128()),
        "later-sheet hydration must not reuse the ID allocated by the intervening edit"
    );
    assert_eq!(active_a1, active_a1_before);
}
