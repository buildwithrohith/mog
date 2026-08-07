import type { CellFormat, SheetId } from '@mog-sdk/contracts/core';
import { jest } from '@jest/globals';

import { ComputeBridge } from '../compute-bridge';

const encoder = new TextEncoder();

function u32(value: number): number[] {
  return [value & 0xff, (value >>> 8) & 0xff, (value >>> 16) & 0xff, (value >>> 24) & 0xff];
}

function f64(value: number): number[] {
  const bytes = new Uint8Array(8);
  new DataView(bytes.buffer).setFloat64(0, value, true);
  return Array.from(bytes);
}

function binary(parts: number[][]): Uint8Array {
  return Uint8Array.from(parts.flat());
}

function denseNumberPayload(): Uint8Array {
  return binary([
    [1, 0, 0, 0], // version, kind, dense mode, f64 encoding
    u32(0),
    u32(0),
    u32(1),
    u32(1),
    u32(1), // one type run
    [1, 1], // number, one value
    f64(42),
  ]);
}

function denseQueryPayload(): Uint8Array {
  return binary([
    [1, 2, 0, 0], // version, query kind, dense mode, f64 encoding
    u32(0),
    u32(0),
    u32(1),
    u32(1),
    u32(1), // cell count
    u32(0), // merge count
    u32(0), // palette bytes
    u32(1), // one type run
    [1, 1], // number, one value
    [0], // empty UUID
    f64(7),
    [0], // no optional fields
  ]);
}

function emptyFormatPayload(): Uint8Array {
  return binary([
    [1, 1, 0, 2], // version, formats kind, dense mode, palette encoding
    u32(0),
    u32(0),
    u32(1),
    u32(1),
    u32(1), // encoded cell count
    u32(0), // palette bytes
    [0], // empty format slot
  ]);
}

function packTuple<T>(bytes: Uint8Array, metadata: T): Uint8Array {
  const metadataBytes = encoder.encode(JSON.stringify(metadata));
  return binary([u32(bytes.byteLength), Array.from(bytes), Array.from(metadataBytes)]);
}

function fakeBridge(raw: unknown): {
  core: {
    query: <T>(promise: Promise<T>) => Promise<T>;
    transport: { call: jest.Mock };
  };
} {
  const call = jest.fn().mockResolvedValue(raw);
  return {
    core: {
      query: <T>(promise: Promise<T>) => promise,
      transport: { call },
    },
  };
}

const sheetId = 'sheet-1' as SheetId;

describe('ComputeBridge binary range overrides', () => {
  it('decodes a WASM-style query tuple and calls the binary sibling', async () => {
    const raw = [
      denseQueryPayload(),
      {
        startRow: 0,
        startCol: 0,
        rows: 1,
        cols: 1,
        cellCount: 1,
        mergeCount: 0,
        encoding: 'F64Le',
      },
    ] as const;
    const bridge = fakeBridge(raw);

    await expect(
      ComputeBridge.prototype.queryRange.call(
        bridge as unknown as ComputeBridge,
        sheetId,
        0,
        0,
        0,
        0,
      ),
    ).resolves.toEqual({
      cells: [{ row: 0, col: 0, cellId: '', value: 7 }],
      merges: [],
    });
    expect(bridge.core.transport.call).toHaveBeenCalledWith('compute_query_range_binary', {
      docId: undefined,
      sheetId,
      startRow: 0,
      startCol: 0,
      endRow: 0,
      endCol: 0,
    });
  });

  it('normalizes a packed NAPI/Tauri values tuple before decoding', async () => {
    const metadata = {
      startRow: 0,
      startCol: 0,
      rows: 1,
      cols: 1,
      encoding: 'F64Le',
    } as const;
    const bridge = fakeBridge(packTuple(denseNumberPayload(), metadata));

    await expect(
      ComputeBridge.prototype.getRangeValues2d.call(
        bridge as unknown as ComputeBridge,
        sheetId,
        0,
        0,
        0,
        0,
      ),
    ).resolves.toEqual([[42]]);
    expect(bridge.core.transport.call).toHaveBeenCalledWith(
      'compute_get_range_values_2d_binary',
      expect.objectContaining({ sheetId }),
    );
  });

  it('decodes palette-backed formats through the existing API name', async () => {
    const metadata = {
      startRow: 0,
      startCol: 0,
      rows: 1,
      cols: 1,
      encoding: 'FormatPalette',
    } as const;
    const bridge = fakeBridge([emptyFormatPayload(), metadata] as [Uint8Array, typeof metadata]);

    await expect(
      ComputeBridge.prototype.getDisplayedRangeProperties.call(
        bridge as unknown as ComputeBridge,
        sheetId,
        0,
        0,
        0,
        0,
      ),
    ).resolves.toEqual([[{} as CellFormat]]);
    expect(bridge.core.transport.call).toHaveBeenCalledWith(
      'compute_get_displayed_range_properties_binary',
      expect.objectContaining({ sheetId }),
    );
  });
});
