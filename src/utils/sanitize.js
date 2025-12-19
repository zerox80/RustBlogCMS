import DOMPurify from 'dompurify';

/**
 * HTML Sanitization Utilities.
 * 
 * Wraps DOMPurify to provide strict, configured sanitization for HTML content.
 * Prevents XSS attacks by stripping dangerous tags and attributes.
 */
const DEFAULT_CONFIG = {
  ALLOWED_TAGS: [
    'p', 'br', 'strong', 'em', 'u', 's', 'a', 'ul', 'ol', 'li',
    'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
    'blockquote', 'code', 'pre',
    'table', 'thead', 'tbody', 'tr', 'th', 'td',
    'img', 'hr', 'div', 'span'
  ],
  ALLOWED_ATTR: [
    'href', 'target', 'rel', 'class', 'id',
    'src', 'alt', 'title', 'width', 'height'
  ],
  ALLOW_DATA_ATTR: false,
  ALLOW_UNKNOWN_PROTOCOLS: false,
};
export const sanitizeHTML = (dirty, config = {}) => {
  if (typeof dirty !== 'string') return '';
  const mergedConfig = { ...DEFAULT_CONFIG, ...config };
  return DOMPurify.sanitize(dirty, mergedConfig);
};
export const sanitizeText = (input) => {
  if (typeof input !== 'string') return '';
  return DOMPurify.sanitize(input, { ALLOWED_TAGS: [] });
};
export const sanitizeURL = (url) => {
  if (typeof url !== 'string') return null;
  try {
    const parsed = new URL(url);
    const allowed = ['http:', 'https:', 'mailto:', 'tel:'];
    if (!allowed.includes(parsed.protocol)) {
      return null;
    }
    return parsed.href;
  } catch {
    return null;
  }
};
export default {
  sanitizeHTML,
  sanitizeText,
  sanitizeURL,
};