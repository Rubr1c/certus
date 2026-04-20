"use client";

import { Network, Plus, Trash2, ChevronDown, ChevronUp, Globe } from "lucide-react";
import { Config, RouteConfig } from "@/lib/config";
import { useState } from "react";
import { ConfigInput, ConfigSwitch, ConfigSelect } from "./shared";

interface RoutesSectionProps {
  config: Config;
  updateConfig: (updater: (prev: Config) => Config) => void;
}

export function RoutesSection({ config, updateConfig }: RoutesSectionProps) {
  const [expandedRoute, setExpandedRoute] = useState<string | null>(null);

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
    setExpandedRoute(newPath);
  };

  const handleRemoveRoute = (path: string) => {
    updateConfig(c => {
      const next = { ...c.routes };
      delete next[path];
      return { ...c, routes: next };
    });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-lg font-bold text-text-main flex items-center gap-2">
          <Network className="w-5 h-5 text-ocean-500" />
          Routing Table
        </h2>
        <button 
          onClick={handleAddRoute}
          className="flex items-center gap-2 px-4 py-2 bg-ocean-500 text-white text-xs font-bold rounded-lg hover:bg-ocean-600 transition-colors shadow-sm shadow-ocean-100 cursor-pointer"
        >
          <Plus className="w-3.5 h-3.5" />
          Add Route
        </button>
      </div>

      <div className="grid grid-cols-1 gap-4">
        {Object.entries(config.routes).map(([path, route]) => (
          <div key={path} className={`card-container overflow-hidden transition-all ${expandedRoute === path ? 'ring-2 ring-ocean-200' : ''}`}>
            <div 
              className="flex items-center justify-between px-6 py-4 cursor-pointer hover:bg-slate-50/50"
              onClick={() => setExpandedRoute(expandedRoute === path ? null : path)}
            >
              <div className="flex items-center gap-4 flex-1">
                <div className="rounded-lg bg-white p-2 shadow-sm border border-slate-100">
                  <Globe className="h-4 w-4 text-ocean-500" />
                </div>
                <div className="flex-1">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-sm font-bold text-slate-800">
                      {path}
                    </span>
                    <span className="text-[10px] uppercase tracking-wider text-slate-400 font-bold bg-slate-100 px-2 py-0.5 rounded">
                      {route.http_version}
                    </span>
                  </div>
                  <p className="text-[10px] text-slate-400 mt-1 font-medium">
                    {route.endpoints.length} Upstreams · {route.max_connections} Max Connections
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <button 
                  onClick={(e) => { e.stopPropagation(); handleRemoveRoute(path); }}
                  className="p-2 text-slate-400 hover:text-rose-500 hover:bg-rose-50 rounded-lg transition-colors cursor-pointer"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
                {expandedRoute === path ? <ChevronUp className="w-4 h-4 text-slate-400" /> : <ChevronDown className="w-4 h-4 text-slate-400" />}
              </div>
            </div>

            {expandedRoute === path && (
              <div className="px-6 py-6 border-t border-slate-100 bg-slate-50/30 space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-slate-400">Endpoints</h4>
                    <div className="space-y-2">
                      {route.endpoints.map((ep, idx) => (
                        <div key={idx} className="flex gap-2">
                          <input 
                            className="input-base text-xs font-mono py-1.5"
                            value={ep}
                            onChange={e => {
                              const next = [...route.endpoints];
                              next[idx] = e.target.value;
                              updateConfig(c => ({
                                ...c,
                                routes: { ...c.routes, [path]: { ...route, endpoints: next } }
                              }));
                            }}
                          />
                          {route.endpoints.length > 1 && (
                            <button 
                              onClick={() => {
                                const next = route.endpoints.filter((_, i) => i !== idx);
                                updateConfig(c => ({
                                  ...c,
                                  routes: { ...c.routes, [path]: { ...route, endpoints: next } }
                                }));
                              }}
                              className="p-1.5 text-slate-400 hover:text-rose-500 transition-colors cursor-pointer"
                            >
                              <Trash2 className="w-3.5 h-3.5" />
                            </button>
                          )}
                        </div>
                      ))}
                      <button 
                        onClick={() => {
                          updateConfig(c => ({
                            ...c,
                            routes: { ...c.routes, [path]: { ...route, endpoints: [...route.endpoints, ""] } }
                          }));
                        }}
                        className="text-[10px] font-bold text-ocean-600 hover:text-ocean-700 flex items-center gap-1 cursor-pointer"
                      >
                        <Plus className="w-3 h-3" /> Add Endpoint
                      </button>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <h4 className="text-xs font-bold uppercase tracking-wider text-slate-400">Performance</h4>
                    <ConfigInput 
                      label="Max Connections" 
                      type="number"
                      value={route.max_connections} 
                      onChange={v => updateConfig(c => ({ 
                        ...c, 
                        routes: { ...c.routes, [path]: { ...route, max_connections: Number(v) } } 
                      }))}
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
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <ConfigSwitch 
                    label="Static Route" 
                    description="Serve static files from the specified endpoints."
                    checked={route.is_static} 
                    onChange={v => updateConfig(c => ({ 
                      ...c, 
                      routes: { ...c.routes, [path]: { ...route, is_static: v } } 
                    }))}
                  />
                  <ConfigSwitch 
                    label="Force No Cache" 
                    description="Never cache responses for this specific route."
                    checked={route.no_cache} 
                    onChange={v => updateConfig(c => ({ 
                      ...c, 
                      routes: { ...c.routes, [path]: { ...route, no_cache: v } } 
                    }))}
                  />
                  <ConfigSwitch 
                    label="Require Auth" 
                    description="Apply the default authentication rules to this route."
                    checked={route.needs_auth} 
                    onChange={v => updateConfig(c => ({ 
                      ...c, 
                      routes: { ...c.routes, [path]: { ...route, needs_auth: v } } 
                    }))}
                  />
                  <div className="p-4 rounded-lg border border-slate-100 bg-white">
                    <ConfigSelect 
                      label="HTTP Version" 
                      value={route.http_version} 
                      options={[{ label: "HTTP/1.1", value: "HTTP1" }, { label: "HTTP/2", value: "HTTP2" }]}
                      onChange={v => updateConfig(c => ({ 
                        ...c, 
                        routes: { ...c.routes, [path]: { ...route, http_version: v as any } } 
                      }))}
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
