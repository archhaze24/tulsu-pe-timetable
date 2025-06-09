import { invoke } from "@tauri-apps/api/core";
import type { ApiResponse } from "./types/common";

// Универсальная обёртка над invoke
export async function safeInvoke<TArgs extends Record<string, unknown> | undefined, TResult>(
  command: string,
  args: TArgs
): Promise<ApiResponse<TResult>> {
  try {
    const data = await invoke<TResult>(command, args);
    return { ok: true, data };
  } catch (e) {
    return {
      ok: false,
      error: e instanceof Error ? e.message : String(e),
    };
  }
}
