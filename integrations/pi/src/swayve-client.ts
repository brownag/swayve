import net from 'node:net';
import fs from 'node:fs';

export type AgentStatus = 'idle' | 'thinking' | 'executing_tool' | 'awaiting_human_input' | 'completed' | 'failed';

export interface SwayveClientConfig {
  socketPath?: string;
  agentId?: string;
  projectId?: string;
  parentAgentId?: string;
}

export class SwayveClient {
  private readonly socketPath: string;
  private readonly agentId: string;
  private readonly projectId: string;
  private readonly parentAgentId: string | null;

  constructor(config: SwayveClientConfig = {}) {
    this.socketPath = config.socketPath || process.env.SWAYVE_SOCKET || '';
    this.agentId = config.agentId || process.env.SWAYVE_AGENT_ID || 'unknown-agent';
    this.projectId = config.projectId || process.env.SWAYVE_PROJECT_ID || 'default-project';
    this.parentAgentId = config.parentAgentId ?? process.env.SWAYVE_PARENT_ID ?? null;
  }

  private async _send(payload: Record<string, any>): Promise<void> {
    if (!this.socketPath) {
      return;
    }

    // Check if socket exists before attempting connection
    if (!this.socketPath || !fs.existsSync(this.socketPath)) {
      return;
    }

    return new Promise((resolve) => {
      const client = net.createConnection(this.socketPath, () => {
        client.write(JSON.stringify(payload) + '\n', () => {
          client.end();
          resolve();
        });
      });

      client.on('error', (err) => {
        console.warn(`[Swayve IPC Warning] Failed to send IPC payload: ${err.message}`);
        resolve(); // Fail gracefully as requested
      });

      // Set a timeout for the connection attempt
      client.setTimeout(1000, () => {
        client.destroy();
        console.warn(`[Swayve IPC Warning] Connection attempt timed out`);
        resolve();
      });
    });
  }

  /**
   * Registers the agent with the Swayve daemon.
   */
  async register(): Promise<void> {
    await this._send({
      method: 'agent.register',
      params: {
        agent_id: this.agentId,
        agent_type: 'pi',
        project_id: this.projectId,
        parent_agent_id: this.parentAgentId,
        pid: process.pid,
        app_id: `agent.${this.agentId}`,
      }
    });
  }

  /**
   * Updates the agent's current status.
   */
  async sendStatus(status: AgentStatus): Promise<void> {
    await this._send({
      method: 'agent.update_status',
      params: {
        agent_id: this.agentId,
        status: status,
      }
    });
  }

  /**
   * Requests the daemon to focus the current agent's terminal.
   */
  async requestFocus(): Promise<void> {
    await this._send({
      method: 'daemon.focus',
    });
  }
}

/**
 * A higher-order function to wrap an agent's execution loop with automatic status management.
 * It sets status to 'thinking' at the start and 'completed' or 'failed' at the end.
 *
 * @param client The SwayveClient instance.
 * @param task An async function representing the agent's main task.
 * @returns The result of the task.
 */
export async function withSwayve<T>(
  client: SwayveClient,
  task: (client: SwayveClient) => Promise<T>
): Promise<T> {
  await client.sendStatus('thinking');
  try {
    const result = await task(client);
    await client.sendStatus('completed');
    return result;
  } catch (error) {
    await client.sendStatus('failed');
    throw error;
  }
}
