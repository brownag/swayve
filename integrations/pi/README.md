# Swayve TypeScript Client

A production-ready TypeScript library and scaffold for integrating agents with **Swayve**, a native Sway window manager orchestrator for autonomous agents.

## Features

- **Type-safe**: Full TypeScript support for agent statuses and client methods.
- **Robust IPC**: Handles UNIX domain socket communication with error resilience and timeouts.
- **Status Management**: Easy methods for updating agent status (`thinking`, `executing_tool`, etc.).
- **Scaffold Included**: A `run-agent.ts` script to quickly test and implement your agent logic.

## Installation

```bash
# Clone this repository
git clone <repository-url>
cd swayve-client

# Install dependencies
npm install
```

## Integration

### 1. Set Environment Variables

Swayve injects the following variables into your agent's environment:

- `SWAYVE_SOCKET`: Path to the UNIX domain socket.
- `SWAYVE_AGENT_ID`: Unique identifier for the agent.
- `SWAYVE_PROJECT_ID`: Project or workspace identifier.

### 2. Using the Client

```typescript
import { SwayveClient, withSwayve } from 'swayve-client';

const client = new SwayveClient();

async function myAgentTask() {
  // Register the agent with Swayve
  await client.register();

  // Wrap your main execution loop for automatic status management
  await withSwayve(client, async (agent) => {
    console.log('Starting my task...');
    
    // Manually update status for specific steps
    await agent.sendStatus('executing_tool');
    
    // ... your agent logic here ...
    
    await agent.requestFocus();
  });
}

myAgentTask().catch(console.error);
```

## Development

To run the provided scaffold script:

```bash
npm run dev:run-agent
```

To build the library:

```bash
npm run build
```

## Testing

To run the integration test suite:

```bash
npm test
```


## API Reference

### `SwayveClient`

| Method | Signature | Description |
|---|---|---|
| `register()` | `Promise<void>` | Registers the agent with the Swayve daemon. |
| `sendStatus(status)` | `Promise<void>` | Updates the agent's status (e.g., `'thinking'`, `'idle'`). |
| `requestFocus()` | `Promise<void>` | Requests the daemon to focus the agent's window. |

### `withSwayve`

A higher-order function that manages agent status automatically:
- Sets status to `'thinking'` before the task runs.
- Sets status to `'completed'` after successful completion.
- Sets status to `'failed'` if an error is thrown.

```typescript
await withSwayve(client, async (agent) => {
  // Your logic
});
```
