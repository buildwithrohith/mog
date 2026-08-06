import { jest } from '@jest/globals';
import type { SheetId } from '@mog-sdk/contracts/core';
import { DocumentLifecycleSystem } from '../document-lifecycle-system';
import { DocumentMaterializationTracker } from '../materialization-tracker';

type SchedulerHarness = Pick<
  DocumentLifecycleSystem,
  | 'scheduleDeferredHydration'
  | 'setWorkbookReadOnly'
  | 'ensureDeferredHydration'
  | 'awaitMaterialized'
  | 'getMaterializationState'
> & {
  actor: {
    getSnapshot: () => {
      context: {
        computeBridge: unknown;
        rustDocument?: unknown;
        initialSheetIds: string[];
      };
      matches: (state: string) => boolean;
      value: string;
    };
  };
  deferredHydrationPending: boolean;
  deferredHydrationPromise: Promise<void> | null;
  deferredHydrationTimer: ReturnType<typeof setTimeout> | null;
  startDeferredHydrationNow: (() => void) | null;
  hostLifecycleInput: unknown;
  environment: 'browser' | 'headless';
  importDurabilityPending: boolean;
  materializationError: null;
  materializationTracker: DocumentMaterializationTracker;
  _storageState: { readOnly: boolean };
};

function createSchedulerHarness(
  completeDeferredHydration: jest.Mock,
  options: {
    readonly deferredImportSheetIds?: string[];
    readonly materializedSheetIds?: string[];
    readonly deferredHydrationPending?: boolean;
  } = {},
): SchedulerHarness {
  const materializationTracker = new DocumentMaterializationTracker();
  materializationTracker.markDeferredImport(
    (options.deferredImportSheetIds ?? ['critical-sheet', 'secondary-sheet']) as SheetId[],
    (options.materializedSheetIds ?? ['critical-sheet']) as SheetId[],
  );

  const harness = Object.create(DocumentLifecycleSystem.prototype) as SchedulerHarness;
  harness.actor = {
    getSnapshot: () => ({
      context: {
        computeBridge: {
          completeDeferredHydration,
        },
        initialSheetIds: ['critical-sheet', 'secondary-sheet'],
      },
      matches: (state: string) => state === 'ready',
      value: 'ready',
    }),
  };
  harness.deferredHydrationPending = options.deferredHydrationPending ?? true;
  harness.deferredHydrationPromise = null;
  harness.deferredHydrationTimer = null;
  harness.startDeferredHydrationNow = null;
  harness.hostLifecycleInput = {};
  harness.environment = 'browser';
  harness.importDurabilityPending = false;
  harness.materializationError = null;
  harness.materializationTracker = materializationTracker;
  harness._storageState = { readOnly: false };
  return harness;
}

describe('DocumentLifecycleSystem deferred hydration scheduling', () => {
  it('promotes host-backed background hydration when an explicit durability barrier waits', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);

    const scheduled = harness.scheduleDeferredHydration();

    expect(completeDeferredHydration).not.toHaveBeenCalled();

    await harness.ensureDeferredHydration();
    await scheduled;

    expect(completeDeferredHydration).toHaveBeenCalledTimes(1);
    expect(harness.deferredHydrationPending).toBe(false);
    expect(harness.importDurabilityPending).toBe(false);
    expect(harness.deferredHydrationTimer).toBeNull();
    expect(harness.startDeferredHydrationNow).toBeNull();
  });

  it('resolves read-only hydration barriers without invoking the bridge write path', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);
    harness._storageState.readOnly = true;

    await expect(harness.scheduleDeferredHydration()).resolves.toBeUndefined();

    expect(completeDeferredHydration).not.toHaveBeenCalled();
    expect(harness.deferredHydrationPending).toBe(false);
    expect(harness.importDurabilityPending).toBe(false);
    expect(
      harness.materializationTracker.requiresDeferredHydration('secondary-sheet' as SheetId),
    ).toBe(false);
  });

  it('uses workbook read-only configuration at the scheduler call site', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);
    harness.setWorkbookReadOnly(true);

    await expect(harness.scheduleDeferredHydration({ immediate: true })).resolves.toBeUndefined();

    expect(completeDeferredHydration).not.toHaveBeenCalled();
    expect(harness.deferredHydrationPending).toBe(false);
    expect(harness.importDurabilityPending).toBe(false);
  });

  it('does not promote read-only hydration while disposing the bridge', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);
    harness.setWorkbookReadOnly(true);
    (
      harness as unknown as { hostProviderMaterializerHandles: unknown[] }
    ).hostProviderMaterializerHandles = [];
    const destroy = jest.fn().mockResolvedValue(undefined);

    await expect(
      (
        harness as unknown as {
          executeDisposeBridge(input: {
            documentContext: null;
            computeBridge: { destroy: typeof destroy };
            rustDocument: null;
          }): Promise<void>;
        }
      ).executeDisposeBridge({
        documentContext: null,
        computeBridge: { destroy },
        rustDocument: null,
      }),
    ).resolves.toBeUndefined();

    expect(completeDeferredHydration).not.toHaveBeenCalled();
    expect(destroy).toHaveBeenCalledTimes(1);
  });

  it('does not force full hydration for the already materialized critical sheet', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);

    await harness.awaitMaterialized('critical-sheet' as SheetId);

    expect(completeDeferredHydration).not.toHaveBeenCalled();
    expect(harness.deferredHydrationPromise).toBeNull();
    expect(harness.getMaterializationState()).toMatchObject({
      phase: 'CriticalSheetReady',
      isDeferred: true,
      isMaterialized: false,
      pendingScope: 'allSheets',
      initialActiveSheetId: 'critical-sheet',
    });
  });

  it('forces full hydration for a non-critical sheet-scoped barrier', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);

    await harness.awaitMaterialized('secondary-sheet' as SheetId);

    expect(completeDeferredHydration).toHaveBeenCalledTimes(1);
    expect(harness.deferredHydrationPending).toBe(false);
    expect(harness.importDurabilityPending).toBe(false);
  });

  it('does not force deferred import hydration for sheets outside the deferred scope', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);

    await expect(harness.awaitMaterialized('added-sheet' as SheetId)).resolves.toBeUndefined();

    expect(completeDeferredHydration).not.toHaveBeenCalled();
  });

  it('reports all-sheets hydrating once the full hydration run is scheduled', async () => {
    const completeDeferredHydration = jest.fn().mockResolvedValue(undefined);
    const harness = createSchedulerHarness(completeDeferredHydration);

    const scheduled = harness.scheduleDeferredHydration();

    expect(harness.getMaterializationState()).toMatchObject({
      phase: 'AllSheetsHydrating',
      isDeferred: true,
      isMaterialized: false,
      pendingScope: 'allSheets',
      initialActiveSheetId: 'critical-sheet',
    });

    harness.startDeferredHydrationNow?.();
    await scheduled;
  });
});
