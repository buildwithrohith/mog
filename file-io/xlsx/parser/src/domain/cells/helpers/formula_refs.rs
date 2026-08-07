use super::a1::{col_to_letters, format_u32};

#[derive(Debug, Clone, Copy)]
struct FormulaReference {
    start: usize,
    end: usize,
    row: u32,
    col: u32,
    row_absolute: bool,
    col_absolute: bool,
}

#[derive(Debug, Clone, Copy)]
enum FormulaSegment {
    Literal { start: usize, end: usize },
    Reference(FormulaReference),
}

/// Tokenized shared-formula text reused while expanding every cell in a group.
#[derive(Debug, Default)]
pub(crate) struct FormulaReferenceTemplate {
    segments: Vec<FormulaSegment>,
}

/// Tokenize a formula into literal spans and parsed A1 references.
pub(crate) fn tokenize_formula_references(formula: &[u8]) -> FormulaReferenceTemplate {
    let mut segments = Vec::new();
    let mut literal_start = 0;
    let mut pos = 0;

    while pos < formula.len() {
        let b = formula[pos];

        // References inside string literals (double-quoted in formulas) and
        // single-quoted sheet names are copied as literal spans.
        if b == b'"' || b == b'\'' {
            pos = skip_quoted(formula, pos, b);
            continue;
        }

        let is_ref_start = (b == b'$' || b.is_ascii_uppercase())
            && (pos == 0
                || (!formula[pos - 1].is_ascii_alphanumeric() && formula[pos - 1] != b'_'));
        if is_ref_start && let Some(parsed) = try_parse_reference(&formula[pos..]) {
            let reference = FormulaReference {
                start: pos,
                end: pos + parsed.consumed,
                row: parsed.row,
                col: parsed.col,
                row_absolute: parsed.row_absolute,
                col_absolute: parsed.col_absolute,
            };
            if literal_start < pos {
                segments.push(FormulaSegment::Literal {
                    start: literal_start,
                    end: pos,
                });
            }
            segments.push(FormulaSegment::Reference(reference));
            pos = reference.end;
            literal_start = pos;
            continue;
        }

        pos += 1;
    }

    if literal_start < formula.len() {
        segments.push(FormulaSegment::Literal {
            start: literal_start,
            end: formula.len(),
        });
    }

    FormulaReferenceTemplate { segments }
}

/// Adjust A1 cell references in a formula string by row and column offsets.
///
/// This function scans a formula for A1-style cell references (e.g., `A1`, `$B$2`,
/// `AA100`) and adjusts them by the given row and column offsets, respecting
/// absolute reference markers (`$`).
///
/// # Rules
/// - `$` before column letters: column is absolute (not adjusted)
/// - `$` before row digits: row is absolute (not adjusted)
/// - References inside string literals (double-quoted) are not adjusted
/// - Sheet-qualified references like `Sheet1!A1` are handled (the A1 part is adjusted)
///
/// # Arguments
/// * `formula` - The formula text as a byte slice
/// * `row_offset` - Number of rows to shift (positive = down, negative = up)
/// * `col_offset` - Number of columns to shift (positive = right, negative = left)
///
/// # Returns
/// The adjusted formula as a String
pub fn adjust_formula_references(formula: &[u8], row_offset: i32, col_offset: i32) -> String {
    if row_offset == 0 && col_offset == 0 {
        return std::str::from_utf8(formula)
            .expect("worksheet formula XML text was validated as UTF-8 at the archive boundary")
            .to_owned();
    }

    let template = tokenize_formula_references(formula);
    expand_formula_references(formula, &template, row_offset, col_offset)
}

/// Expand one shared-formula cell from a previously tokenized master formula.
pub(crate) fn expand_formula_references(
    formula: &[u8],
    template: &FormulaReferenceTemplate,
    row_offset: i32,
    col_offset: i32,
) -> String {
    let formula_text = std::str::from_utf8(formula)
        .expect("worksheet formula XML text was validated as UTF-8 at the archive boundary");
    let capacity = template
        .segments
        .iter()
        .map(|segment| match segment {
            FormulaSegment::Literal { start, end } => end - start,
            FormulaSegment::Reference(reference) => {
                adjusted_reference_len(*reference, row_offset, col_offset)
            }
        })
        .sum();
    let mut result = String::with_capacity(capacity);

    for segment in &template.segments {
        match segment {
            FormulaSegment::Literal { start, end } => result.push_str(&formula_text[*start..*end]),
            FormulaSegment::Reference(reference) => append_reference(
                &mut result,
                formula_text,
                *reference,
                row_offset,
                col_offset,
            ),
        }
    }

    result
}

fn skip_quoted(formula: &[u8], start: usize, quote: u8) -> usize {
    let mut pos = start + 1;
    while pos < formula.len() {
        if formula[pos] == quote {
            return pos + 1;
        }
        pos += 1;
    }
    formula.len()
}

#[derive(Debug, Clone, Copy)]
struct ParsedReference {
    consumed: usize,
    row: u32,
    col: u32,
    row_absolute: bool,
    col_absolute: bool,
}

/// Try to parse a single A1 reference at the start of `input`.
fn try_parse_reference(input: &[u8]) -> Option<ParsedReference> {
    let mut pos = 0;

    // Check for $ before column
    let col_absolute = if pos < input.len() && input[pos] == b'$' {
        pos += 1;
        true
    } else {
        false
    };

    // Parse column letters (must have at least one)
    let col_start = pos;
    let mut col_val: u32 = 0;
    while pos < input.len() && input[pos].is_ascii_uppercase() {
        col_val = col_val
            .saturating_mul(26)
            .saturating_add((input[pos] - b'A' + 1) as u32);
        pos += 1;
    }

    if pos == col_start || col_val == 0 {
        return None;
    }
    let col = col_val - 1; // Convert to 0-indexed

    // Check for $ before row
    let row_absolute = if pos < input.len() && input[pos] == b'$' {
        pos += 1;
        true
    } else {
        false
    };

    // Parse row digits (must have at least one)
    let row_start = pos;
    let mut row_val: u32 = 0;
    while pos < input.len() && input[pos].is_ascii_digit() {
        row_val = row_val
            .saturating_mul(10)
            .saturating_add((input[pos] - b'0') as u32);
        pos += 1;
    }

    if pos == row_start || row_val == 0 {
        return None;
    }
    let row = row_val - 1; // Convert to 0-indexed

    // Make sure the character after the reference is not alphanumeric
    // (to avoid partial matches like "A1B" being treated as ref "A1" + "B")
    if pos < input.len() && (input[pos].is_ascii_alphanumeric() || input[pos] == b'_') {
        return None;
    }

    Some(ParsedReference {
        consumed: pos,
        row,
        col,
        row_absolute,
        col_absolute,
    })
}

fn adjusted_coordinates(
    reference: FormulaReference,
    row_offset: i32,
    col_offset: i32,
) -> Option<(u32, u32)> {
    let col = if reference.col_absolute {
        reference.col
    } else {
        let adjusted = reference.col as i32 + col_offset;
        if !(0..=16383).contains(&adjusted) {
            return None;
        }
        adjusted as u32
    };
    let row = if reference.row_absolute {
        reference.row
    } else {
        let adjusted = reference.row as i32 + row_offset;
        if !(0..=1048575).contains(&adjusted) {
            return None;
        }
        adjusted as u32
    };
    Some((row, col))
}

fn decimal_len(mut value: u32) -> usize {
    let mut len = 1;
    while value >= 10 {
        value /= 10;
        len += 1;
    }
    len
}

fn adjusted_reference_len(reference: FormulaReference, row_offset: i32, col_offset: i32) -> usize {
    let Some((row, col)) = adjusted_coordinates(reference, row_offset, col_offset) else {
        return reference.end - reference.start;
    };
    reference.col_absolute as usize
        + col_to_letters(col)
            .iter()
            .filter(|&&letter| letter != 0)
            .count()
        + reference.row_absolute as usize
        + decimal_len(row + 1)
}

fn append_reference(
    result: &mut String,
    formula: &str,
    reference: FormulaReference,
    row_offset: i32,
    col_offset: i32,
) {
    let Some((row, col)) = adjusted_coordinates(reference, row_offset, col_offset) else {
        result.push_str(&formula[reference.start..reference.end]);
        return;
    };

    if reference.col_absolute {
        result.push('$');
    }
    for &letter in &col_to_letters(col) {
        if letter != 0 {
            result.push(letter as char);
        }
    }
    if reference.row_absolute {
        result.push('$');
    }
    let mut row_buffer = [0u8; 10];
    result.push_str(format_u32(row + 1, &mut row_buffer));
}

#[cfg(test)]
mod tests {
    use super::{
        adjust_formula_references, expand_formula_references, tokenize_formula_references,
    };

    #[test]
    fn cached_segments_expand_references_without_touching_quoted_text() {
        let formula = br#"SUM(A1,$B$2,Sheet1!C3,"E4",'Sheet 1'!D5)"#;
        let template = tokenize_formula_references(formula);

        assert_eq!(
            expand_formula_references(formula, &template, 1, 2),
            r#"SUM(C2,$B$2,Sheet1!E4,"E4",'Sheet 1'!F6)"#
        );
    }

    #[test]
    fn cached_expansion_matches_the_public_adjustment_helper_at_boundaries() {
        let formula = b"A1+B1+C1";
        let template = tokenize_formula_references(formula);

        assert_eq!(
            expand_formula_references(formula, &template, -1, -1),
            "A1+B1+C1"
        );
        assert_eq!(
            expand_formula_references(formula, &template, 1, 1),
            adjust_formula_references(formula, 1, 1)
        );
    }
}
