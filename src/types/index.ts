export type LogLevel = "info" | "warn" | "error" | "debug";

export interface SecurityLogEntry {
  id: string;
  timestamp: number;
  level: LogLevel;
  source: string;
  message: string;
  metadata?: Record<string, unknown>;
}

export interface AIReasoningStep {
  id: string;
  timestamp: number;
  type: "thought" | "tool_call" | "tool_result" | "response";
  content: string;
  toolName?: string;
}

export interface TerminalLine {
  id: string;
  timestamp: number;
  content: string;
  stream: "stdout" | "stderr" | "system";
}

export interface PortScanResult {
  port: number;
  open: boolean;
  serviceHint?: string;
}

export interface ScanResponse {
  target: string;
  results: PortScanResult[];
  elapsed_ms: number;
  total_open: number;
}
