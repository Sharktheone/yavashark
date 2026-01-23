#!/usr/bin/env -S deno run --allow-net=localhost --allow-read

// YavaShark Test262 Scripting Runtime
// This is the main entry point for the Deno subprocess
// It reads JSON-RPC requests from stdin, executes scripts, and writes responses to stdout

import type { RPCRequest, RPCResponse, ExecuteParams } from "./types.ts";
import { ys, setServerUrl, setCurrentSession, resetOutput, getOutput, setAbortSignal } from "./api.ts";

// Track current execution for cancellation
let currentExecutionId: number | null = null;
let currentAbortController: AbortController | null = null;

// Mutex for writing responses (ensure responses aren't interleaved)
let writePromise: Promise<void> = Promise.resolve();

// Write a JSON-RPC response to stdout (serialized)
function writeResponse(response: RPCResponse): void {
  writePromise = writePromise.then(() => {
    const json = JSON.stringify(response);
    console.log(json);
  });
}

// Execute a user script using Function constructor
async function executeScript(params: ExecuteParams, abortSignal: AbortSignal): Promise<{ output: string[]; result: unknown }> {
  setCurrentSession(params.sessionId);
  resetOutput(); // Clear output from previous execution
  
  // Set server URL if provided
  if (params.serverUrl) {
    setServerUrl(params.serverUrl);
  }
  
  // Set abort signal for API calls
  setAbortSignal(abortSignal);
  
  // Wrap script in async function
  const asyncFn = new Function("ys", `
    return (async () => {
      ${params.script}
    })();
  `);

  // Create abort promise that rejects when cancelled
  const abortPromise = new Promise((_, reject) => {
    if (abortSignal.aborted) {
      reject(new DOMException("Script execution was cancelled", "AbortError"));
      return;
    }
    abortSignal.addEventListener("abort", () => {
      reject(new DOMException("Script execution was cancelled", "AbortError"));
    }, { once: true });
  });

  const result = await Promise.race([
    asyncFn(ys),
    abortPromise,
  ]);

  return {
    output: getOutput(),
    result,
  };
}

// Handle cancel request - returns response immediately, doesn't await
function handleCancelRequest(request: RPCRequest): RPCResponse {
  const params = request.params as { id?: number } | undefined;
  const targetId = params?.id;
  
  if (currentAbortController && (targetId === undefined || targetId === currentExecutionId)) {
    currentAbortController.abort();
    return {
      jsonrpc: "2.0",
      id: request.id,
      result: { cancelled: true, executionId: currentExecutionId },
    };
  } else {
    return {
      jsonrpc: "2.0",
      id: request.id,
      result: { cancelled: false, reason: "No matching execution to cancel" },
    };
  }
}

// Handle execute request - this is async and can be cancelled
async function handleExecuteRequest(request: RPCRequest): Promise<RPCResponse> {
  const params = request.params as ExecuteParams;
  
  // Set up cancellation
  currentExecutionId = request.id;
  currentAbortController = new AbortController();
  
  try {
    const { output, result } = await executeScript(params, currentAbortController.signal);
    
    // Build a combined result with output and return value
    let finalResult: unknown;
    
    if (output.length > 0 && result !== undefined) {
      finalResult = {
        output: output.join('\n'),
        result,
      };
    } else if (output.length > 0) {
      finalResult = {
        output: output.join('\n'),
      };
    } else if (result !== undefined) {
      if (typeof result === 'object' && result !== null) {
        finalResult = result;
      } else {
        finalResult = { value: result };
      }
    } else {
      finalResult = { message: "Script completed with no output" };
    }
    
    return {
      jsonrpc: "2.0",
      id: request.id,
      result: finalResult,
    };
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      return {
        jsonrpc: "2.0",
        id: request.id,
        error: {
          code: -32001,
          message: "Script execution was cancelled",
        },
      };
    } else {
      return {
        jsonrpc: "2.0",
        id: request.id,
        error: {
          code: -32000,
          message: error instanceof Error ? error.message : String(error),
        },
      };
    }
  } finally {
    currentExecutionId = null;
    currentAbortController = null;
    setAbortSignal(null);
  }
}

// Handle an incoming JSON-RPC request
function handleRequest(request: RPCRequest): Promise<RPCResponse> | RPCResponse {
  switch (request.method) {
    case "execute":
      return handleExecuteRequest(request);
    
    case "cancel":
      return handleCancelRequest(request);
    
    default:
      return {
        jsonrpc: "2.0",
        id: request.id,
        error: {
          code: -32601,
          message: `Method not found: ${request.method}`,
        },
      };
  }
}

// Main entry point - reads lines and dispatches requests concurrently
async function main(): Promise<void> {
  const decoder = new TextDecoder();
  const reader = Deno.stdin.readable.getReader();
  let buffer = "";
  
  // Track the current execute promise so we can ensure proper ordering
  let currentExecutePromise: Promise<void> | null = null;

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";

      for (const line of lines) {
        if (!line.trim()) continue;
        
        let request: RPCRequest;
        try {
          request = JSON.parse(line) as RPCRequest;
        } catch (error) {
          writeResponse({
            jsonrpc: "2.0",
            id: 0,
            error: {
              code: -32700,
              message: `Parse error: ${error instanceof Error ? error.message : String(error)}`,
            },
          });
          continue;
        }

        // Handle cancel requests immediately (synchronously)
        if (request.method === "cancel") {
          const response = handleRequest(request);
          writeResponse(response as RPCResponse);
          continue;
        }

        // For execute requests, we need to wait for any previous execute to finish
        // to maintain ordering, but we dispatch them asynchronously so cancel can interrupt
        if (request.method === "execute") {
          // Wait for previous execute to complete (if any)
          if (currentExecutePromise) {
            await currentExecutePromise;
          }
          
          // Start new execute and track it
          currentExecutePromise = (async () => {
            const responsePromise = handleRequest(request);
            const response = responsePromise instanceof Promise 
              ? await responsePromise 
              : responsePromise;
            writeResponse(response);
          })();
          
          // Don't await here - let the main loop continue to read cancel requests
          // The response will be written when the execute completes
        } else {
          // Other requests are handled synchronously
          const response = handleRequest(request);
          if (response instanceof Promise) {
            writeResponse(await response);
          } else {
            writeResponse(response);
          }
        }
      }
    }

    // Handle remaining buffer
    if (buffer.trim()) {
      try {
        const request = JSON.parse(buffer) as RPCRequest;
        const response = handleRequest(request);
        if (response instanceof Promise) {
          writeResponse(await response);
        } else {
          writeResponse(response);
        }
      } catch (error) {
        writeResponse({
          jsonrpc: "2.0",
          id: 0,
          error: {
            code: -32700,
            message: `Parse error: ${error instanceof Error ? error.message : String(error)}`,
          },
        });
      }
    }
    
    // Wait for any pending execute to complete
    if (currentExecutePromise) {
      await currentExecutePromise;
    }
  } finally {
    reader.releaseLock();
  }
}

main().catch(console.error);
