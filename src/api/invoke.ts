import {invoke} from "@tauri-apps/api/core";

export interface ApiSuccess<T = unknown> {
  ok: true;
  data: T;
}

export interface ApiFailure {
  ok: false;
  error: string;
}

export type ApiResponse<T = unknown> = ApiSuccess<T> | ApiFailure;

/**
 * Безопасный вызов Rust-команд через Tauri `invoke`.
 * Гарантирует единый формат ответа `{ ok, data | error }`.
 */
export async function safeInvoke<TResult = unknown>(
  command: string,
  args?: Record<string, unknown>
): Promise<ApiResponse<TResult>> {
  try {
    // Пытаемся вызвать команду
    const res = await invoke<ApiResponse<TResult> | TResult>(command, args);

    // 1) Backend уже вернул объект формата ApiResponse
    if (typeof res === "object" && res !== null && "ok" in res) {
      return res as ApiResponse<TResult>;
    }

    // 2) Backend вернул «сырые» данные — оборачиваем сами
    return { ok: true, data: res as TResult };
  } catch (err) {
    // 3) Исключение JS-слоя → приводим к единому виду
    const message = err instanceof Error ? err.message : String(err);
    return { ok: false, error: message };
  }
}
