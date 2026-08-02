import { SwayveClient, withSwayve } from './swayve-client.js';

// Helper to simulate work
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  console.log('Starting Swayve agent simulation...');

  // In a real Swayve environment, these would be set by environment variables
  // For simulation, we'll assume they are present or use defaults
  const client = new SwayveClient();

  try {
    console.log('Registering with Swayve...');
    await client.register();

    await withSwayve(client, async (agent) => {
      console.log('Task started: Simulating tool execution...');
      await agent.sendStatus('executing_tool');
      await sleep(2000);

      console.log('Task continuing: Simulating waiting for human input...');
      await agent.sendStatus('awaiting_human_input');
      await sleep(3000);

      console.log('Task continuing: Requesting focus...');
      await agent.requestFocus();
      await sleep(1000);

      console.log('Task finished successfully.');
    });

    console.log('Agent simulation completed.');
  } catch (error) {
    console.error('Agent simulation failed:', error);
    process.exit(1);
  }
}

main();
