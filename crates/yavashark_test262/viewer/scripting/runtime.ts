#!/usr/bin/env -S deno run --allow-net=localhost --allow-read

// YavaShark Test262 Scripting Runtime
// This is the main entry point for the Deno subprocess
// It reads JSON-RPC requests from stdin, executes scripts, and writes responses to stdout

import type { RPCRequest, RPCResponse, ExecuteParams } from "./types.ts";
import { ys, setServerUrl, setCurrentSession, resetOutput, getOutput } from "./api.ts";

// Read lines from stdin
async function* readLines(): AsyncGenerator<string> {
  const decoder = new TextDecoder();
  const reader = Deno.stdin.readable.getReader();
  let buffer = "";

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";

      for (const line of lines) {
        if (line.trim()) {
          yield line;
        }
      }
    }

    // Handle remaining buffer
    if (buffer.trim()) {
      yield buffer;
    }
  } finally {
    reader.releaseLock();
  }
}

// Write a JSON-RPC response to stdout
function writeResponse(response: RPCResponse): void {
  const json = JSON.stringify(response);
  console.log(json);
}

// Execute a user script using Function constructor
async function executeScript(params: ExecuteParams): Promise<{ output: string[]; result: unknown }> {
  setCurrentSession(params.sessionId);
  resetOutput(); // Clear output from previous execution
  
  // Set server URL if provided
  if (params.serverUrl) {
    setServerUrl(params.serverUrl);
  }
  
  const timeoutMs = (params.timeout || 10) * 1000;
  
  // Wrap script in async function
  const asyncFn = new Function("ys", `
    return (async () => {
      ${params.script}
    })();
  `);

  const timeoutPromise = new Promise((_, reject) => {
    setTimeout(() => reject(new Error("Script execution timed out")), timeoutMs);
  });

  const result = await Promise.race([
    asyncFn(ys),
    timeoutPromise,
  ]);

  return {
    output: getOutput(),
    result,
  };
}

// Handle an incoming JSON-RPC request
async function handleRequest(request: RPCRequest): Promise<void> {
  let response: RPCResponse;

  try {
    switch (request.method) {
      case "execute": {
        const params = request.params as ExecuteParams;
        const { output, result } = await executeScript(params);
        
        // Build a combined result with output and return value
        // If there's printed output, include it
        // If there's a return value, include it
        let finalResult: unknown;
        
        if (output.length > 0 && result !== undefined) {
          // Both output and return value
          finalResult = {
            output: output.join('\n'),
            result,
          };
        } else if (output.length > 0) {
          // Only output (no return)
          finalResult = {
            output: output.join('\n'),
          };
        } else if (result !== undefined) {
          // Only return value (no output)
          // Wrap primitives in an object for structured content compatibility
          if (typeof result === 'object' && result !== null) {
            finalResult = result;
          } else {
            finalResult = { value: result };
          }
        } else {
          // Nothing returned and nothing printed
          finalResult = { message: "Script completed with no output" };
        }
        
        response = {
          jsonrpc: "2.0",
          id: request.id,
          result: finalResult,
        };
        break;
      }

      default:
        response = {
          jsonrpc: "2.0",
          id: request.id,
          error: {
            code: -32601,
            message: `Method not found: ${request.method}`,
          },
        };
    }
  } catch (error) {
    response = {
      jsonrpc: "2.0",
      id: request.id,
      error: {
        code: -32000,
        message: error instanceof Error ? error.message : String(error),
      },
    };
  }

  writeResponse(response);
}

// Main entry point
async function main(): Promise<void> {
  // Process incoming requests
  for await (const line of readLines()) {
    try {
      const request = JSON.parse(line) as RPCRequest;
      await handleRequest(request);
    } catch (error) {
      // JSON parse error
      const response: RPCResponse = {
        jsonrpc: "2.0",
        id: 0,
        error: {
          code: -32700,
          message: `Parse error: ${error instanceof Error ? error.message : String(error)}`,
        },
      };
      writeResponse(response);
    }
  }
}

main().catch(console.error);
