import { safeInvoke } from "./invoke";
import type { CountRequest, CountResult } from "./types/count";
import type { ApiResponse } from "./types/common";

export default async function count(
  req: CountRequest
): Promise<ApiResponse<CountResult>> {
  return safeInvoke("count", req);
}
