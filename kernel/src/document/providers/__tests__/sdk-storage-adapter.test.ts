import { jest } from '@jest/globals';
import type { MogSdkStorageProvider } from '@mog-sdk/contracts/sdk';
import { createSdkStorageAdapter } from '../sdk-storage-adapter';
import type { ProviderDoc } from '../provider';

function sdkProvider(overrides: Partial<MogSdkStorageProvider> = {}): MogSdkStorageProvider {
  return {
    name: 'test-sdk',
    attach: jest.fn(async () => ({ ok: true, initialUpdate: new Uint8Array([19]) })),
    appendUpdate: jest.fn(),
    flush: jest.fn(async () => undefined),
    checkpoint: jest.fn(async () => ({ ok: true })),
    flushSync: jest.fn(),
    detach: jest.fn(async () => undefined),
    flushFailed: false,
    ...overrides,
  };
}

function providerDoc(): ProviderDoc {
  return {
    docId: 'schema-gate-doc',
    applyUpdate: jest.fn(async () => undefined),
    encodeDiff: jest.fn(async (sv: Uint8Array) => {
      expect(Array.from(sv)).toEqual([0]);
      return new Uint8Array([20, 1, 2]);
    }),
    currentStateVector: jest.fn(async () => new Uint8Array([7])),
    inspectStorageSchemaVersion: jest.fn(async (update: Uint8Array) => ({
      incomingSchemaVersion: update[0] ?? 0,
      currentSchemaVersion: 20,
    })),
    prepareStorageSchemaBaseline: jest.fn(async () => undefined),
  };
}

describe('createSdkStorageAdapter schema gate', () => {
  it('discards a v19 baseline, never applies it live, and checkpoints fresh v20 state', async () => {
    const sdk = sdkProvider();
    const doc = providerDoc();
    const result = await createSdkStorageAdapter(sdk).attach(doc);

    expect(result).toEqual(expect.objectContaining({ status: 'ready' }));
    expect(doc.inspectStorageSchemaVersion).toHaveBeenCalledWith(new Uint8Array([19]));
    expect(doc.applyUpdate).not.toHaveBeenCalled();
    expect(doc.prepareStorageSchemaBaseline).toHaveBeenCalledTimes(1);
    expect(sdk.checkpoint).toHaveBeenCalledTimes(1);
    const checkpointDoc = (sdk.checkpoint as jest.Mock).mock.calls[0]?.[0] as {
      encodeStateAsUpdate(): Uint8Array;
    };
    expect(Array.from(checkpointDoc.encodeStateAsUpdate())).toEqual([20, 1, 2]);
  });

  it('blocks attach when the replacement checkpoint fails', async () => {
    const sdk = sdkProvider({
      checkpoint: jest.fn(async () => ({ ok: false, error: 'disk full' })),
    });
    const doc = providerDoc();
    const adapter = createSdkStorageAdapter(sdk);
    const result = await adapter.attach(doc);

    expect(result).toEqual(
      expect.objectContaining({ status: 'blocked', reason: 'unavailable', message: 'disk full' }),
    );
    expect(doc.applyUpdate).not.toHaveBeenCalled();
    await adapter.detach();
    expect(sdk.detach).toHaveBeenCalledTimes(1);
  });

  it('refuses a future-schema baseline without overwriting it', async () => {
    const sdk = sdkProvider({
      attach: jest.fn(async () => ({ ok: true, initialUpdate: new Uint8Array([21]) })),
    });
    const doc = providerDoc();
    const adapter = createSdkStorageAdapter(sdk);

    const result = await adapter.attach(doc);

    expect(result).toEqual(expect.objectContaining({ status: 'blocked', reason: 'unavailable' }));
    expect(doc.applyUpdate).not.toHaveBeenCalled();
    expect(doc.prepareStorageSchemaBaseline).not.toHaveBeenCalled();
    expect(sdk.checkpoint).not.toHaveBeenCalled();
    await adapter.detach();
    expect(sdk.detach).toHaveBeenCalledTimes(1);
  });

  it('returns blocked for malformed initial state and remains detachable', async () => {
    const sdk = sdkProvider();
    const doc = providerDoc();
    (doc.inspectStorageSchemaVersion as jest.Mock).mockRejectedValueOnce(
      new Error('malformed yrs update'),
    );
    const adapter = createSdkStorageAdapter(sdk);

    const result = await adapter.attach(doc);

    expect(result).toEqual(
      expect.objectContaining({
        status: 'blocked',
        reason: 'unavailable',
        message: 'malformed yrs update',
      }),
    );
    await adapter.detach();
    expect(sdk.detach).toHaveBeenCalledTimes(1);
  });
});
