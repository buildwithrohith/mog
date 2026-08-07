use std::collections::{BTreeMap, BTreeSet};

use xlsx_api::{DependencyBlocker, ParserZipWriter, parse_sheet_dependency_manifest};

fn workbook_fixture(sheets: &[(&str, Option<&str>)], defined_names: &[(&str, &str)]) -> Vec<u8> {
    let sheet_elements = sheets
        .iter()
        .enumerate()
        .map(|(index, (name, _))| {
            format!(
                r#"<sheet name="{name}" sheetId="{}" r:id="rId{}"/>"#,
                index + 1,
                index + 1
            )
        })
        .collect::<String>();
    let names = if defined_names.is_empty() {
        String::new()
    } else {
        format!(
            "<definedNames>{}</definedNames>",
            defined_names
                .iter()
                .map(|(name, formula)| format!(
                    r#"<definedName name="{name}">{formula}</definedName>"#
                ))
                .collect::<String>()
        )
    };
    let workbook = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>{sheet_elements}</sheets>{names}
</workbook>"#
    );
    let relationships = sheets
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#,
                index + 1,
                index + 1
            )
        })
        .collect::<String>();
    let workbook_rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{relationships}</Relationships>"#
    );
    let overrides = sheets
        .iter()
        .enumerate()
        .map(|(index, _)| {
            format!(
                r#"<Override PartName="/xl/worksheets/sheet{}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
                index + 1
            )
        })
        .collect::<String>();
    let content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  {overrides}
</Types>"#
    );

    let mut zip = ParserZipWriter::new();
    zip.add_file("[Content_Types].xml", content_types.into_bytes());
    zip.add_file(
        "_rels/.rels",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#
            .to_vec(),
    );
    zip.add_file("xl/workbook.xml", workbook.into_bytes());
    zip.add_file("xl/_rels/workbook.xml.rels", workbook_rels.into_bytes());
    for (index, (_, formula)) in sheets.iter().enumerate() {
        let cell = formula
            .map(|formula| format!(r#"<c r="A1"><f>{formula}</f><v>0</v></c>"#))
            .unwrap_or_else(|| r#"<c r="A1"><v>1</v></c>"#.to_string());
        let worksheet = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1">{cell}</row></sheetData>
</worksheet>"#
        );
        zip.add_file(
            &format!("xl/worksheets/sheet{}.xml", index + 1),
            worksheet.into_bytes(),
        );
    }
    zip.finish().expect("dependency fixture")
}

#[test]
fn direct_sheet_reference_uses_workbook_order_indices() {
    let bytes = workbook_fixture(&[("Inputs", None), ("Model", Some("Inputs!A1"))], &[]);
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.sheet_names,
        BTreeMap::from([(0, "Inputs".into()), (1, "Model".into())])
    );
    assert_eq!(
        manifest.precedents_by_dependent,
        BTreeMap::from([(1, BTreeSet::from([0]))])
    );
}

#[test]
fn three_sheet_formula_collects_both_precedents() {
    let bytes = workbook_fixture(
        &[
            ("Inputs", None),
            ("Revenue", None),
            ("DCF", Some("Inputs!A1+Revenue!A1")),
        ],
        &[],
    );
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.precedents_by_dependent.get(&2),
        Some(&BTreeSet::from([0, 1]))
    );
    assert!(manifest.blockers_by_dependent.is_empty());
}

#[test]
fn static_defined_name_resolves_recursively_for_imported_and_prospective_formulas() {
    let bytes = workbook_fixture(
        &[("Inputs", None), ("Model", Some("InputCell*2"))],
        &[("InputCell", "Inputs!$A$1")],
    );
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.precedents_by_dependent.get(&1),
        Some(&BTreeSet::from([0]))
    );
    let prospective = manifest.analyze_formula(1, "=InputCell+Inputs!B2");
    assert_eq!(prospective.precedents, BTreeSet::from([0]));
    assert!(prospective.blockers.is_empty());
}

#[test]
fn dynamic_defined_name_fails_closed_for_its_formula_owner() {
    let bytes = workbook_fixture(
        &[("Inputs", None), ("Model", Some("DynamicInput"))],
        &[("DynamicInput", "INDIRECT(\"Inputs!A1\")")],
    );
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.blockers_by_dependent.get(&1),
        Some(&BTreeSet::from([DependencyBlocker::DynamicIndirect]))
    );
}

#[test]
fn external_reference_fails_closed_for_its_formula_owner() {
    let bytes = workbook_fixture(&[("Model", Some("[Book.xlsx]Inputs!A1"))], &[]);
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.blockers_by_dependent.get(&0),
        Some(&BTreeSet::from([DependencyBlocker::ExternalReference]))
    );
}

#[test]
fn malformed_formula_fails_closed_for_its_formula_owner() {
    let bytes = workbook_fixture(&[("Model", Some("SUM(A1"))], &[]);
    let manifest = parse_sheet_dependency_manifest(&bytes).expect("manifest");

    assert_eq!(
        manifest.blockers_by_dependent.get(&0),
        Some(&BTreeSet::from([DependencyBlocker::MalformedFormula]))
    );
}
