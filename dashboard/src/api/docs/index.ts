import type {
  DocsGenerateErrorBody,
  EndpointDoc,
  GeneratedApiDocs,
} from "@/lib/docs/types";

const BASE = "/_certus/api/v1";

function toStringArray(value: unknown): string[] {
  if (Array.isArray(value)) {
    return value.filter((item): item is string => typeof item === "string");
  }
  if (typeof value === "string" && value.trim().length > 0) {
    return [value];
  }
  return [];
}

function normalizeEndpoint(input: unknown): EndpointDoc {
  const endpoint = (input ?? {}) as Record<string, unknown>;

  return {
    path: typeof endpoint.path === "string" ? endpoint.path : "",
    method: typeof endpoint.method === "string" ? endpoint.method : "GET",
    status_code:
      typeof endpoint.status_code === "number" ? endpoint.status_code : 0,
    title: typeof endpoint.title === "string" ? endpoint.title : "Endpoint",
    summary: typeof endpoint.summary === "string" ? endpoint.summary : "",
    observed_request: toStringArray(endpoint.observed_request),
    observed_response: toStringArray(endpoint.observed_response),
    markdown: typeof endpoint.markdown === "string" ? endpoint.markdown : "",
    limitations: toStringArray(endpoint.limitations),
  };
}

function normalizeDocs(input: unknown): GeneratedApiDocs {
  const docs = (input ?? {}) as Record<string, unknown>;

  return {
    title: typeof docs.title === "string" ? docs.title : "API documentation",
    introduction:
      typeof docs.introduction === "string" ? docs.introduction : "",
    highlights: toStringArray(docs.highlights),
    endpoints: Array.isArray(docs.endpoints)
      ? docs.endpoints.map(normalizeEndpoint)
      : [],
  };
}

async function generate(): Promise<GeneratedApiDocs> {
  const res = await fetch(`${BASE}/docs/generate`, {
    method: "POST",
    headers: { Accept: "application/json" },
  });

  const text = await res.text();

  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    try {
      const parsed = JSON.parse(text) as DocsGenerateErrorBody;
      if (typeof parsed.error === "string" && parsed.error.length > 0) {
        message = parsed.error;
      }
    } catch {
      /* use status */
    }
    throw new Error(message);
  }

  if (!text) {
    throw new Error("Empty response from documentation service");
  }

  return normalizeDocs(JSON.parse(text));
}

async function latest(): Promise<GeneratedApiDocs | null> {
  const res = await fetch(`${BASE}/docs`, {
    method: "GET",
    headers: { Accept: "application/json" },
  });

  if (res.status === 404) {
    return null;
  }

  const text = await res.text();

  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    try {
      const parsed = JSON.parse(text) as DocsGenerateErrorBody;
      if (typeof parsed.error === "string" && parsed.error.length > 0) {
        message = parsed.error;
      }
    } catch {
      /* use status */
    }
    throw new Error(message);
  }

  if (!text) {
    return null;
  }

  return normalizeDocs(JSON.parse(text));
}

export const docsApi = {
  generate,
  latest,
};
