import { useEffect, useRef, useState, useCallback } from "react";

const WS_BASE = "ws://localhost:8080/_certus/api/v1/ws";

export type WSStatus = "connecting" | "open" | "error" | "closed";

export function useWS<T>(url: string, enabled?: boolean, limit = 100) {
  const [data, setData] = useState<T[]>([]);
  const [status, setStatus] = useState<WSStatus>("closed");
  const socketRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const mountedRef = useRef(true);

  const connect = useCallback(() => {
    if (!enabled || !mountedRef.current) return;

    setStatus("connecting");
    const sock = new WebSocket(`${WS_BASE}/${url}`);
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
          connect();
        }, 3000);
      }
    };
  }, [url, enabled, limit]);

  useEffect(() => {
    mountedRef.current = true;
    if (enabled) {
      setData([]);
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
