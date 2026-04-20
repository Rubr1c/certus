import { request } from "../client"
import type { RouteInfo, UpstreamHealth } from "@/lib/types"

export const routesApi = {
  get: () =>
    request<RouteInfo[]>({
      path: "routes",
    }),
  health: {
    all: () =>
      request<UpstreamHealth[]>({
        path: "routes/health",
      }),
    one: (addr: string) =>
      request<UpstreamHealth>({
        path: `routes/health/${addr}`,
      }),
  },
}
