import { describe, expect, it } from "vitest";

import { applyFieldErrors, applyProblem } from "./form-errors";
import { isProblemDetails, ProblemError, parseProblem } from "./problem";

describe("parseProblem", () => {
  it("wraps RFC 9457 bodies including field errors", () => {
    const error = parseProblem({
      type: "about:blank",
      title: "Unprocessable Entity",
      status: 422,
      detail: "One or more fields failed validation.",
      errors: { email: ["not a valid email"] },
    });
    expect(error).toBeInstanceOf(ProblemError);
    expect(error?.fieldErrors.email?.[0]).toBe("not a valid email");
  });

  it("rejects non-problem JSON", () => {
    expect(isProblemDetails({ error: "nope" })).toBe(false);
    expect(parseProblem("plain")).toBeNull();
  });
});

describe("applyProblem", () => {
  it("maps fieldErrors onto setError", () => {
    const calls: Array<{ name: string; message: string }> = [];
    const setError = (name: string, error: { message?: string }) => {
      calls.push({ name, message: error.message ?? "" });
    };
    const problem = new ProblemError({
      type: "about:blank",
      title: "Unprocessable Entity",
      status: 422,
      detail: "One or more fields failed validation.",
      errors: { email: ["invalid"], password: ["too short"] },
    });
    expect(applyProblem(problem, setError)).toBe(true);
    expect(calls).toEqual([
      { name: "email", message: "invalid" },
      { name: "password", message: "too short" },
    ]);
  });

  it("uses root when there are no field errors", () => {
    const calls: Array<{ name: string; message: string }> = [];
    const setError = (name: string, error: { message?: string }) => {
      calls.push({ name, message: error.message ?? "" });
    };
    applyFieldErrors({}, setError);
    expect(calls).toEqual([]);
    const problem = new ProblemError({
      type: "about:blank",
      title: "Unauthorized",
      status: 401,
      detail: "Authentication is required.",
    });
    applyProblem(problem, setError);
    expect(calls).toEqual([
      { name: "root", message: "Authentication is required." },
    ]);
  });
});
