import { useAegisStore } from "../store";

const TYPE_STYLES: Record<string, { color: string; label: string }> = {
  thought:     { color: "#d29922", label: "THINK" },
  tool_call:   { color: "#58a6ff", label: "CALL" },
  tool_result: { color: "#3fb950", label: "RESULT" },
  response:    { color: "#c9d1d9", label: "REPLY" },
};

export default function AIReasoningTrace() {
  const steps = useAegisStore((s) => s.reasoningSteps);
  const clearTrace = useAegisStore((s) => s.clearReasoningTrace);

  return (
    <div className="h-full flex flex-col bg-[#0d1117]">
      <div className="flex items-center justify-between px-4 py-1 bg-[#161b22] border-b border-[#30363d]">
        <span className="text-xs text-[#8b949e]">AI reasoning trace — {steps.length} steps</span>
        <button
          onClick={clearTrace}
          className="text-xs text-[#8b949e] hover:text-[#c9d1d9] transition"
        >
          clear
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {steps.length === 0 && (
          <p className="text-[#8b949e] text-xs italic">
            AI reasoning steps will appear here during active sessions.
          </p>
        )}
        {steps.map((step) => {
          const style = TYPE_STYLES[step.type] ?? TYPE_STYLES.response;
          return (
            <div key={step.id} className="border border-[#30363d] rounded p-3">
              <div className="flex items-center gap-2 mb-1">
                <span
                  className="text-xs font-bold px-1.5 py-0.5 rounded"
                  style={{ color: style.color, border: `1px solid ${style.color}` }}
                >
                  {style.label}
                </span>
                {step.toolName && (
                  <span className="text-xs text-[#58a6ff]">{step.toolName}</span>
                )}
                <span className="text-xs text-[#30363d] ml-auto">
                  {new Date(step.timestamp).toISOString().slice(11, 23)}
                </span>
              </div>
              <pre className="text-xs text-[#c9d1d9] whitespace-pre-wrap break-all">
                {step.content}
              </pre>
            </div>
          );
        })}
      </div>
    </div>
  );
}
