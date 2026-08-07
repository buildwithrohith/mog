//! Layout report for the fixed per-cell types that dominate large imports.
//!
//! Run with `--nocapture` to preserve the printed table for memory-budget
//! decisions. The assertions are deliberately generous sanity ceilings: this
//! test reports layout rather than locking an optimization to one ABI.

use std::mem::size_of;

use cell_types::CellId;
use compute_core::mirror::CellEntry;
use domain_types::{CellData, FormulaCacheProvenance};
use value_types::CellValue;

#[test]
fn report_fixed_cell_type_sizes() {
    let rows = [
        (
            "domain_types::parse_output::CellData",
            size_of::<CellData>(),
        ),
        (
            "domain_types::parse_output::FormulaCacheProvenance",
            size_of::<FormulaCacheProvenance>(),
        ),
        ("compute_core::mirror::CellEntry", size_of::<CellEntry>()),
        ("value_types::CellValue", size_of::<CellValue>()),
        (
            "value_types::Option<CellValue>",
            size_of::<Option<CellValue>>(),
        ),
        ("cell_types::CellId", size_of::<CellId>()),
    ];

    println!("=== Mog fixed type size report ===");
    println!("{:<56} {:>8}", "type", "bytes");
    println!("{}", "-".repeat(67));
    for (name, bytes) in rows {
        println!("{name:<56} {bytes:>8}");
        assert!(
            bytes < 4096,
            "unexpectedly large fixed type {name}: {bytes} bytes"
        );
    }

    const CADENCE_CELL_COUNT: usize = 1_407_921;

    let cell_value_size = size_of::<CellValue>();
    let cell_entry_size = size_of::<CellEntry>();
    let option_cell_value_size = size_of::<Option<CellValue>>();
    assert!(
        cell_value_size <= 32,
        "CellValue layout regression: {cell_value_size} bytes (expected <= 32)"
    );
    assert!(
        cell_entry_size <= 40,
        "CellEntry layout regression: {cell_entry_size} bytes (expected <= 40)"
    );
    assert!(
        option_cell_value_size <= 32,
        "Option<CellValue> layout regression: {option_cell_value_size} bytes (expected <= 32)"
    );

    let fixed_parse_bytes = size_of::<CellData>() * CADENCE_CELL_COUNT;
    println!(
        "fixed parse overhead at Cadence scale: {fixed_parse_bytes} bytes ({:.2} MB)",
        fixed_parse_bytes as f64 / 1_000_000.0
    );
}
