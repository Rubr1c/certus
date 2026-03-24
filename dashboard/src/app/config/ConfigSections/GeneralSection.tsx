"use client";

import { Server, Globe, Network, Clock, Activity } from "lucide-react";
import { Config } from "@/lib/config";
import { SectionCard, ConfigInput } from "./shared";

interface GeneralSectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

export function GeneralSection({ config, updateConfig }: GeneralSectionProps) {
  return (
    <div className="space-y-6">
      <SectionCard title="Server Settings" icon={Server}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <ConfigInput 
            label="Port" 
            type="number" 
            value={config.server.port ?? 8080} 
            onChange={v => updateConfig(c => ({ ...c, server: { ...c.server, port: Number(v) } }))}
            icon={Globe} 
          />
          <ConfigInput 
            label="Default Upstream" 
            value={config.default_server ?? ""} 
            onChange={v => updateConfig(c => ({ ...c, default_server: v as string }))}
            icon={Network} 
          />
        </div>
      </SectionCard>

      <SectionCard title="Performance & Timeouts" icon={Clock}>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <ConfigInput 
            label="Connection Timeout (ms)" 
            type="number"
            value={config.server.timeout_ms || 30000} 
            onChange={v => updateConfig(c => ({ ...c, server: { ...c.server, timeout_ms: Number(v) } }))}
          />
          <ConfigInput 
            label="Max Concurrent Streams" 
            type="number"
            value={config.server.max_concurrent_streams || 100} 
            onChange={v => updateConfig(c => ({ ...c, server: { ...c.server, max_concurrent_streams: Number(v) } }))}
          />
        </div>
      </SectionCard>
    </div>
  );
}
