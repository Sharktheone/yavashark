// YavaShark Test262 Scripting API
// This file implements the `ys` namespace available to user scripts

import type {
  Status,
  TestEntry,
  TestStatus,
  TestOutput,
  RerunOptions,
  RerunResult,
  RerunJob,
  SpecMatch,
} from "./types.ts";

// Session storage (in-memory, persisted per session ID)
const sessions = new Map<string, Map<string, unknown>>();
let currentSessionId: string | undefined;

// Server URL for API calls (set by runtime)
let serverUrl = "http://localhost:1215";

// Output collection for ys.print()
let outputLines: string[] = [];

export function setServerUrl(url: string) {
  serverUrl = url;
}

export function setCurrentSession(sessionId: string | undefined) {
  currentSessionId = sessionId;
}

// Reset output collection before each script execution
export function resetOutput(): void {
  outputLines = [];
}

// Get collected output
export function getOutput(): string[] {
  return outputLines;
}

function getSessionStore(): Map<string, unknown> {
  if (!currentSessionId) {
    // Use a default session if none specified
    currentSessionId = "__default__";
  }
  if (!sessions.has(currentSessionId)) {
    sessions.set(currentSessionId, new Map());
  }
  return sessions.get(currentSessionId)!;
}

// Make an API call to the Go server via HTTP
async function apiCall(method: string, params?: unknown): Promise<unknown> {
  const response = await fetch(`${serverUrl}/api/script/call`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ method, params }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`API call failed: ${response.status} ${text}`);
  }

  const result = await response.json();
  if (result.error) {
    throw new Error(result.error);
  }

  return result.result;
}

// TestQuery - fluent API for querying tests
class TestQuery {
  private filters: Array<(t: TestEntry) => boolean> = [];
  private dirPath?: string;
  private recursive = true;
  private statusFilter?: Status | Status[];
  private searchQuery?: string;

  constructor() {}

  filter(predicate: (t: TestEntry) => boolean): TestQuery {
    const q = this.clone();
    q.filters.push(predicate);
    return q;
  }

  withStatus(status: Status | Status[]): TestQuery {
    const q = this.clone();
    q.statusFilter = status;
    return q;
  }

  inDir(path: string, recursive = true): TestQuery {
    const q = this.clone();
    q.dirPath = path;
    q.recursive = recursive;
    return q;
  }

  search(query: string): TestQuery {
    const q = this.clone();
    q.searchQuery = query;
    return q;
  }

  private clone(): TestQuery {
    const q = new TestQuery();
    q.filters = [...this.filters];
    q.dirPath = this.dirPath;
    q.recursive = this.recursive;
    q.statusFilter = this.statusFilter;
    q.searchQuery = this.searchQuery;
    return q;
  }

  private async fetchTests(): Promise<TestEntry[]> {
    const params: Record<string, unknown> = {};
    
    if (this.dirPath) {
      params.dir = this.dirPath;
      params.recursive = this.recursive;
    }
    
    if (this.statusFilter) {
      params.status = this.statusFilter;
    }
    
    if (this.searchQuery) {
      params.query = this.searchQuery;
    }

    const result = await apiCall("tests.query", params);
    let tests = result as TestEntry[];

    // Apply local filters
    for (const filter of this.filters) {
      tests = tests.filter(filter);
    }

    return tests;
  }

  async toArray(): Promise<TestEntry[]> {
    return this.fetchTests();
  }

  async count(): Promise<number> {
    const tests = await this.fetchTests();
    return tests.length;
  }

  async first(n = 1): Promise<TestEntry[]> {
    const tests = await this.fetchTests();
    return tests.slice(0, n);
  }

  async paths(): Promise<string[]> {
    const tests = await this.fetchTests();
    return tests.map(t => t.path);
  }

  async groupByStatus(): Promise<Record<Status, TestEntry[]>> {
    const tests = await this.fetchTests();
    const groups: Record<string, TestEntry[]> = {};
    
    for (const test of tests) {
      if (!groups[test.status]) {
        groups[test.status] = [];
      }
      groups[test.status].push(test);
    }
    
    return groups as Record<Status, TestEntry[]>;
  }

  async groupByDir(depth = 1): Promise<Record<string, TestEntry[]>> {
    const tests = await this.fetchTests();
    const groups: Record<string, TestEntry[]> = {};
    
    for (const test of tests) {
      const parts = test.path.split("/");
      const dir = parts.slice(0, depth).join("/");
      
      if (!groups[dir]) {
        groups[dir] = [];
      }
      groups[dir].push(test);
    }
    
    return groups;
  }
}

// The ys namespace - main API exposed to scripts
export const ys = {
  // Print output - use this to add text to the result
  print(...args: unknown[]): void {
    const line = args.map(arg => 
      typeof arg === 'string' ? arg : JSON.stringify(arg, null, 2)
    ).join(' ');
    outputLines.push(line);
  },

  tests: {
    all(): TestQuery {
      return new TestQuery();
    },

    inDir(path: string, recursive = true): TestQuery {
      return new TestQuery().inDir(path, recursive);
    },

    search(query: string): TestQuery {
      return new TestQuery().search(query);
    },

    withStatus(status: Status): TestQuery {
      return new TestQuery().withStatus(status);
    },

    failing(): TestQuery {
      return new TestQuery().withStatus("FAIL");
    },

    async getStatus(path: string): Promise<TestStatus> {
      return await apiCall("tests.getStatus", { path }) as TestStatus;
    },

    async getOutput(path: string): Promise<TestOutput> {
      return await apiCall("tests.getOutput", { path }) as TestOutput;
    },

    async getCode(path: string): Promise<string> {
      return await apiCall("tests.getCode", { path }) as string;
    },

    async setCode(path: string, code: string): Promise<void> {
      await apiCall("tests.setCode", { path, code });
    },
  },

  harness: {
    async getCode(name: string): Promise<string> {
      return await apiCall("harness.getCode", { name }) as string;
    },

    async listForTest(testPath: string): Promise<string[]> {
      return await apiCall("harness.listForTest", { testPath }) as string[];
    },

    async getForTest(testPath: string): Promise<Record<string, string>> {
      return await apiCall("harness.getForTest", { testPath }) as Record<string, string>;
    },

    async setCode(name: string, code: string): Promise<void> {
      await apiCall("harness.setCode", { name, code });
    },
  },

  spec: {
    async get(section: string): Promise<string> {
      return await apiCall("spec.get", { section }) as string;
    },

    async search(query: string): Promise<SpecMatch[]> {
      return await apiCall("spec.search", { query }) as SpecMatch[];
    },

    async forIntrinsic(name: string): Promise<string> {
      return await apiCall("spec.forIntrinsic", { name }) as string;
    },
  },

  runner: {
    async rerun(opts: RerunOptions): Promise<RerunResult> {
      return await apiCall("runner.rerun", opts) as RerunResult;
    },

    async rerunAsync(opts: RerunOptions): Promise<RerunJob> {
      return await apiCall("runner.rerunAsync", opts) as RerunJob;
    },

    async getJob(jobId: string): Promise<RerunJob> {
      return await apiCall("runner.getJob", { jobId }) as RerunJob;
    },

    async cancelJob(jobId: string): Promise<void> {
      await apiCall("runner.cancelJob", { jobId });
    },
  },

  session: {
    get<T>(key: string): T | undefined {
      return getSessionStore().get(key) as T | undefined;
    },

    set<T>(key: string, value: T): void {
      getSessionStore().set(key, value);
    },

    delete(key: string): void {
      getSessionStore().delete(key);
    },

    clear(): void {
      getSessionStore().clear();
    },
  },

  output: {
    /**
     * Set the maximum number of characters for tool output.
     * @param maxChars - Maximum characters (0 for unlimited)
     */
    async setMaxChars(maxChars: number): Promise<void> {
      await apiCall("output.setMaxChars", { maxChars });
    },

    /**
     * Get the current maximum output characters setting.
     */
    async getMaxChars(): Promise<{ maxChars: number; unlimited: boolean }> {
      return await apiCall("output.getMaxChars", {}) as { maxChars: number; unlimited: boolean };
    },

    /**
     * Set the offset for truncated output (skip first N chars).
     * @param offset - Number of characters to skip from the start
     */
    async setOffset(offset: number): Promise<void> {
      await apiCall("output.setOffset", { offset });
    },

    /**
     * Get the current offset setting.
     */
    async getOffset(): Promise<number> {
      return (await apiCall("output.getOffset", {}) as { offset: number }).offset;
    },

    /**
     * Set whether to take characters from head or tail of output.
     * @param mode - "head" to take from start, "tail" to take from end
     */
    async setMode(mode: "head" | "tail"): Promise<void> {
      await apiCall("output.setMode", { mode });
    },

    /**
     * Get the current mode setting.
     */
    async getMode(): Promise<"head" | "tail"> {
      return (await apiCall("output.getMode", {}) as { mode: "head" | "tail" }).mode;
    },

    /**
     * Configure all output settings at once.
     * @param config - Output configuration
     */
    async configure(config: {
      maxChars?: number;
      offset?: number;
      mode?: "head" | "tail";
    }): Promise<void> {
      await apiCall("output.configure", config);
    },

    /**
     * Get all current output settings.
     */
    async getConfig(): Promise<{
      maxChars: number;
      offset: number;
      mode: "head" | "tail";
      unlimited: boolean;
    }> {
      return await apiCall("output.getConfig", {}) as {
        maxChars: number;
        offset: number;
        mode: "head" | "tail";
        unlimited: boolean;
      };
    },

    /**
     * Get the length of the last tool output (before truncation).
     * Useful for deciding how to paginate large outputs.
     */
    async getLastLength(): Promise<number> {
      return (await apiCall("output.getLastLength", {}) as { length: number }).length;
    },
  },
};
