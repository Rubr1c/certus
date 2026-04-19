import fs from "node:fs/promises";
import path from "node:path";
import crypto from "node:crypto";
import { Database } from "bun:sqlite";

const ROOT_DIR = path.resolve(import.meta.dir, "..", "..");
const DEV_DB_PATH = path.join(ROOT_DIR, "dev.db");

const GATEWAY_URL = "http://127.0.0.1:8080";
const API_BASE = `${GATEWAY_URL}/_certus/api/v1`;

const GATEWAY_CMD = [
  "cargo",
  "run",
  "-p",
  "gateway",
  "--",
  "-c",
  "examples/test.certus.config.yaml",
  "--ws",
  "logs,metrics",
];

const SERVER_CMD = [
  "bun",
  "test/src/index.ts",
  "--http1",
  "3100,3101,3102,3105,3106",
  "--http2",
  "3103,3104",
  "--https1",
  "3107,3108",
  "--https2",
  "3109,3110",
  "--jiq",
  "3105,3106",
];

interface SeedCounters {
  total: number;
  ok: number;
  expectedError: number;
  unexpected: number;
}

interface HitOptions {
  timeoutMs?: number;
  strict?: boolean;
}

const counters: SeedCounters = {
  total: 0,
  ok: 0,
  expectedError: 0,
  unexpected: 0,
};

const children: Bun.Subprocess[] = [];

function logInfo(message: string): void {
  console.log(`[seed] ${message}`);
}

function logWarn(message: string): void {
  console.warn(`[seed] WARN: ${message}`);
}

function logError(message: string): void {
  console.error(`[seed] ERROR: ${message}`);
}

function b64Url(input: Buffer | string): string {
  return Buffer.from(input)
    .toString("base64")
    .replace(/=/g, "")
    .replace(/\+/g, "-")
    .replace(/\//g, "_");
}

function signJwtHs256(
  payload: Record<string, unknown>,
  secret: string,
): string {
  const header = { alg: "HS256", typ: "JWT" };
  const encodedHeader = b64Url(JSON.stringify(header));
  const encodedPayload = b64Url(JSON.stringify(payload));
  const unsigned = `${encodedHeader}.${encodedPayload}`;

  const signature = crypto
    .createHmac("sha256", secret)
    .update(unsigned)
    .digest();

  return `${unsigned}.${b64Url(signature)}`;
}

async function removeIfExists(filePath: string): Promise<void> {
  try {
    await fs.unlink(filePath);
    logInfo(`Removed ${path.relative(ROOT_DIR, filePath)}`);
  } catch (error) {
    if (
      typeof error === "object" &&
      error !== null &&
      "code" in error &&
      (error as { code?: string }).code === "ENOENT"
    ) {
      return;
    }

    throw error;
  }
}

async function nukeDevDb(): Promise<void> {
  logInfo("Nuking sqlite dev database files");
  await removeIfExists(DEV_DB_PATH);
  await removeIfExists(`${DEV_DB_PATH}-wal`);
  await removeIfExists(`${DEV_DB_PATH}-shm`);
}

async function fetchWithTimeout(
  url: string,
  init: RequestInit = {},
  timeoutMs = 8000,
): Promise<Response> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    return await fetch(url, {
      ...init,
      signal: controller.signal,
    });
  } finally {
    clearTimeout(timeout);
  }
}

async function waitForReady(
  name: string,
  url: string,
  timeoutMs: number,
): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetchWithTimeout(url, {}, 1200);
      if (res.ok) {
        logInfo(`${name} is ready (${res.status})`);
        return;
      }
    } catch {
      // ignore transient startup errors
    }

    await Bun.sleep(250);
  }

  throw new Error(`Timed out waiting for ${name} readiness: ${url}`);
}

function spawnService(name: string, cmd: string[]): Bun.Subprocess {
  logInfo(`Starting ${name}: ${cmd.join(" ")}`);

  const child = Bun.spawn({
    cmd,
    cwd: ROOT_DIR,
    stdout: "inherit",
    stderr: "inherit",
  });

  children.push(child);
  return child;
}

async function stopChildren(): Promise<void> {
  if (children.length === 0) {
    return;
  }

  logInfo("Stopping spawned processes");

  for (const child of children.reverse()) {
    try {
      child.kill("SIGTERM");
    } catch {
      // ignore
    }
  }

  await Promise.allSettled(children.map((child) => child.exited));
}

async function hit(
  label: string,
  pathOrUrl: string,
  init: RequestInit,
  expectedStatus: number[],
  options: HitOptions = {},
): Promise<Response | null> {
  counters.total += 1;

  const isAbsolute = /^https?:\/\//.test(pathOrUrl);
  const url = isAbsolute ? pathOrUrl : `${GATEWAY_URL}${pathOrUrl}`;

  const reqHeaders = new Headers(init.headers ?? {});
  if (!reqHeaders.has("x-seed-key")) {
    reqHeaders.set("x-seed-key", `seed-${counters.total}`);
  }

  const strict = options.strict ?? true;
  const timeoutMs = options.timeoutMs ?? 12_000;

  try {
    const res = await fetchWithTimeout(
      url,
      {
        ...init,
        headers: reqHeaders,
      },
      timeoutMs,
    );
    const expected = expectedStatus.includes(res.status);
    const method = (init.method ?? "GET").toUpperCase();

    if (expected) {
      if (res.status >= 400) {
        counters.expectedError += 1;
      } else {
        counters.ok += 1;
      }

      logInfo(`${method} ${pathOrUrl} -> ${res.status} (${label})`);
      return res;
    }

    if (!strict) {
      logWarn(
        `${method} ${pathOrUrl} unexpected status ${res.status}; expected [${expectedStatus.join(
          ", ",
        )}] (soft-fail ${label})`,
      );
      return res;
    }

    counters.unexpected += 1;

    let bodyPreview = "";
    try {
      const text = await res.text();
      bodyPreview = text.slice(0, 160).replace(/\s+/g, " ");
    } catch {
      bodyPreview = "<unreadable>";
    }

    logWarn(
      `${method} ${pathOrUrl} unexpected status ${res.status}; expected [${expectedStatus.join(
        ", ",
      )}] body=${bodyPreview}`,
    );
    return res;
  } catch (error) {
    if (!strict) {
      logWarn(`${label} request failed (soft-fail): ${String(error)}`);
      return null;
    }

    counters.unexpected += 1;
    logWarn(`${label} request failed: ${String(error)}`);
    return null;
  }
}

async function hitJson<T>(
  label: string,
  pathOrUrl: string,
  init: RequestInit,
  expectedStatus: number[],
  options: HitOptions = {},
): Promise<T | null> {
  const res = await hit(label, pathOrUrl, init, expectedStatus, options);
  if (!res) {
    return null;
  }

  try {
    return (await res.json()) as T;
  } catch {
    return null;
  }
}

async function seedTraffic(authToken: string): Promise<void> {
  logInfo("Starting gateway traffic seeding");

  const jsonHeaders = {
    "content-type": "application/json",
  };

  await hit("users list cold miss", "/api/users", { method: "GET" }, [200]);
  await hit("users list slight delay", "/api/users?delay=12", { method: "GET" }, [200]);
  await hit("users list cache hit", "/api/users", { method: "GET" }, [200]);
  await hit(
    "users list with query miss",
    "/api/users?include_inactive=1",
    { method: "GET" },
    [200],
  );
  await hit(
    "users list with query cache hit",
    "/api/users?include_inactive=1",
    { method: "GET" },
    [200],
  );
  await hit("single user miss", "/api/users/1", { method: "GET" }, [200]);
  await hit("single user delayed", "/api/users/1?delay=24", { method: "GET" }, [200]);
  await hit("single user cache hit", "/api/users/1", { method: "GET" }, [200]);
  await hit("http2 item miss", "/api/http2/items", { method: "GET" }, [200]);
  await hit("http2 item delayed", "/api/http2/items?delay=18", { method: "GET" }, [200]);
  await hit("http2 item cache hit", "/api/http2/items", { method: "GET" }, [200]);
  await hit("static route hit", "/api/static", { method: "GET" }, [200]);
  await hit("static route repeat", "/api/static", { method: "GET" }, [200]);

  await hit("users head", "/api/users", { method: "HEAD" }, [200]);
  await hit("users options", "/api/users", { method: "OPTIONS" }, [200, 204]);

  await hit(
    "create user success",
    "/api/users",
    {
      method: "POST",
      headers: jsonHeaders,
      body: JSON.stringify({
        name: "Seed User",
        email: "seed.user@example.test",
        role: "analyst",
        tags: ["seed", "qa"],
      }),
    },
    [201],
  );

  await hit(
    "create user validation error",
    "/api/users",
    {
      method: "POST",
      headers: jsonHeaders,
      body: JSON.stringify({ name: "Broken User", email: "not-an-email" }),
    },
    [422],
  );

  await hit(
    "patch user",
    "/api/users/1",
    {
      method: "PATCH",
      headers: jsonHeaders,
      body: JSON.stringify({ role: "owner", tags: ["core", "seed"] }),
    },
    [200],
  );

  await hit(
    "http2 create order",
    "/api/http2/items",
    {
      method: "POST",
      headers: jsonHeaders,
      body: JSON.stringify({
        user_id: 1,
        items: ["monitor", "dock"],
        total: 329.5,
      }),
    },
    [201],
  );

  await hit(
    "http2 create order invalid user",
    "/api/http2/items",
    {
      method: "POST",
      headers: jsonHeaders,
      body: JSON.stringify({ user_id: 9999, items: ["ghost-item"] }),
    },
    [404],
  );

  await hit(
    "users redirect",
    "/api/users/redirect",
    { method: "GET", redirect: "manual" },
    [302],
  );
  await hit("users server error", "/api/users/error", { method: "GET" }, [500]);
  await hit("users forced 400", "/api/users?error=400", { method: "GET" }, [400]);
  await hit("users blob content-length", "/api/users/blob?size=8192", { method: "GET" }, [200]);

  await hit("public text", "/api/public/text", { method: "GET" }, [200]);
  await hit("public text delayed", "/api/public/text?delay=16", { method: "GET" }, [200]);
  await hit("public text with length", "/api/public/text/length", { method: "GET" }, [200]);
  await hit("public text chunk-style", "/api/public/text/chunked", { method: "GET" }, [200]);
  await hit(
    "public text echo post",
    "/api/public/text/echo",
    {
      method: "POST",
      headers: { "content-type": "text/plain" },
      body: "seed echo payload line one\nline two\nline three",
    },
    [200],
  );
  await hit("public forced 503", "/api/public/text?error=503", { method: "GET" }, [503]);

  await hit(
    "private profile unauthorized missing token",
    "/api/private/profile",
    { method: "GET" },
    [401],
  );
  await hit(
    "private profile unauthorized bad token",
    "/api/private/profile",
    {
      method: "GET",
      headers: { authorization: "Bearer broken.token.signature" },
    },
    [401],
  );
  await hit(
    "private profile authorized",
    "/api/private/profile",
    {
      method: "GET",
      headers: { authorization: `Bearer ${authToken}` },
    },
    [200],
  );
  await hit(
    "private nocache authorized",
    "/api/private/nocache",
    {
      method: "POST",
      headers: {
        authorization: `Bearer ${authToken}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({ test: true, mode: "seed" }),
    },
    [200],
  );

  await Promise.all(
    Array.from({ length: 12 }, (_, idx) =>
      hit(
        `rate limit burst #${idx + 1}`,
        "/api/limited",
        {
          method: "GET",
          headers: { "x-seed-key": "rate-main" },
        },
        [200, 429, 503],
      ),
    ),
  );

  await Promise.all(
    Array.from({ length: 8 }, (_, idx) =>
      hit(
        `overload concurrency #${idx + 1}`,
        "/api/overload?delay=1200",
        {
          method: "GET",
          headers: { "x-seed-key": "overload-main" },
        },
        [200, 503],
      ),
    ),
  );

  await hit("https1 route", "/api/https1/data", { method: "GET" }, [200]);
  await hit("https1 route delayed", "/api/https1/data?delay=15", { method: "GET" }, [200]);
  await hit("https2 route", "/api/https2/data", { method: "GET" }, [200]);
  await hit("https2 route delayed", "/api/https2/data?delay=20", { method: "GET" }, [200]);

  await hit(
    "invalid request metrics interval",
    `${API_BASE}/metrics/requests/aggregate?interval=7m`,
    { method: "GET" },
    [400],
  );
  await hit(
    "invalid cache metrics interval",
    `${API_BASE}/metrics/cache/aggregate?interval=13m`,
    { method: "GET" },
    [400],
  );
  await hit(
    "invalid summary group_by",
    `${API_BASE}/metrics/requests/summary?group_by=badgroup`,
    { method: "GET" },
    [400],
  );
  await hit(
    "missing upstream health lookup",
    `${API_BASE}/upstreams/does-not-exist/health`,
    { method: "GET" },
    [404],
  );

  await hit("internal routes snapshot", `${API_BASE}/routes`, { method: "GET" }, [200]);
  await hit("internal config snapshot", `${API_BASE}/config`, { method: "GET" }, [200]);
  await hit("internal logs fetch", `${API_BASE}/logs?page=0&per_page=5`, { method: "GET" }, [200]);
}

function spreadTableTimestamps(
  db: Database,
  tableName: "request_metrics" | "cache_metrics" | "logs",
  format: string,
  minutesStep: number,
  secondsSeed: number,
): void {
  db.exec(`
    UPDATE ${tableName}
    SET timestamp = strftime(
      '${format}',
      'now',
      '-' || ((id - COALESCE((SELECT MIN(id) FROM ${tableName}), id)) * ${minutesStep}) || ' minutes',
      '-' || ((((id * ${secondsSeed}) % 55) + 2)) || ' seconds'
    )
    WHERE id IS NOT NULL;
  `);
}

function countRows(db: Database, tableName: "request_metrics" | "cache_metrics" | "logs"): number {
  const row = db.query(`SELECT COUNT(*) AS c FROM ${tableName}`).get() as
    | { c: number }
    | null;

  return Number(row?.c ?? 0);
}

async function spreadTimeline(): Promise<void> {
  logInfo("Spreading seed timestamps across timeline for charts");

  const maxRetries = 6;
  for (let attempt = 1; attempt <= maxRetries; attempt += 1) {
    let db: Database | null = null;

    try {
      db = new Database(DEV_DB_PATH);
      db.exec("PRAGMA busy_timeout = 5000;");

      const requestCount = countRows(db, "request_metrics");
      const cacheCount = countRows(db, "cache_metrics");
      const logCount = countRows(db, "logs");

      if (requestCount > 0) {
        spreadTableTimestamps(
          db,
          "request_metrics",
          "%Y-%m-%d %H:%M:%f UTC",
          4,
          11,
        );
      }

      if (cacheCount > 0) {
        spreadTableTimestamps(
          db,
          "cache_metrics",
          "%Y-%m-%d %H:%M:%f UTC",
          5,
          13,
        );
      }

      if (logCount > 0) {
        spreadTableTimestamps(db, "logs", "%Y-%m-%dT%H:%M:%fZ", 3, 7);
      }

      db.close();

      logInfo(
        `Timestamp spread complete (request=${requestCount}, cache=${cacheCount}, logs=${logCount})`,
      );
      return;
    } catch (error) {
      const message = String(error).toLowerCase();
      if (!message.includes("locked") || attempt === maxRetries) {
        throw error;
      }

      logWarn(`dev.db locked during timeline spread (retry ${attempt}/${maxRetries})`);
      await Bun.sleep(400 * attempt);
    } finally {
      if (db) {
        try {
          db.close();
        } catch {
          // ignore
        }
      }
    }
  }
}

async function generateDocsAndSummarize(): Promise<void> {
  logInfo("Waiting for flusher tasks before timeline spread and docs generation");
  await Bun.sleep(2200);

  await spreadTimeline();

  const docsGenerate = await hitJson<Record<string, unknown>>(
    "docs generate",
    `${API_BASE}/docs/generate`,
    { method: "POST" },
    [200, 502, 503],
    {
      timeoutMs: 180_000,
      strict: false,
    },
  );

  if (docsGenerate && "error" in docsGenerate) {
    logWarn(`docs/generate returned error: ${String(docsGenerate.error)}`);
  }

  const docsLatest = await hitJson<Record<string, unknown> | null>(
    "docs latest",
    `${API_BASE}/docs`,
    { method: "GET" },
    [200, 404],
  );

  const requestRows = await hitJson<Array<Record<string, unknown>>>(
    "request metrics sample",
    `${API_BASE}/metrics/requests?page=0&per_page=120`,
    { method: "GET" },
    [200],
  );

  const cacheRows = await hitJson<Array<Record<string, unknown>>>(
    "cache metrics sample",
    `${API_BASE}/metrics/cache?page=0&per_page=120`,
    { method: "GET" },
    [200],
  );

  const logRows = await hitJson<Array<Record<string, unknown>>>(
    "logs sample",
    `${API_BASE}/logs?page=0&per_page=120`,
    { method: "GET" },
    [200],
  );

  const schemaRows = await hitJson<Array<Record<string, unknown>>>(
    "schema sample",
    `${API_BASE}/schemas?page=0&per_page=120`,
    { method: "GET" },
    [200],
  );

  const docsEndpoints =
    docsLatest && typeof docsLatest === "object" && Array.isArray(docsLatest.endpoints)
      ? docsLatest.endpoints.length
      : 0;

  logInfo(`Request metric rows fetched: ${requestRows?.length ?? 0}`);
  logInfo(`Cache metric rows fetched: ${cacheRows?.length ?? 0}`);
  logInfo(`Log rows fetched: ${logRows?.length ?? 0}`);
  logInfo(`Schema rows fetched: ${schemaRows?.length ?? 0}`);
  logInfo(`Generated docs endpoints: ${docsEndpoints}`);
}

async function main(): Promise<void> {
  const authToken = signJwtHs256(
    {
      user_id: "seed-user-1",
      role: "seed-admin",
      exp: Math.floor(Date.now() / 1000) + 60 * 60,
    },
    "test-seed-jwt-secret",
  );

  process.on("SIGINT", () => {
    logWarn("Received SIGINT");
    void stopChildren().finally(() => process.exit(130));
  });

  process.on("SIGTERM", () => {
    logWarn("Received SIGTERM");
    void stopChildren().finally(() => process.exit(143));
  });

  await nukeDevDb();

  const servers = spawnService("test servers", SERVER_CMD);
  const gateway = spawnService("gateway", GATEWAY_CMD);

  try {
    await waitForReady("test server", "http://127.0.0.1:3101/", 30_000);
    await waitForReady("gateway", `${API_BASE}/args`, 120_000);

    if (servers.exitCode !== null) {
      throw new Error(`Test servers exited early with code ${servers.exitCode}`);
    }
    if (gateway.exitCode !== null) {
      throw new Error(`Gateway exited early with code ${gateway.exitCode}`);
    }

    await hit("reset in-memory service state", "http://127.0.0.1:3101/_seed/reset", { method: "POST" }, [200]);

    await seedTraffic(authToken);
    await generateDocsAndSummarize();

    logInfo(
      `Seed complete: total=${counters.total}, ok=${counters.ok}, expected_errors=${counters.expectedError}, unexpected=${counters.unexpected}`,
    );

    if (counters.unexpected > 0) {
      process.exitCode = 1;
    }
  } finally {
    await stopChildren();
  }
}

void main().catch(async (error) => {
  logError(String(error));
  await stopChildren();
  process.exit(1);
});
