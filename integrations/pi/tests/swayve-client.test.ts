import { test, describe } from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import net from 'node:net';
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

  // Runs register() against a temp-socket server and returns the captured params.
  async function captureRegister(config: ConstructorParameters<typeof SwayveClient>[0], env?: string) {
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'swayve-pi-'));
    const prev = process.env.SWAYVE_PARENT_ID;
    if (env === undefined) delete process.env.SWAYVE_PARENT_ID;
    else process.env.SWAYVE_PARENT_ID = env;
    const sock = path.join(dir, 's.sock');
    const server = net.createServer();
    try {
      const line = new Promise<string>((resolve) => {
        server.on('connection', (c) => {
          let buf = '';
          c.on('data', (d) => (buf += d));
          c.on('end', () => resolve(buf));
        });
      });
      await new Promise<void>((r) => server.listen(sock, r));
      await new SwayveClient({ ...config, socketPath: sock }).register();
      return JSON.parse((await line).split('\n')[0]).params;
    } finally {
      await new Promise((r) => server.close(r));
      if (prev === undefined) delete process.env.SWAYVE_PARENT_ID;
      else process.env.SWAYVE_PARENT_ID = prev;
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  test('register sends configured parent_agent_id', async () => {
    const params = await captureRegister({ parentAgentId: 'parent-1' });
    assert.strictEqual(params.parent_agent_id, 'parent-1');
  });

  test('register sends null parent_agent_id when unset', async () => {
    const params = await captureRegister({});
    assert.ok('parent_agent_id' in params);
    assert.strictEqual(params.parent_agent_id, null);
  });

  test('register reads parent_agent_id from SWAYVE_PARENT_ID', async () => {
    const params = await captureRegister({}, 'env-parent');
    assert.strictEqual(params.parent_agent_id, 'env-parent');
  });

  test('register sends null parent_agent_id when SWAYVE_PARENT_ID is empty', async () => {
    const params = await captureRegister({}, '');
    assert.strictEqual(params.parent_agent_id, null);
  });
});
