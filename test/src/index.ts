import http2, { type IncomingHttpHeaders, type ServerHttp2Stream } from 'node:http2'
import https from 'node:https'
import fs from 'node:fs'
import path from 'node:path'
import { parseArgs } from 'node:util';

const { values } = parseArgs({
  args: Bun.argv,
  options: {
    http1: {
      type: "string",
    },
    http2: {
      type: "string",
    },
    https1: {
      type: "string",
    },
    https2: {
      type: "string",
    },
    log: {
      type: "boolean",
    }
  },
  strict: true,
  allowPositionals: true,
});

const http1Ports = values.http1?.split(",").map(port => parseInt(port));
const http2Ports = values.http2?.split(",").map(port => parseInt(port));
const https1Ports = values.https1?.split(",").map(port => parseInt(port));
const https2Ports = values.https2?.split(",").map(port => parseInt(port));

const hostname = '127.0.0.1';

const certsDir = path.resolve(import.meta.dir, '..', 'certs');
const tlsKey = fs.readFileSync(path.join(certsDir, 'key.pem'));
const tlsCert = fs.readFileSync(path.join(certsDir, 'cert.pem'));

function createServer(port: number) {
  const socketAddr = `${hostname}:${port}`;
  console.log(`[http1] Running server on ${socketAddr}`)
  return Bun.serve({
    port,
    hostname,
    fetch(req) {
      if (values.log) {
        console.log(`[INFO] bun::server::http1 headers=${JSON.stringify(Object.fromEntries(req.headers))}`)
      }

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

    if (values.log) {
      console.log(`[INFO] bun::server::http2 headers=${JSON.stringify(headers)}`)
    }

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

function createHttps1Server(port: number) {
  const socketAddr = `${hostname}:${port}`;

  const server = https.createServer({ key: tlsKey, cert: tlsCert }, (req, res) => {
    if (values.log) {
      console.log(`[INFO] bun::server::https1 headers=${JSON.stringify(req.headers)}`)
    }

    res.writeHead(200, { 'content-type': 'text/plain' });
    res.end(`${req.url} => ${socketAddr}`);
  });

  server.listen(port, hostname, () => {
    console.log(`[https1](TLS) Running server on ${hostname}:${port}`);
  });
}

function createHttps2Server(port: number) {
  const socketAddr = `${hostname}:${port}`;

  const server = http2.createSecureServer({ key: tlsKey, cert: tlsCert });

  server.on('stream', (stream: ServerHttp2Stream, headers: IncomingHttpHeaders) => {
    const reqPath = headers[':path'];

    if (values.log) {
      console.log(`[INFO] bun::server::https2 headers=${JSON.stringify(headers)}`)
    }

    stream.respond({
      ':status': 200,
      'content-type': 'text/plain',
    });

    stream.end(`${reqPath} => ${socketAddr}`);
  });

  server.listen(port, hostname, () => {
    console.log(`[https2](TLS) Running server on ${hostname}:${port}`);
  });
}

if (!http1Ports) process.exit(1);
if (!http2Ports) process.exit(1);

for (const port of http1Ports) {
  createServer(port);
}

for (const port of http2Ports) {
  createHttp2Server(port);
}

if (https1Ports) {
  for (const port of https1Ports) {
    createHttps1Server(port);
  }
}

if (https2Ports) {
  for (const port of https2Ports) {
    createHttps2Server(port);
  }
}
