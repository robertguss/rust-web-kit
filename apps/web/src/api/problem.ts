/** RFC 9457 field errors: field name → messages. */
export type FieldErrors = Record<string, string[]>;

/** RFC 9457 Problem Details body as returned by the API. */
export type ProblemDetails = {
  type: string;
  title: string;
  status: number;
  detail: string;
  instance?: string;
  errors?: FieldErrors;
};

/** Typed error wrapping Problem Details, including per-field messages. */
export class ProblemError extends Error {
  readonly type: string;
  readonly title: string;
  readonly status: number;
  readonly detail: string;
  readonly instance?: string;
  readonly fieldErrors: FieldErrors;

  constructor(problem: ProblemDetails) {
    super(problem.detail || problem.title);
    this.name = "ProblemError";
    this.type = problem.type;
    this.title = problem.title;
    this.status = problem.status;
    this.detail = problem.detail;
    this.instance = problem.instance;
    this.fieldErrors = problem.errors ?? {};
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/** Narrow an unknown JSON body to Problem Details. */
export function isProblemDetails(value: unknown): value is ProblemDetails {
  if (!isRecord(value)) {
    return false;
  }
  return (
    typeof value.type === "string" &&
    typeof value.title === "string" &&
    typeof value.status === "number" &&
    typeof value.detail === "string"
  );
}

/** Parse a JSON body into a [`ProblemError`], or `null` if it is not Problem Details. */
export function parseProblem(value: unknown): ProblemError | null {
  if (!isProblemDetails(value)) {
    return null;
  }
  return new ProblemError(value);
}
