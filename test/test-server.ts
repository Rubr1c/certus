#!/usr/bin/env bun

const port = parseInt(process.argv[2] || '3000', 10)

if (isNaN(port)) {
  console.error('Invalid port number')
  process.exit(1)
}

const socketAddr = `127.0.0.1:${port}`

const server = Bun.serve({
  port,
  hostname: '127.0.0.1',
  fetch(req) {
    return new Response(socketAddr, {
      headers: { 'Content-Type': 'text/plain' },
    })
  },
})

console.log(`Test server running on ${socketAddr}`)
