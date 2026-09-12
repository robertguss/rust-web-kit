const DEFAULT_RETURN_TO = "/app/projects";

/** Only in-app relative paths (blocks open redirects). */
export function safeReturnTo(value: unknown): string {
  if (typeof value !== "string") {
    return DEFAULT_RETURN_TO;
  }
  if (!value.startsWith("/app")) {
    return DEFAULT_RETURN_TO;
  }
  if (value.startsWith("//") || value.includes("://")) {
    return DEFAULT_RETURN_TO;
  }
  return value;
}
