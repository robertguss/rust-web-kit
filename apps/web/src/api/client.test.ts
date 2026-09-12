import { afterEach, describe, expect, it, vi } from "vitest";

import { installProblemInterceptor } from "./client";
import { client } from "./generated/client.gen";
import { login } from "./generated/sdk.gen";
import { ProblemError } from "./problem";

installProblemInterceptor(client);

describe("problem interceptor", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("throws ProblemError for non-2xx problem+json", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        return new Response(
          JSON.stringify({
            type: "about:blank",
            title: "Unprocessable Entity",
            status: 422,
            detail: "One or more fields failed validation.",
            errors: { email: ["not a valid email"] },
          }),
          {
            status: 422,
            headers: { "content-type": "application/problem+json" },
          },
        );
      }),
    );

    try {
      await login({
        body: { email: "bad", password: "password12" },
        throwOnError: true,
      });
      expect.unreachable();
    } catch (error) {
      expect(error).toBeInstanceOf(ProblemError);
      const problem = error as ProblemError;
      expect(problem.status).toBe(422);
      expect(problem.fieldErrors.email?.[0]).toBe("not a valid email");
    }
  });
});
