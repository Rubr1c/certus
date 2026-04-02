"use client";

import { Database, Zap, Clock, Layers } from "lucide-react";
import { Config, type CacheConfig, type StorageType } from "@/lib/config";
import { SectionCard, ConfigSelect, ConfigInput } from "./shared";

interface CacheSectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

/** Matches `gateway::config::defaults::default_cache_size`. */
const DEFAULT_CACHE_CAPACITY = 1000;

const MIB = 1024 * 1024;

function isRedisStorage(t: StorageType): t is { redis: { url: string } } {
  return typeof t === "object" && t !== null && "redis" in t;
}

function storageSelectValue(type: StorageType): string {
  return type === "in_memory" ? "in_memory" : "redis";
}

function applyStorageSelection(prev: CacheConfig, selectValue: string): CacheConfig {
  const redisUrl = isRedisStorage(prev.type) ? prev.type.redis.url : "redis://127.0.0.1:6379";
  const nextType: StorageType =
    selectValue === "in_memory" ? "in_memory" : { redis: { url: redisUrl } };
  return {
    ...prev,
    type: nextType,
    static: { type: nextType },
  };
}

function bytesToMibField(bytes: number): number {
  if (bytes <= 0) return 0;
  return Math.round((bytes / MIB) * 100) / 100;
}

function mibFieldToBytes(mib: number): number {
  if (mib <= 0) return 0;
  return Math.round(mib * MIB);
}

export function CacheSection({ config, updateConfig }: CacheSectionProps) {
  const storageOptions = [
    { label: "In-Memory", value: "in_memory" },
    { label: "Redis", value: "redis" },
  ];

  const cache = config.cache;

  return (
    <div className="space-y-6">
      <SectionCard title="Caching Strategy" icon={Database}>
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
            <ConfigInput
              label="Default TTL (seconds)"
              description="Leave empty to omit TTL (Moka / Redis use gateway defaults for expiry)."
              type="text"
              value={cache.ttl !== undefined ? String(cache.ttl) : ""}
              placeholder="e.g. 300"
              onChange={v => {
                const s = String(v).trim();
                updateConfig(c => ({
                  ...c,
                  cache: {
                    ...c.cache,
                    ttl:
                      s === ""
                        ? undefined
                        : Math.max(0, Math.floor(Number(s))),
                  },
                }));
              }}
              icon={Clock}
            />
            <ConfigInput
              label="Dynamic cache entry capacity"
              description="Maximum entries in the response cache (Moka max_capacity)."
              type="number"
              value={cache.size ?? DEFAULT_CACHE_CAPACITY}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  cache: { ...c.cache, size: Math.max(1, Math.floor(Number(v))) },
                }))
              }
              icon={Layers}
            />
            <ConfigInput
              label="Max response body to cache (MiB)"
              description="Stored as bytes in the gateway; responses larger than this skip caching."
              type="number"
              value={bytesToMibField(cache.max_size)}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  cache: { ...c.cache, max_size: mibFieldToBytes(Number(v)) },
                }))
              }
              icon={Zap}
            />
            <ConfigSelect
              label="Storage engine"
              value={storageSelectValue(cache.type)}
              options={storageOptions}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  cache: applyStorageSelection(c.cache, v),
                }))
              }
            />
          </div>
        </div>
      </SectionCard>

      {isRedisStorage(cache.type) && (
        <SectionCard title="Redis configuration" icon={Database}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <ConfigInput
              label="Redis connection URL"
              placeholder="redis://127.0.0.1:6379"
              value={cache.type.redis.url}
              onChange={v => {
                const url = String(v);
                const nextType: StorageType = { redis: { url } };
                updateConfig(c => ({
                  ...c,
                  cache: { ...c.cache, type: nextType, static: { type: nextType } },
                }));
              }}
            />
          </div>
        </SectionCard>
      )}
    </div>
  );
}
