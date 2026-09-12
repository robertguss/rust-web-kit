import type { FieldValues, Path, UseFormSetError } from "react-hook-form";

import { type FieldErrors, ProblemError } from "./problem";

/** Map RFC 9457 `errors` onto react-hook-form fields. */
export function applyFieldErrors<T extends FieldValues>(
  fieldErrors: FieldErrors,
  setError: UseFormSetError<T>,
): void {
  for (const [name, messages] of Object.entries(fieldErrors)) {
    const message = messages[0];
    if (!message) {
      continue;
    }
    setError(name as Path<T>, { type: "server", message });
  }
}

/**
 * Apply a thrown API error to the form. Field errors win; otherwise `root`.
 * Returns true when the error was a [`ProblemError`].
 */
export function applyProblem<T extends FieldValues>(
  error: unknown,
  setError: UseFormSetError<T>,
): boolean {
  if (error instanceof ProblemError) {
    if (Object.keys(error.fieldErrors).length > 0) {
      applyFieldErrors(error.fieldErrors, setError);
    } else {
      setError("root", { type: "server", message: error.detail });
    }
    return true;
  }
  const message =
    error instanceof Error ? error.message : "Something went wrong.";
  setError("root", { type: "server", message });
  return false;
}
