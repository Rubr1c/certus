import http2, { type IncomingHttpHeaders, type ServerHttp2Stream } from 'node:http2'
import { parseArgs } from 'node:util';

const hostname = '127.0.0.1';

function createServer(port: number) {
  const socketAddr = `${hostname}:${port}`;
  console.log(`[http1] Running server on ${socketAddr}`)
  return Bun.serve({
    port,
    hostname,
    fetch(req) {
      return new Response(`${req.url} => ${socketAddr}`, {
        headers: { 'Content-Type': 'text/plain' },
      })
    },
  });
}

function createHttp2Server(port: number) {
  const socketAddr = `${hostname}:${port}`;

  const server = http2.createServer();

  server.on('stream', (stream: ServerHttp2Stream, headers: IncomingHttpHeaders) => {
    const path = headers[':path'];

    stream.respond({
      ':status': 200,
      'content-type': 'text/plain',
    });

    stream.end(`${path} => ${socketAddr}`);
  });

  server.listen(port, hostname, () => {
    console.log(`[http2](Cleartext) Running server on ${hostname}:${port}`);
  });
}

const { values } = parseArgs({
  args: Bun.argv,
  options: {
    http1: {
      type: "string",
    },
    http2: {
      type: "string",
    },
  },
  strict: true,
  allowPositionals: true,
});


let http1Ports = values.http1?.split(",").map(port => parseInt(port));
let http2Ports = values.http2?.split(",").map(port => parseInt(port));

if (!http1Ports) process.exit(1);
if (!http2Ports) process.exit(1);
for (const port of http1Ports) {
  createServer(port);
}

for (const port of http2Ports) {
  createHttp2Server(port);
}
