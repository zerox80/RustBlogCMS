import { describe, it, expect } from 'vitest'
import { sanitizeExternalUrl, isSafeExternalUrl } from '../urlValidation'

describe('urlValidation', () => {
    describe('sanitizeExternalUrl', () => {
        it('returns valid https URLs', () => {
            expect(sanitizeExternalUrl('https://example.com')).toBe('https://example.com/')
        })

        it('returns valid http URLs', () => {
            expect(sanitizeExternalUrl('http://example.com')).toBe('http://example.com/')
        })

        it('returns valid mailto URLs', () => {
            expect(sanitizeExternalUrl('mailto:user@example.com')).toBe('mailto:user@example.com')
        })

        it('returns valid tel URLs', () => {
            expect(sanitizeExternalUrl('tel:+1234567890')).toBe('tel:+1234567890')
        })

        it('rejects ftp URLs', () => {
            expect(sanitizeExternalUrl('ftp://example.com')).toBeNull()
        })

        it('rejects protocol-relative URLs', () => {
            expect(sanitizeExternalUrl('//example.com')).toBeNull()
        })

        it('rejects javascript: URLs', () => {
            expect(sanitizeExternalUrl('javascript:alert(1)')).toBeNull()
        })

        it('returns safe relative paths/domains without protocol', () => {
            expect(sanitizeExternalUrl('example.com')).toBe('example.com')
            expect(sanitizeExternalUrl('/foo/bar')).toBe('/foo/bar')
        })

        it('returns null for non-string inputs', () => {
            expect(sanitizeExternalUrl(123)).toBeNull()
            expect(sanitizeExternalUrl(null)).toBeNull()
        })
    })

    describe('isSafeExternalUrl', () => {
        it('returns true for safe URLs', () => {
            expect(isSafeExternalUrl('https://google.com')).toBe(true)
        })

        it('returns false for unsafe URLs', () => {
            expect(isSafeExternalUrl('javascript:alert(1)')).toBe(false)
        })
    })
})
