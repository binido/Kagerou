/** Tauri rejects `invoke` with whatever the command's `Err` serialises to —
 * for our `Result<_, String>` commands that is a bare string, not an `Error`.
 * Checking only `instanceof Error` therefore discards every backend message
 * we have, which is how "502 Bad Gateway" reaches the user as "failed". */
export const backendErrorMessage = (error: unknown, fallback: string): string => {
  if (typeof error === 'string' && error.trim()) return error
  if (error instanceof Error && error.message.trim()) return error.message
  return fallback
}
