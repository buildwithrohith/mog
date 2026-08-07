import type { CellFormat } from '@mog-sdk/contracts/core';

import type {
  QueryRangeBinaryMeta,
  RangeBinaryEncoding,
  RangeBinaryMeta,
} from '../../compute/types';
import { encodePaletteBinary } from '../palette-binary';
import {
  decodeQueryRangeBinary,
  decodeRangeFormatsBinary,
  decodeRangeValuesBinary,
  RangeBinaryDecodeError,
} from '../range-binary';

const VERSION = 1;
const KIND_VALUES = 0;
const KIND_FORMATS = 1;
const KIND_QUERY = 2;
const MODE_DENSE = 0;
const MODE_SPARSE = 1;
const ENCODING_F64 = 0;
const ENCODING_MIXED = 1;
const ENCODING_FORMAT_PALETTE = 2;

const TAG_NULL = 0;
const TAG_NUMBER = 1;
const TAG_TEXT = 2;
const TAG_BOOLEAN = 3;
const TAG_ERROR = 4;
const TAG_ARRAY = 5;

const encoder = new TextEncoder();

class Writer {
  private readonly output: number[] = [];

  byte(value: number): void {
    this.output.push(value & 0xff);
  }

  u16(value: number): void {
    this.output.push(value & 0xff, (value >>> 8) & 0xff);
  }

  u32(value: number): void {
    this.output.push(
      value & 0xff,
      (value >>> 8) & 0xff,
      (value >>> 16) & 0xff,
      (value >>> 24) & 0xff,
    );
  }

  varU32(value: number): void {
    let remaining = value >>> 0;
    while (remaining >= 0x80) {
      this.byte((remaining & 0x7f) | 0x80);
      remaining >>>= 7;
    }
    this.byte(remaining);
  }

  f64(value: number): void {
    const bytes = new Uint8Array(8);
    new DataView(bytes.buffer).setFloat64(0, value, true);
    this.bytes(bytes);
  }

  bytes(bytes: Uint8Array): void {
    for (const byte of bytes) this.byte(byte);
  }

  string(value: string): void {
    const bytes = encoder.encode(value);
    this.u32(bytes.byteLength);
    this.bytes(bytes);
  }

  varString(value: string): void {
    const bytes = encoder.encode(value);
    this.varU32(bytes.byteLength);
    this.bytes(bytes);
  }

  finish(): Uint8Array {
    return Uint8Array.from(this.output);
  }
}

function header(
  writer: Writer,
  kind: number,
  mode: number,
  encoding: number,
  startRow: number,
  startCol: number,
  rows: number,
  cols: number,
): void {
  writer.byte(VERSION);
  writer.byte(kind);
  writer.byte(mode);
  writer.byte(encoding);
  writer.u32(startRow);
  writer.u32(startCol);
  writer.u32(rows);
  writer.u32(cols);
}

function typeRuns(writer: Writer, runs: Array<[number, number]>): void {
  writer.u32(runs.length);
  for (const [tag, count] of runs) {
    writer.byte(tag);
    writer.varU32(count);
  }
}

function metadata(
  startRow: number,
  startCol: number,
  rows: number,
  cols: number,
  encoding: RangeBinaryEncoding,
): RangeBinaryMeta {
  return { startRow, startCol, rows, cols, encoding };
}

function uuidBytes(): Uint8Array {
  return Uint8Array.from([
    0x55, 0x0e, 0x84, 0x00, 0xe2, 0x9b, 0x41, 0xd4,
    0xa7, 0x16, 0x44, 0x66, 0x55, 0x44, 0x00, 0x00,
  ]);
}

describe('range binary decoder', () => {
  it('decodes a dense pure-numeric f64 block with one type run', () => {
    const writer = new Writer();
    header(writer, KIND_VALUES, MODE_DENSE, ENCODING_F64, 2, 3, 2, 2);
    typeRuns(writer, [[TAG_NUMBER, 4]]);
    for (const value of [1.25, -2.5, 3, 4]) writer.f64(value);

    expect(
      decodeRangeValuesBinary(
        writer.finish(),
        metadata(2, 3, 2, 2, 'F64Le'),
      ),
    ).toEqual([
      [1.25, -2.5],
      [3, 4],
    ]);
  });

  it('decodes mixed values and recursively tagged arrays', () => {
    const writer = new Writer();
    header(writer, KIND_VALUES, MODE_DENSE, ENCODING_MIXED, 0, 0, 2, 3);
    typeRuns(writer, [
      [TAG_NUMBER, 1],
      [TAG_TEXT, 1],
      [TAG_BOOLEAN, 1],
      [TAG_ERROR, 1],
      [TAG_NULL, 1],
      [TAG_ARRAY, 1],
    ]);
    writer.f64(1.25);
    writer.string('hello');
    writer.byte(1);
    writer.byte(5); // #REF!
    writer.byte(1);
    writer.string('bad reference');
    writer.u32(1);
    writer.u32(2);
    writer.byte(TAG_NUMBER);
    writer.f64(7.5);
    writer.byte(TAG_TEXT);
    writer.string('nested');

    expect(
      decodeRangeValuesBinary(
        writer.finish(),
        metadata(0, 0, 2, 3, 'MixedLe'),
      ),
    ).toEqual([
      [1.25, 'hello', true],
      [
        { type: 'error', value: 'Ref', message: 'bad reference' },
        null,
        [[7.5, 'nested']],
      ],
    ]);
  });

  it('decodes palette-backed formats and preserves empty format slots', () => {
    const format: CellFormat = { bold: true, numberFormat: '0.00' };
    const palette = encodePaletteBinary(0, [format]);
    const writer = new Writer();
    header(writer, KIND_FORMATS, MODE_DENSE, ENCODING_FORMAT_PALETTE, 4, 5, 1, 3);
    writer.u32(3);
    writer.u32(palette.byteLength);
    writer.byte(1);
    writer.u16(0);
    writer.byte(0);
    writer.byte(1);
    writer.u16(0);
    writer.bytes(palette);

    expect(
      decodeRangeFormatsBinary(
        writer.finish(),
        metadata(4, 5, 1, 3, 'FormatPalette'),
      ),
    ).toEqual([[format, {}, format]]);
  });

  it('decodes a dense query with UUIDs, optional fields, formats, and merges', () => {
    const format: CellFormat = { italic: true };
    const palette = encodePaletteBinary(0, [format]);
    const writer = new Writer();
    header(writer, KIND_QUERY, MODE_DENSE, ENCODING_MIXED, 10, 4, 1, 2);
    writer.u32(2);
    writer.u32(1);
    writer.u32(palette.byteLength);
    typeRuns(writer, [
      [TAG_NUMBER, 1],
      [TAG_TEXT, 1],
    ]);

    writer.byte(1);
    writer.bytes(uuidBytes());
    writer.f64(42);
    writer.byte(0b1111);
    writer.varString('=A1+1');
    writer.varString('42.00');
    writer.byte(1);
    writer.u16(0);
    writer.varString('https://example.test');

    writer.byte(2);
    writer.varString('custom-id');
    writer.string('hello');
    writer.byte(0);

    writer.varU32(10);
    writer.varU32(4);
    writer.varU32(10);
    writer.varU32(5);
    writer.bytes(palette);

    const queryMetadata: QueryRangeBinaryMeta = {
      ...metadata(10, 4, 1, 2, 'MixedLe'),
      cellCount: 2,
      mergeCount: 1,
    };
    expect(decodeQueryRangeBinary(writer.finish(), queryMetadata)).toEqual({
      cells: [
        {
          row: 10,
          col: 4,
          cellId: '550e8400-e29b-41d4-a716-446655440000',
          value: 42,
          formula: '=A1+1',
          formatted: '42.00',
          format,
          hyperlinkUrl: 'https://example.test',
        },
        {
          row: 10,
          col: 5,
          cellId: 'custom-id',
          value: 'hello',
        },
      ],
      merges: [{ startRow: 10, startCol: 4, endRow: 10, endCol: 5 }],
    });
  });

  it('decodes sparse query coordinates as absolute row and column varints', () => {
    const writer = new Writer();
    header(writer, KIND_QUERY, MODE_SPARSE, ENCODING_MIXED, 100, 200, 50, 50);
    writer.u32(2);
    writer.u32(0);
    writer.u32(0);
    typeRuns(writer, [
      [TAG_NUMBER, 1],
      [TAG_TEXT, 1],
    ]);
    writer.varU32(101);
    writer.varU32(203);
    writer.byte(0);
    writer.f64(9);
    writer.byte(0);
    writer.varU32(149);
    writer.varU32(249);
    writer.byte(0);
    writer.string('sparse');
    writer.byte(0);

    expect(
      decodeQueryRangeBinary(writer.finish(), {
        ...metadata(100, 200, 50, 50, 'MixedLe'),
        cellCount: 2,
        mergeCount: 0,
      }),
    ).toEqual({
      cells: [
        { row: 101, col: 203, cellId: '', value: 9 },
        { row: 149, col: 249, cellId: '', value: 'sparse' },
      ],
      merges: [],
    });
  });

  it('rejects truncated and incompatible payloads', () => {
    const writer = new Writer();
    header(writer, KIND_VALUES, MODE_DENSE, ENCODING_F64, 0, 0, 1, 1);
    typeRuns(writer, [[TAG_NUMBER, 1]]);
    writer.f64(1);
    const bytes = writer.finish();

    expect(() => decodeRangeValuesBinary(bytes.subarray(0, bytes.length - 1), metadata(0, 0, 1, 1, 'F64Le'))).toThrow(RangeBinaryDecodeError);
    expect(() => decodeRangeValuesBinary(bytes, metadata(0, 0, 2, 1, 'F64Le'))).toThrow(RangeBinaryDecodeError);
    const badVersion = bytes.slice();
    badVersion[0] = 2;
    expect(() => decodeRangeValuesBinary(badVersion, metadata(0, 0, 1, 1, 'F64Le'))).toThrow(RangeBinaryDecodeError);
  });
});
