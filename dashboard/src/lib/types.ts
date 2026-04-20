export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
export type HttpVersion = "HTTP1" | "HTTP2";

export const WEB_SOCKET_TYPE = {
  Logs: "Logs",
  Metrics: "Metrics",
} as const;

export type WebSocketType = (typeof WEB_SOCKET_TYPE)[keyof typeof WEB_SOCKET_TYPE];

export interface CmdArgs {
  ws: WebSocketType[];
}

export interface UpstreamInfo {
  address: string;
  healthy: boolean;
  active_connections: number;
  total_connections: number;
  idle_connections: number;
}

export interface RouteInfo {
  path: string;
  upstreams: UpstreamInfo[];
}

export interface UpstreamHealth {
  address: string;
  healthy: boolean;
}

export interface Pagination {
  page?: number;
  per_page?: number;
}

export interface BaseMetricQuery {
  from?: string;
  to?: string;
  route?: string;
}

export interface RequestMetricQuery extends BaseMetricQuery {
  status?: number;
  ip?: string;
  method?: string;
}

export interface AggregateRequestQuery {
  from?: string;
  to?: string;
  route?: string;
  status?: number;
  method?: string;
  interval?: string;
}

export interface AggregateCacheQuery {
  from?: string;
  to?: string;
  route?: string;
  interval?: string;
}

export interface SummaryQuery {
  from?: string;
  to?: string;
  group_by?: string;
}

export interface LogQuery {
  from?: string;
  to?: string;
  level?: string;
  target?: string;
  search?: string;
}

export const VALID_INTERVALS = ["1m", "5m", "15m", "30m", "1h", "6h", "1d"] as const;
export type Interval = (typeof VALID_INTERVALS)[number];

export const VALID_GROUP_BY = ["route", "method", "status", "upstream"] as const;
export type GroupBy = (typeof VALID_GROUP_BY)[number];
