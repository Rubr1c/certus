import { useEffect, useRef, useState } from "react";

const WS_BASE = "ws://localhost:8080/_certus/api/v1/ws";

export function useWS<T>(url: string, enabled?: boolean, limit = 100) {
  const [data, setData] = useState<T[]>([]);

  useEffect(() => {
    if (!enabled) return;

    setData([]);
    let mounted = true;
    const sock = new WebSocket(`${WS_BASE}/${url}`);

    sock.onmessage = (e) => {
      if (!mounted) return;
      try {
        const parsed = JSON.parse(e.data) as T;
        setData((p) => {
          const next = [...p, parsed];
          if (next.length > limit) return next.slice(next.length - limit);
          return next;
        });
      } catch {
        // ignore malformed messages
      }
    };

    return () => {
      mounted = false;
      sock.close();
    };
  }, [url, enabled]);

  return enabled ? data : [];
}
