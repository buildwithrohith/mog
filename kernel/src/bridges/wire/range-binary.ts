/**
 * Decoder for the self-describing bulk range wire format.
 *
 * Layout (all multi-byte values are little-endian):
 *
 *   [version:u8][kind:u8][mode:u8][encoding:u8]
 *   [start_row:u32][start_col:u32][rows:u32][cols:u32]
 *   [payload]
 *
 * Dense values are row-major. Their payload begins with a u32 type-run count
 * followed by `(type:u8, run_length:var_u32)` records and then typed values
 * without repeated type bytes. Query payloads add cell/merge/palette counts;
 * sparse query cells carry absolute row/column varints. Format payloads use
 * the existing palette-binary protocol for their tail.
 */

import type { CellError, CellFormat, CellValue } from '@mog-sdk/contracts/core';

import type {
  RangeCellData,
  RangeQueryResult,
  ViewportMerge,
} from '../compute/compute-types.gen';
import type { QueryRangeBinaryMeta, RangeBinaryEncoding, RangeBinaryMeta } from '../compute/types';
import { decodePaletteBinary } from './palette-binary';

const RANGE_BINARY_VERSION = 1;
const KIND_VALUES = 0;
const KIND_FORMATS = 1;
const KIND_QUERY = 2;
const MODE_DENSE = 0;
const MODE_SPARSE = 1;
const ENCODING_F64 = 0;
const ENCODING_MIXED = 1;
const ENCODING_FORMAT_PALETTE = 2;

const VALUE_TAG_NULL = 0;
const VALUE_TAG_NUMBER = 1;
const VALUE_TAG_TEXT = 2;
const VALUE_TAG_BOOLEAN = 3;
const VALUE_TAG_ERROR = 4;
const VALUE_TAG_ARRAY = 5;
const VALUE_TAG_CONTROL = 6;
const VALUE_TAG_IMAGE = 7;

const OPTIONAL_FORMULA = 1 << 0;
const OPTIONAL_FORMATTED = 1 << 1;
const OPTIONAL_FORMAT = 1 << 2;
const OPTIONAL_HYPERLINK = 1 << 3;
const OPTIONAL_MASK = OPTIONAL_FORMULA | OPTIONAL_FORMATTED | OPTIONAL_FORMAT | OPTIONAL_HYPERLINK;

const MAX_U32 = 0xffff_ffff;
const ERROR_VARIANTS = [
  'Div0',
  'Na',
  'Name',
  'Null',
  'Num',
  'Ref',
  'Value',
  'Spill',
  'Calc',
  'GettingData',
  'Circ',
] as const;

const sharedDecoder = new TextDecoder('utf-8', { fatal: true });

/** Error raised for malformed or incompatible range-binary payloads. */
export class RangeBinaryDecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'RangeBinaryDecodeError';
  }
}

function fail(message: string): never {
  throw new RangeBinaryDecodeError(message);
}

function asBytes(bytes: Uint8Array): Uint8Array {
  if (!(bytes instanceof Uint8Array)) {
    return fail('range binary payload is not a Uint8Array');
  }
  return bytes;
}

function checkedCellCount(rows: number, cols: number): number {
  if (
    !Number.isSafeInteger(rows) ||
    !Number.isSafeInteger(cols) ||
    rows < 0 ||
    cols < 0 ||
    (rows !== 0 && cols > Math.floor(Number.MAX_SAFE_INTEGER / rows))
  ) {
    return fail('range binary rectangle shape overflows JavaScript');
  }
  return rows * cols;
}

function encodingCode(encoding: RangeBinaryEncoding): number {
  switch (encoding) {
    case 'F64Le':
      return ENCODING_F64;
    case 'MixedLe':
      return ENCODING_MIXED;
    case 'FormatPalette':
      return ENCODING_FORMAT_PALETTE;
    default:
      return fail(`range binary encoding ${String(encoding)} is invalid`);
  }
}

function encodingFromCode(code: number): RangeBinaryEncoding {
  switch (code) {
    case ENCODING_F64:
      return 'F64Le';
    case ENCODING_MIXED:
      return 'MixedLe';
    case ENCODING_FORMAT_PALETTE:
      return 'FormatPalette';
    default:
      return fail(`range binary encoding ${code} is invalid`);
  }
}

class Reader {
  private readonly view: DataView;
  private pos = 0;
  private limit: number;

  constructor(private readonly bytes: Uint8Array) {
    this.view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    this.limit = bytes.byteLength;
  }

  get position(): number {
    return this.pos;
  }

  remaining(): number {
    return this.limit - this.pos;
  }

  setLimit(limit: number): void {
    if (!Number.isSafeInteger(limit) || limit < 0 || limit > this.bytes.byteLength) {
      fail('range binary reader limit is invalid');
    }
    if (this.pos > limit) {
      fail('range binary reader limit overlaps decoded data');
    }
    this.limit = limit;
  }

  private ensure(length: number): void {
    if (
      !Number.isSafeInteger(length) ||
      length < 0 ||
      this.pos + length > this.limit
    ) {
      fail('range binary payload is truncated');
    }
  }

  take(length: number): Uint8Array {
    this.ensure(length);
    const result = this.bytes.subarray(this.pos, this.pos + length);
    this.pos += length;
    return result;
  }

  byte(): number {
    return this.take(1)[0];
  }

  u16(): number {
    this.ensure(2);
    const value = this.view.getUint16(this.pos, true);
    this.pos += 2;
    return value;
  }

  u32(): number {
    this.ensure(4);
    const value = this.view.getUint32(this.pos, true);
    this.pos += 4;
    return value;
  }

  f64(): number {
    this.ensure(8);
    const value = this.view.getFloat64(this.pos, true);
    this.pos += 8;
    return value;
  }

  varU32(): number {
    let value = 0;
    for (let shift = 0; shift <= 28; shift += 7) {
      const byte = this.byte();
      const part = byte & 0x7f;
      if (shift === 28 && part > 0x0f) {
        fail('range binary varint overflows u32');
      }
      value += part * 2 ** shift;
      if ((byte & 0x80) === 0) {
        return value;
      }
    }
    return fail('range binary varint is too long');
  }

  bytes32(): Uint8Array {
    return this.take(this.u32());
  }

  varBytes(): Uint8Array {
    return this.take(this.varU32());
  }

  string(): string {
    return decodeUtf8(this.bytes32());
  }

  varString(): string {
    return decodeUtf8(this.varBytes());
  }
}

function decodeUtf8(bytes: Uint8Array): string {
  try {
    return sharedDecoder.decode(bytes);
  } catch {
    return fail('range binary string is not valid UTF-8');
  }
}

function readRectHeader(
  reader: Reader,
  expectedKind: number,
  metadata: RangeBinaryMeta,
): { mode: number; encoding: RangeBinaryEncoding } {
  if (reader.byte() !== RANGE_BINARY_VERSION) {
    fail('range binary version is invalid');
  }
  if (reader.byte() !== expectedKind) {
    fail('range binary payload kind is invalid');
  }
  const mode = reader.byte();
  const encoding = encodingFromCode(reader.byte());
  const startRow = reader.u32();
  const startCol = reader.u32();
  const rows = reader.u32();
  const cols = reader.u32();

  if (
    startRow !== metadata.startRow ||
    startCol !== metadata.startCol ||
    rows !== metadata.rows ||
    cols !== metadata.cols
  ) {
    fail('range binary rectangle does not match metadata');
  }
  if (encodingCode(metadata.encoding) !== encodingCode(encoding)) {
    fail('range binary encoding does not match metadata');
  }
  return { mode, encoding };
}

function isValueTag(tag: number): boolean {
  return tag >= VALUE_TAG_NULL && tag <= VALUE_TAG_IMAGE;
}

function readTypeRuns(reader: Reader, valueCount: number): Uint8Array {
  const runCount = reader.u32();
  if (runCount > valueCount) {
    fail('range binary type stream has too many runs');
  }

  const tags = new Uint8Array(valueCount);
  let covered = 0;
  for (let run = 0; run < runCount; run++) {
    const tag = reader.byte();
    if (!isValueTag(tag)) {
      fail(`range binary type tag ${tag} is invalid`);
    }
    const count = reader.varU32();
    if (count === 0) {
      fail('range binary type run is empty');
    }
    const next = covered + count;
    if (next > valueCount) {
      fail('range binary type stream exceeds cell count');
    }
    tags.fill(tag, covered, next);
    covered = next;
  }
  if (covered !== valueCount) {
    fail('range binary type stream does not cover cell count');
  }
  return tags;
}

function readCellValue(reader: Reader, depth: number): unknown {
  if (depth > 64) {
    fail('range binary nested value depth exceeds 64');
  }
  return readCellPayload(reader, reader.byte(), depth);
}

function readCellPayload(reader: Reader, tag: number, depth: number): unknown {
  if (depth > 64) {
    fail('range binary nested value depth exceeds 64');
  }

  switch (tag) {
    case VALUE_TAG_NULL:
      return null;
    case VALUE_TAG_NUMBER:
      return reader.f64();
    case VALUE_TAG_TEXT:
      return reader.string();
    case VALUE_TAG_BOOLEAN:
      return reader.byte() !== 0;
    case VALUE_TAG_ERROR: {
      const variant = ERROR_VARIANTS[reader.byte()];
      if (variant === undefined) {
        fail('range binary cell error code is invalid');
      }
      const error: CellError = { type: 'error', value: variant };
      if (reader.byte() !== 0) {
        error.message = reader.string();
      }
      return error;
    }
    case VALUE_TAG_ARRAY: {
      const rows = reader.u32();
      const cols = reader.u32();
      const count = checkedCellCount(rows, cols);
      const values = new Array<unknown>(count);
      for (let index = 0; index < count; index++) {
        values[index] = readCellValue(reader, depth + 1);
      }
      const result: unknown[][] = new Array(rows);
      for (let row = 0; row < rows; row++) {
        result[row] = values.slice(row * cols, (row + 1) * cols);
      }
      return result;
    }
    case VALUE_TAG_CONTROL: {
      const controlType = reader.byte();
      if (controlType !== 0) {
        fail(`range binary control type code ${controlType} is invalid`);
      }
      return {
        type: 'control',
        controlType: 'checkbox',
        checked: reader.byte() !== 0,
        value: reader.byte() !== 0,
      };
    }
    case VALUE_TAG_IMAGE: {
      const source = reader.string();
      const altText = reader.byte() !== 0 ? reader.string() : null;
      const sizingCode = reader.byte();
      const sizing = ['fit', 'fill', 'original', 'custom'][sizingCode];
      if (sizing === undefined) {
        fail(`range binary image sizing code ${sizingCode} is invalid`);
      }
      const height = reader.byte() !== 0 ? reader.u32() : null;
      const width = reader.byte() !== 0 ? reader.u32() : null;
      return { type: 'image', source, altText, sizing, height, width };
    }
    default:
      return fail(`range binary value tag ${tag} is invalid`);
  }
}

function reshape<T>(values: T[], rows: number, cols: number): T[][] {
  if (cols === 0) {
    return Array.from({ length: rows }, () => [] as T[]);
  }
  return Array.from({ length: rows }, (_, row) => values.slice(row * cols, (row + 1) * cols));
}

/** Decode a dense `get_range_values_2d_binary` payload. */
export function decodeRangeValuesBinary(
  bytesInput: Uint8Array,
  metadata: RangeBinaryMeta,
): CellValue[][] {
  const bytes = asBytes(bytesInput);
  const count = checkedCellCount(metadata.rows, metadata.cols);
  const reader = new Reader(bytes);
  const { mode, encoding } = readRectHeader(reader, KIND_VALUES, metadata);
  if (mode !== MODE_DENSE) {
    fail('range binary value payload mode is invalid');
  }
  if (encoding === 'FormatPalette') {
    fail('range binary value payload encoding is invalid');
  }

  const tags = readTypeRuns(reader, count);
  if (encoding === 'F64Le' && Array.from(tags).some((tag) => tag !== VALUE_TAG_NUMBER)) {
    fail('range binary f64 payload has a non-numeric type run');
  }

  const values = new Array<CellValue>(count);
  for (let index = 0; index < count; index++) {
    values[index] = readCellPayload(reader, tags[index], 0) as CellValue;
  }
  if (reader.remaining() !== 0) {
    fail('range binary dense payload has trailing bytes');
  }
  return reshape(values, metadata.rows, metadata.cols);
}

type FormatSlot =
  | { kind: 'none' }
  | { kind: 'palette'; index: number }
  | { kind: 'inline'; bytes: Uint8Array };

function readFormatSlot(reader: Reader, variableLength: boolean): FormatSlot {
  switch (reader.byte()) {
    case 0:
      return { kind: 'none' };
    case 1:
      return { kind: 'palette', index: reader.u16() };
    case 2:
      return {
        kind: 'inline',
        bytes: variableLength ? reader.varBytes() : reader.bytes32(),
      };
    default:
      return fail('range binary format slot kind is invalid');
  }
}

function decodePalette(
  bytes: Uint8Array,
  offset: number,
  length: number,
): CellFormat[] {
  if (length === 0) {
    return [];
  }
  try {
    return decodePaletteBinary(
      new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength),
      offset,
      length,
    ).formats;
  } catch (error) {
    if (error instanceof RangeBinaryDecodeError) {
      throw error;
    }
    return fail(`range binary format palette: ${String(error)}`);
  }
}

function resolveFormatSlot(slot: FormatSlot, palette: CellFormat[]): unknown {
  switch (slot.kind) {
    case 'none':
      return {};
    case 'palette': {
      const format = palette[slot.index];
      if (format === undefined) {
        fail(`range binary format palette index ${slot.index} is invalid`);
      }
      return format;
    }
    case 'inline':
      try {
        return JSON.parse(decodeUtf8(slot.bytes));
      } catch (error) {
        if (error instanceof RangeBinaryDecodeError) {
          throw error;
        }
        return fail(`range binary inline format: ${String(error)}`);
      }
  }
}

function paletteStart(reader: Reader, bytes: Uint8Array, paletteLength: number): number {
  if (paletteLength > bytes.byteLength) {
    fail('range binary format palette length is invalid');
  }
  const start = bytes.byteLength - paletteLength;
  if (start < reader.position) {
    fail('range binary format slots overlap palette');
  }
  reader.setLimit(start);
  return start;
}

/** Decode a dense `get_displayed_range_properties_binary` payload. */
export function decodeRangeFormatsBinary(
  bytesInput: Uint8Array,
  metadata: RangeBinaryMeta,
): CellFormat[][] {
  const bytes = asBytes(bytesInput);
  const count = checkedCellCount(metadata.rows, metadata.cols);
  const reader = new Reader(bytes);
  const { mode, encoding } = readRectHeader(reader, KIND_FORMATS, metadata);
  if (mode !== MODE_DENSE || encoding !== 'FormatPalette') {
    fail('range binary format payload mode or encoding is invalid');
  }

  const encodedCount = reader.u32();
  const paletteLength = reader.u32();
  if (encodedCount !== count) {
    fail(`range binary format count ${encodedCount} does not match metadata count ${count}`);
  }
  const paletteOffset = paletteStart(reader, bytes, paletteLength);
  const slots = new Array<FormatSlot>(count);
  for (let index = 0; index < count; index++) {
    slots[index] = readFormatSlot(reader, false);
  }
  if (reader.position !== paletteOffset) {
    fail('range binary format slots have trailing bytes');
  }
  const palette = decodePalette(bytes, paletteOffset, paletteLength);
  const formats = slots.map((slot) => resolveFormatSlot(slot, palette) as CellFormat);
  return reshape(formats, metadata.rows, metadata.cols);
}

function uuidString(bytes: Uint8Array): string {
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

function readUuid(reader: Reader): string {
  switch (reader.byte()) {
    case 0:
      return '';
    case 1:
      return uuidString(reader.take(16));
    case 2:
      return reader.varString();
    default:
      return fail('range binary UUID kind is invalid');
  }
}

function addU32Saturating(base: number, offset: number): number {
  return Math.min(MAX_U32, base + offset);
}

/** Decode the sparse-or-dense `query_range_binary` payload. */
export function decodeQueryRangeBinary(
  bytesInput: Uint8Array,
  metadata: QueryRangeBinaryMeta,
): RangeQueryResult {
  const bytes = asBytes(bytesInput);
  const rectangle: RangeBinaryMeta = {
    startRow: metadata.startRow,
    startCol: metadata.startCol,
    rows: metadata.rows,
    cols: metadata.cols,
    encoding: metadata.encoding,
  };
  const reader = new Reader(bytes);
  const { mode, encoding } = readRectHeader(reader, KIND_QUERY, rectangle);
  if (mode !== MODE_DENSE && mode !== MODE_SPARSE) {
    fail('range binary query mode is invalid');
  }
  if (encoding === 'FormatPalette') {
    fail('range binary query encoding is invalid');
  }

  const cellCount = reader.u32();
  const mergeCount = reader.u32();
  const paletteLength = reader.u32();
  if (cellCount !== metadata.cellCount || mergeCount !== metadata.mergeCount) {
    fail('range binary query counts do not match metadata');
  }
  const rectangleCount = checkedCellCount(metadata.rows, metadata.cols);
  if (mode === MODE_DENSE && rectangleCount !== cellCount) {
    fail('range binary dense query count does not match rectangle');
  }
  const paletteOffset = paletteStart(reader, bytes, paletteLength);
  const tags = readTypeRuns(reader, cellCount);
  if (encoding === 'F64Le' && Array.from(tags).some((tag) => tag !== VALUE_TAG_NUMBER)) {
    fail('range binary f64 query has a non-numeric type run');
  }

  const cells = new Array<RangeCellData>(cellCount);
  for (let index = 0; index < cellCount; index++) {
    const row =
      mode === MODE_DENSE
        ? addU32Saturating(metadata.startRow, Math.floor(index / metadata.cols))
        : reader.varU32();
    const col =
      mode === MODE_DENSE
        ? addU32Saturating(metadata.startCol, index % metadata.cols)
        : reader.varU32();
    const cell: RangeCellData = {
      row,
      col,
      cellId: readUuid(reader),
      value: readCellPayload(reader, tags[index], 0) as CellValue,
    };

    const mask = reader.byte();
    if ((mask & ~OPTIONAL_MASK) !== 0) {
      fail('range binary query optional-field mask has unknown bits');
    }
    if (mask & OPTIONAL_FORMULA) {
      cell.formula = reader.varString();
    }
    if (mask & OPTIONAL_FORMATTED) {
      cell.formatted = reader.varString();
    }
    if (mask & OPTIONAL_FORMAT) {
      // Query slots use varint lengths for inline JSON, unlike the dense
      // format payload's fixed u32 lengths.
      const slot = readFormatSlot(reader, true);
      // Resolve after the palette tail has been decoded below.
      cell.format = slot;
    }
    if (mask & OPTIONAL_HYPERLINK) {
      cell.hyperlinkUrl = reader.varString();
    }
    cells[index] = cell;
  }

  const merges = new Array<ViewportMerge>(mergeCount);
  for (let index = 0; index < mergeCount; index++) {
    merges[index] = {
      startRow: reader.varU32(),
      startCol: reader.varU32(),
      endRow: reader.varU32(),
      endCol: reader.varU32(),
    };
  }
  if (reader.position !== paletteOffset) {
    fail('range binary query records have trailing bytes');
  }
  const palette = decodePalette(bytes, paletteOffset, paletteLength);
  for (const cell of cells) {
    if (cell.format !== undefined) {
      cell.format = resolveFormatSlot(cell.format as FormatSlot, palette);
    }
  }
  return { cells, merges };
}
