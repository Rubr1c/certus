#!/bin/bash

# Start all test servers in the background
echo "Starting test servers..."

cd "$(dirname "$0")"
bun run test-server.ts 3001 &
bun run test-server.ts 3002 &
bun run test-server.ts 3003 &
bun run test-server.ts 3004 &
bun run test-server.ts 3005 &
bun run test-server.ts 3006 &

echo "All test servers started!"
echo "Press Ctrl+C to stop all servers"
echo "To stop manually, run: pkill -f test-server.ts"

# Wait for user interrupt
wait

