import type { Client } from "./generated/client";
import type { CreateClientConfig } from "./generated/client.gen";
import { ProblemError, parseProblem } from "./problem";

export const createClientConfig: CreateClientConfig = (config) => ({
  ...config,
  baseUrl: "/api",
  credentials: "include",
});

let installed = false;

/**
 * Turn non-2xx Problem Details into [`ProblemError`] so forms can map
 * `fieldErrors` onto react-hook-form.
 */
export function installProblemInterceptor(client: Client): void {
  if (installed) {
    return;
  }
  installed = true;

  client.interceptors.response.use(async (response) => {
    if (response.ok) {
      return response;
    }
    const text = await response.clone().text();
    let json: unknown;
    try {
      json = JSON.parse(text);
    } catch {
      return response;
    }
    const problem = parseProblem(json);
    if (problem) {
      throw problem;
    }
    return response;
  });

  client.interceptors.error.use((error) => {
    if (error instanceof ProblemError) {
      return error;
    }
    return parseProblem(error) ?? error;
  });
}
