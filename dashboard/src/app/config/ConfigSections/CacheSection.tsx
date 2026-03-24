"use client";

import { Database, Zap, Clock, Shield } from "lucide-react";
import { Config, StorageType } from "@/lib/config";
import { SectionCard, ConfigSelect, ConfigInput, ConfigSwitch } from "./shared";

interface CacheSectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

export function CacheSection({ config, updateConfig }: CacheSectionProps) {
  const storageOptions = [
    { label: "In-Memory", value: "Memory" },
    { label: "Redis", value: "Redis" },
  ];

  return (
    <div className="space-y-6">
      <SectionCard title="Caching Strategy" icon={Database}>
        <div className="space-y-6">
          <ConfigSwitch 
            label="Enable Result Caching" 
            description="Improve performance by serving repeated requests from cache."
            checked={!!config.cache} 
            onChange={checked => updateConfig(c => ({
              ...c,
              cache: checked ? (c.cache || {
                ttl_seconds: 300,
                max_size_mb: 100,
                storage: "Memory"
              }) : undefined
            }))}
          />
          
          {config.cache && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <ConfigInput 
                label="Default TTL (seconds)" 
                type="number"
                value={config.cache.ttl_seconds ?? 300} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  cache: { ...c.cache!, ttl_seconds: Number(v) } 
                }))}
                icon={Clock}
              />
              <ConfigInput 
                label="Max Cache Size (MB)" 
                type="number"
                value={config.cache.max_size_mb ?? 100} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  cache: { ...c.cache!, max_size_mb: Number(v) } 
                }))}
                icon={Zap}
              />
              <ConfigSelect 
                label="Storage Engine" 
                value={config.cache.storage} 
                options={storageOptions}
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  cache: { ...c.cache!, storage: v as StorageType } 
                }))}
              />
            </div>
          )}
        </div>
      </SectionCard>

      {config.cache?.storage === "Redis" && (
        <SectionCard title="Redis Configuration" icon={Database}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <ConfigInput 
              label="Redis Connection URL" 
              placeholder="redis://127.0.0.1:6379"
              value={config.cache.redis_url || ""} 
              onChange={v => updateConfig(c => ({ 
                ...c, 
                cache: { ...c.cache!, redis_url: v as string } 
              }))}
            />
          </div>
        </SectionCard>
      )}
    </div>
  );
}
