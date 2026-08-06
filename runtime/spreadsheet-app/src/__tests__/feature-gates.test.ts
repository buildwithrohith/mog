import assert from 'node:assert/strict';
import test from 'node:test';

import { enforceAttachmentDocumentReadOnly } from '../attachment-runtime';
import { mergeFeatureGates } from '../feature-gates';

test('request-level document read-only cannot be weakened by attachment editing policy', () => {
  const hostWritablePolicy = { editing: true, ribbon: true };

  assert.deepEqual(enforceAttachmentDocumentReadOnly(hostWritablePolicy, true), {
    editing: false,
    ribbon: true,
  });
  assert.equal(enforceAttachmentDocumentReadOnly(hostWritablePolicy, false), hostWritablePolicy);
});

test('version-control feature gates fail closed without weakening host capability policy', () => {
  const unavailable = mergeFeatureGates(
    {
      capabilities: {
        versionControl: true,
        versionControlMerge: true,
        'versionControl.merge': true,
        customHostCapability: true,
      },
    },
    undefined,
    undefined,
    undefined,
    { versionControl: false },
  );

  assert.equal(unavailable.capabilities?.versionControl, false);
  assert.equal(unavailable.capabilities?.versionControlMerge, false);
  assert.equal(unavailable.capabilities?.['versionControl.merge'], false);
  assert.equal(unavailable.capabilities?.customHostCapability, true);

  const available = mergeFeatureGates(
    {
      capabilities: {
        versionControl: false,
        versionControlMerge: false,
        'versionControl.merge': false,
      },
    },
    undefined,
    undefined,
    undefined,
    { versionControl: true },
  );

  assert.equal(available.capabilities?.versionControl, false);
  assert.equal(available.capabilities?.versionControlMerge, false);
  assert.equal(available.capabilities?.['versionControl.merge'], false);
});
