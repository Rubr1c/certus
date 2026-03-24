"use client";

import { Shield, Lock, Key, Activity } from "lucide-react";
import { Config, AuthType, RateLimitKey } from "@/lib/config";
import { SectionCard, ConfigSelect, ConfigInput, ConfigSwitch } from "./shared";

interface SecuritySectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

export function SecuritySection({ config, updateConfig }: SecuritySectionProps) {
  const authOptions = [
    { label: "None", value: "None" },
    { label: "Basic", value: "Basic" },
    { label: "Bearer", value: "Bearer" },
    { label: "ApiKey", value: "ApiKey" },
  ];

  const rateLimitKeyOptions = [
    { label: "IP Address", value: "IP" },
    { label: "Route Path", value: "Route" },
    { label: "API Key / Token", value: "Key" },
  ];

  return (
    <div className="space-y-6">
      <SectionCard title="Authentication" icon={Lock}>
        <div className="space-y-4">
          <ConfigSelect 
            label="Default Auth Type" 
            value={config.auth?.auth_type || "None"} 
            options={authOptions}
            onChange={v => updateConfig(c => ({ 
              ...c, 
              auth: { ...c.auth!, auth_type: v as AuthType } 
            }))}
          />
          {config.auth?.auth_type && config.auth.auth_type !== "None" && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-slate-50">
              <ConfigInput 
                label="Auth Token / Secret" 
                type="password"
                value={config.auth.token || ""} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  auth: { ...c.auth!, token: v as string } 
                }))}
                icon={Key} 
              />
            </div>
          )}
        </div>
      </SectionCard>

      <SectionCard title="Rate Limiting" icon={Activity}>
        <div className="space-y-6">
          <ConfigSwitch 
            label="Enable Global Rate Limiting" 
            description="Protect your backend from being overwhelmed by too many requests."
            checked={!!config.rate_limit} 
            onChange={checked => updateConfig(c => ({
              ...c,
              rate_limit: checked ? (c.rate_limit || {
                requests_per_second: 100,
                burst: 20,
                key: "IP"
              }) : undefined
            }))}
          />
          
          {config.rate_limit && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <ConfigInput 
                label="Requests Per Second" 
                type="number"
                value={config.rate_limit.requests_per_second ?? 100} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  rate_limit: { ...c.rate_limit!, requests_per_second: Number(v) } 
                }))}
              />
              <ConfigInput 
                label="Burst Limit" 
                type="number"
                value={config.rate_limit.burst ?? 20} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  rate_limit: { ...c.rate_limit!, burst: Number(v) } 
                }))}
              />
              <ConfigSelect 
                label="Identification Key" 
                value={config.rate_limit.key} 
                options={rateLimitKeyOptions}
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  rate_limit: { ...c.rate_limit!, key: v as RateLimitKey } 
                }))}
              />
            </div>
          )}
        </div>
      </SectionCard>

      <SectionCard title="TLS / SSL" icon={Shield}>
        <div className="space-y-4">
          <ConfigSwitch 
            label="Enable HTTPS" 
            description="Encrypt all traffic between the client and the gateway."
            checked={!!config.tls} 
            onChange={checked => updateConfig(c => ({
              ...c,
              tls: checked ? (c.tls || {
                cert_path: "certs/cert.pem",
                key_path: "certs/key.pem",
              }) : undefined
            }))}
          />
          
          {config.tls && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <ConfigInput 
                label="Certificate Path" 
                value={config.tls.cert_path} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  tls: { ...c.tls!, cert_path: v as string } 
                }))}
              />
              <ConfigInput 
                label="Private Key Path" 
                value={config.tls.key_path} 
                onChange={v => updateConfig(c => ({ 
                  ...c, 
                  tls: { ...c.tls!, key_path: v as string } 
                }))}
              />
            </div>
          )}
        </div>
      </SectionCard>
    </div>
  );
}
