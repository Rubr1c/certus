"use client";

import { useQuery } from "@tanstack/react-query";
import { 
  Settings, 
  Server, 
  Shield, 
  Zap, 
  Database, 
  Network, 
  Lock, 
  Clock,
  Activity,
  Globe,
  Key,
  Plus,
  X,
  ChevronDown,
  Trash2
} from "lucide-react";
import { api } from "@/api";
import { useState, useEffect } from "react";
import { Config, AuthType, RateLimitKey, StorageType, RouteConfig } from "@/lib/config";

type Section = "general" | "routes" | "security" | "cache";

export default function ConfigPage() {
  const { data: initialConfig, isLoading, error } = useQuery({
    queryKey: ["config"],
    queryFn: () => api.config.get(),
  });

  const [config, setConfig] = useState<Config | null>(null);
  const [activeSection, setActiveSection] = useState<Section>("general");

  useEffect(() => {
    if (initialConfig && !config) {
      setConfig(initialConfig);
    }
  }, [initialConfig, config]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-ocean-500"></div>
      </div>
    );
  }

  if (error || !config) {
    return (
      <div className="p-6 bg-red-50 border border-red-200 rounded-xl text-red-600">
        <h3 className="font-semibold mb-1">Failed to load configuration</h3>
        <p className="text-sm">Please check if the gateway is online and try again.</p>
      </div>
    );
  }

  const navItems = [
    { id: "general" as Section, label: "General", icon: Server },
    { id: "routes" as Section, label: "Routes", icon: Network },
    { id: "security" as Section, label: "Security", icon: Shield },
    { id: "cache" as Section, label: "Cache", icon: Database },
  ];

  const updateConfig = (updater: (prev: Config) => Config) => {
    setConfig(prev => prev ? updater(prev) : null);
  };

  const handleAddRoute = () => {
    const newPath = "/new-route-" + Math.floor(Math.random() * 1000);
    const newRoute: RouteConfig = {
      endpoints: ["http://localhost:8080"],
      is_static: false,
      needs_auth: false,
      http_version: "HTTP1",
      max_connections: 100,
      token_weight: 0,
      no_cache: false,
    };
    updateConfig(c => ({
      ...c,
      routes: { ...c.routes, [newPath]: newRoute }
    }));
  };

  const handleRemoveRoute = (path: string) => {
    updateConfig(c => {
      const next = { ...c.routes };
      delete next[path];
      return { ...c, routes: next };
    });
  };

  const handleRenameRoute = (oldPath: string, newPath: string) => {
    if (!newPath || oldPath === newPath) return;
    updateConfig(c => {
      const next = { ...c.routes };
      const route = next[oldPath];
      delete next[oldPath];
      next[newPath] = route;
      return { ...c, routes: next };
    });
  };

  return (
    <div className="max-w-6xl mx-auto">
      <header className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-text-main flex items-center gap-2">
            <Settings className="w-6 h-6 text-ocean-500" />
            Gateway Configuration
          </h1>
          <p className="text-text-muted mt-1">
            Manage your API gateway settings and routing rules.
          </p>
        </div>
        <div className="flex gap-3">
          <button 
            className="px-4 py-2 text-sm font-semibold text-text-muted hover:text-text-main transition-colors"
            onClick={() => setConfig(initialConfig || null)}
          >
            Reset Changes
          </button>
          <button className="px-6 py-2 bg-ocean-500 text-white text-sm font-semibold rounded-lg hover:bg-ocean-600 transition-colors shadow-sm shadow-ocean-200">
            Save Config
          </button>
        </div>
      </header>

      <div className="flex flex-col md:flex-row gap-8 items-start">
        {/* Sidebar Nav */}
        <aside className="w-full md:w-64 shrink-0 bg-white rounded-xl border border-slate-200 p-2 overflow-hidden sticky top-24">
          <nav className="space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = activeSection === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm font-medium rounded-lg transition-colors ${
                    isActive 
                      ? "bg-ocean-50 text-ocean-600" 
                      : "text-text-muted hover:bg-slate-50 hover:text-text-main"
                  }`}
                >
                  <Icon className={`w-4 h-4 ${isActive ? "text-ocean-500" : "text-slate-400"}`} />
                  {item.label}
                </button>
              );
            })}
          </nav>
        </aside>

        {/* Content Area */}
        <main className="flex-1 min-w-0 w-full space-y-6">
          {activeSection === "general" && (
            <div className="space-y-6">
              <SectionCard title="Server Settings" icon={Server}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <ConfigInput 
                    label="Port" 
                    type="number" 
                    value={config.server.port} 
                    onChange={v => updateConfig(c => ({ ...c, server: { ...c.server, port: Number(v) } }))}
                    icon={Globe} 
                  />
                  <ConfigInput 
                    label="Default Upstream" 
                    value={config.default_server} 
                    onChange={v => updateConfig(c => ({ ...c, default_server: v as string }))}
                    icon={Network} 
                  />
                  <div className="md:col-span-2">
                    <ConfigList 
                      label="Allowed Origins" 
                      items={config.server.origins} 
                      onChange={items => updateConfig(c => ({ ...c, server: { ...c.server, origins: items } }))}
                      placeholder="e.g. http://localhost:3000"
                    />
                  </div>
                </div>
              </SectionCard>

              <SectionCard title="Connection Settings" icon={Zap}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <ConfigInput 
                    label="Connect Timeout (ms)" 
                    type="number"
                    value={config.connection.connect_timeout} 
                    onChange={v => updateConfig(c => ({ ...c, connection: { ...c.connection, connect_timeout: Number(v) } }))}
                    icon={Clock} 
                  />
                </div>
              </SectionCard>
            </div>
          )}

          {activeSection === "routes" && (
            <div className="space-y-6">
              <div className="flex justify-between items-center bg-white p-4 rounded-xl border border-slate-200 shadow-sm">
                <div>
                  <h3 className="font-semibold text-text-main">Route Management</h3>
                  <p className="text-sm text-text-muted">Total routes: {Object.keys(config.routes).length}</p>
                </div>
                <button
                  onClick={handleAddRoute}
                  className="flex items-center gap-2 px-4 py-2 bg-ocean-500 text-white text-sm font-semibold rounded-lg hover:bg-ocean-600 transition-colors shadow-sm shadow-ocean-200"
                >
                  <Plus size={18} />
                  Add New Route
                </button>
              </div>

              {Object.entries(config.routes).map(([path, route]) => (
                <SectionCard 
                  key={path} 
                  title={`Route: ${path}`} 
                  icon={Network}
                  action={
                    <button 
                      onClick={() => handleRemoveRoute(path)}
                      className="p-2 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all"
                      title="Delete Route"
                    >
                      <Trash2 size={18} />
                    </button>
                  }
                >
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-6">
                    <div className="md:col-span-2">
                      <ConfigInput 
                        label="Route Path" 
                        value={path} 
                        onChange={v => handleRenameRoute(path, v as string)}
                        icon={Globe}
                        placeholder="/api/v1/..."
                      />
                    </div>
                    <ConfigToggle 
                      label="Static Content" 
                      value={route.is_static} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, is_static: v } }
                      }))}
                    />
                    <ConfigToggle 
                      label="Needs Auth" 
                      value={route.needs_auth} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, needs_auth: v } }
                      }))}
                      icon={Lock} 
                    />
                    <ConfigSelect
                      label="HTTP Version"
                      value={route.http_version}
                      options={["HTTP1", "HTTP2"]}
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, http_version: v as any } }
                      }))}
                    />
                    <ConfigInput 
                      label="Max Connections" 
                      type="number"
                      value={route.max_connections} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, max_connections: Number(v) } }
                      }))}
                      icon={Activity} 
                    />
                    <ConfigInput 
                      label="Token Weight" 
                      type="number"
                      value={route.token_weight} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, token_weight: Number(v) } }
                      }))}
                    />
                    <ConfigToggle 
                      label="Enable Caching" 
                      value={!route.no_cache} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        routes: { ...c.routes, [path]: { ...route, no_cache: !v } }
                      }))}
                      icon={Database} 
                    />
                    <div className="md:col-span-2">
                      <ConfigList 
                        label="Upstream Endpoints" 
                        items={route.endpoints} 
                        onChange={items => updateConfig(c => ({
                          ...c,
                          routes: { ...c.routes, [path]: { ...route, endpoints: items } }
                        }))}
                        placeholder="e.g. http://10.0.0.1:8080"
                      />
                    </div>
                  </div>
                </SectionCard>
              ))}
            </div>
          )}

          {activeSection === "security" && (
            <div className="space-y-6">
              <SectionCard title="Authentication" icon={Lock}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <ConfigSelect
                    label="Method"
                    value={typeof config.auth.method === "string" ? "none" : "jwt"}
                    options={[{ label: "None", value: "none" }, { label: "JWT", value: "jwt" }]}
                    onChange={v => updateConfig(c => ({
                      ...c,
                      auth: { 
                        ...c.auth, 
                        method: v === "none" ? "none" : { jwt: { secret: "", algorithm: "HS256" } } 
                      }
                    }))}
                    icon={Key}
                  />
                  <ConfigInput 
                    label="Prefix" 
                    value={config.auth.prefix} 
                    onChange={v => updateConfig(c => ({ ...c, auth: { ...c.auth, prefix: v as string } }))}
                  />
                  {typeof config.auth.method !== "string" && (
                    <div className="md:col-span-2 p-4 bg-slate-50 rounded-xl border border-slate-100 space-y-4">
                      <ConfigSelect
                        label="JWT Algorithm"
                        value={config.auth.method.jwt.algorithm}
                        options={["HS256", "HS384", "HS512", "RS256"]}
                        onChange={v => updateConfig(c => {
                          const m = c.auth.method as { jwt: any };
                          return { ...c, auth: { ...c.auth, method: { jwt: { ...m.jwt, algorithm: v } } } };
                        })}
                      />
                      <ConfigInput 
                        label="JWT Secret" 
                        type="password"
                        value={config.auth.method.jwt.secret} 
                        onChange={v => updateConfig(c => {
                          const m = c.auth.method as { jwt: any };
                          return { ...c, auth: { ...c.auth, method: { jwt: { ...m.jwt, secret: v } } } };
                        })}
                      />
                    </div>
                  )}
                </div>
              </SectionCard>

              <SectionCard title="Rate Limiting" icon={Activity}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <ConfigInput 
                    label="Max Tokens" 
                    type="number"
                    value={config.rate_limit.max_tokens} 
                    onChange={v => updateConfig(c => ({ ...c, rate_limit: { ...c.rate_limit, max_tokens: Number(v) } }))}
                  />
                  <ConfigInput 
                    label="Refill Rate" 
                    type="number"
                    value={config.rate_limit.refill_rate} 
                    onChange={v => updateConfig(c => ({ ...c, rate_limit: { ...c.rate_limit, refill_rate: Number(v) } }))}
                    icon={Zap} 
                  />
                  <ConfigSelect
                    label="Key Strategy"
                    value={typeof config.rate_limit.key === "string" ? config.rate_limit.key : "header"}
                    options={[
                      { label: "IP Address", value: "ip" },
                      { label: "Auth Token", value: "token" },
                      { label: "Custom Header", value: "header" }
                    ]}
                    onChange={v => updateConfig(c => ({
                      ...c,
                      rate_limit: { 
                        ...c.rate_limit, 
                        key: v === "header" ? { header: "X-API-Key" } : v as any
                      }
                    }))}
                  />
                  {typeof config.rate_limit.key !== "string" && (
                    <ConfigInput 
                      label="Header Name" 
                      value={config.rate_limit.key.header} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        rate_limit: { ...c.rate_limit, key: { header: v as string } }
                      }))}
                    />
                  )}
                  <ConfigSelect
                    label="Storage"
                    value={typeof config.rate_limit.type === "string" ? "in_memory" : "redis"}
                    options={[
                      { label: "In-Memory", value: "in_memory" },
                      { label: "Redis", value: "redis" }
                    ]}
                    onChange={v => updateConfig(c => ({
                      ...c,
                      rate_limit: { 
                        ...c.rate_limit, 
                        type: v === "in_memory" ? "in_memory" : { redis: { url: "redis://localhost:6379" } }
                      }
                    }))}
                    icon={Database}
                  />
                  {typeof config.rate_limit.type !== "string" && (
                    <ConfigInput 
                      label="Redis URL" 
                      value={config.rate_limit.type.redis.url} 
                      onChange={v => updateConfig(c => ({
                        ...c,
                        rate_limit: { ...c.rate_limit, type: { redis: { url: v as string } } }
                      }))}
                    />
                  )}
                </div>
              </SectionCard>
            </div>
          )}

          {activeSection === "cache" && (
            <div className="space-y-6">
              <SectionCard title="Cache Policy" icon={Database}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <ConfigInput 
                    label="Max Cache Entries" 
                    type="number"
                    value={config.cache.size} 
                    onChange={v => updateConfig(c => ({ ...c, cache: { ...c.cache, size: Number(v) } }))}
                  />
                  <ConfigInput 
                    label="Max Entry Size (Bytes)" 
                    type="number"
                    value={config.cache.max_size} 
                    onChange={v => updateConfig(c => ({ ...c, cache: { ...c.cache, max_size: Number(v) } }))}
                  />
                  <ConfigInput 
                    label="TTL (Seconds)" 
                    type="number"
                    value={config.cache.ttl || 0} 
                    onChange={v => updateConfig(c => ({ ...c, cache: { ...c.cache, ttl: Number(v) } }))}
                    icon={Clock} 
                  />
                  <ConfigInput 
                    label="TTI (Idle Seconds)" 
                    type="number"
                    value={config.cache.tti || 0} 
                    onChange={v => updateConfig(c => ({ ...c, cache: { ...c.cache, tti: Number(v) } }))}
                    icon={Clock} 
                  />
                  <ConfigSelect
                    label="Storage"
                    value={typeof config.cache.type === "string" ? "in_memory" : "redis"}
                    options={[
                      { label: "In-Memory", value: "in_memory" },
                      { label: "Redis", value: "redis" }
                    ]}
                    onChange={v => updateConfig(c => ({
                      ...c,
                      cache: { 
                        ...c.cache, 
                        type: v === "in_memory" ? "in_memory" : { redis: { url: "redis://localhost:6379" } }
                      }
                    }))}
                  />
                </div>
              </SectionCard>

              <SectionCard title="Static Assets Cache" icon={Zap}>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                   <ConfigSelect
                    label="Storage"
                    value={typeof config.cache.static.type === "string" ? "in_memory" : "redis"}
                    options={[
                      { label: "In-Memory", value: "in_memory" },
                      { label: "Redis", value: "redis" }
                    ]}
                    onChange={v => updateConfig(c => ({
                      ...c,
                      cache: { 
                        ...c.cache, 
                        static: { type: v === "in_memory" ? "in_memory" : { redis: { url: "redis://localhost:6379" } } }
                      }
                    }))}
                  />
                </div>
              </SectionCard>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

function SectionCard({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) {
  return (
    <section className="bg-white rounded-2xl border border-slate-200 overflow-hidden shadow-sm hover:shadow-md transition-shadow duration-200">
      <div className="px-6 py-4 border-b border-slate-100 bg-slate-50/50 flex items-center gap-3">
        <div className="p-2 bg-white rounded-lg shadow-sm">
          <Icon className="w-4 h-4 text-ocean-500" />
        </div>
        <h2 className="font-semibold text-text-main">{title}</h2>
      </div>
      <div className="p-6">
        {children}
      </div>
    </section>
  );
}

function ConfigInput({ 
  label, 
  value, 
  type = "text",
  placeholder,
  icon: Icon,
  onChange
}: { 
  label: string; 
  value: string | number; 
  type?: "text" | "number" | "password";
  placeholder?: string;
  icon?: any;
  onChange: (v: string | number) => void;
}) {
  return (
    <div>
      <label className="block text-xs font-semibold uppercase tracking-wider text-slate-400 mb-1.5">
        {label}
      </label>
      <div className="relative group">
        {Icon && (
          <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-ocean-500 transition-colors">
            <Icon size={16} />
          </div>
        )}
        <input
          type={type}
          value={value}
          placeholder={placeholder}
          step={type === "number" ? "any" : undefined}
          onChange={e => onChange(e.target.value)}
          className={`w-full bg-slate-50 border border-slate-100 rounded-xl py-2.5 text-sm font-medium text-text-main focus:outline-none focus:ring-2 focus:ring-ocean-500/10 focus:border-ocean-500 focus:bg-white transition-all ${Icon ? 'pl-10' : 'pl-4'} pr-4`}
        />
      </div>
    </div>
  );
}

function ConfigSelect({ 
  label, 
  value, 
  options,
  icon: Icon,
  onChange
}: { 
  label: string; 
  value: string; 
  options: (string | { label: string, value: string })[];
  icon?: any;
  onChange: (v: string) => void;
}) {
  return (
    <div>
      <label className="block text-xs font-semibold uppercase tracking-wider text-slate-400 mb-1.5">
        {label}
      </label>
      <div className="relative group">
        {Icon && (
          <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-slate-400 group-focus-within:text-ocean-500 transition-colors pointer-events-none">
            <Icon size={16} />
          </div>
        )}
        <select
          value={value}
          onChange={e => onChange(e.target.value)}
          className={`w-full appearance-none bg-slate-50 border border-slate-100 rounded-xl py-2.5 text-sm font-medium text-text-main focus:outline-none focus:ring-2 focus:ring-ocean-500/10 focus:border-ocean-500 focus:bg-white transition-all ${Icon ? 'pl-10' : 'pl-4'} pr-10 cursor-pointer`}
        >
          {options.map(opt => (
            <option key={typeof opt === 'string' ? opt : opt.value} value={typeof opt === 'string' ? opt : opt.value}>
              {typeof opt === 'string' ? opt : opt.label}
            </option>
          ))}
        </select>
        <div className="absolute right-3.5 top-1/2 -translate-y-1/2 text-slate-400 pointer-events-none">
          <ChevronDown size={16} />
        </div>
      </div>
    </div>
  );
}

function ConfigToggle({ 
  label, 
  value, 
  icon: Icon,
  onChange
}: { 
  label: string; 
  value: boolean; 
  icon?: any;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between p-3 bg-slate-50 rounded-xl border border-slate-100">
      <div className="flex items-center gap-3">
        {Icon && <Icon className={`w-4 h-4 ${value ? 'text-ocean-500' : 'text-slate-400'}`} />}
        <span className="text-sm font-medium text-text-main">{label}</span>
      </div>
      <button
        onClick={() => onChange(!value)}
        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${
          value ? 'bg-ocean-500' : 'bg-slate-200'
        }`}
      >
        <span
          className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
            value ? 'translate-x-5' : 'translate-x-0'
          }`}
        />
      </button>
    </div>
  );
}

function ConfigList({ 
  label, 
  items, 
  placeholder,
  onChange
}: { 
  label: string; 
  items: string[]; 
  placeholder?: string;
  onChange: (items: string[]) => void;
}) {
  const [newVal, setNewVal] = useState("");

  const handleAdd = () => {
    if (newVal.trim()) {
      onChange([...items, newVal.trim()]);
      setNewVal("");
    }
  };

  const handleRemove = (index: number) => {
    onChange(items.filter((_, i) => i !== index));
  };

  return (
    <div>
      <label className="block text-xs font-semibold uppercase tracking-wider text-slate-400 mb-2">
        {label}
      </label>
      <div className="space-y-2">
        {items.map((item, idx) => (
          <div key={idx} className="flex items-center justify-between gap-3 px-4 py-2 bg-white rounded-xl border border-slate-100 shadow-sm group hover:border-ocean-200 transition-colors">
            <span className="text-sm font-mono text-slate-600 truncate">{item}</span>
            <button 
              onClick={() => handleRemove(idx)}
              className="p-1 text-slate-400 hover:text-red-500 hover:bg-red-50 rounded-lg transition-all"
            >
              <X size={14} />
            </button>
          </div>
        ))}
        <div className="flex gap-2">
          <input
            type="text"
            value={newVal}
            onChange={e => setNewVal(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleAdd()}
            placeholder={placeholder}
            className="flex-1 bg-slate-50 border border-slate-100 rounded-xl py-2 px-4 text-sm focus:outline-none focus:ring-2 focus:ring-ocean-500/10 focus:border-ocean-500 focus:bg-white transition-all"
          />
          <button
            onClick={handleAdd}
            disabled={!newVal.trim()}
            className="p-2.5 bg-ocean-500 text-white rounded-xl hover:bg-ocean-600 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm shadow-ocean-100 transition-all"
          >
            <Plus size={18} />
          </button>
        </div>
      </div>
    </div>
  );
}
