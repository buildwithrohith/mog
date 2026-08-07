//! Layout report for the fixed per-cell types that dominate large imports.
//!
//! Run with `--nocapture` to preserve the printed table for memory-budget
//! decisions. The assertions are deliberately generous sanity ceilings: this
//! test reports layout rather than locking an optimization to one ABI.

use std::mem::size_of;

use cell_types::CellId;
use compute_core::mirror::CellEntry;
use domain_types::{CellData, CellDataExtras, FormulaCacheProvenance};
use value_types::CellValue;

#[test]
fn report_fixed_cell_type_sizes() {
    let rows = [
        (
            "domain_types::parse_output::CellData",
            size_of::<CellData>(),
            150,
        ),
        (
            "domain_types::parse_output::CellDataExtras",
            size_of::<CellDataExtras>(),
            4096,
        ),
        (
            "domain_types::parse_output::FormulaCacheProvenance",
            size_of::<FormulaCacheProvenance>(),
            4096,
        ),
        (
            "compute_core::mirror::CellEntry",
            size_of::<CellEntry>(),
            4096,
        ),
        ("value_types::CellValue", size_of::<CellValue>(), 4096),
        ("cell_types::CellId", size_of::<CellId>(), 4096),
    ];

    println!("=== Mog fixed type size report ===");
    println!("{:<56} {:>8}", "type", "bytes");
    println!("{}", "-".repeat(67));
    for (name, bytes, ceiling) in rows {
        println!("{name:<56} {bytes:>8}");
        assert!(
            bytes < ceiling,
            "unexpectedly large fixed type {name}: {bytes} bytes (ceiling {ceiling})"
        );
    }

    const CADENCE_CELL_COUNT: usize = 1_407_921;
    let fixed_parse_bytes = size_of::<CellData>() * CADENCE_CELL_COUNT;
    println!(
        "fixed parse overhead at Cadence scale: {fixed_parse_bytes} bytes ({:.2} MB)",
        fixed_parse_bytes as f64 / 1_000_000.0
    );
}
