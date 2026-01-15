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
