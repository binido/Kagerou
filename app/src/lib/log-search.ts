/** The log viewer highlights the search query inside each message, which means
 * turning user input into a RegExp. Every metacharacter has to be escaped
 * first: an unescaped `(` or `[` makes the RegExp constructor throw, and the
 * throw happens during render, where there is nothing to catch it. */
const METACHARACTERS = /[.*+?^${}()|[\]\\]/g

/** Query to a case-insensitive capturing pattern for `String.split`, or null
 * when there is nothing to highlight. */
export const highlightPattern = (query: string): RegExp | null => {
  const needle = query.trim()
  return needle ? new RegExp(`(${needle.replace(METACHARACTERS, '\\$&')})`, 'ig') : null
}
