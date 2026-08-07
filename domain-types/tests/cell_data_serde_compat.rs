use domain_types::{
    CellData, CellDataExtras, FormulaCacheProvenance, FormulaCacheState,
    FormulaCachedValuePresence, ImportedCellProjectionRole, RichSharedString, RichTextRun,
};
use ooxml_types::worksheet::{CellFormula, CellFormulaType};
use value_types::CellValue;

fn fully_populated_formula_cell() -> CellData {
    CellData {
        row: 4,
        col: 2,
        value: CellValue::from(42.5_f64),
        style_id: Some(7),
        phonetic: true,
        projection_role: ImportedCellProjectionRole::DynamicArraySource,
        extras: Some(Box::new(CellDataExtras {
            rich_string: None,
            formula: Some("SUM(A1:B2)".into()),
            array_ref: Some("A1:B2".into()),
            cell_formula: Some(CellFormula {
                text: "SUM(A1:B2)".into(),
                t: CellFormulaType::Array,
                si: Some(3),
                r#ref: Some("A1:B2".into()),
                aca: true,
                dt2d: true,
                del1: true,
                del2: true,
                r1: Some("A1".into()),
                r2: Some("B2".into()),
                ca: true,
                bx: true,
                dtr: true,
            }),
            cell_metadata_index: Some(11),
            formula_result_type: Some(6),
            has_empty_cached_value: true,
            formula_cache_provenance: FormulaCacheProvenance {
                state: FormulaCacheState::ImportedCurrent,
                force_recalc: true,
                advanced_calc: true,
                formula_preserve_space: true,
                value_preserve_space: true,
                cached_value_kind: Some(6),
                cached_value_presence: FormulaCachedValuePresence::NonEmpty,
                cached_value_lexeme: Some("42.5".into()),
                formula_identity_fingerprint: Some("formula-fingerprint".into()),
                formula_metadata_fingerprint: Some("metadata-fingerprint".into()),
                cached_semantic_value_fingerprint: Some("value-fingerprint".into()),
                owner_generation: Some(9),
                workbook_generation: Some(10),
            },
            vm: Some(12),
            date_lexical_value: Some("2026-08-06".into()),
            original_sst_index: Some(13),
            original_value: Some("4.25e1".into()),
        })),
    }
}

fn plain_value_cell() -> CellData {
    CellData {
        row: 8,
        col: 5,
        value: CellValue::from(12.0_f64),
        ..CellData::default()
    }
}

fn rich_string_cell() -> CellData {
    CellData {
        row: 12,
        col: 1,
        value: CellValue::from("Styled"),
        extras: Some(Box::new(CellDataExtras {
            rich_string: Some(RichSharedString {
                plain_text: "Styled".into(),
                runs: vec![RichTextRun {
                    text: "Styled".into(),
                    bold: true,
                    ..RichTextRun::default()
                }],
                ..RichSharedString::default()
            }),
            ..CellDataExtras::default()
        })),
        ..CellData::default()
    }
}

#[test]
fn cell_data_wire_json_is_pinned_before_layout_refactor() {
    let cases = [
        (
            "formula",
            fully_populated_formula_cell(),
            "{\"row\":4,\"col\":2,\"value\":42.5,\"formula\":\"SUM(A1:B2)\",\"arrayRef\":\"A1:B2\",\"styleId\":7,\"cellFormula\":{\"text\":\"SUM(A1:B2)\",\"t\":\"Array\",\"si\":3,\"ref\":\"A1:B2\",\"aca\":true,\"dt2d\":true,\"del1\":true,\"del2\":true,\"r1\":\"A1\",\"r2\":\"B2\",\"ca\":true,\"bx\":true,\"dtr\":true},\"cellMetadataIndex\":11,\"formulaResultType\":6,\"hasEmptyCachedValue\":true,\"formulaCacheProvenance\":{\"state\":\"importedCurrent\",\"forceRecalc\":true,\"advancedCalc\":true,\"formulaPreserveSpace\":true,\"valuePreserveSpace\":true,\"cachedValueKind\":6,\"cachedValuePresence\":\"nonEmpty\",\"cachedValueLexeme\":\"42.5\",\"formulaIdentityFingerprint\":\"formula-fingerprint\",\"formulaMetadataFingerprint\":\"metadata-fingerprint\",\"cachedSemanticValueFingerprint\":\"value-fingerprint\",\"ownerGeneration\":9,\"workbookGeneration\":10},\"vm\":12,\"phonetic\":true,\"dateLexicalValue\":\"2026-08-06\",\"originalSstIndex\":13,\"originalValue\":\"4.25e1\",\"projectionRole\":\"dynamicArraySource\"}",
        ),
        (
            "plain",
            plain_value_cell(),
            "{\"row\":8,\"col\":5,\"value\":12.0,\"formula\":null,\"arrayRef\":null,\"styleId\":null}",
        ),
        (
            "rich",
            rich_string_cell(),
            "{\"row\":12,\"col\":1,\"value\":\"Styled\",\"richString\":{\"plainText\":\"Styled\",\"runs\":[{\"text\":\"Styled\",\"fontName\":null,\"fontSize\":null,\"bold\":true,\"italic\":false,\"underline\":false,\"strikethrough\":false,\"color\":null,\"charset\":null,\"family\":null,\"scheme\":null}]},\"formula\":null,\"arrayRef\":null,\"styleId\":null}",
        ),
    ];

    for (name, cell, expected) in cases {
        let actual = serde_json::to_string(&cell).expect("serialize CellData");
        assert_eq!(actual, expected, "wire JSON changed for {name}: {actual}");
        let restored: CellData = serde_json::from_str(&actual).expect("deserialize CellData");
        assert_eq!(restored, cell, "round-trip changed {name} cell");
    }
}
