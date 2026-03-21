import { useEffect, useRef, useState } from "react";

const WS_BASE = "ws://localhost:8080/_certus/api/v1/ws";

export function useWS<T>(url: string, enabled?: boolean) {
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
        setData((p) => [...p, parsed]);
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
