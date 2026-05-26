import { invoke } from "@tauri-apps/api/core";
import { useAegisStore } from "./store";
import Terminal from "./components/Terminal";
import AIReasoningTrace from "./components/AIReasoningTrace";
import SecurityDashboard from "./components/SecurityDashboard";
import type { ScanResponse } from "./types";

const TABS = [
  { id: "terminal" as const,   label: "Terminal" },
  { id: "ai-trace" as const,   label: "AI Trace" },
  { id: "dashboard" as const,  label: "Dashboard" },
];

export default function App() {
  const { activePanel, setActivePanel, isScanning, setIsScanning, setLastScan, addLog, appendLine } =
    useAegisStore();

  const handleQuickScan = async () => {
    const targetIp = window.prompt("Target IP address:");
    if (!targetIp?.trim()) return;

    setIsScanning(true);
    appendLine({ content: `[aegisforge] starting scan on ${targetIp}`, stream: "system" });
    addLog({ level: "info", source: "recon", message: `scan initiated — target: ${targetIp}` });

    try {
      const result = await invoke<ScanResponse>("scan_ports", {
        request: { target: targetIp, ports: [22, 80, 443, 3389, 8080], timeout_ms: 1500 },
      });
      setLastScan(result);
      appendLine({
        content: `[aegisforge] scan complete — ${result.total_open} open port(s) in ${result.elapsed_ms}ms`,
        stream: "stdout",
      });
      addLog({ level: "info", source: "recon", message: `scan finished — ${result.total_open} open` });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      appendLine({ content: `[ERROR] ${msg}`, stream: "stderr" });
      addLog({ level: "error", source: "recon", message: msg });
    } finally {
      setIsScanning(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-[#0d1117] text-[#c9d1d9]">
      {/* Header */}
      <header className="flex items-center gap-4 px-4 py-2 bg-[#161b22] border-b border-[#30363d]">
        <span className="text-[#3fb950] font-bold tracking-wider">AegisForge</span>
        <span className="text-[#8b949e] text-xs">AI Security Research Engine</span>
        <div className="flex gap-2 ml-auto">
          <button
            onClick={handleQuickScan}
            disabled={isScanning}
            className="px-3 py-1 text-xs rounded bg-[#3fb950] text-black font-semibold disabled:opacity-40 hover:bg-green-400 transition"
          >
            {isScanning ? "Scanning..." : "Quick Scan"}
          </button>
        </div>
      </header>

      {/* Tab bar */}
      <nav className="flex gap-1 px-4 pt-2 bg-[#161b22] border-b border-[#30363d]">
        {TABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setActivePanel(t.id)}
            className={`px-4 py-1 text-xs rounded-t transition ${
              activePanel === t.id
                ? "bg-[#0d1117] text-[#58a6ff] border-t border-x border-[#30363d]"
                : "text-[#8b949e] hover:text-[#c9d1d9]"
            }`}
          >
            {t.label}
          </button>
        ))}
      </nav>

      {/* Panel content */}
      <main className="flex-1 overflow-hidden">
        {activePanel === "terminal"   && <Terminal />}
        {activePanel === "ai-trace"   && <AIReasoningTrace />}
        {activePanel === "dashboard"  && <SecurityDashboard />}
      </main>
    </div>
  );
}
