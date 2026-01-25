// Type definitions for the YavaShark Test262 scripting API

export type Status = 
  | "PASS" 
  | "FAIL" 
  | "SKIP" 
  | "TIMEOUT" 
  | "CRASH" 
  | "PARSE_ERROR" 
  | "NOT_IMPLEMENTED"
  | "RUNNER_ERROR";

export interface TestEntry {
  path: string;
  status: Status;
}

export interface TestStatus {
  path: string;
  status: Status;
}

export interface TestOutput {
  path: string;
  status: Status;
  message: string;
  duration: number;
}

export interface RerunOptions {
  paths?: string[];
  dir?: string;
  failedOnly?: boolean;
  rebuild?: boolean;
}

export interface RerunResult {
  id: string;
  before: TestEntry[];
  after: TestEntry[];
  diff: DiffResult;
  duration: number;
  status: "complete" | "timeout" | "cancelled";
}

export interface RerunJob {
  id: string;
  status: "pending" | "running" | "complete" | "cancelled";
  progress?: {
    completed: number;
    total: number;
  };
  result?: RerunResult;
}

export interface DiffResult {
  gained: TestEntry[];  // Now passing
  lost: TestEntry[];    // Now failing
  changed: TestEntry[]; // Status changed (any)
}

export interface SpecMatch {
  section: string;
  title: string;
  preview: string;
}

// Run history types

export interface ChangedTest {
  path: string;
  oldStatus: string;
  newStatus: string;
}

export interface RunHistoryEntry {
  id: string;
  path: string;
  paths?: string[];
  profile?: string;
  source?: string;  // "http", "mcp", "stream"
  startedAt: string;
  completedAt?: string;
  phase: string;
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  crashed: number;
  timeout: number;
  gained: number;
  lost: number;
  failedOnly?: boolean;
  rebuild?: boolean;
  baselineRef?: string;
  changedTests?: ChangedTest[];
  buildOutput?: string[];
}

export interface RunDetails {
  id: string;
  before: TestEntry[];
  after: TestEntry[];
  diff: DiffResult;
  duration: number;
  status: string;
  options: RerunOptions;
}

// Compare types

export interface CompareSource {
  type: "current" | "run";
  runId?: string;  // Required when type is "run"
}

export interface CompareOptions {
  left: CompareSource;
  right: CompareSource;
}

export interface CompareStats {
  total: number;
  passed: number;
  failed: number;
  skipped: number;
  crashed: number;
  timeout: number;
}

export interface CompareChangedTest {
  path: string;
  leftStatus: string;
  rightStatus: string;
}

export interface CompareResult {
  left: CompareStats;
  right: CompareStats;
  gained: number;      // Tests that went from non-pass to pass
  lost: number;        // Tests that went from pass to non-pass
  changed: number;     // Total tests with different status
  unchanged: number;   // Total tests with same status
  changedTests: CompareChangedTest[];
}

// JSON-RPC types

export interface RPCRequest {
  jsonrpc: "2.0";
  id: number;
  method: string;
  params?: unknown;
}

export interface RPCResponse {
  jsonrpc: "2.0";
  id: number;
  result?: unknown;
  error?: RPCError;
}

export interface RPCError {
  code: number;
  message: string;
  data?: unknown;
}

export interface ExecuteParams {
  script: string;
  sessionId?: string;
  timeout: number;
  serverUrl?: string;
}

// API call types (script -> Go server)

export interface APICallRequest {
  id: number;
  method: string;
  params?: unknown;
}

export interface APICallResponse {
  id: number;
  result?: unknown;
  error?: RPCError;
}
