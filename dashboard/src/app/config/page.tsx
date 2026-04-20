"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Settings, 
  Server, 
  Shield, 
  Database, 
  Network, 
} from "lucide-react";
import { api } from "@/api";
import { useState, useEffect } from "react";
import { Config } from "@/lib/config";
import { toast } from "sonner";

import { GeneralSection } from "./ConfigSections/GeneralSection";
import { RoutesSection } from "./ConfigSections/RoutesSection";
import { SecuritySection } from "./ConfigSections/SecuritySection";
import { CacheSection } from "./ConfigSections/CacheSection";

type Section = "general" | "routes" | "security" | "cache";

export default function ConfigPage() {
  const queryClient = useQueryClient();
  const { data: initialConfig, isLoading, error } = useQuery({
    queryKey: ["config"],
    queryFn: () => api.config.get(),
  });

  const [config, setConfig] = useState<Config | null>(null);
  const [activeSection, setActiveSection] = useState<Section>("general");

  const saveMutation = useMutation({
    mutationFn: (newConfig: Config) => api.config.update(newConfig),
    onSuccess: () => {
      toast.success("Configuration saved successfully!");
      queryClient.invalidateQueries({ queryKey: ["config"] });
    },
    onError: (err) => {
      toast.error("Failed to save configuration: " + (err instanceof Error ? err.message : "Unknown error"));
    },
  });

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

  const handleSave = () => {
    if (config) {
      saveMutation.mutate(config);
    }
  };

  return (
    <div className="max-w-6xl mx-auto pb-20">
      <header className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold text-text-main flex items-center gap-2">
            <Settings className="w-6 h-6 text-ocean-500" />
            Gateway Configuration
          </h1>
          <p className="text-text-muted mt-1 text-sm">
            Manage your API gateway settings and routing rules.
          </p>
        </div>
        <div className="flex gap-3">
          <button 
            className="px-4 py-2 text-sm font-semibold text-text-muted hover:text-text-main transition-colors cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
            onClick={() => setConfig(initialConfig || null)}
            disabled={saveMutation.isPending}
          >
            Reset
          </button>
          <button 
            className="px-6 py-2 bg-ocean-500 text-white text-sm font-bold rounded-lg hover:bg-ocean-600 transition-colors shadow-sm shadow-ocean-200 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50 flex items-center gap-2"
            onClick={handleSave}
            disabled={saveMutation.isPending || !config}
          >
            {saveMutation.isPending && <div className="animate-spin rounded-full h-3 w-3 border-b-2 border-white"></div>}
            {saveMutation.isPending ? "Saving..." : "Save Changes"}
          </button>
        </div>
      </header>

      <div className="flex flex-col md:flex-row gap-8 items-start">
        {/* Sidebar Nav */}
        <aside className="w-full md:w-64 shrink-0 bg-white rounded-xl border border-slate-200 p-2 overflow-hidden sticky top-24 shadow-sm">
          <nav className="space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = activeSection === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveSection(item.id)}
                  className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm font-bold rounded-lg transition-colors cursor-pointer ${
                    isActive 
                      ? "bg-ocean-50 text-ocean-600 shadow-sm border border-ocean-100/50 cursor-default" 
                      : "text-text-muted hover:bg-slate-50 hover:text-text-main border border-transparent"
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
        <main className="flex-1 min-w-0 w-full">
          {activeSection === "general" && (
            <GeneralSection config={config} updateConfig={updateConfig} />
          )}
          {activeSection === "routes" && (
            <RoutesSection config={config} updateConfig={updateConfig} />
          )}
          {activeSection === "security" && (
            <SecuritySection config={config} updateConfig={updateConfig} />
          )}
          {activeSection === "cache" && (
            <CacheSection config={config} updateConfig={updateConfig} />
          )}
        </main>
      </div>
    </div>
  );
}
