"use client";

import { ReactNode } from "react";
import { type LucideIcon } from "lucide-react";

interface SectionCardProps {
  title: string;
  icon: LucideIcon;
  children: ReactNode;
}

export function SectionCard({ title, icon: Icon, children }: SectionCardProps) {
  return (
    <div className="card-container overflow-hidden">
      <div className="flex items-center gap-2 border-b border-slate-100 bg-slate-50/30 px-6 py-4">
        <Icon className="w-4 h-4 text-ocean-500" />
        <h2 className="font-semibold text-text-main">{title}</h2>
      </div>
      <div className="p-6">
        {children}
      </div>
    </div>
  );
}

interface ConfigInputProps {
  label: string;
  value: string | number;
  onChange: (value: string | number) => void;
  type?: string;
  icon?: LucideIcon;
  placeholder?: string;
  className?: string;
}

export function ConfigInput({ label, value, onChange, type = "text", icon: Icon, placeholder, className }: ConfigInputProps) {
  return (
    <div className={className}>
      <label className="block text-sm font-medium text-text-muted mb-2">
        {label}
      </label>
      <div className="relative">
        {Icon && <Icon className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />}
        <input 
          type={type}
          value={value ?? ""}
          onChange={e => onChange(type === "number" ? Number(e.target.value) : e.target.value)}
          className={`input-base ${Icon ? 'pl-10' : ''}`}
          placeholder={placeholder}
        />
      </div>
    </div>
  );
}

interface ConfigSelectProps {
  label: string;
  value: string;
  options: { label: string; value: string }[];
  onChange: (value: string) => void;
  className?: string;
}

export function ConfigSelect({ label, value, options, onChange, className }: ConfigSelectProps) {
  return (
    <div className={className}>
      <label className="block text-sm font-medium text-text-muted mb-2">
        {label}
      </label>
      <select 
        value={value}
        onChange={e => onChange(e.target.value)}
        className="input-base"
      >
        {options.map(opt => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    </div>
  );
}

interface ConfigSwitchProps {
  label: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function ConfigSwitch({ label, description, checked, onChange }: ConfigSwitchProps) {
  return (
    <div className="flex items-center justify-between p-4 rounded-lg border border-slate-100 hover:bg-slate-50 transition-colors">
      <div>
        <h4 className="text-sm font-medium text-text-main">{label}</h4>
        <p className="text-xs text-text-muted">{description}</p>
      </div>
      <button
        onClick={() => onChange(!checked)}
        className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${checked ? 'bg-ocean-500' : 'bg-slate-200'}`}
      >
        <span
          className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${checked ? 'translate-x-5' : 'translate-x-0'}`}
        />
      </button>
    </div>
  );
}
