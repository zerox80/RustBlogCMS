const ALLOWED_PROTOCOLS = new Set(['http:', 'https:', 'mailto:', 'tel:'])
const hasProtocol = (value) => /^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(value)
/**
 * Security-focused URL Sanitizer.
 * 
 * Prevents:
 * - Protocol-relative hijacking (//evil.com).
 * - XSS via `javascript:` protocols.
 * - Unauthorized protocols (only http, https, mailto, tel allowed).
 */
export const sanitizeExternalUrl = (value) => {
  // Input validation
  if (typeof value !== 'string') {
    return null
  }
  const trimmed = value.trim()
  if (!trimmed) {
    return null
  }
  // Block protocol-relative URLs for security (e.g., //example.com)
  if (trimmed.startsWith('//')) {
    return null
  }
  // If no protocol, treat as relative path (considered safe)
  if (!hasProtocol(trimmed)) {
    return trimmed
  }
  // Parse and validate URLs with protocols
  try {
    const parsed = new URL(trimmed)
    // Validate protocol against allowed whitelist
    return ALLOWED_PROTOCOLS.has(parsed.protocol.toLowerCase()) ? parsed.toString() : null
  } catch (error) {
    // Log malformed URLs for debugging
    console.warn('Failed to parse external URL:', error)
    return null
  }
}
export const isSafeExternalUrl = (value) => {
  return sanitizeExternalUrl(value) !== null
}