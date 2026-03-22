import { HttpVersion } from "@/lib/types"

/** Mirrors gateway/src/config/types.rs */

export interface ServerConfig {
  port: number
  origins: string[]
}

export type AuthType =
  | "none"
  | {
    jwt: {
      secret: string
      algorithm: string
    }
  }

export interface AuthConfig {
  method: AuthType
  prefix: string
}

export interface TLSConfig {
  cert_path: string
  key_path: string
}

export interface RouteConfig {
  endpoints: string[]
  is_static: boolean
  needs_auth: boolean
  http_version: HttpVersion
  max_connections: number
  token_weight: number
  no_cache: boolean
}

export type RateLimitKey =
  | "ip"
  | "token"
  | { header: string }

export type StorageType =
  | "in_memory"
  | { redis: { url: string } }

export interface RateLimitConfig {
  max_tokens: number
  refill_rate: number
  key: RateLimitKey
  type: StorageType
}

export interface ConnectionConfig {
  connect_timeout: number
}

export interface StaticCacheConfig {
  type: StorageType
}

export interface CacheConfig {
  size: number
  type: StorageType
  ttl?: number
  tti?: number
  max_size: number
  static: StaticCacheConfig
}

export interface Config {
  server: ServerConfig
  routes: Record<string, RouteConfig>
  tls?: TLSConfig | null
  auth: AuthConfig
  rate_limit: RateLimitConfig
  connection: ConnectionConfig
  cache: CacheConfig
  default_server: string
}
