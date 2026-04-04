import type {
  DocsGenerateErrorBody,
  GeneratedApiDocs,
} from "@/lib/docs/types";

const BASE = "/_certus/api/v1";

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

  return JSON.parse(text) as GeneratedApiDocs;
}

export const docsApi = {
  generate,
};
