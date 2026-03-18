/** Mirrors gateway/src/db/models/request_metric.rs and cache_metric.rs */

export interface RequestMetricRow {
  timestamp: string
  route: string
  status_code: number
  duration_total_ms: number
  duration_upstream_ms: number
  bytes_in: number
  bytes_out: number
  client_ip: string
  method: string
  upstream_addr: string | null
  early_exit: string | null
}

export interface RequestMetricBucket {
  bucket: string
  count: number
  avg_duration_ms: number
  min_duration_ms: number
  max_duration_ms: number
  avg_upstream_ms: number
  bytes_in: number
  bytes_out: number
  status_2xx: number
  status_3xx: number
  status_4xx: number
  status_5xx: number
  error_count: number
}

export interface RequestMetricSummary {
  key: string
  count: number
  avg_duration_ms: number
  min_duration_ms: number
  max_duration_ms: number
  avg_upstream_ms: number
  bytes_in: number
  bytes_out: number
  error_count: number
  status_2xx: number
  status_3xx: number
  status_4xx: number
  status_5xx: number
}

export interface CacheMetricRow {
  timestamp: string
  route: string
  result: string
}

export interface CacheMetricBucket {
  bucket: string
  total: number
  hits: number
  misses: number
  bypasses: number
  hit_rate: number
}
