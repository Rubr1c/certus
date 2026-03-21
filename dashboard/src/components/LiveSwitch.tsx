interface LiveSwitchProps {
  isActive: boolean;
  onToggle: (state: boolean) => void;
  label?: string;
  disabled?: boolean;
  tooltipMessage?: string;
}

export function LiveSwitch({
  isActive,
  onToggle,
  label = "Live",
  disabled = false,
  tooltipMessage,
}: LiveSwitchProps) {
  const handleClick = () => {
    if (!disabled) onToggle(!isActive);
  };

  return (
    <div
      className={`group relative flex select-none items-center gap-3 ${
        disabled ? "cursor-not-allowed opacity-50" : ""
      }`}
    >
      {disabled && tooltipMessage && (
        <span
          className="absolute top-full right-0 mt-2 hidden w-max max-w-[200px] whitespace-normal rounded bg-slate-800 px-2 py-1 text-center text-xs text-white shadow-lg group-hover:block z-50"
          role="tooltip"
        >
          {tooltipMessage}
        </span>
      )}
      <button
        type="button"
        onClick={handleClick}
        disabled={disabled}
        className={`
          relative h-6 w-10 shrink-0 cursor-pointer rounded-full transition-colors duration-200
          focus:outline-none focus:ring-2 focus:ring-ocean-500 focus:ring-offset-2
          disabled:cursor-not-allowed
          ${
            isActive
              ? "bg-emerald-500"
              : "bg-slate-200"
          }
        `}
      >
        <span
          className={`
            absolute top-1 block h-4 w-4 rounded-full bg-white shadow-sm
            transition-transform duration-200
            ${isActive ? "left-1 translate-x-4" : "left-1"}
          `}
        />
      </button>
      <div className="flex items-center gap-2">
        <span
          className={`h-2 w-2 shrink-0 rounded-full transition-colors ${
            isActive ? "bg-emerald-500 animate-pulse" : "bg-transparent"
          }`}
          aria-hidden
        />
        <span className="text-sm font-semibold text-slate-900">{label}</span>
      </div>
    </div>
  );
}
