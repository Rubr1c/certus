"use client";

import { Shield, Lock, Key, Activity } from "lucide-react";
import {
  Config,
  type AuthType,
  type RateLimitConfig,
  type RateLimitKey,
  type StorageType,
} from "@/lib/config";
import { SectionCard, ConfigSelect, ConfigInput, ConfigSwitch } from "./shared";

interface SecuritySectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

function isJwtMethod(m: AuthType): m is { jwt: { secret: string; algorithm: string } } {
  return m !== "none";
}

function authMethodSelectValue(m: AuthType): string {
  return m === "none" ? "none" : "jwt";
}

function isRedisStorage(t: StorageType): t is { redis: { url: string } } {
  return typeof t === "object" && t !== null && "redis" in t;
}

function storageSelectValue(type: StorageType): string {
  return type === "in_memory" ? "in_memory" : "redis";
}

function applyRateLimitStorage(prev: RateLimitConfig, selectValue: string): RateLimitConfig {
  const redisUrl = isRedisStorage(prev.type) ? prev.type.redis.url : "redis://127.0.0.1:6379";
  const nextType: StorageType =
    selectValue === "in_memory" ? "in_memory" : { redis: { url: redisUrl } };
  return { ...prev, type: nextType };
}

function rateLimitKeySelectValue(k: RateLimitKey): string {
  if (k === "ip") return "ip";
  if (k === "token") return "token";
  return "header";
}

function rateLimitHeaderName(k: RateLimitKey): string {
  return typeof k === "object" && k !== null && "header" in k ? k.header : "";
}

export function SecuritySection({ config, updateConfig }: SecuritySectionProps) {
  const authMethodOptions = [
    { label: "None", value: "none" },
    { label: "JWT", value: "jwt" },
  ];

  const jwtAlgorithmOptions = [
    { label: "HS256", value: "HS256" },
    { label: "HS384", value: "HS384" },
    { label: "HS512", value: "HS512" },
    { label: "RS256", value: "RS256" },
    { label: "ES256", value: "ES256" },
  ];

  const rateLimitKeyOptions = [
    { label: "Client IP", value: "ip" },
    { label: "Bearer token", value: "token" },
    { label: "HTTP header", value: "header" },
  ];

  const rateLimitStorageOptions = [
    { label: "In-Memory", value: "in_memory" },
    { label: "Redis", value: "redis" },
  ];

  return (
    <div className="space-y-6">
      <SectionCard title="Authentication" icon={Lock}>
        <div className="space-y-4">
          <ConfigSelect
            label="Auth method"
            value={authMethodSelectValue(config.auth.method)}
            options={authMethodOptions}
            onChange={v =>
              updateConfig(c => ({
                ...c,
                auth: {
                  ...c.auth,
                  method:
                    v === "jwt"
                      ? { jwt: { secret: "", algorithm: "HS256" } }
                      : "none",
                },
              }))
            }
          />
          <ConfigInput
            label="Authorization prefix"
            value={config.auth.prefix}
            onChange={v =>
              updateConfig(c => ({
                ...c,
                auth: { ...c.auth, prefix: String(v) },
              }))
            }
            icon={Key}
          />
          {isJwtMethod(config.auth.method) && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-4 border-t border-slate-50">
              <ConfigInput
                label="JWT secret"
                type="password"
                value={config.auth.method.jwt.secret}
                onChange={v =>
                  updateConfig(c => {
                    const m = c.auth.method;
                    if (m === "none") return c;
                    return {
                      ...c,
                      auth: {
                        ...c.auth,
                        method: {
                          jwt: { ...m.jwt, secret: String(v) },
                        },
                      },
                    };
                  })}
                icon={Key}
              />
              <ConfigSelect
                label="JWT algorithm"
                value={config.auth.method.jwt.algorithm}
                options={jwtAlgorithmOptions}
                onChange={v =>
                  updateConfig(c => {
                    const m = c.auth.method;
                    if (m === "none") return c;
                    return {
                      ...c,
                      auth: {
                        ...c.auth,
                        method: {
                          jwt: { ...m.jwt, algorithm: String(v) },
                        },
                      },
                    };
                  })}
              />
            </div>
          )}
        </div>
      </SectionCard>

      <SectionCard title="Rate Limiting" icon={Activity}>
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <ConfigInput
              label="Max tokens (bucket capacity)"
              type="number"
              value={config.rate_limit.max_tokens ?? 100}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  rate_limit: { ...c.rate_limit, max_tokens: Number(v) },
                }))
              }
            />
            <ConfigInput
              label="Refill rate (tokens per second)"
              type="number"
              value={config.rate_limit.refill_rate ?? 1}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  rate_limit: { ...c.rate_limit, refill_rate: Number(v) },
                }))
              }
            />
            <ConfigSelect
              label="Rate limit key"
              value={rateLimitKeySelectValue(config.rate_limit.key)}
              options={rateLimitKeyOptions}
              onChange={v =>
                updateConfig(c => {
                  let key: RateLimitKey;
                  if (v === "ip") key = "ip";
                  else if (v === "token") key = "token";
                  else key = { header: rateLimitHeaderName(c.rate_limit.key) || "X-Api-Key" };
                  return { ...c, rate_limit: { ...c.rate_limit, key } };
                })
              }
            />
            <ConfigSelect
              label="Rate limit storage"
              value={storageSelectValue(config.rate_limit.type)}
              options={rateLimitStorageOptions}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  rate_limit: applyRateLimitStorage(c.rate_limit, v),
                }))
              }
            />
          </div>
          {rateLimitKeySelectValue(config.rate_limit.key) === "header" && (
            <ConfigInput
              label="Header name"
              value={rateLimitHeaderName(config.rate_limit.key)}
              onChange={v =>
                updateConfig(c => ({
                  ...c,
                  rate_limit: { ...c.rate_limit, key: { header: String(v) } },
                }))
              }
            />
          )}
          {isRedisStorage(config.rate_limit.type) && (
            <div className="pt-2 border-t border-slate-100">
              <p className="text-sm font-medium text-text-muted mb-4">Rate limit Redis</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <ConfigInput
                  label="Redis URL"
                  placeholder="redis://127.0.0.1:6379"
                  value={config.rate_limit.type.redis.url}
                  onChange={v => {
                    const url = String(v);
                    const nextType: StorageType = { redis: { url } };
                    updateConfig(c => ({
                      ...c,
                      rate_limit: { ...c.rate_limit, type: nextType },
                    }));
                  }}
                />
              </div>
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
            onChange={checked =>
              updateConfig(c => ({
                ...c,
                tls: checked
                  ? c.tls ?? {
                      cert_path: "certs/cert.pem",
                      key_path: "certs/key.pem",
                    }
                  : undefined,
              }))
            }
          />

          {config.tls && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6 pt-2">
              <ConfigInput
                label="Certificate Path"
                value={config.tls.cert_path}
                onChange={v =>
                  updateConfig(c => ({
                    ...c,
                    tls: { ...c.tls!, cert_path: v as string },
                  }))
                }
              />
              <ConfigInput
                label="Private Key Path"
                value={config.tls.key_path}
                onChange={v =>
                  updateConfig(c => ({
                    ...c,
                    tls: { ...c.tls!, key_path: v as string },
                  }))
                }
              />
            </div>
          )}
        </div>
      </SectionCard>
    </div>
  );
}
