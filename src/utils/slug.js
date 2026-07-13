export const sanitizeSlug = (value) => {
  if (!value) return ''
  // Unicode normalization to separate base characters from diacritics
  const withoutDiacritics = value.normalize('NFKD').replace(/[̀-ͯ]/g, '')
  // Character filtering, case conversion, and cleanup
  const cleaned = withoutDiacritics
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-') // Replace any sequence of non-alphanumeric chars with single hyphen
    .replace(/-+/g, '-') // Collapse multiple consecutive hyphens into single hyphen
    .replace(/^-|-$/g, '') // Remove hyphens from start and end of string
  // Apply length limit for URL compatibility
  const MAX_LENGTH = 100
  const truncated = cleaned.slice(0, MAX_LENGTH)
  return truncated
}
const SLUG_REGEX = /^[a-z0-9]+(?:-[a-z0-9]+)*$/
export const isValidSlug = (value) => SLUG_REGEX.test(value)
