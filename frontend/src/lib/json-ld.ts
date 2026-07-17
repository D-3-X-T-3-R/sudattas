const LINE_SEPARATOR_RE = new RegExp("\\u2028", "g");
const PARAGRAPH_SEPARATOR_RE = new RegExp("\\u2029", "g");

/**
 * JSON.stringify does not escape `<`, so a value containing `</script>` can terminate the
 * enclosing <script type="application/ld+json"> tag early and inject markup/script. Escape the
 * characters an HTML parser treats specially inside a script body, plus the JS line terminators
 * that are illegal in a JSON string literal per spec but accepted by some parsers.
 */
export function safeJsonLd(value: unknown): string {
  return JSON.stringify(value)
    .replace(/</g, "\\u003c")
    .replace(/>/g, "\\u003e")
    .replace(/&/g, "\\u0026")
    .replace(LINE_SEPARATOR_RE, "\\u2028")
    .replace(PARAGRAPH_SEPARATOR_RE, "\\u2029");
}
