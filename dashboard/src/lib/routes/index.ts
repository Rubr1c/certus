/** Mirrors gateway/src/controllers/route.rs */

export interface UpstreamInfo {
  address: string
  healthy: boolean
  active_connections: number
  total_connections: number
  idle_connections: number
}

export interface RouteInfo {
  path: string
  upstreams: UpstreamInfo[]
}

export interface UpstreamHealth {
  address: string
  healthy: boolean
}
