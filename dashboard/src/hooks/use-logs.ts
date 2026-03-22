import { useQuery } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { api } from "@/api";
import { useWS } from "./use-ws";
import { WEB_SOCKET_TYPE } from "@/lib/args";
import { normalizeWSLog } from "@/lib/log/utils";
import type { LogEntry } from "@/lib/log";
import type { LogQuery } from "@/lib/query";

interface UseLogsProps {
  query: LogQuery & { page: number; per_page: number };
  isLive: boolean;
  wsEnabled: boolean;
}

export function useLogs({ query, isLive, wsEnabled }: UseLogsProps) {
  const {
    data: rawLogs = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ["logs", query, isLive],
    queryFn: () =>
      api.logs.get({
        page: Math.max(0, query.page - 1),
        per_page: query.per_page,
        level: query.level === "All" ? undefined : query.level,
        search: query.search || undefined,
        target: query.target || undefined,
        from: query.from || undefined,
        to: query.to || undefined,
      }),
    enabled: !isLive,
    placeholderData: (previousData) => previousData,
  });

  const wsLogs = useWS<Omit<LogEntry, "id">>(
    WEB_SOCKET_TYPE.Logs.toLowerCase(),
    wsEnabled && isLive,
    100
  );

  const logs = useMemo(() => {
    if (!isLive) return rawLogs;

    const normalizedWs = wsLogs
      .map((log, i) => normalizeWSLog(log, i))
      .reverse();

    let combined = [...normalizedWs, ...rawLogs];

    // Client-side filtering for live stream
    if (query.level && query.level !== "All") {
      combined = combined.filter((l) => l.level === query.level);
    }
    if (query.search) {
      const term = query.search.toLowerCase();
      combined = combined.filter(
        (l) =>
          l.message.toLowerCase().includes(term) ||
          l.target.toLowerCase().includes(term)
      );
    }
    if (query.target) {
      const term = query.target.toLowerCase();
      combined = combined.filter((l) => l.target.toLowerCase().includes(term));
    }

    return combined;
  }, [isLive, rawLogs, wsLogs, query.level, query.search, query.target]);

  return {
    logs,
    isLoading,
    error,
  };
}
