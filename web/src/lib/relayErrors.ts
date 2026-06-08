/**
 * Extracts a human-readable message from an error thrown by a Relay mutation's
 * `onError` callback (or any rejected promise).
 *
 * Our GraphQL network layer returns the raw `{ data, errors }` payload, so when
 * the server reports a GraphQL error Relay surfaces a network error that carries
 * the original response (with its `errors` array) on `.source`. We prefer those
 * server-provided messages, falling back to the Error's own message.
 */
export function getErrorMessage(err: unknown): string {
  if (err == null) return "Unknown error";

  // Relay attaches the raw GraphQL response to network errors as `.source`.
  const source = (
    err as {
      source?: { errors?: ReadonlyArray<{ message?: string | null }> };
    }
  ).source;
  const gqlMessages = source?.errors
    ?.map((e) => e?.message)
    .filter((m): m is string => Boolean(m));
  if (gqlMessages && gqlMessages.length > 0) {
    return gqlMessages.join("; ");
  }

  if (err instanceof Error && err.message) return err.message;
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
