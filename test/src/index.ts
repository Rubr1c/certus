import http2, { type IncomingHttpHeaders, type ServerHttp2Stream } from "node:http2";
import https from "node:https";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";

interface UserRecord {
  id: number;
  name: string;
  email: string;
  role: string;
  active: boolean;
  tags: string[];
  created_at: string;
}

interface OrderRecord {
  id: number;
  user_id: number;
  total: number;
  currency: string;
  items: string[];
  created_at: string;
}

interface EventRecord {
  id: number;
  type: string;
  timestamp: string;
  details: Record<string, unknown>;
}

interface ServiceState {
  users: Map<number, UserRecord>;
  orders: Map<number, OrderRecord>;
  events: EventRecord[];
  nextUserId: number;
  nextOrderId: number;
  nextEventId: number;
}

interface ServiceRequest {
  method: string;
  rawPath: string;
  headers: Record<string, string>;
  bodyText: string;
  socketAddr: string;
  protocol: "http" | "https";
}

interface ServiceResponse {
  status: number;
  headers?: Record<string, string>;
  body?: string;
  delayMs?: number;
}

const { values } = parseArgs({
  args: Bun.argv,
  options: {
    http1: { type: "string" },
    http2: { type: "string" },
    https1: { type: "string" },
    https2: { type: "string" },
    jiq: { type: "string" },
    log: { type: "boolean" },
  },
  strict: true,
  allowPositionals: true,
});

const hostname = "127.0.0.1";
const GATEWAY_IDLE_URL = "http://127.0.0.1:8080/_certus/api/v1/idle";
const JIQ_WORK_MS = 2000;

const http1Ports = parsePorts(values.http1);
const http2Ports = parsePorts(values.http2);
const https1Ports = parsePorts(values.https1);
const https2Ports = parsePorts(values.https2);
const jiqPorts = new Set(parsePorts(values.jiq));

const certsDir = path.resolve(import.meta.dir, "..", "certs");
const tlsKey = fs.readFileSync(path.join(certsDir, "key.pem"));
const tlsCert = fs.readFileSync(path.join(certsDir, "cert.pem"));

const state: ServiceState = {
  users: new Map<number, UserRecord>(),
  orders: new Map<number, OrderRecord>(),
  events: [],
  nextUserId: 1,
  nextOrderId: 1,
  nextEventId: 1,
};

seedInitialState();

function parsePorts(raw?: string): number[] {
  if (!raw) {
    return [];
  }

  return raw
    .split(",")
    .map((part) => Number.parseInt(part, 10))
    .filter((part) => Number.isFinite(part));
}

function nowIso(): string {
  return new Date().toISOString();
}

function withDefaultHeaders(
  headers?: Record<string, string>,
): Record<string, string> {
  return {
    "x-test-server": "certus-mini-service",
    ...headers,
  };
}

function noContentResponse(status = 204): ServiceResponse {
  return {
    status,
    headers: withDefaultHeaders({
      "content-length": "0",
    }),
  };
}

function textResponse(
  status: number,
  body: string,
  withLength = false,
  extraHeaders?: Record<string, string>,
): ServiceResponse {
  const headers: Record<string, string> = {
    "content-type": "text/plain; charset=utf-8",
    ...extraHeaders,
  };

  if (withLength) {
    headers["content-length"] = String(Buffer.byteLength(body));
  }

  return {
    status,
    headers: withDefaultHeaders(headers),
    body,
  };
}

function jsonResponse(
  status: number,
  data: unknown,
  withLength = true,
  extraHeaders?: Record<string, string>,
): ServiceResponse {
  const body = JSON.stringify(data);
  const headers: Record<string, string> = {
    "content-type": "application/json; charset=utf-8",
    ...extraHeaders,
  };

  if (withLength) {
    headers["content-length"] = String(Buffer.byteLength(body));
  }

  return {
    status,
    headers: withDefaultHeaders(headers),
    body,
  };
}

function redirectResponse(location: string): ServiceResponse {
  return {
    status: 302,
    headers: withDefaultHeaders({
      location,
      "content-length": "0",
    }),
  };
}

function methodNotAllowed(allow: string[]): ServiceResponse {
  return jsonResponse(
    405,
    { error: "method_not_allowed", allow },
    true,
    { allow: allow.join(", ") },
  );
}

function optionsResponse(allow: string[]): ServiceResponse {
  return {
    status: 204,
    headers: withDefaultHeaders({
      allow: allow.join(", "),
      "access-control-allow-origin": "*",
      "access-control-allow-methods": allow.join(", "),
      "access-control-allow-headers":
        "content-type, authorization, x-seed-key",
      "content-length": "0",
    }),
  };
}

function parseJsonBody(bodyText: string): Record<string, unknown> | null {
  if (bodyText.trim().length === 0) {
    return null;
  }

  try {
    const parsed = JSON.parse(bodyText) as unknown;
    if (parsed !== null && typeof parsed === "object") {
      return parsed as Record<string, unknown>;
    }
    return null;
  } catch {
    return null;
  }
}

function parseDelay(value: string | null): number | undefined {
  if (!value) {
    return undefined;
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return undefined;
  }

  return Math.min(parsed, 5000);
}

function parseForcedStatus(value: string | null): number | null {
  if (!value) {
    return null;
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    return null;
  }

  const allowed = new Set([400, 401, 403, 404, 409, 422, 429, 500, 502, 503]);
  return allowed.has(parsed) ? parsed : null;
}

function pushEvent(type: string, details: Record<string, unknown>): void {
  const entry: EventRecord = {
    id: state.nextEventId++,
    type,
    timestamp: nowIso(),
    details,
  };

  state.events.push(entry);

  if (state.events.length > 200) {
    state.events.shift();
  }
}

function seedInitialState(): void {
  state.users.clear();
  state.orders.clear();
  state.events.length = 0;
  state.nextUserId = 1;
  state.nextOrderId = 1;
  state.nextEventId = 1;

  const baseUsers: Omit<UserRecord, "id">[] = [
    {
      name: "Ada Lovelace",
      email: "ada@example.test",
      role: "admin",
      active: true,
      tags: ["founder", "math"],
      created_at: nowIso(),
    },
    {
      name: "Linus Torvalds",
      email: "linus@example.test",
      role: "maintainer",
      active: true,
      tags: ["kernel", "oss"],
      created_at: nowIso(),
    },
    {
      name: "Grace Hopper",
      email: "grace@example.test",
      role: "analyst",
      active: false,
      tags: ["compiler", "navy"],
      created_at: nowIso(),
    },
  ];

  for (const user of baseUsers) {
    const id = state.nextUserId++;
    state.users.set(id, { id, ...user });
  }

  const baseOrders: Omit<OrderRecord, "id">[] = [
    {
      user_id: 1,
      total: 149.99,
      currency: "USD",
      items: ["keyboard", "mouse"],
      created_at: nowIso(),
    },
    {
      user_id: 2,
      total: 45.0,
      currency: "USD",
      items: ["book"],
      created_at: nowIso(),
    },
  ];

  for (const order of baseOrders) {
    const id = state.nextOrderId++;
    state.orders.set(id, { id, ...order });
  }

  pushEvent("bootstrap", {
    users: state.users.size,
    orders: state.orders.size,
  });
}

function normalizeNodeHeaders(
  headers: IncomingHttpHeaders,
): Record<string, string> {
  const normalized: Record<string, string> = {};

  for (const [key, value] of Object.entries(headers)) {
    if (key.startsWith(":")) {
      continue;
    }

    if (typeof value === "string") {
      normalized[key.toLowerCase()] = value;
      continue;
    }

    if (Array.isArray(value)) {
      normalized[key.toLowerCase()] = value.join(",");
    }
  }

  return normalized;
}

async function readNodeBody(stream: NodeJS.ReadableStream): Promise<string> {
  const chunks: Buffer[] = [];

  for await (const chunk of stream) {
    if (typeof chunk === "string") {
      chunks.push(Buffer.from(chunk));
    } else {
      chunks.push(Buffer.from(chunk));
    }
  }

  return Buffer.concat(chunks).toString("utf-8");
}

function applyHead(method: string, response: ServiceResponse): ServiceResponse {
  if (method !== "HEAD") {
    return response;
  }

  return {
    ...response,
    body: undefined,
    headers: withDefaultHeaders({
      ...(response.headers ?? {}),
      "content-length": "0",
    }),
  };
}

function handleUsersRoute(
  method: string,
  pathname: string,
  url: URL,
  bodyText: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (pathname === "/api/users" || pathname === "/api/users/") {
    if (method === "GET") {
      const includeInactive = url.searchParams.get("include_inactive") === "1";
      const users = Array.from(state.users.values()).filter(
        (user) => includeInactive || user.active,
      );

      return jsonResponse(200, {
        users,
        count: users.length,
        include_inactive: includeInactive,
        server: socketAddr,
      });
    }

    if (method === "POST") {
      const payload = parseJsonBody(bodyText);
      if (!payload) {
        return jsonResponse(400, {
          error: "invalid_json_payload",
          expected: {
            name: "string",
            email: "string",
            role: "string (optional)",
          },
        });
      }

      const name =
        typeof payload.name === "string" ? payload.name.trim() : "";
      const email =
        typeof payload.email === "string" ? payload.email.trim() : "";

      if (!name || !email || !email.includes("@")) {
        return jsonResponse(422, {
          error: "validation_failed",
          fields: {
            name: "required",
            email: "required and must contain @",
          },
        });
      }

      const role =
        typeof payload.role === "string" && payload.role.trim().length > 0
          ? payload.role.trim()
          : "viewer";
      const tags = Array.isArray(payload.tags)
        ? payload.tags.filter((tag): tag is string => typeof tag === "string")
        : [];

      const id = state.nextUserId++;
      const user: UserRecord = {
        id,
        name,
        email,
        role,
        active: true,
        tags,
        created_at: nowIso(),
      };

      state.users.set(id, user);
      pushEvent("user_created", { id, email, server: socketAddr });

      return jsonResponse(201, {
        message: "user_created",
        user,
      });
    }

    if (method === "DELETE") {
      const removed = state.users.size;
      state.users.clear();
      pushEvent("users_cleared", { removed, server: socketAddr });

      return jsonResponse(200, {
        message: "users_cleared",
        removed,
      });
    }

    return methodNotAllowed(allow);
  }

  if (pathname === "/api/users/redirect") {
    return redirectResponse("/api/users");
  }

  if (pathname === "/api/users/error") {
    return jsonResponse(500, {
      error: "simulated_upstream_error",
      server: socketAddr,
      recovery_hint: "Retry with smaller payload",
    });
  }

  if (pathname === "/api/users/text") {
    return textResponse(
      200,
      `users-service on ${socketAddr} has ${state.users.size} users`,
      true,
    );
  }

  if (pathname === "/api/users/blob") {
    const sizeRaw = url.searchParams.get("size");
    const sizeParsed = sizeRaw ? Number.parseInt(sizeRaw, 10) : 256;
    const size =
      Number.isFinite(sizeParsed) && sizeParsed > 0
        ? Math.min(sizeParsed, 16_384)
        : 256;
    const body = "x".repeat(size);

    return textResponse(200, body, true, {
      "x-content-size": String(size),
    });
  }

  const userIdMatch = pathname.match(/^\/api\/users\/(\d+)$/);
  const userIdRaw = userIdMatch?.[1];
  if (!userIdRaw) {
    return jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  const userId = Number.parseInt(userIdRaw, 10);
  const user = state.users.get(userId);

  if (method === "GET") {
    if (!user) {
      return jsonResponse(404, {
        error: "user_not_found",
        user_id: userId,
      });
    }

    return jsonResponse(200, {
      user,
      server: socketAddr,
      path: pathname,
    });
  }

  if (method === "DELETE") {
    if (!user) {
      return jsonResponse(404, {
        error: "user_not_found",
        user_id: userId,
      });
    }

    state.users.delete(userId);
    pushEvent("user_deleted", { id: userId, server: socketAddr });

    return noContentResponse();
  }

  if (method === "PUT" || method === "PATCH") {
    if (!user) {
      return jsonResponse(404, {
        error: "user_not_found",
        user_id: userId,
      });
    }

    const payload = parseJsonBody(bodyText);
    if (!payload) {
      return jsonResponse(400, {
        error: "invalid_json_payload",
        user_id: userId,
      });
    }

    const updated: UserRecord = {
      ...user,
      name:
        typeof payload.name === "string" && payload.name.trim().length > 0
          ? payload.name.trim()
          : user.name,
      email:
        typeof payload.email === "string" && payload.email.includes("@")
          ? payload.email
          : user.email,
      role:
        typeof payload.role === "string" && payload.role.trim().length > 0
          ? payload.role.trim()
          : user.role,
      active:
        typeof payload.active === "boolean" ? payload.active : user.active,
      tags: Array.isArray(payload.tags)
        ? payload.tags.filter((tag): tag is string => typeof tag === "string")
        : user.tags,
    };

    state.users.set(userId, updated);
    pushEvent("user_updated", { id: userId, method, server: socketAddr });

    return jsonResponse(200, {
      message: "user_updated",
      user: updated,
    });
  }

  return methodNotAllowed(allow);
}

function handleHttp2ItemsRoute(
  method: string,
  pathname: string,
  bodyText: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "POST", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (pathname !== "/api/http2/items") {
    return jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  if (method === "GET") {
    const orders = Array.from(state.orders.values());

    return jsonResponse(200, {
      items: orders,
      total: orders.length,
      source: {
        socket: socketAddr,
        transport: "http2",
      },
      cached: true,
      note: null,
    });
  }

  if (method === "POST") {
    const payload = parseJsonBody(bodyText);
    if (!payload) {
      return jsonResponse(400, {
        error: "invalid_json_payload",
      });
    }

    const userId =
      typeof payload.user_id === "number" ? payload.user_id : Number.NaN;
    if (!Number.isFinite(userId) || !state.users.has(userId)) {
      return jsonResponse(404, {
        error: "user_not_found",
        user_id: payload.user_id ?? null,
      });
    }

    const items = Array.isArray(payload.items)
      ? payload.items.filter((item): item is string => typeof item === "string")
      : [];

    if (items.length === 0) {
      return jsonResponse(422, {
        error: "validation_failed",
        fields: {
          items: "must be a non-empty string array",
        },
      });
    }

    const total =
      typeof payload.total === "number" && payload.total > 0
        ? payload.total
        : Number((Math.random() * 90 + 10).toFixed(2));

    const id = state.nextOrderId++;
    const order: OrderRecord = {
      id,
      user_id: userId,
      total,
      currency:
        typeof payload.currency === "string" ? payload.currency : "USD",
      items,
      created_at: nowIso(),
    };

    state.orders.set(id, order);
    pushEvent("order_created", {
      id,
      user_id: userId,
      total,
      server: socketAddr,
    });

    return jsonResponse(201, {
      message: "order_created",
      order,
    });
  }

  return methodNotAllowed(allow);
}

function handlePublicTextRoute(
  method: string,
  pathname: string,
  bodyText: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "POST", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (pathname === "/api/public/text") {
    if (method !== "GET") {
      return methodNotAllowed(allow);
    }

    return textResponse(
      200,
      `plain text response from ${socketAddr} at ${nowIso()}`,
      false,
      {
        "cache-control": "no-store",
      },
    );
  }

  if (pathname === "/api/public/text/length") {
    if (method !== "GET") {
      return methodNotAllowed(allow);
    }

    return textResponse(
      200,
      `with-content-length ${socketAddr}`,
      true,
      {
        "cache-control": "no-store",
      },
    );
  }

  if (pathname === "/api/public/text/chunked") {
    if (method !== "GET") {
      return methodNotAllowed(allow);
    }

    return textResponse(
      200,
      `chunk-like text (${socketAddr})\nline-1\nline-2\nline-3`,
      false,
      {
        "cache-control": "no-store",
      },
    );
  }

  if (pathname === "/api/public/text/echo") {
    if (method !== "POST") {
      return methodNotAllowed(allow);
    }

    return textResponse(
      200,
      `echo:${bodyText.length}:${bodyText.slice(0, 120)}`,
      true,
      {
        "cache-control": "no-store",
      },
    );
  }

  return jsonResponse(404, {
    error: "route_not_found",
    route: pathname,
    method,
  });
}

function handlePrivateRoute(
  method: string,
  pathname: string,
  bodyText: string,
  headers: Record<string, string>,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "POST", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (pathname === "/api/private/profile") {
    if (method !== "GET") {
      return methodNotAllowed(allow);
    }

    return jsonResponse(200, {
      message: "private_profile",
      auth_forwarded: {
        user_id: headers["x-user-id"] ?? null,
        role: headers["x-user-role"] ?? null,
      },
      server: socketAddr,
    });
  }

  if (pathname === "/api/private/nocache") {
    if (method !== "POST") {
      return methodNotAllowed(allow);
    }

    const payload = parseJsonBody(bodyText);

    return jsonResponse(
      200,
      {
        message: "private_no_cache_ok",
        auth_forwarded: {
          user_id: headers["x-user-id"] ?? null,
          role: headers["x-user-role"] ?? null,
        },
        payload_present: payload !== null,
      },
      true,
      {
        "cache-control": "no-store",
      },
    );
  }

  return jsonResponse(404, {
    error: "route_not_found",
    route: pathname,
    method,
  });
}

function handleLimitedRoute(
  method: string,
  pathname: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (!pathname.startsWith("/api/limited")) {
    return jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  if (method !== "GET") {
    return methodNotAllowed(allow);
  }

  return textResponse(
    200,
    `limited route served by ${socketAddr}`,
    true,
    {
      "cache-control": "no-store",
    },
  );
}

function handleOverloadRoute(
  method: string,
  pathname: string,
  url: URL,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (!pathname.startsWith("/api/overload")) {
    return jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  if (method !== "GET") {
    return methodNotAllowed(allow);
  }

  const delayMs = parseDelay(url.searchParams.get("delay")) ?? 800;

  return {
    ...jsonResponse(
      200,
      {
        message: "overload_test_response",
        delay_ms: delayMs,
        server: socketAddr,
      },
      true,
      {
        "cache-control": "no-store",
      },
    ),
    delayMs,
  };
}

function handleStaticRoute(
  method: string,
  pathname: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (
    pathname !== "/api/static" &&
    pathname !== "/api/static/" &&
    pathname !== "/api/static/content"
  ) {
    return jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  if (method !== "GET") {
    return methodNotAllowed(allow);
  }

  return textResponse(
    200,
    `static content from ${socketAddr}\nversion=v1\ncacheable=true`,
    true,
    {
      "cache-control": "public, max-age=120",
      etag: '"static-content-v1"',
    },
  );
}

function handleHttpsRoute(
  method: string,
  pathname: string,
  socketAddr: string,
): ServiceResponse {
  const allow = ["GET", "HEAD", "OPTIONS"];

  if (method === "OPTIONS") {
    return optionsResponse(allow);
  }

  if (method !== "GET") {
    return methodNotAllowed(allow);
  }

  if (pathname === "/api/https1/data") {
    return jsonResponse(200, {
      source: "https1",
      encrypted: true,
      socket: socketAddr,
      payload: {
        ok: true,
        latency_hint_ms: 22,
      },
    });
  }

  if (pathname === "/api/https2/data") {
    return jsonResponse(200, {
      source: "https2",
      encrypted: true,
      socket: socketAddr,
      payload: {
        ok: true,
        protocol: "h2",
      },
    });
  }

  return jsonResponse(404, {
    error: "route_not_found",
    route: pathname,
    method,
  });
}

function routeRequest(request: ServiceRequest): ServiceResponse {
  const methodOriginal = request.method.toUpperCase();
  const method = methodOriginal === "HEAD" ? "GET" : methodOriginal;

  const url = new URL(
    request.rawPath,
    `${request.protocol}://certus.test.local`,
  );
  const pathname = url.pathname;

  const forcedStatus = parseForcedStatus(url.searchParams.get("error"));
  const forcedDelay = parseDelay(url.searchParams.get("delay"));

  if (forcedStatus !== null) {
    const forced = jsonResponse(forcedStatus, {
      error: `forced_error_${forcedStatus}`,
      path: pathname,
      method,
      server: request.socketAddr,
    });

    forced.delayMs = forcedDelay;
    return applyHead(methodOriginal, forced);
  }

  if (pathname === "/") {
    const root = textResponse(
      200,
      `ok ${request.socketAddr} ${nowIso()}`,
      true,
      {
        "cache-control": "no-store",
      },
    );
    root.delayMs = forcedDelay;
    return applyHead(methodOriginal, root);
  }

  if (pathname === "/_seed/events") {
    if (method === "OPTIONS") {
      return optionsResponse(["GET", "HEAD", "OPTIONS"]);
    }

    if (method !== "GET") {
      return methodNotAllowed(["GET", "HEAD", "OPTIONS"]);
    }

    const events = jsonResponse(200, {
      total: state.events.length,
      events: state.events,
    });
    events.delayMs = forcedDelay;
    return applyHead(methodOriginal, events);
  }

  if (pathname === "/_seed/reset") {
    if (method === "OPTIONS") {
      return optionsResponse(["POST", "OPTIONS"]);
    }

    if (method !== "POST") {
      return methodNotAllowed(["POST", "OPTIONS"]);
    }

    seedInitialState();
    const reset = jsonResponse(200, {
      message: "state_reset",
      users: state.users.size,
      orders: state.orders.size,
    });
    reset.delayMs = forcedDelay;
    return applyHead(methodOriginal, reset);
  }

  let response: ServiceResponse;

  if (pathname.startsWith("/api/users")) {
    response = handleUsersRoute(
      method,
      pathname,
      url,
      request.bodyText,
      request.socketAddr,
    );
  } else if (pathname.startsWith("/api/http2/items")) {
    response = handleHttp2ItemsRoute(
      method,
      pathname,
      request.bodyText,
      request.socketAddr,
    );
  } else if (pathname.startsWith("/api/public/text")) {
    response = handlePublicTextRoute(
      method,
      pathname,
      request.bodyText,
      request.socketAddr,
    );
  } else if (pathname.startsWith("/api/private")) {
    response = handlePrivateRoute(
      method,
      pathname,
      request.bodyText,
      request.headers,
      request.socketAddr,
    );
  } else if (pathname.startsWith("/api/limited")) {
    response = handleLimitedRoute(method, pathname, request.socketAddr);
  } else if (pathname.startsWith("/api/overload")) {
    response = handleOverloadRoute(method, pathname, url, request.socketAddr);
  } else if (pathname.startsWith("/api/static")) {
    response = handleStaticRoute(method, pathname, request.socketAddr);
  } else if (pathname.startsWith("/api/https1") || pathname.startsWith("/api/https2")) {
    response = handleHttpsRoute(method, pathname, request.socketAddr);
  } else {
    response = jsonResponse(404, {
      error: "route_not_found",
      route: pathname,
      method,
    });
  }

  if (!response.delayMs && forcedDelay) {
    response.delayMs = forcedDelay;
  }

  return applyHead(methodOriginal, response);
}

async function announceIdle(serverAddr: string): Promise<void> {
  try {
    await fetch(GATEWAY_IDLE_URL, {
      method: "POST",
      body: serverAddr,
    });

    if (values.log) {
      console.log(`[jiq] ${serverAddr} announced idle`);
    }
  } catch (err) {
    if (values.log) {
      console.log(`[jiq] ${serverAddr} failed to announce idle: ${err}`);
    }
  }
}

function toBunResponse(serviceResponse: ServiceResponse): Response {
  return new Response(serviceResponse.body ?? null, {
    status: serviceResponse.status,
    headers: serviceResponse.headers,
  });
}

async function respondHttp2(
  stream: ServerHttp2Stream,
  serviceResponse: ServiceResponse,
): Promise<void> {
  if (serviceResponse.delayMs) {
    await Bun.sleep(serviceResponse.delayMs);
  }

  stream.respond({
    ":status": serviceResponse.status,
    ...(serviceResponse.headers ?? {}),
  });
  stream.end(serviceResponse.body ?? "");
}

function createHttp1Server(port: number): void {
  const socketAddr = `${hostname}:${port}`;
  const isJiq = jiqPorts.has(port);
  console.log(`[http1${isJiq ? "+jiq" : ""}] Running server on ${socketAddr}`);

  if (isJiq) {
    setTimeout(() => {
      void announceIdle(socketAddr);
    }, 1000);
  }

  Bun.serve({
    port,
    hostname,
    async fetch(req) {
      const headers = Object.fromEntries(req.headers.entries());

      if (values.log) {
        console.log(`[INFO] bun::server::http1 headers=${JSON.stringify(headers)}`);
      }

      const bodyText = await req.text();
      const request: ServiceRequest = {
        method: req.method,
        rawPath: req.url,
        headers,
        bodyText,
        socketAddr,
        protocol: "http",
      };

      if (isJiq) {
        await Bun.sleep(JIQ_WORK_MS);
      }

      const response = routeRequest(request);

      if (response.delayMs) {
        await Bun.sleep(response.delayMs);
      }

      if (isJiq) {
        void announceIdle(socketAddr);
      }

      return toBunResponse(response);
    },
  });
}

function createHttp2Server(port: number): void {
  const socketAddr = `${hostname}:${port}`;
  const server = http2.createServer();

  server.on(
    "stream",
    (stream: ServerHttp2Stream, headers: IncomingHttpHeaders) => {
      void (async () => {
        if (values.log) {
          console.log(`[INFO] bun::server::http2 headers=${JSON.stringify(headers)}`);
        }

        try {
          const bodyText = await readNodeBody(stream);
          const pathHeader = headers[":path"];
          const methodHeader = headers[":method"];

          const request: ServiceRequest = {
            method:
              typeof methodHeader === "string"
                ? methodHeader
                : Array.isArray(methodHeader)
                  ? methodHeader[0] ?? "GET"
                  : "GET",
            rawPath:
              typeof pathHeader === "string"
                ? pathHeader
                : Array.isArray(pathHeader)
                  ? pathHeader[0] ?? "/"
                  : "/",
            headers: normalizeNodeHeaders(headers),
            bodyText,
            socketAddr,
            protocol: "http",
          };

          const response = routeRequest(request);
          await respondHttp2(stream, response);
        } catch (error) {
          const response = jsonResponse(500, {
            error: "http2_handler_failure",
            details: String(error),
          });
          await respondHttp2(stream, response);
        }
      })();
    },
  );

  server.listen(port, hostname, () => {
    console.log(`[http2](Cleartext) Running server on ${socketAddr}`);
  });
}

function createHttps1Server(port: number): void {
  const socketAddr = `${hostname}:${port}`;

  const server = https.createServer(
    { key: tlsKey, cert: tlsCert },
    (req, res) => {
      void (async () => {
        if (values.log) {
          console.log(`[INFO] bun::server::https1 headers=${JSON.stringify(req.headers)}`);
        }

        try {
          const bodyText = await readNodeBody(req);

          const request: ServiceRequest = {
            method: req.method ?? "GET",
            rawPath: req.url ?? "/",
            headers: normalizeNodeHeaders(req.headers),
            bodyText,
            socketAddr,
            protocol: "https",
          };

          const response = routeRequest(request);
          if (response.delayMs) {
            await Bun.sleep(response.delayMs);
          }

          res.writeHead(response.status, response.headers ?? {});
          res.end(response.body ?? "");
        } catch (error) {
          const response = jsonResponse(500, {
            error: "https1_handler_failure",
            details: String(error),
          });
          res.writeHead(response.status, response.headers ?? {});
          res.end(response.body ?? "");
        }
      })();
    },
  );

  server.listen(port, hostname, () => {
    console.log(`[https1](TLS) Running server on ${socketAddr}`);
  });
}

function createHttps2Server(port: number): void {
  const socketAddr = `${hostname}:${port}`;
  const server = http2.createSecureServer({ key: tlsKey, cert: tlsCert });

  server.on(
    "stream",
    (stream: ServerHttp2Stream, headers: IncomingHttpHeaders) => {
      void (async () => {
        if (values.log) {
          console.log(`[INFO] bun::server::https2 headers=${JSON.stringify(headers)}`);
        }

        try {
          const bodyText = await readNodeBody(stream);
          const pathHeader = headers[":path"];
          const methodHeader = headers[":method"];

          const request: ServiceRequest = {
            method:
              typeof methodHeader === "string"
                ? methodHeader
                : Array.isArray(methodHeader)
                  ? methodHeader[0] ?? "GET"
                  : "GET",
            rawPath:
              typeof pathHeader === "string"
                ? pathHeader
                : Array.isArray(pathHeader)
                  ? pathHeader[0] ?? "/"
                  : "/",
            headers: normalizeNodeHeaders(headers),
            bodyText,
            socketAddr,
            protocol: "https",
          };

          const response = routeRequest(request);
          await respondHttp2(stream, response);
        } catch (error) {
          const response = jsonResponse(500, {
            error: "https2_handler_failure",
            details: String(error),
          });
          await respondHttp2(stream, response);
        }
      })();
    },
  );

  server.listen(port, hostname, () => {
    console.log(`[https2](TLS) Running server on ${socketAddr}`);
  });
}

if (http1Ports.length === 0) {
  console.error("No HTTP/1 test ports configured (--http1)");
  process.exit(1);
}

if (http2Ports.length === 0) {
  console.error("No HTTP/2 test ports configured (--http2)");
  process.exit(1);
}

for (const port of http1Ports) {
  createHttp1Server(port);
}

for (const port of http2Ports) {
  createHttp2Server(port);
}

for (const port of https1Ports) {
  createHttps1Server(port);
}

for (const port of https2Ports) {
  createHttps2Server(port);
}
