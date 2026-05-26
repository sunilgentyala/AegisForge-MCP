import { useAegisStore } from "../store";

const LEVEL_COLOR: Record<string, string> = {
  info:  "#3fb950",
  warn:  "#d29922",
  error: "#f85149",
  debug: "#8b949e",
};

export default function SecurityDashboard() {
  const logs = useAegisStore((s) => s.logs);
  const lastScan = useAegisStore((s) => s.lastScan);
  const clearLogs = useAegisStore((s) => s.clearLogs);

  const openPorts = lastScan?.results.filter((r) => r.open) ?? [];

  return (
    <div className="h-full flex gap-0 bg-[#0d1117]">
      {/* Scan results panel */}
      <div className="w-80 flex flex-col border-r border-[#30363d]">
        <div className="px-4 py-1 bg-[#161b22] border-b border-[#30363d]">
          <span className="text-xs text-[#8b949e]">last scan results</span>
        </div>
        <div className="flex-1 overflow-y-auto p-4">
          {!lastScan && (
            <p className="text-xs text-[#8b949e] italic">No scan results yet.</p>
          )}
          {lastScan && (
            <>
              <div className="mb-3">
                <p className="text-xs text-[#8b949e]">target</p>
                <p className="text-sm text-[#58a6ff]">{lastScan.target}</p>
              </div>
              <div className="flex gap-4 mb-3">
                <div>
                  <p className="text-xs text-[#8b949e]">open</p>
                  <p className="text-lg font-bold text-[#3fb950]">{lastScan.total_open}</p>
                </div>
                <div>
                  <p className="text-xs text-[#8b949e]">elapsed</p>
                  <p className="text-lg font-bold text-[#c9d1d9]">{lastScan.elapsed_ms}ms</p>
                </div>
              </div>
              <div className="space-y-1">
                {openPorts.map((r) => (
                  <div key={r.port} className="flex justify-between text-xs border border-[#30363d] rounded px-2 py-1">
                    <span className="text-[#3fb950] font-mono">{r.port}</span>
                    <span className="text-[#8b949e]">{r.serviceHint ?? "unknown"}</span>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Security event log */}
      <div className="flex-1 flex flex-col">
        <div className="flex items-center justify-between px-4 py-1 bg-[#161b22] border-b border-[#30363d]">
          <span className="text-xs text-[#8b949e]">security events — {logs.length}</span>
          <button
            onClick={clearLogs}
            className="text-xs text-[#8b949e] hover:text-[#c9d1d9] transition"
          >
            clear
          </button>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-1">
          {logs.slice().reverse().map((log) => (
            <div key={log.id} className="flex gap-2 text-xs">
              <span style={{ color: LEVEL_COLOR[log.level] }} className="w-12 shrink-0 uppercase">
                {log.level}
              </span>
              <span className="text-[#30363d] shrink-0">
                {new Date(log.timestamp).toISOString().slice(11, 23)}
              </span>
              <span className="text-[#8b949e] shrink-0">[{log.source}]</span>
              <span className="text-[#c9d1d9]">{log.message}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
