import { useEffect, useRef } from "react";
import { useAegisStore } from "../store";

const STREAM_COLOR: Record<string, string> = {
  stdout: "#c9d1d9",
  stderr: "#f85149",
  system: "#58a6ff",
};

export default function Terminal() {
  const lines = useAegisStore((s) => s.terminalLines);
  const clearTerminal = useAegisStore((s) => s.clearTerminal);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to newest output without blocking the main thread
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [lines.length]);

  return (
    <div className="h-full flex flex-col bg-[#0d1117]">
      <div className="flex items-center justify-between px-4 py-1 bg-[#161b22] border-b border-[#30363d]">
        <span className="text-xs text-[#8b949e]">terminal — {lines.length} lines</span>
        <button
          onClick={clearTerminal}
          className="text-xs text-[#8b949e] hover:text-[#c9d1d9] transition"
        >
          clear
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 font-mono text-xs leading-5">
        {lines.map((line) => (
          <div key={line.id} style={{ color: STREAM_COLOR[line.stream] ?? "#c9d1d9" }}>
            <span className="text-[#30363d] mr-2">
              {new Date(line.timestamp).toISOString().slice(11, 23)}
            </span>
            {line.content}
          </div>
        ))}
        <div ref={bottomRef} />
      </div>
    </div>
  );
}
