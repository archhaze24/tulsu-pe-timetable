import { safeInvoke } from "./invoke";
import type { ApiResponse } from "./types/common";
import {GreetRequest, GreetResult} from "./types/greet.ts";

export default async function greet(
  req: GreetRequest
): Promise<ApiResponse<GreetResult>> {
  return safeInvoke("greet", req);
}
