import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type {
  SecurityLogEntry,
  AIReasoningStep,
  TerminalLine,
  ScanResponse,
} from "@/types";

// Monotonic ID generator — avoids Date collisions under rapid updates
let _seq = 0;
const nextId = () => `${Date.now()}-${++_seq}`;

// Ring-buffer sizes — bounds memory usage for long-running sessions
const MAX_LOGS = 10_000;
const MAX_TERMINAL = 5_000;

export type ActivePanel = "terminal" | "ai-trace" | "dashboard";

interface AegisState {
  // Security event log
  logs: SecurityLogEntry[];
  addLog(entry: Omit<SecurityLogEntry, "id" | "timestamp">): void;
  clearLogs(): void;

  // AI reasoning trace (tool calls, thoughts, results from Claude)
  reasoningSteps: AIReasoningStep[];
  addReasoningStep(step: Omit<AIReasoningStep, "id" | "timestamp">): void;
  clearReasoningTrace(): void;

  // Raw terminal I/O
  terminalLines: TerminalLine[];
  appendLine(line: Omit<TerminalLine, "id" | "timestamp">): void;
  clearTerminal(): void;

  // Latest scan results
  lastScan: ScanResponse | null;
  setLastScan(scan: ScanResponse): void;

  // UI state
  activePanel: ActivePanel;
  setActivePanel(panel: ActivePanel): void;
  isScanning: boolean;
  setIsScanning(v: boolean): void;
}

export const useAegisStore = create<AegisState>()(
  immer((set) => ({
    // ── logs ──────────────────────────────────────────────────────────────
    logs: [],
    addLog: (entry) =>
      set((s) => {
        if (s.logs.length >= MAX_LOGS) s.logs.shift();
        s.logs.push({ ...entry, id: nextId(), timestamp: Date.now() });
      }),
    clearLogs: () => set((s) => { s.logs = []; }),

    // ── AI trace ──────────────────────────────────────────────────────────
    reasoningSteps: [],
    addReasoningStep: (step) =>
      set((s) => {
        s.reasoningSteps.push({ ...step, id: nextId(), timestamp: Date.now() });
      }),
    clearReasoningTrace: () => set((s) => { s.reasoningSteps = []; }),

    // ── terminal ──────────────────────────────────────────────────────────
    terminalLines: [],
    appendLine: (line) =>
      set((s) => {
        if (s.terminalLines.length >= MAX_TERMINAL) s.terminalLines.shift();
        s.terminalLines.push({ ...line, id: nextId(), timestamp: Date.now() });
      }),
    clearTerminal: () => set((s) => { s.terminalLines = []; }),

    // ── scan results ──────────────────────────────────────────────────────
    lastScan: null,
    setLastScan: (scan) => set((s) => { s.lastScan = scan; }),

    // ── UI ────────────────────────────────────────────────────────────────
    activePanel: "terminal",
    setActivePanel: (panel) => set((s) => { s.activePanel = panel; }),
    isScanning: false,
    setIsScanning: (v) => set((s) => { s.isScanning = v; }),
  }))
);
