//! Binary wire codecs for bulk range reads.
//!
//! Every payload is self-describing and starts with version byte `1`, followed
//! by a payload kind. Rectangular payloads then carry one shared header:
//! `start_row`, `start_col`, `rows`, and `cols` as little-endian `u32`s. Dense
//! values are row-major. Mixed ranges put `(type, run_length)` records in a
//! run-length type stream before the values, so a numeric block costs one type
//! record plus 8 bytes per cell rather than one type byte per cell. Nested
//! arrays retain the same tagged value format recursively.
//!
//! ```text
//! [version][kind][mode][encoding][start_row][start_col][rows][cols] ...
//! ```
//!
//! The format payload uses the existing `compute-wire` palette binary protocol.
//! Query payloads use the same rectangle header and type stream when all cells
//! fill the requested rectangle; sparse queries retain row/column varints. A
//! query cell stores a compact UUID, typed value, and optional-field mask for
//! formula, formatted text, format, and hyperlink. This keeps the query wire
//! additive and lossless while avoiding per-cell JSON field names.

use super::cell_semantics::{RangeBinaryEncoding, RangeBinaryMeta};
use super::queries::QueryRangeBinaryMeta;
use crate::snapshot::{RangeCellData, RangeQueryResult, ViewportMerge};
use domain_types::CellFormat;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use value_types::{
    CellArray, CellControl, CellControlType, CellError, CellImage, CellImageSizing, CellValue,
};

const VALUE_TAG_NULL: u8 = 0;
const VALUE_TAG_NUMBER: u8 = 1;
const VALUE_TAG_TEXT: u8 = 2;
const VALUE_TAG_BOOLEAN: u8 = 3;
const VALUE_TAG_ERROR: u8 = 4;
const VALUE_TAG_ARRAY: u8 = 5;
const VALUE_TAG_CONTROL: u8 = 6;
const VALUE_TAG_IMAGE: u8 = 7;

pub(crate) const RANGE_BINARY_VERSION: u8 = 1;
const KIND_VALUES: u8 = 0;
const KIND_FORMATS: u8 = 1;
const KIND_QUERY: u8 = 2;
const MODE_DENSE: u8 = 0;
const MODE_SPARSE: u8 = 1;
const ENCODING_F64: u8 = 0;
const ENCODING_MIXED: u8 = 1;
const ENCODING_FORMAT_PALETTE: u8 = 2;
const OPTIONAL_FORMULA: u8 = 1 << 0;
const OPTIONAL_FORMATTED: u8 = 1 << 1;
const OPTIONAL_FORMAT: u8 = 1 << 2;
const OPTIONAL_HYPERLINK: u8 = 1 << 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecodeError(String);

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DecodeError {}

fn error(message: impl Into<String>) -> DecodeError {
    DecodeError(message.into())
}

fn checked_len(rows: u32, cols: u32) -> Result<usize, DecodeError> {
    (rows as usize)
        .checked_mul(cols as usize)
        .ok_or_else(|| error("range binary shape overflows usize"))
}

fn push_u32(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(
        &u32::try_from(value)
            .expect("range binary payload exceeds the u32 wire length")
            .to_le_bytes(),
    );
}

fn push_var_u32(out: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn push_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    push_u32(out, bytes.len());
    out.extend_from_slice(bytes);
}

fn push_var_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    push_var_u32(
        out,
        u32::try_from(bytes.len()).expect("range binary string exceeds the u32 wire length"),
    );
    out.extend_from_slice(bytes);
}

fn push_string(out: &mut Vec<u8>, value: &str) {
    push_bytes(out, value.as_bytes());
}

fn push_var_string(out: &mut Vec<u8>, value: &str) {
    push_var_bytes(out, value.as_bytes());
}

fn encoding_code(encoding: RangeBinaryEncoding) -> u8 {
    match encoding {
        RangeBinaryEncoding::F64Le => ENCODING_F64,
        RangeBinaryEncoding::MixedLe => ENCODING_MIXED,
        RangeBinaryEncoding::FormatPalette => ENCODING_FORMAT_PALETTE,
    }
}

fn encoding_from_code(code: u8) -> Result<RangeBinaryEncoding, DecodeError> {
    match code {
        ENCODING_F64 => Ok(RangeBinaryEncoding::F64Le),
        ENCODING_MIXED => Ok(RangeBinaryEncoding::MixedLe),
        ENCODING_FORMAT_PALETTE => Ok(RangeBinaryEncoding::FormatPalette),
        _ => Err(error(format!("range binary encoding {code} is invalid"))),
    }
}

fn push_rect_header(
    out: &mut Vec<u8>,
    kind: u8,
    mode: u8,
    encoding: RangeBinaryEncoding,
    start_row: u32,
    start_col: u32,
    rows: u32,
    cols: u32,
) {
    out.push(RANGE_BINARY_VERSION);
    out.push(kind);
    out.push(mode);
    out.push(encoding_code(encoding));
    out.extend_from_slice(&start_row.to_le_bytes());
    out.extend_from_slice(&start_col.to_le_bytes());
    out.extend_from_slice(&rows.to_le_bytes());
    out.extend_from_slice(&cols.to_le_bytes());
}

fn read_rect_header_any(
    reader: &mut Reader<'_>,
    expected_kind: u8,
    metadata: RangeBinaryMeta,
) -> Result<(u8, RangeBinaryEncoding), DecodeError> {
    if reader.byte()? != RANGE_BINARY_VERSION {
        return Err(error("range binary version is invalid"));
    }
    if reader.byte()? != expected_kind {
        return Err(error("range binary payload kind is invalid"));
    }
    let mode = reader.byte()?;
    let encoding = encoding_from_code(reader.byte()?)?;
    let start_row = reader.u32()?;
    let start_col = reader.u32()?;
    let rows = reader.u32()?;
    let cols = reader.u32()?;
    if (start_row, start_col, rows, cols)
        != (
            metadata.start_row,
            metadata.start_col,
            metadata.rows,
            metadata.cols,
        )
    {
        return Err(error("range binary rectangle does not match metadata"));
    }
    Ok((mode, encoding))
}

fn read_rect_header(
    reader: &mut Reader<'_>,
    expected_kind: u8,
    expected_mode: u8,
    metadata: RangeBinaryMeta,
) -> Result<RangeBinaryEncoding, DecodeError> {
    let (mode, encoding) = read_rect_header_any(reader, expected_kind, metadata)?;
    if mode != expected_mode {
        return Err(error("range binary payload mode is invalid"));
    }
    Ok(encoding)
}

fn push_optional_string(out: &mut Vec<u8>, value: Option<&str>) {
    match value {
        Some(value) => push_bytes(out, value.as_bytes()),
        None => out.extend_from_slice(&u32::MAX.to_le_bytes()),
    }
}

fn read_error(message: impl Into<String>) -> DecodeError {
    error(message)
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    limit: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            limit: bytes.len(),
        }
    }

    fn with_limit(bytes: &'a [u8], limit: usize) -> Result<Self, DecodeError> {
        if limit > bytes.len() {
            return Err(error("range binary limit exceeds buffer"));
        }
        Ok(Self {
            bytes,
            pos: 0,
            limit,
        })
    }

    fn remaining(&self) -> usize {
        self.limit.saturating_sub(self.pos)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| error("range binary cursor overflows usize"))?;
        if end > self.limit {
            return Err(error("range binary payload is truncated"));
        }
        let bytes = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }

    fn byte(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn f64(&mut self) -> Result<f64, DecodeError> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    fn var_u32(&mut self) -> Result<u32, DecodeError> {
        let mut value = 0u32;
        let mut shift = 0u32;
        for _ in 0..5 {
            let byte = self.byte()?;
            let part = u32::from(byte & 0x7f);
            value |= part
                .checked_shl(shift)
                .ok_or_else(|| error("range binary varint overflows u32"))?;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
        }
        Err(error("range binary varint is too long"))
    }

    fn bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.u32()? as usize;
        self.take(len)
    }

    fn var_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        let len = self.var_u32()? as usize;
        self.take(len)
    }

    fn string(&mut self) -> Result<String, DecodeError> {
        String::from_utf8(self.bytes()?.to_vec())
            .map_err(|_| error("range binary string is not valid UTF-8"))
    }

    fn var_string(&mut self) -> Result<String, DecodeError> {
        String::from_utf8(self.var_bytes()?.to_vec())
            .map_err(|_| error("range binary string is not valid UTF-8"))
    }

    fn optional_string(&mut self) -> Result<Option<String>, DecodeError> {
        let len = self.u32()?;
        if len == u32::MAX {
            return Ok(None);
        }
        String::from_utf8(self.take(len as usize)?.to_vec())
            .map(Some)
            .map_err(|_| error("range binary optional string is not valid UTF-8"))
    }

    fn expect(&mut self, expected: &[u8]) -> Result<(), DecodeError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(error("range binary magic/version is invalid"))
        }
    }
}

fn cell_value_tag(value: &CellValue) -> u8 {
    match value {
        CellValue::Null => VALUE_TAG_NULL,
        CellValue::Number(_) => VALUE_TAG_NUMBER,
        CellValue::Text(_) => VALUE_TAG_TEXT,
        CellValue::Boolean(_) => VALUE_TAG_BOOLEAN,
        CellValue::Error(_, _) => VALUE_TAG_ERROR,
        CellValue::Array(_) => VALUE_TAG_ARRAY,
        CellValue::Control(_) => VALUE_TAG_CONTROL,
        CellValue::Image(_) => VALUE_TAG_IMAGE,
    }
}

fn write_cell_payload(out: &mut Vec<u8>, value: &CellValue) {
    match value {
        CellValue::Null => {}
        CellValue::Number(value) => out.extend_from_slice(&value.get().to_le_bytes()),
        CellValue::Text(value) => push_string(out, value),
        CellValue::Boolean(value) => out.push(u8::from(*value)),
        CellValue::Error(kind, message) => {
            out.push(cell_error_code(*kind));
            match message {
                Some(message) => {
                    out.push(1);
                    push_string(out, message);
                }
                None => out.push(0),
            }
        }
        CellValue::Array(array) => {
            push_u32(out, array.rows());
            push_u32(out, array.cols());
            for value in array.iter() {
                write_cell_value(out, value);
            }
        }
        CellValue::Control(control) => {
            out.push(control_type_code(control.control_type));
            out.push(u8::from(control.checked));
            out.push(u8::from(control.value));
        }
        CellValue::Image(image) => {
            push_string(out, &image.source);
            match &image.alt_text {
                Some(value) => {
                    out.push(1);
                    push_string(out, value);
                }
                None => out.push(0),
            }
            out.push(image_sizing_code(image.sizing));
            match image.height {
                Some(value) => {
                    out.push(1);
                    out.extend_from_slice(&value.to_le_bytes());
                }
                None => out.push(0),
            }
            match image.width {
                Some(value) => {
                    out.push(1);
                    out.extend_from_slice(&value.to_le_bytes());
                }
                None => out.push(0),
            }
        }
    }
}

fn write_cell_value(out: &mut Vec<u8>, value: &CellValue) {
    out.push(cell_value_tag(value));
    write_cell_payload(out, value);
}

fn write_typed_value(out: &mut Vec<u8>, value: &CellValue, tag: u8) {
    debug_assert_eq!(cell_value_tag(value), tag);
    write_cell_payload(out, value);
}

fn read_cell_payload(
    reader: &mut Reader<'_>,
    tag: u8,
    depth: u8,
) -> Result<CellValue, DecodeError> {
    if depth > 64 {
        return Err(error("range binary nested value depth exceeds 64"));
    }
    match tag {
        VALUE_TAG_NULL => Ok(CellValue::Null),
        VALUE_TAG_NUMBER => Ok(CellValue::number(reader.f64()?)),
        VALUE_TAG_TEXT => Ok(CellValue::Text(Arc::from(reader.string()?))),
        VALUE_TAG_BOOLEAN => Ok(CellValue::Boolean(reader.byte()? != 0)),
        VALUE_TAG_ERROR => {
            let kind = cell_error_from_code(reader.byte()?)?;
            let message = if reader.byte()? != 0 {
                Some(Arc::from(reader.string()?))
            } else {
                None
            };
            Ok(CellValue::Error(kind, message))
        }
        VALUE_TAG_ARRAY => {
            let rows = reader.u32()? as usize;
            let cols = reader.u32()? as usize;
            let len = rows
                .checked_mul(cols)
                .ok_or_else(|| error("range binary array shape overflows usize"))?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(read_cell_value(reader, depth + 1)?);
            }
            let array = CellArray::try_new(values, cols)
                .map_err(|array_error| error(format!("range binary array: {array_error}")))?;
            Ok(CellValue::Array(Arc::new(array)))
        }
        VALUE_TAG_CONTROL => {
            let control_type = control_type_from_code(reader.byte()?)?;
            Ok(CellValue::Control(CellControl {
                control_type,
                checked: reader.byte()? != 0,
                value: reader.byte()? != 0,
            }))
        }
        VALUE_TAG_IMAGE => {
            let source = reader.string()?;
            let alt_text = if reader.byte()? != 0 {
                Some(Arc::from(reader.string()?))
            } else {
                None
            };
            let sizing = image_sizing_from_code(reader.byte()?)?;
            let height = if reader.byte()? != 0 {
                Some(reader.u32()?)
            } else {
                None
            };
            let width = if reader.byte()? != 0 {
                Some(reader.u32()?)
            } else {
                None
            };
            Ok(CellValue::Image(Arc::new(CellImage::new(
                source, alt_text, sizing, height, width,
            ))))
        }
        tag => Err(error(format!("range binary value tag {tag} is invalid"))),
    }
}

fn read_cell_value(reader: &mut Reader<'_>, depth: u8) -> Result<CellValue, DecodeError> {
    let tag = reader.byte()?;
    read_cell_payload(reader, tag, depth)
}

fn read_typed_value(reader: &mut Reader<'_>, tag: u8, depth: u8) -> Result<CellValue, DecodeError> {
    read_cell_payload(reader, tag, depth)
}

fn is_value_tag(tag: u8) -> bool {
    matches!(
        tag,
        VALUE_TAG_NULL
            | VALUE_TAG_NUMBER
            | VALUE_TAG_TEXT
            | VALUE_TAG_BOOLEAN
            | VALUE_TAG_ERROR
            | VALUE_TAG_ARRAY
            | VALUE_TAG_CONTROL
            | VALUE_TAG_IMAGE
    )
}

fn push_type_runs<'a, I>(out: &mut Vec<u8>, values: I)
where
    I: IntoIterator<Item = &'a CellValue>,
{
    let mut runs = Vec::<(u8, u32)>::new();
    for value in values {
        let tag = cell_value_tag(value);
        if let Some((last_tag, count)) = runs.last_mut()
            && *last_tag == tag
            && *count < u32::MAX
        {
            *count += 1;
        } else {
            runs.push((tag, 1));
        }
    }
    push_u32(out, runs.len());
    for (tag, count) in runs {
        out.push(tag);
        push_var_u32(out, count);
    }
}

fn read_type_runs(reader: &mut Reader<'_>, value_count: usize) -> Result<Vec<u8>, DecodeError> {
    let run_count = reader.u32()? as usize;
    let mut tags = Vec::with_capacity(value_count);
    for _ in 0..run_count {
        let tag = reader.byte()?;
        if !is_value_tag(tag) {
            return Err(error(format!("range binary type tag {tag} is invalid")));
        }
        let count = reader.var_u32()? as usize;
        if count == 0 {
            return Err(error("range binary type run is empty"));
        }
        let next_len = tags
            .len()
            .checked_add(count)
            .ok_or_else(|| error("range binary type stream overflows usize"))?;
        if next_len > value_count {
            return Err(error("range binary type stream exceeds cell count"));
        }
        tags.extend(std::iter::repeat_n(tag, count));
    }
    if tags.len() != value_count {
        return Err(error("range binary type stream does not cover cell count"));
    }
    Ok(tags)
}

fn cell_error_code(error: CellError) -> u8 {
    match error {
        CellError::Div0 => 0,
        CellError::Na => 1,
        CellError::Name => 2,
        CellError::Null => 3,
        CellError::Num => 4,
        CellError::Ref => 5,
        CellError::Value => 6,
        CellError::Spill => 7,
        CellError::Calc => 8,
        CellError::GettingData => 9,
        CellError::Circ => 10,
    }
}

fn cell_error_from_code(code: u8) -> Result<CellError, DecodeError> {
    match code {
        0 => Ok(CellError::Div0),
        1 => Ok(CellError::Na),
        2 => Ok(CellError::Name),
        3 => Ok(CellError::Null),
        4 => Ok(CellError::Num),
        5 => Ok(CellError::Ref),
        6 => Ok(CellError::Value),
        7 => Ok(CellError::Spill),
        8 => Ok(CellError::Calc),
        9 => Ok(CellError::GettingData),
        10 => Ok(CellError::Circ),
        _ => Err(error(format!(
            "range binary cell error code {code} is invalid"
        ))),
    }
}

fn control_type_code(control_type: CellControlType) -> u8 {
    match control_type {
        CellControlType::Checkbox => 0,
        _ => 0,
    }
}

fn control_type_from_code(code: u8) -> Result<CellControlType, DecodeError> {
    match code {
        0 => Ok(CellControlType::Checkbox),
        _ => Err(error(format!(
            "range binary control type code {code} is invalid"
        ))),
    }
}

fn image_sizing_code(sizing: CellImageSizing) -> u8 {
    match sizing {
        CellImageSizing::Fit => 0,
        CellImageSizing::Fill => 1,
        CellImageSizing::Original => 2,
        CellImageSizing::Custom => 3,
    }
}

fn image_sizing_from_code(code: u8) -> Result<CellImageSizing, DecodeError> {
    match code {
        0 => Ok(CellImageSizing::Fit),
        1 => Ok(CellImageSizing::Fill),
        2 => Ok(CellImageSizing::Original),
        3 => Ok(CellImageSizing::Custom),
        _ => Err(error(format!(
            "range binary image sizing code {code} is invalid"
        ))),
    }
}

/// Encode a dense 2D value range and its bridge metadata.
pub(crate) fn encode_values(
    start_row: u32,
    start_col: u32,
    values: &[Vec<CellValue>],
) -> (Vec<u8>, RangeBinaryMeta) {
    let rows = values.len();
    let cols = values.first().map_or(0, Vec::len);
    let rectangular = values.iter().all(|row| row.len() == cols);
    let numeric = rectangular
        && !values.is_empty()
        && values
            .iter()
            .flatten()
            .all(|value| matches!(value, CellValue::Number(_)));
    let encoding = if numeric {
        RangeBinaryEncoding::F64Le
    } else {
        RangeBinaryEncoding::MixedLe
    };
    let rows = u32::try_from(rows).expect("range binary row count exceeds u32");
    let cols = u32::try_from(cols).expect("range binary column count exceeds u32");
    let mut bytes = Vec::new();
    push_rect_header(
        &mut bytes,
        KIND_VALUES,
        MODE_DENSE,
        encoding,
        start_row,
        start_col,
        rows,
        cols,
    );
    push_type_runs(&mut bytes, values.iter().flatten());
    for value in values.iter().flatten() {
        write_typed_value(&mut bytes, value, cell_value_tag(value));
    }
    (
        bytes,
        RangeBinaryMeta {
            start_row,
            start_col,
            rows,
            cols,
            encoding,
        },
    )
}

/// Decode a dense value range. This reference decoder is also used by Rust
/// round-trip tests; the production consumer is `kernel/src/bridges/compute`.
pub(crate) fn decode_values(
    bytes: &[u8],
    metadata: RangeBinaryMeta,
) -> Result<Vec<Vec<CellValue>>, DecodeError> {
    let count = checked_len(metadata.rows, metadata.cols)?;
    let mut reader = Reader::new(bytes);
    let encoding = read_rect_header(&mut reader, KIND_VALUES, MODE_DENSE, metadata)?;
    if encoding != metadata.encoding {
        return Err(error("range binary value encoding does not match metadata"));
    }
    let tags = read_type_runs(&mut reader, count)?;
    if encoding == RangeBinaryEncoding::F64Le && tags.iter().any(|&tag| tag != VALUE_TAG_NUMBER) {
        return Err(error("range binary f64 payload has a non-numeric type run"));
    }
    let mut values = Vec::with_capacity(count);
    for tag in tags {
        values.push(read_typed_value(&mut reader, tag, 0)?);
    }
    if reader.remaining() != 0 {
        return Err(error("range binary dense payload has trailing bytes"));
    }
    Ok(values
        .chunks(metadata.cols as usize)
        .map(<[CellValue]>::to_vec)
        .collect())
}

#[derive(Debug, Clone)]
enum FormatSlot {
    None,
    Palette { index: u16, fallback: Vec<u8> },
    Inline(Vec<u8>),
}

fn palette_is_lossless(palette_bytes: &[u8], palette: &[CellFormat]) -> bool {
    compute_wire::palette_binary::deserialize_palette_binary(palette_bytes)
        .map(|(_, decoded)| decoded == palette)
        .unwrap_or(false)
}

/// Encode a format slot. Dense format payloads use fixed `u32` byte lengths;
/// query inline formats use the varint length consumed by the query decoder.
fn encode_format_slot(out: &mut Vec<u8>, slot: &FormatSlot, variable_length: bool) {
    match slot {
        FormatSlot::None => out.push(0),
        FormatSlot::Palette { index, .. } => {
            out.push(1);
            out.extend_from_slice(&index.to_le_bytes());
        }
        FormatSlot::Inline(bytes) => {
            out.push(2);
            if variable_length {
                push_var_bytes(out, bytes);
            } else {
                push_bytes(out, bytes);
            }
        }
    }
}

fn prepare_format_slots<I>(formats: I) -> (Vec<FormatSlot>, Vec<CellFormat>, Vec<u8>)
where
    I: IntoIterator<Item = Option<(CellFormat, Vec<u8>)>>,
{
    let mut slots = Vec::new();
    let mut palette = Vec::new();
    let mut palette_indices = HashMap::<CellFormat, u16>::new();
    for format in formats {
        let Some((format, fallback)) = format else {
            slots.push(FormatSlot::None);
            continue;
        };
        if let Some(&index) = palette_indices.get(&format) {
            slots.push(FormatSlot::Palette { index, fallback });
        } else if palette.len() < u16::MAX as usize {
            let index = u16::try_from(palette.len()).expect("format palette index exceeds u16");
            palette_indices.insert(format.clone(), index);
            palette.push(format);
            slots.push(FormatSlot::Palette { index, fallback });
        } else {
            slots.push(FormatSlot::Inline(fallback));
        }
    }

    let palette_bytes = compute_wire::palette_binary::serialize_palette_binary(&palette, 0);
    if !palette.is_empty() && !palette_is_lossless(&palette_bytes, &palette) {
        let fallback_slots = slots
            .into_iter()
            .map(|slot| match slot {
                FormatSlot::Palette { fallback, .. } => FormatSlot::Inline(fallback),
                other => other,
            })
            .collect();
        (fallback_slots, Vec::new(), Vec::new())
    } else {
        (slots, palette, palette_bytes)
    }
}

/// Encode a dense format range as palette indices plus the existing palette
/// protocol. `None` slots decode as the JSON sibling's default format.
pub(crate) fn encode_formats(
    start_row: u32,
    start_col: u32,
    rows: &[Vec<Option<CellFormat>>],
) -> (Vec<u8>, RangeBinaryMeta) {
    let row_count = rows.len();
    let col_count = rows.first().map_or(0, Vec::len);
    let formats = rows.iter().flatten().map(|format| {
        format.clone().map(|format| {
            let json = serde_json::to_vec(&format).expect("CellFormat must serialize");
            (format, json)
        })
    });
    let (slots, _palette, palette_bytes) = prepare_format_slots(formats);
    let mut bytes = Vec::new();
    push_rect_header(
        &mut bytes,
        KIND_FORMATS,
        MODE_DENSE,
        RangeBinaryEncoding::FormatPalette,
        start_row,
        start_col,
        u32::try_from(row_count).expect("format row count exceeds u32"),
        u32::try_from(col_count).expect("format column count exceeds u32"),
    );
    push_u32(&mut bytes, slots.len());
    push_u32(&mut bytes, palette_bytes.len());
    for slot in &slots {
        encode_format_slot(&mut bytes, slot, false);
    }
    bytes.extend_from_slice(&palette_bytes);
    (
        bytes,
        RangeBinaryMeta {
            start_row,
            start_col,
            rows: u32::try_from(row_count).expect("format row count exceeds u32"),
            cols: u32::try_from(col_count).expect("format column count exceeds u32"),
            encoding: RangeBinaryEncoding::FormatPalette,
        },
    )
}

fn decode_palette(bytes: &[u8]) -> Result<Vec<CellFormat>, DecodeError> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    compute_wire::palette_binary::deserialize_palette_binary(bytes)
        .map(|(_, formats)| formats)
        .map_err(|palette_error| error(format!("range binary format palette: {palette_error}")))
}

fn decode_format_slots(
    reader: &mut Reader<'_>,
    count: usize,
) -> Result<Vec<FormatSlot>, DecodeError> {
    let mut slots = Vec::with_capacity(count);
    for _ in 0..count {
        slots.push(match reader.byte()? {
            0 => FormatSlot::None,
            1 => FormatSlot::Palette {
                index: reader.u16()?,
                fallback: Vec::new(),
            },
            2 => FormatSlot::Inline(reader.bytes()?.to_vec()),
            kind => {
                return Err(error(format!(
                    "range binary format slot kind {kind} is invalid"
                )));
            }
        });
    }
    Ok(slots)
}

fn resolve_format_slot(
    slot: FormatSlot,
    palette: &[CellFormat],
) -> Result<CellFormat, DecodeError> {
    match slot {
        FormatSlot::None => Ok(CellFormat::default()),
        FormatSlot::Palette { index, .. } => {
            palette.get(index as usize).cloned().ok_or_else(|| {
                error(format!(
                    "range binary format palette index {index} is invalid"
                ))
            })
        }
        FormatSlot::Inline(bytes) => serde_json::from_slice(&bytes)
            .map_err(|json_error| error(format!("range binary inline format: {json_error}"))),
    }
}

/// Decode a dense format range.
pub(crate) fn decode_formats(
    bytes: &[u8],
    metadata: RangeBinaryMeta,
) -> Result<Vec<Vec<CellFormat>>, DecodeError> {
    let count = checked_len(metadata.rows, metadata.cols)?;
    let mut header = Reader::new(bytes);
    let encoding = read_rect_header(&mut header, KIND_FORMATS, MODE_DENSE, metadata)?;
    if encoding != metadata.encoding || encoding != RangeBinaryEncoding::FormatPalette {
        return Err(error(
            "range binary format encoding does not match metadata",
        ));
    }
    let encoded_count = header.u32()? as usize;
    let palette_len = header.u32()? as usize;
    if encoded_count != count {
        return Err(error(format!(
            "range binary format count {encoded_count} != metadata count {count}"
        )));
    }
    let palette_start = bytes
        .len()
        .checked_sub(palette_len)
        .ok_or_else(|| error("range binary format palette length is invalid"))?;
    if palette_start < header.pos {
        return Err(error("range binary format slots overlap palette"));
    }
    header.limit = palette_start;
    let slots = decode_format_slots(&mut header, count)?;
    if header.pos != palette_start {
        return Err(error("range binary format slots have trailing bytes"));
    }
    let palette = decode_palette(&bytes[palette_start..])?;
    let formats = slots
        .into_iter()
        .map(|slot| resolve_format_slot(slot, &palette))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(formats
        .chunks(metadata.cols as usize)
        .map(<[CellFormat]>::to_vec)
        .collect())
}

fn encode_uuid(out: &mut Vec<u8>, cell_id: &str) {
    if cell_id.is_empty() {
        out.push(0);
    } else if let Ok(uuid) = Uuid::parse_str(cell_id) {
        out.push(1);
        out.extend_from_slice(uuid.as_bytes());
    } else {
        out.push(2);
        push_var_string(out, cell_id);
    }
}

fn decode_uuid(reader: &mut Reader<'_>) -> Result<String, DecodeError> {
    match reader.byte()? {
        0 => Ok(String::new()),
        1 => Ok(Uuid::from_bytes(reader.take(16)?.try_into().unwrap()).to_string()),
        2 => reader.var_string(),
        kind => Err(error(format!("range binary UUID kind {kind} is invalid"))),
    }
}

fn encode_query_format_slots(cells: &[RangeCellData]) -> (Vec<FormatSlot>, Vec<u8>) {
    let formats = cells.iter().map(|cell| {
        cell.format.as_ref().map(|format| {
            let fallback = serde_json::to_vec(format).expect("JSON value must serialize");
            serde_json::from_value::<CellFormat>(format.clone())
                .map(|format| (format, fallback.clone()))
                .unwrap_or_else(|_| (CellFormat::default(), fallback))
        })
    });
    let (mut slots, palette, palette_bytes) = prepare_format_slots(formats);
    // `CellFormat::default()` is only a carrier for values that failed to
    // parse. Those slots must remain inline rather than becoming `{}`.
    for (slot, cell) in slots.iter_mut().zip(cells) {
        if cell
            .format
            .as_ref()
            .is_some_and(|format| serde_json::from_value::<CellFormat>(format.clone()).is_err())
        {
            let fallback = match slot {
                FormatSlot::Palette { fallback, .. } => Some(fallback.clone()),
                _ => None,
            };
            if let Some(fallback) = fallback {
                *slot = FormatSlot::Inline(fallback);
            }
        }
    }
    let _ = palette;
    (slots, palette_bytes)
}

fn encode_optional_query_fields(out: &mut Vec<u8>, cell: &RangeCellData, slots: &FormatSlot) {
    let mut mask = 0;
    if cell.formula.is_some() {
        mask |= OPTIONAL_FORMULA;
    }
    if cell.formatted.is_some() {
        mask |= OPTIONAL_FORMATTED;
    }
    if !matches!(slots, FormatSlot::None) {
        mask |= OPTIONAL_FORMAT;
    }
    if cell.hyperlink_url.is_some() {
        mask |= OPTIONAL_HYPERLINK;
    }
    out.push(mask);
    if let Some(formula) = &cell.formula {
        push_var_string(out, formula);
    }
    if let Some(formatted) = &cell.formatted {
        push_var_string(out, formatted);
    }
    if mask & OPTIONAL_FORMAT != 0 {
        encode_format_slot(out, slots, true);
    }
    if let Some(hyperlink) = &cell.hyperlink_url {
        push_var_string(out, hyperlink);
    }
}

/// Return the most compact value encoding applicable to sparse query cells.
pub(crate) fn query_value_encoding(cells: &[RangeCellData]) -> RangeBinaryEncoding {
    if !cells.is_empty()
        && cells
            .iter()
            .all(|cell| matches!(cell.value, CellValue::Number(_)))
    {
        RangeBinaryEncoding::F64Le
    } else {
        RangeBinaryEncoding::MixedLe
    }
}

fn is_dense_query(result: &RangeQueryResult, metadata: QueryRangeBinaryMeta) -> bool {
    let Some(expected_count) = (metadata.rows as usize).checked_mul(metadata.cols as usize) else {
        return false;
    };
    if expected_count == 0 || result.cells.len() != expected_count {
        return false;
    }
    result.cells.iter().enumerate().all(|(index, cell)| {
        let row_offset = (index / metadata.cols as usize) as u32;
        let col_offset = (index % metadata.cols as usize) as u32;
        cell.row == metadata.start_row.saturating_add(row_offset)
            && cell.col == metadata.start_col.saturating_add(col_offset)
    })
}

/// Encode a sparse `query_range` result while leaving the JSON sibling intact.
pub(crate) fn encode_query_range(
    result: &RangeQueryResult,
    metadata: QueryRangeBinaryMeta,
) -> (Vec<u8>, QueryRangeBinaryMeta) {
    let (format_slots, palette_bytes) = encode_query_format_slots(&result.cells);
    let encoding = query_value_encoding(&result.cells);
    let dense = is_dense_query(result, metadata);
    let mode = if dense { MODE_DENSE } else { MODE_SPARSE };
    let mut bytes = Vec::new();
    push_rect_header(
        &mut bytes,
        KIND_QUERY,
        mode,
        encoding,
        metadata.start_row,
        metadata.start_col,
        metadata.rows,
        metadata.cols,
    );
    push_u32(&mut bytes, result.cells.len());
    push_u32(&mut bytes, result.merges.len());
    push_u32(&mut bytes, palette_bytes.len());
    push_type_runs(&mut bytes, result.cells.iter().map(|cell| &cell.value));
    for (cell, format_slot) in result.cells.iter().zip(format_slots.iter()) {
        if !dense {
            push_var_u32(&mut bytes, cell.row);
            push_var_u32(&mut bytes, cell.col);
        }
        encode_uuid(&mut bytes, &cell.cell_id);
        write_typed_value(&mut bytes, &cell.value, cell_value_tag(&cell.value));
        encode_optional_query_fields(&mut bytes, cell, format_slot);
    }
    for merge in &result.merges {
        push_var_u32(&mut bytes, merge.start_row);
        push_var_u32(&mut bytes, merge.start_col);
        push_var_u32(&mut bytes, merge.end_row);
        push_var_u32(&mut bytes, merge.end_col);
    }
    bytes.extend_from_slice(&palette_bytes);
    (
        bytes,
        QueryRangeBinaryMeta {
            cell_count: u32::try_from(result.cells.len())
                .expect("query range cell count exceeds u32"),
            merge_count: u32::try_from(result.merges.len())
                .expect("query range merge count exceeds u32"),
            encoding,
            ..metadata
        },
    )
}

#[derive(Debug)]
struct DecodedQueryCell {
    row: u32,
    col: u32,
    cell_id: String,
    value: CellValue,
    formula: Option<String>,
    formatted: Option<String>,
    format: Option<FormatSlot>,
    hyperlink_url: Option<String>,
}

fn read_query_format_slot(reader: &mut Reader<'_>) -> Result<FormatSlot, DecodeError> {
    match reader.byte()? {
        0 => Ok(FormatSlot::None),
        1 => Ok(FormatSlot::Palette {
            index: reader.u16()?,
            fallback: Vec::new(),
        }),
        2 => Ok(FormatSlot::Inline(reader.var_bytes()?.to_vec())),
        kind => Err(error(format!(
            "range binary query format kind {kind} is invalid"
        ))),
    }
}

/// Decode a query result for Rust reference tests and parity checks.
pub(crate) fn decode_query_range(
    bytes: &[u8],
    metadata: QueryRangeBinaryMeta,
) -> Result<RangeQueryResult, DecodeError> {
    let mut header = Reader::new(bytes);
    let rect_metadata = RangeBinaryMeta {
        start_row: metadata.start_row,
        start_col: metadata.start_col,
        rows: metadata.rows,
        cols: metadata.cols,
        encoding: metadata.encoding,
    };
    let (mode, encoding) = read_rect_header_any(&mut header, KIND_QUERY, rect_metadata)?;
    if !matches!(mode, MODE_DENSE | MODE_SPARSE) {
        return Err(error("range binary query mode is invalid"));
    }
    if encoding != metadata.encoding {
        return Err(error("range binary query encoding does not match metadata"));
    }
    let cell_count = header.u32()? as usize;
    let merge_count = header.u32()? as usize;
    let palette_len = header.u32()? as usize;
    if cell_count != metadata.cell_count as usize || merge_count != metadata.merge_count as usize {
        return Err(error("range binary query counts do not match metadata"));
    }
    let palette_start = bytes
        .len()
        .checked_sub(palette_len)
        .ok_or_else(|| error("range binary query palette length is invalid"))?;
    if palette_start < header.pos {
        return Err(error("range binary query records overlap palette"));
    }
    header.limit = palette_start;
    let tags = read_type_runs(&mut header, cell_count)?;
    if encoding == RangeBinaryEncoding::F64Le && tags.iter().any(|&tag| tag != VALUE_TAG_NUMBER) {
        return Err(error("range binary f64 query has a non-numeric type run"));
    }
    if mode == MODE_DENSE && checked_len(metadata.rows, metadata.cols)? != cell_count {
        return Err(error(
            "range binary dense query count does not match rectangle",
        ));
    }
    let mut cells = Vec::with_capacity(cell_count);
    for (index, tag) in tags.into_iter().enumerate() {
        let (row, col) = if mode == MODE_DENSE {
            (
                metadata
                    .start_row
                    .saturating_add((index / metadata.cols as usize) as u32),
                metadata
                    .start_col
                    .saturating_add((index % metadata.cols as usize) as u32),
            )
        } else {
            (header.var_u32()?, header.var_u32()?)
        };
        let cell_id = decode_uuid(&mut header)?;
        let value = read_typed_value(&mut header, tag, 0)?;
        let mask = header.byte()?;
        if mask & !(OPTIONAL_FORMULA | OPTIONAL_FORMATTED | OPTIONAL_FORMAT | OPTIONAL_HYPERLINK)
            != 0
        {
            return Err(error(
                "range binary query optional-field mask has unknown bits",
            ));
        }
        let formula = if mask & OPTIONAL_FORMULA != 0 {
            Some(header.var_string()?)
        } else {
            None
        };
        let formatted = if mask & OPTIONAL_FORMATTED != 0 {
            Some(header.var_string()?)
        } else {
            None
        };
        let format = if mask & OPTIONAL_FORMAT != 0 {
            Some(read_query_format_slot(&mut header)?)
        } else {
            None
        };
        let hyperlink_url = if mask & OPTIONAL_HYPERLINK != 0 {
            Some(header.var_string()?)
        } else {
            None
        };
        cells.push(DecodedQueryCell {
            row,
            col,
            cell_id,
            value,
            formula,
            formatted,
            format,
            hyperlink_url,
        });
    }
    let mut merges = Vec::with_capacity(merge_count);
    for _ in 0..merge_count {
        merges.push(ViewportMerge {
            start_row: header.var_u32()?,
            start_col: header.var_u32()?,
            end_row: header.var_u32()?,
            end_col: header.var_u32()?,
        });
    }
    if header.pos != palette_start {
        return Err(error("range binary query records have trailing bytes"));
    }
    let palette = decode_palette(&bytes[palette_start..])?;
    let cells = cells
        .into_iter()
        .map(|cell| {
            let format = cell
                .format
                .map(|slot| resolve_format_slot(slot, &palette))
                .transpose()?
                .map(|format| serde_json::to_value(format).expect("CellFormat must serialize"));
            Ok(RangeCellData {
                row: cell.row,
                col: cell.col,
                cell_id: cell.cell_id,
                value: cell.value,
                formula: cell.formula,
                formatted: cell.formatted,
                format,
                hyperlink_url: cell.hyperlink_url,
            })
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    Ok(RangeQueryResult { cells, merges })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::RangeCellData;
    use domain_types::CellFormat;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    fn mixed_values() -> Vec<Vec<CellValue>> {
        vec![
            vec![
                CellValue::number(42.5),
                CellValue::Text(Arc::from("hello")),
                CellValue::Error(CellError::Div0, Some(Arc::from("division"))),
                CellValue::Null,
            ],
            vec![
                CellValue::Boolean(true),
                CellValue::Array(Arc::new(CellArray::from_rows(vec![vec![
                    CellValue::number(1.0),
                    CellValue::Text(Arc::from("nested")),
                ]]))),
                CellValue::Control(CellControl::checkbox(true)),
                CellValue::Image(Arc::new(CellImage::new(
                    "https://example.test/image.png",
                    Some(Arc::from("alt")),
                    CellImageSizing::Custom,
                    Some(20),
                    Some(30),
                ))),
            ],
        ]
    }

    fn real_fixture_path() -> PathBuf {
        std::env::var_os("MOG_RANGE_BINARY_REAL_FIXTURE")
            .map(PathBuf::from)
            .expect("set MOG_RANGE_BINARY_REAL_FIXTURE to run the real-workbook repro test")
    }

    // JSON query results may preserve compact 32-hex IDs; kind-1 binary UUIDs
    // decode to the canonical hyphenated spelling used by the TS decoder.
    fn normalize_query_uuid_spellings(value: &mut serde_json::Value) {
        let Some(cells) = value
            .get_mut("cells")
            .and_then(serde_json::Value::as_array_mut)
        else {
            return;
        };
        for cell in cells {
            let Some(cell_id) = cell
                .get("cellId")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
            else {
                continue;
            };
            let Ok(uuid) = Uuid::parse_str(&cell_id) else {
                continue;
            };
            cell["cellId"] = serde_json::Value::String(uuid.to_string());
        }
    }

    #[test]
    #[ignore = "requires the external repro194.xlsx fixture; run explicitly with --ignored"]
    fn repro194_query_range_binary_matches_json_sibling_on_every_sheet() {
        let fixture = real_fixture_path();
        let bytes = fs::read(&fixture).expect("read repro194.xlsx");
        let (engine, _) = crate::storage::engine::YrsComputeEngine::from_xlsx_bytes(&bytes)
            .expect("import repro194.xlsx");
        let dump_dir = std::env::var_os("MOG_RANGE_BINARY_DUMP_DIR").map(PathBuf::from);
        let sheet_ids = engine.get_all_sheet_ids();
        assert!(!sheet_ids.is_empty(), "repro194.xlsx has no sheets");

        for (index, sheet_id_text) in sheet_ids.iter().enumerate() {
            let sheet_id = cell_types::SheetId::from_uuid_str(sheet_id_text)
                .expect("imported sheet ID is a UUID");
            let bounds = engine.get_data_bounds(&sheet_id);
            let (start_row, start_col, end_row, end_col) = bounds
                .map(|bounds| {
                    (
                        bounds.min_row,
                        bounds.min_col,
                        bounds.max_row,
                        bounds.max_col,
                    )
                })
                .unwrap_or((0, 0, 0, 0));
            let json_result = engine.query_range(&sheet_id, start_row, start_col, end_row, end_col);
            let (binary, metadata) =
                engine.query_range_binary(&sheet_id, start_row, start_col, end_row, end_col);
            if let Some(directory) = &dump_dir {
                fs::create_dir_all(directory).expect("create range-binary dump directory");
                fs::write(directory.join(format!("sheet-{index:02}.bin")), &binary)
                    .expect("write range-binary payload dump");
                fs::write(
                    directory.join(format!("sheet-{index:02}.metadata.json")),
                    serde_json::to_vec_pretty(&metadata).expect("serialize range metadata"),
                )
                .expect("write range-binary metadata dump");
            }
            eprintln!(
                "sheet {index}: {} bounds=({start_row},{start_col})..({end_row},{end_col}) cells={} merges={} bytes={}",
                engine
                    .get_sheet_name(&sheet_id)
                    .unwrap_or_else(|| sheet_id_text.clone()),
                metadata.cell_count,
                metadata.merge_count,
                binary.len(),
            );
            let decoded = decode_query_range(&binary, metadata)
                .unwrap_or_else(|error| panic!("sheet {index} Rust decode failed: {error}"));
            let decoded_json = serde_json::to_value(&decoded).expect("serialize decoded query");
            let mut json_result_json =
                serde_json::to_value(&json_result).expect("serialize JSON query");
            normalize_query_uuid_spellings(&mut json_result_json);
            if decoded_json != json_result_json {
                let decoded_cells = decoded_json
                    .get("cells")
                    .and_then(serde_json::Value::as_array)
                    .expect("decoded query cells array");
                let json_cells = json_result_json
                    .get("cells")
                    .and_then(serde_json::Value::as_array)
                    .expect("JSON query cells array");
                if let Some((cell_index, (decoded_cell, json_cell))) = decoded_cells
                    .iter()
                    .zip(json_cells)
                    .enumerate()
                    .find(|(_, (decoded_cell, json_cell))| decoded_cell != json_cell)
                {
                    panic!(
                        "sheet {index} query_range binary differs at cell {cell_index}: decoded={decoded_cell:?} json={json_cell:?}"
                    );
                }
                panic!("sheet {index} query_range binary differs from JSON sibling");
            }
        }
    }

    #[test]
    fn mixed_values_round_trip_matches_json_sibling() {
        let values = mixed_values();
        let (bytes, metadata) = encode_values(7, 11, &values);
        assert_eq!(metadata.encoding, RangeBinaryEncoding::MixedLe);
        let decoded = decode_values(&bytes, metadata).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(values).unwrap()
        );
    }

    #[test]
    fn numeric_values_use_exact_f64_little_endian() {
        let values = vec![vec![CellValue::number(-1.25), CellValue::number(2.5)]];
        let (bytes, metadata) = encode_values(2, 3, &values);
        assert_eq!(metadata.encoding, RangeBinaryEncoding::F64Le);
        assert_eq!(bytes[0], RANGE_BINARY_VERSION);
        assert_eq!(metadata.start_row, 2);
        assert_eq!(metadata.start_col, 3);
        assert_eq!(bytes.len(), 4 + 16 + 4 + 1 + 1 + 16);
        assert_eq!(decode_values(&bytes, metadata).unwrap(), values);
    }

    #[test]
    fn formats_round_trip_through_palette() {
        let mut number_format = CellFormat::default();
        number_format.number_format = Some("0.00".to_string());
        number_format.bold = Some(true);
        let mut fill = CellFormat::default();
        fill.background_color = Some("#FFFF00".to_string());
        let formats = vec![vec![Some(number_format.clone()), Some(fill.clone())]];
        let (bytes, metadata) = encode_formats(5, 6, &formats);
        let decoded = decode_formats(&bytes, metadata).unwrap();
        assert_eq!(decoded, vec![vec![number_format, fill]]);
    }

    #[test]
    fn query_round_trip_preserves_mixed_cells_and_merges() {
        let result = RangeQueryResult {
            cells: vec![
                RangeCellData {
                    row: 3,
                    col: 4,
                    cell_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                    value: CellValue::number(10.0),
                    formula: Some("=A1+1".to_string()),
                    formatted: Some("10.00".to_string()),
                    format: Some(json!({"numberFormat":"0.00"})),
                    hyperlink_url: Some("https://example.test".to_string()),
                },
                RangeCellData {
                    row: 99,
                    col: 1000,
                    cell_id: "not-a-uuid".to_string(),
                    value: CellValue::Error(CellError::Na, None),
                    formula: None,
                    formatted: None,
                    format: None,
                    hyperlink_url: None,
                },
            ],
            merges: vec![ViewportMerge {
                start_row: 3,
                start_col: 4,
                end_row: 4,
                end_col: 5,
            }],
        };
        let metadata = QueryRangeBinaryMeta {
            start_row: 3,
            start_col: 4,
            rows: 97,
            cols: 997,
            cell_count: 0,
            merge_count: 0,
            encoding: query_value_encoding(&result.cells),
        };
        let (bytes, metadata) = encode_query_range(&result, metadata);
        let decoded = decode_query_range(&bytes, metadata).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(result).unwrap()
        );
    }

    #[test]
    fn query_inline_format_round_trip_uses_varint_length() {
        let result = RangeQueryResult {
            cells: vec![
                RangeCellData {
                    row: 0,
                    col: 0,
                    cell_id: String::new(),
                    value: CellValue::Text(Arc::from("inline format")),
                    formula: None,
                    formatted: None,
                    format: Some(json!({
                        "backgroundColor": "#FFFF00",
                        "backgroundColorTint": 0.2
                    })),
                    hyperlink_url: None,
                },
                RangeCellData {
                    row: 0,
                    col: 1,
                    cell_id: String::new(),
                    value: CellValue::number(2.0),
                    formula: None,
                    formatted: None,
                    format: None,
                    hyperlink_url: None,
                },
            ],
            merges: Vec::new(),
        };
        let metadata = QueryRangeBinaryMeta {
            start_row: 0,
            start_col: 0,
            rows: 1,
            cols: 2,
            cell_count: 0,
            merge_count: 0,
            encoding: query_value_encoding(&result.cells),
        };

        let (bytes, metadata) = encode_query_range(&result, metadata);
        assert_eq!(&bytes[28..32], &[0, 0, 0, 0]);
        let decoded = decode_query_range(&bytes, metadata).unwrap();
        assert_eq!(
            serde_json::to_value(decoded).unwrap(),
            serde_json::to_value(result).unwrap()
        );
    }

    #[test]
    fn ten_thousand_mixed_metadata_query_cells_report_size() {
        let cells = (0..10_000u32)
            .map(|index| RangeCellData {
                row: index / 100,
                col: index % 100,
                cell_id: Uuid::from_u128(0x550e8400e29b41d4a716446655440000 + u128::from(index))
                    .to_string(),
                value: CellValue::number(f64::from(index) * 1_000_000.125),
                formula: None,
                formatted: Some(format!("{:.3}", f64::from(index) * 1_000_000.125)),
                format: Some(json!({"numberFormat":"#,##0.000"})),
                hyperlink_url: None,
            })
            .collect::<Vec<_>>();
        let result = RangeQueryResult {
            cells,
            merges: Vec::new(),
        };
        let metadata = QueryRangeBinaryMeta {
            start_row: 0,
            start_col: 0,
            rows: 100,
            cols: 100,
            cell_count: 0,
            merge_count: 0,
            encoding: query_value_encoding(&result.cells),
        };
        let (binary, metadata) = encode_query_range(&result, metadata);
        let json_bytes = serde_json::to_vec(&result).unwrap();
        let ratio = binary.len() as f64 / json_bytes.len() as f64;
        println!(
            "10k mixed-metadata query range: binary={} JSON={} ratio={:.2}%",
            binary.len(),
            json_bytes.len(),
            ratio * 100.0
        );
        assert_eq!(
            decode_query_range(&binary, metadata).unwrap().cells.len(),
            10_000
        );
    }

    #[test]
    fn ten_thousand_pure_numeric_query_cells_report_size_breakdown() {
        let cells = (0..10_000u32)
            .map(|index| RangeCellData {
                row: index / 100,
                col: index % 100,
                cell_id: Uuid::from_u128(0x550e8400e29b41d4a71644644000000 + u128::from(index))
                    .to_string(),
                value: CellValue::number(f64::from(index) * 1_000_000.125),
                formula: None,
                formatted: None,
                format: None,
                hyperlink_url: None,
            })
            .collect::<Vec<_>>();
        let result = RangeQueryResult {
            cells,
            merges: Vec::new(),
        };
        let metadata = QueryRangeBinaryMeta {
            start_row: 0,
            start_col: 0,
            rows: 100,
            cols: 100,
            cell_count: 0,
            merge_count: 0,
            encoding: query_value_encoding(&result.cells),
        };
        let (binary, metadata) = encode_query_range(&result, metadata);
        let json_bytes = serde_json::to_vec(&result).unwrap();
        let ratio = binary.len() as f64 / json_bytes.len() as f64;
        println!(
            "10k pure-numeric query range: binary={} JSON={} ratio={:.2}% (header=20B, counts=12B, type-runs=7B, coordinates=0B, UUIDs={}B, f64-values={}B, optional-masks={}B, palette=8B)",
            binary.len(),
            json_bytes.len(),
            ratio * 100.0,
            10_000 * 17,
            10_000 * std::mem::size_of::<f64>(),
            10_000
        );
        assert!(binary.len() * 10 < json_bytes.len() * 3);
        assert_eq!(
            decode_query_range(&binary, metadata).unwrap().cells.len(),
            10_000
        );
    }
}
