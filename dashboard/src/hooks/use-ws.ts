import { useEffect, useRef, useState, useCallback, startTransition } from "react";

function wsBaseUrl(): string {
  if (typeof window === "undefined") {
    return "ws://127.0.0.1:8080/_certus/api/v1/ws";
  }
  const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
  return `${proto}//${window.location.host}/_certus/api/v1/ws`;
}

export type WSStatus = "connecting" | "open" | "error" | "closed";

export function useWS<T>(url: string, enabled?: boolean, limit = 100) {
  const [data, setData] = useState<T[]>([]);
  const [status, setStatus] = useState<WSStatus>("closed");
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);
  const connectRef = useRef<(() => void) | undefined>(undefined);

  const connect = useCallback(() => {
    if (!enabled || !mountedRef.current) return;

    setStatus("connecting");
    const sock = new WebSocket(`${wsBaseUrl()}/${url}`);
    socketRef.current = sock;

    sock.onopen = () => {
      if (mountedRef.current) setStatus("open");
    };

    sock.onmessage = (e) => {
      if (!mountedRef.current) return;
      try {
        const parsed = JSON.parse(e.data) as T;
        setData((p) => {
          const next = [...p, parsed];
          if (next.length > limit) return next.slice(next.length - limit);
          return next;
        });
      } catch {
      }
    };

    sock.onerror = () => {
      if (mountedRef.current) setStatus("error");
    };

    sock.onclose = () => {
      if (mountedRef.current) {
        setStatus("closed");
        reconnectTimeoutRef.current = setTimeout(() => {
          connectRef.current?.();
        }, 3000);
      }
    };
  }, [url, enabled, limit]);

  useEffect(() => {
    connectRef.current = connect;
  }, [connect]);

  useEffect(() => {
    mountedRef.current = true;
    if (enabled) {
      startTransition(() => setData([]));
      // WebSocket `connect` updates `status`/`data`; must run when `enabled` flips.
      // eslint-disable-next-line react-hooks/set-state-in-effect -- intentional subscription on toggle
      connect();
    }

    return () => {
      mountedRef.current = false;
      if (socketRef.current) {
        socketRef.current.onclose = null; 
        socketRef.current.close();
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
    };
  }, [enabled, connect]);

  return { data: enabled ? data : [], status };
}
