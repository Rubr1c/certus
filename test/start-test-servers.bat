@echo off
echo Starting test servers...

cd /d "%~dp0"
start "Test Server 3001" bun run test-server.ts 3001
start "Test Server 3002" bun run test-server.ts 3002
start "Test Server 3003" bun run test-server.ts 3003
start "Test Server 3004" bun run test-server.ts 3004
start "Test Server 3005" bun run test-server.ts 3005
start "Test Server 3006" bun run test-server.ts 3006

echo All test servers started!
echo Close the windows to stop the servers
pause

