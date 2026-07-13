/**
 * Mojibake Encoding Repair Map.
 *
 * Maps common UTF-8 interpreted as ISO-8859-1 character sequences back to
 * their original German and Latin-1 characters. This is a critical fallback
 * for data that might have been mangled during database migrations or
 * legacy API interactions.
 */
const MOJIBAKE_REPLACEMENTS = [
  ['Ã¤', 'ä'], // ä - German umlaut a
  ['Ã', 'Ä'], // Ä - German umlaut A (uppercase)
  ['Ã¶', 'ö'], // ö - German umlaut o
  ['Ã', 'Ö'], // Ö - German umlaut O (uppercase)
  ['Ã¼', 'ü'], // ü - German umlaut u
  ['Ã', 'Ü'], // Ü - German umlaut U (uppercase)
  ['Ã', 'ß'], // ß - German eszett (sharp s)
  ['Ã¡', 'á'], // á - Latin small letter a with acute
  ['Ã ', 'à'], // à - Latin small letter a with grave
  ['Ã©', 'é'], // é - Latin small letter e with acute
  ['Ã¨', 'è'], // è - Latin small letter e with grave
  ['Ãº', 'ú'], // ú - Latin small letter u with acute
  ['Ã¹', 'ù'], // ù - Latin small letter u with grave
  ['Ã³', 'ó'], // ó - Latin small letter o with acute
  ['Ã²', 'ò'], // ò - Latin small letter o with grave
  ['Ã±', 'ñ'], // ñ - Latin small letter n with tilde
  ['Ã§', 'ç'], // ç - Latin small letter c with cedilla
  ['Ã¥', 'å'], // å - Latin small letter a with ring above
  ['Ã', 'Ø'], // Ø - Latin capital letter O with stroke
  ['Ã¸', 'ø'], // ø - Latin small letter o with stroke
  ['–', '–'], // – - En dash
  ['—', '—'], // — - Em dash
  ['„', '„'], // „ - Double low-9 quotation mark
  ['"', '"'], // " - Left double quotation mark
  ['"', '"'], // " - Right double quotation mark
  ["'", "'"], // ' - Left single quotation mark
  ["'", "'"], // ' - Right single quotation mark
  ['…', '…'], // … - Horizontal ellipsis
  ['•', '•'], // • - Bullet
  [' ', ' '], // Regular space (non-breaking space fix)
  ['Â', ''], // Remove stray Â characters
  ['�', ''], // Remove replacement characters
]
/**
 * Fixes character encoding issues (Mojibake) in a string.
 *
 * Performance: Includes a pre-flight regex check to bypass clean strings
 * and avoid unnecessary iterations.
 */
const fixMojibake = (value) => {
  if (typeof value !== 'string' || value.length === 0) {
    return value
  }
  // Quick detection to avoid processing clean text
  if (!/[Ã¤ÃÃ¶ÃÃ¼ÃÃâ]/.test(value)) {
    return value
  }
  let result = value
  for (const [search, replacement] of MOJIBAKE_REPLACEMENTS) {
    if (result.includes(search)) {
      result = result.split(search).join(replacement)
    }
  }
  return result
}
const normalizeStringArray = (values, joiner = ' ') => {
  const cleaned = values
    .filter(Boolean)
    .map((part) => (typeof part === 'string' ? fixMojibake(part.trim()) : null))
    .filter((part) => part && part.length)
  return cleaned.join(joiner)
}
export const normalizeTitle = (title, fallback) => {
  if (!title) return fallback
  if (typeof title === 'string') return fixMojibake(title)
  if (Array.isArray(title)) {
    const joined = normalizeStringArray(title)
    if (joined) {
      return joined
    }
  }
  if (typeof title === 'object') {
    const values = Object.values(title).filter((part) => typeof part === 'string' && part.trim())
    if (values.length) {
      return normalizeStringArray(values)
    }
  }
  return fallback
}
export const normalizeText = (value, fallback = '') => {
  if (!value) return fallback
  if (typeof value === 'string') return fixMojibake(value)
  if (Array.isArray(value)) {
    const text = normalizeStringArray(value, '\n')
    return text || fallback
  }
  if (typeof value === 'object') {
    if (typeof value.text === 'string' && value.text.trim()) {
      return fixMojibake(value.text)
    }
    const values = Object.values(value).filter((part) => typeof part === 'string' && part.trim())
    if (values.length) {
      return normalizeStringArray(values, '\n')
    }
  }
  return fallback
}
export const formatDate = (isoString, locale = 'de-DE') => {
  if (!isoString) return null
  const date = new Date(isoString)
  if (Number.isNaN(date.getTime())) return null
  return new Intl.DateTimeFormat(locale, {
    day: '2-digit',
    month: 'long',
    year: 'numeric',
  }).format(date)
}
export const buildPreviewText = (post, maxLength = 240, minCutoff = 180) => {
  const excerpt = normalizeText(post?.excerpt)
  const fallback = normalizeText(post?.content_markdown)
  const source = (excerpt || fallback || '').replace(/\s+/g, ' ').trim()
  if (!source) {
    return ''
  }
  if (source.length <= maxLength) {
    return source
  }
  const truncated = source.slice(0, maxLength)
  const lastSpace = truncated.lastIndexOf(' ')
  const safeCut = lastSpace > minCutoff ? truncated.slice(0, lastSpace) : truncated
  return `${safeCut.trim()}.`
}
/**
 * Generates a clean, URL-safe and SEO-friendly slug.
 *
 * Algorithm:
 * 1. Normalize Unicode (NFKD) to decompose combined characters (e.g., 'ü' -> 'u' + '..').
 * 2. Strip diacritics using regex.
 * 3. Filter for alphanumeric characters and spaces.
 * 4. Collapse consecutive separators.
 * 5. Enforce safety limits (length, reserved names).
 */
export const normalizeSlug = (value) => {
  if (typeof value !== 'string') {
    return ''
  }
  const trimmed = value.trim()
  if (!trimmed) {
    return ''
  }
  // Remove diacritics and normalize Unicode
  const ascii = trimmed.normalize('NFKD').replace(/[\u0300-\u036f]/g, '') // drop diacritics
  // Clean and sanitize the slug
  const sanitized = ascii
    .replace(/[^0-9A-Za-z\s-]/g, '')
    .trim()
    .replace(/[-_\s]+/g, '-') // collapse separators
    .replace(/^-+|-+$/g, '')
    .toLowerCase()
  // Security filtering for dangerous patterns
  if (!sanitized || sanitized === '.' || sanitized === '..') {
    return ''
  }
  return sanitized.slice(0, 128)
}
