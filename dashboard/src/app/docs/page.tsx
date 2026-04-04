"use client";

import { api } from "@/api";
import type { EndpointDoc, GeneratedApiDocs } from "@/lib/docs/types";
import { BookOpen, Loader2, Sparkles } from "lucide-react";
import { useCallback, useState } from "react";
import ReactMarkdown from "react-markdown";
import { toast } from "sonner";
import type { Components } from "react-markdown";

const markdownComponents = {
  h2: ({ children }) => (
    <h2 className="mt-5 text-base font-semibold text-slate-900 first:mt-0">
      {children}
    </h2>
  ),
  h3: ({ children }) => (
    <h3 className="mt-3 text-sm font-semibold text-slate-800">{children}</h3>
  ),
  p: ({ children }) => (
    <p className="mt-2 text-sm leading-relaxed text-slate-700">{children}</p>
  ),
  ul: ({ children }) => (
    <ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-slate-700">
      {children}
    </ul>
  ),
  ol: ({ children }) => (
    <ol className="mt-2 list-decimal space-y-1 pl-5 text-sm text-slate-700">
      {children}
    </ol>
  ),
  li: ({ children }) => <li className="leading-relaxed">{children}</li>,
  code: ({ children, className }) => {
    if (!className) {
      return (
        <code className="rounded bg-slate-100 px-1.5 py-0.5 font-mono text-xs text-slate-800">
          {children}
        </code>
      );
    }
    return (
      <code className={`font-mono text-xs text-slate-100 ${className}`}>
        {children}
      </code>
    );
  },
  pre: ({ children }) => (
    <pre className="mt-2 overflow-x-auto rounded-lg bg-slate-900 p-3">
      {children}
    </pre>
  ),
  strong: ({ children }) => (
    <strong className="font-semibold text-slate-900">{children}</strong>
  ),
} satisfies Partial<Components>;

function MethodBadge({ method }: { method: string }) {
  const m = method.toUpperCase();
  const color =
    m === "GET"
      ? "bg-emerald-50 text-emerald-800 border-emerald-200"
      : m === "POST"
        ? "bg-ocean-50 text-ocean-800 border-ocean-200"
        : m === "PUT" || m === "PATCH"
          ? "bg-amber-50 text-amber-900 border-amber-200"
          : m === "DELETE"
            ? "bg-red-50 text-red-800 border-red-200"
            : "bg-slate-100 text-slate-700 border-slate-200";

  return (
    <span
      className={`inline-flex rounded-md border px-2 py-0.5 font-mono text-xs font-bold ${color}`}
    >
      {m}
    </span>
  );
}

function EndpointCard({ ep }: { ep: EndpointDoc }) {
  return (
    <article className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
      <div className="flex flex-wrap items-center gap-2 gap-y-2">
        <MethodBadge method={ep.method} />
        <span className="font-mono text-sm font-medium text-slate-900">
          {ep.path}
        </span>
        <span className="rounded-md bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-600">
          {ep.status_code}
        </span>
      </div>
      <h2 className="mt-4 text-lg font-semibold text-slate-900">{ep.title}</h2>
      <p className="mt-1 text-sm text-slate-600">{ep.summary}</p>
      <div className="mt-4 border-t border-slate-100 pt-4">
        <ReactMarkdown components={markdownComponents}>{ep.markdown}</ReactMarkdown>
      </div>
      {ep.limitations.trim().length > 0 ? (
        <p className="mt-4 rounded-lg border border-amber-100 bg-amber-50/80 px-3 py-2 text-xs leading-relaxed text-amber-900">
          <span className="font-semibold">Limitations: </span>
          {ep.limitations}
        </p>
      ) : null}
    </article>
  );
}

export default function DocsPage() {
  const [docs, setDocs] = useState<GeneratedApiDocs | null>(null);
  const [loading, setLoading] = useState(false);

  const handleGenerate = useCallback(async () => {
    setLoading(true);
    try {
      const result = await api.docs.generate();
      setDocs(result);
      toast.success("Documentation generated");
    } catch (e) {
      const message = e instanceof Error ? e.message : "Generation failed";
      toast.error(message);
      setDocs(null);
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">Documentation</h1>
          <p className="mt-1 text-sm text-slate-500">
            API documentation generation from observed gateway traffic.
          </p>
        </div>
        <button
          type="button"
          onClick={handleGenerate}
          disabled={loading}
          className="inline-flex shrink-0 items-center justify-center gap-2 px-6 py-2 text-sm font-bold text-white shadow-sm shadow-ocean-200 transition-colors cursor-pointer rounded-xl bg-ocean-500 hover:bg-ocean-600 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {loading ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" aria-hidden />
              Generating…
            </>
          ) : (
            <>
              <Sparkles className="h-4 w-4" aria-hidden />
              Generate
            </>
          )}
        </button>
      </div>

      {!docs ? (
        <div className="flex h-64 flex-col items-center justify-center rounded-xl border border-dashed border-slate-200 bg-white text-slate-400">
          <BookOpen className="mb-2 h-10 w-10 text-slate-300" aria-hidden />
          <p className="text-sm font-medium text-slate-500">
            Click Generate to build documentation from stored schemas.
          </p>
        </div>
      ) : (
        <div className="space-y-8">
          <section className="rounded-xl border border-slate-200 bg-white p-6 shadow-sm">
            <h2 className="text-xl font-bold text-slate-900">{docs.title}</h2>
            <div className="mt-3 text-sm leading-relaxed text-slate-700">
              <ReactMarkdown components={markdownComponents}>
                {docs.introduction}
              </ReactMarkdown>
            </div>
          </section>

          {docs.endpoints.length === 0 ? (
            <div className="rounded-xl border border-slate-200 bg-slate-50/80 p-6 text-center text-sm text-slate-600">
              No endpoints in this run — there may be no schema rows yet, or the
              model returned an empty list.
            </div>
          ) : (
            <div className="space-y-4">
              <h3 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
                Endpoints ({docs.endpoints.length})
              </h3>
              <div className="space-y-4">
                {docs.endpoints.map((ep, i) => (
                  <EndpointCard
                    key={`${ep.method}-${ep.path}-${ep.status_code}-${i}`}
                    ep={ep}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
