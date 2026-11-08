import { test, describe } from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { SwayveClient } from '../src/swayve-client.js';

describe('SwayveClient', () => {
  const socketPath = path.join(process.cwd(), 'test.sock');

  // Cleanup after tests
  test('cleanup', async () => {
    if (fs.existsSync(socketPath)) {
      fs.unlinkSync(socketPath);
    }
  });

  test('handles missing socket gracefully', async () => {
    const client = new SwayveClient({ socketPath: path.join(os.tmpdir(), 'swayve-missing-' + process.pid + '.sock') });
    // Should not throw
    await assert.doesNotReject(client.sendStatus('thinking'));
  });

  test('handles connection errors gracefully', async () => {
    // Create a file that isn't a socket to force a connection error
    const dummyFile = path.join(process.cwd(), 'dummy.sock');
    fs.writeFileSync(dummyFile, 'not a socket');
    
    const client = new SwayveClient({ socketPath: dummyFile });
    
    try {
      // Should not throw due to internal error handling
      await assert.doesNotReject(client.sendStatus('thinking'));
    } finally {
      if (fs.existsSync(dummyFile)) {
        fs.unlinkSync(dummyFile);
      }
    }
  });
});
