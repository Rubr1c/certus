import path from "node:path";

const ROOT_DIR = path.resolve(import.meta.dir, "..", "..");

const SERVER_CMD = [
  "bun",
  "test/src/index.ts",
  "--http1",
  "3101,3102",
  "--http2",
  "3103",
];

const BENCH_CMD = ["cargo", "bench", "-p", "gateway", "--bench", "flow"];

const children: Bun.Subprocess[] = [];
let isStopping = false;

function logInfo(message: string): void {
  console.log(`[bench] ${message}`);
}

function logWarn(message: string): void {
  console.warn(`[bench] WARN: ${message}`);
}

function logError(message: string): void {
  console.error(`[bench] ERROR: ${message}`);
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
  if (isStopping) {
    return;
  }

  isStopping = true;

  if (children.length === 0) {
    return;
  }

  logInfo("Stopping spawned processes");

  for (const child of children.reverse()) {
    try {
      child.kill("SIGTERM");
    } catch {
      // no-op
    }
  }

  await Promise.race([
    Promise.allSettled(children.map((child) => child.exited)),
    Bun.sleep(4000),
  ]);

  for (const child of children) {
    if (child.exitCode === null) {
      try {
        child.kill("SIGKILL");
      } catch {
        // no-op
      }
    }
  }

  await Promise.allSettled(children.map((child) => child.exited));
}

async function fetchWithTimeout(url: string, timeoutMs = 1200): Promise<Response> {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    return await fetch(url, { signal: controller.signal });
  } finally {
    clearTimeout(timeout);
  }
}

async function waitForReady(name: string, url: string, timeoutMs: number): Promise<void> {
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetchWithTimeout(url);
      if (res.ok) {
        logInfo(`${name} is ready (${res.status}): ${url}`);
        return;
      }
    } catch {
      // no-op
    }

    await Bun.sleep(200);
  }

  throw new Error(`Timed out waiting for ${name}: ${url}`);
}

async function runBench(): Promise<number> {
  logInfo(`Running benchmark: ${BENCH_CMD.join(" ")}`);

  const bench = Bun.spawn({
    cmd: BENCH_CMD,
    cwd: ROOT_DIR,
    stdout: "inherit",
    stderr: "inherit",
  });

  const exitCode = await bench.exited;
  logInfo(`Benchmark process exited with code ${exitCode}`);
  return exitCode;
}

async function main(): Promise<void> {
  process.on("SIGINT", () => {
    logWarn("Received SIGINT");
    void stopChildren().finally(() => process.exit(130));
  });

  process.on("SIGTERM", () => {
    logWarn("Received SIGTERM");
    void stopChildren().finally(() => process.exit(143));
  });

  const servers = spawnService("benchmark test servers", SERVER_CMD);

  try {
    await waitForReady("http1 upstream", "http://127.0.0.1:3101/", 30_000);
    await waitForReady("http1 upstream", "http://127.0.0.1:3102/", 30_000);
    await waitForReady(
      "public route endpoint",
      "http://127.0.0.1:3101/api/public/text",
      30_000,
    );
    await waitForReady("users route endpoint", "http://127.0.0.1:3102/api/users", 30_000);

    if (servers.exitCode !== null) {
      throw new Error(`Test servers exited early with code ${servers.exitCode}`);
    }

    const benchExitCode = await runBench();
    if (benchExitCode !== 0) {
      process.exitCode = benchExitCode;
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
