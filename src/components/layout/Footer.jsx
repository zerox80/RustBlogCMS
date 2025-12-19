import { useCallback, useMemo } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { Heart } from 'lucide-react'
import { useContent } from '../../context/ContentContext'
import { navigateContentTarget } from '../../utils/contentNavigation'
import { getIconComponent } from '../../utils/iconMap'
import { sanitizeExternalUrl } from '../../utils/urlValidation'

/**
 * Smart icon resolver for contact links.
 * 
 * Attempts to guess the appropriate Lucide icon based on:
 * 1. Explicitly defined icon in CMS
 * 2. URL protocol (mailto:, tel:)
 * 3. Domain detection (github.com)
 */
const resolveContactFallbackIcon = (contact) => {
    if (!contact) {
        return 'Terminal'
    }

    if (contact.icon) {
        return contact.icon
    }

    const href = typeof contact.href === 'string' ? contact.href : contact.url
    if (typeof href === 'string') {
        const value = href.toLowerCase()
        if (value.startsWith('mailto:')) {
            return 'Mail'
        }
        if (value.startsWith('tel:')) {
            return 'Phone'
        }
        if (value.includes('github.com')) {
            return 'Github'
        }
    }

    if (typeof contact.type === 'string') {
        const type = contact.type.toLowerCase()
        if (type === 'email') {
            return 'Mail'
        }
        if (type === 'phone') {
            return 'Phone'
        }
        if (type === 'github') {
            return 'Github'
        }
    }

    return 'Terminal'
}

import EditableText from '../cms/EditableText'

const Footer = () => {
    const { getSection, navigation } = useContent()
    const footerContent = getSection('footer') ?? {}
    const currentYear = new Date().getFullYear()
    const navigate = useNavigate()
    const location = useLocation()

    const BrandIcon = useMemo(
        () => getIconComponent(footerContent?.brand?.icon, 'Terminal'),
        [footerContent?.brand?.icon],
    )

    const contactLinks = Array.isArray(footerContent?.contactLinks) ? footerContent.contactLinks : []

    const staticNavigationItems = useMemo(
        () => (Array.isArray(navigation?.static) ? navigation.static : []),
        [navigation?.static],
    )

    const dynamicNavigationItems = useMemo(
        () => (Array.isArray(navigation?.dynamic) ? navigation.dynamic : []),
        [navigation?.dynamic],
    )

    const effectiveNavigationItems = useMemo(() => {
        const combined = [...staticNavigationItems, ...dynamicNavigationItems]
        if (combined.length > 0) {
            return combined
        }
        const allItems = Array.isArray(navigation?.items) ? navigation.items : []
        return allItems
    }, [staticNavigationItems, dynamicNavigationItems, navigation?.items])

    /**
     * Normalizes various link target types into a standard browser href.
     * 
     * Handles:
     * - `section`: Prepends # for anchor navigation.
     * - `route`/`page`: Internal SPA routing paths.
     * - `external`: Validates and sanitizes third-party URLs.
     */
    const buildTargetHref = useCallback((target) => {
        if (!target || typeof target !== 'object') {
            return null
        }
        const value = typeof target.value === 'string' ? target.value : target.path || target.href || ''
        switch (target.type) {
            case 'section':
                if (value) {
                    return `#${value.replace(/^#/, '')}`
                }
                return '#'
            case 'route':
            case 'page':
                return value || '/'
            case 'external':
            case 'href':
                return sanitizeExternalUrl(value) || null
            default:
                return null
        }
    }, [])

    const quickLinks = useMemo(() => {
        const contentLinks = Array.isArray(footerContent?.quickLinks) ? footerContent.quickLinks : []
        const normalizedContentLinks = contentLinks
            .map((link, index) => {
                if (!link) return null
                if (link.target) {
                    const href = buildTargetHref(link.target)
                    return {
                        label: link.label || link.target?.value || 'Link',
                        target: link.target,
                        href,
                        field: `quickLinks.${index}.label`
                    }
                }
                if (link.href) {
                    const safeHref = sanitizeExternalUrl(link.href)
                    if (!safeHref) {
                        console.warn('Blocked unsafe footer quick link:', link.href)
                        return null
                    }
                    return {
                        label: link.label || link.href,
                        href: safeHref,
                        field: `quickLinks.${index}.label`
                    }
                }
                if (link.path) {
                    return {
                        label: link.label || link.path,
                        target: { type: 'route', value: link.path },
                        href: link.path,
                        field: `quickLinks.${index}.label`
                    }
                }
                if (link.slug) {
                    const slug = link.slug.trim().replace(/^\//, '')
                    if (!slug) return null
                    return {
                        label: link.label || slug,
                        target: { type: 'route', value: `/pages/${slug}` },
                        href: `/pages/${slug}`,
                        field: `quickLinks.${index}.label`
                    }
                }
                return null
            })
            .filter(Boolean)

        if (normalizedContentLinks.length > 0) {
            return normalizedContentLinks
        }

        return effectiveNavigationItems
            .map((item) => {
                if (!item) return null
                if (item.target) {
                    const href = buildTargetHref(item.target)
                    return {
                        label: item.label || item.slug || 'Link',
                        target: item.target,
                        href,
                    }
                }
                if (item.href) {
                    const safeHref = sanitizeExternalUrl(item.href)
                    if (!safeHref) {
                        console.warn('Blocked unsafe navigation href in footer:', item.href)
                        return null
                    }
                    return {
                        label: item.label || item.slug || item.href,
                        href: safeHref,
                    }
                }
                if (item.type === 'route' && item.path) {
                    return {
                        label: item.label || item.slug || item.path,
                        target: { type: 'route', value: item.path },
                        href: item.path,
                    }
                }
                if (item.type === 'section') {
                    const sectionValue = item.path || item.value || item.id
                    if (!sectionValue) return null
                    return {
                        label: item.label || 'Link',
                        target: { type: 'section', value: sectionValue },
                        href: `#${sectionValue.replace(/^#/, '')}`,
                    }
                }
                if (item.slug) {
                    return {
                        label: item.label || item.slug,
                        target: { type: 'route', value: `/pages/${item.slug}` },
                        href: `/pages/${item.slug}`,
                    }
                }
                return null
            })
            .filter(Boolean)
    }, [buildTargetHref, effectiveNavigationItems, footerContent?.quickLinks])

    /**
     * Centralized click handler for footer links.
     * 
     * Leverages `navigateContentTarget` for SPA-friendly section/route jumps
     * and falls back to standard anchor behavior for external/protocol links.
     */
    const handleQuickLink = (event, link) => {
        if (!link) return

        const target = link.target

        if (target) {
            event?.preventDefault?.()
            navigateContentTarget(target, { navigate, location })
            return
        }

        const href = sanitizeExternalUrl(link.href || link.url)
        if (href) {
            // Check for protocols that should NOT be intercepted by the SPA router
            const isExternal = href.startsWith('http://') || href.startsWith('https://')
            const isSpecialProtocol = href.startsWith('mailto:') || href.startsWith('tel:')

            if (isExternal || isSpecialProtocol) {
                return
            }

            event?.preventDefault?.()
            window.location.assign(href)
            return
        }

        const path = link.path
        if (path) {
            event?.preventDefault?.()
            navigate(path)
            return
        }
    }

    return (
        <footer className="bg-gray-900 text-gray-300 py-12">
            <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
                    {/* Brand */}
                    <div>
                        <div className="flex items-center space-x-3 mb-4">
                            <div className="bg-gradient-to-r from-primary-600 to-primary-800 p-2 rounded-lg">
                                <BrandIcon className="w-6 h-6 text-white" />
                            </div>
                            <span className="text-xl font-bold text-white">
                                <EditableText
                                    section="footer"
                                    field="brand.name"
                                    value={footerContent?.brand?.title || footerContent?.brand?.name || 'IT Portal'}
                                />
                            </span>
                        </div>
                        <p className="text-gray-400">
                            <EditableText
                                section="footer"
                                field="brand.description"
                                value={footerContent?.brand?.description || 'Dein Portal für IT Security, Programmierung und Administration.'}
                                multiline
                            />
                        </p>
                    </div>

                    {/* Quick Links */}
                    <div>
                        <h4 className="text-white font-semibold mb-4">Quick Links</h4>
                        <ul className="space-y-2">
                            {quickLinks.length > 0 ? (
                                quickLinks.map((link, index) => {
                                    const href = link.href || '#'
                                    const isExternal = typeof href === 'string' && (href.startsWith('http://') || href.startsWith('https://'))
                                    return (
                                        <li key={link.label || link.target?.value || `quick-${index}`}>
                                            <a
                                                href={href}
                                                onClick={(event) => handleQuickLink(event, link)}
                                                className="hover:text-primary-400 transition-colors duration-200"
                                                target={isExternal ? '_blank' : undefined}
                                                rel={isExternal ? 'noopener noreferrer' : undefined}
                                            >
                                                {link.field ? (
                                                    <EditableText section="footer" field={link.field} value={link.label} />
                                                ) : (
                                                    link.label || 'Link'
                                                )}
                                            </a>
                                        </li>
                                    )
                                })
                            ) : (
                                <li className="text-sm text-gray-500">Noch keine Quick Links definiert.</li>
                            )}
                        </ul>
                    </div>

                    {/* Contact */}
                    <div>
                        <h4 className="text-white font-semibold mb-4">Kontakt</h4>
                        <div className="space-y-3">
                            {contactLinks.length > 0 ? (
                                contactLinks.map((contact, index) => {
                                    const safeHref = sanitizeExternalUrl(contact.href || contact.url)
                                    const ContactIcon = getIconComponent(resolveContactFallbackIcon(contact), 'Terminal')

                                    if (!safeHref) {
                                        return (
                                            <div
                                                key={contact.label || `contact-${index}`}
                                                className="flex items-center space-x-2 text-gray-500"
                                            >
                                                <ContactIcon className="w-5 h-5" />
                                                <span>
                                                    <EditableText section="footer" field={`contactLinks.${index}.label`} value={contact.label || 'Kontakt'} />
                                                </span>
                                            </div>
                                        )
                                    }

                                    const isHttp = safeHref.startsWith('http://') || safeHref.startsWith('https://')
                                    const isExternal = isHttp

                                    return (
                                        <a
                                            key={safeHref || contact.label || `contact-${index}`}
                                            href={safeHref}
                                            target={isExternal ? '_blank' : undefined}
                                            rel={isExternal ? 'noopener noreferrer' : undefined}
                                            className="flex items-center space-x-2 hover:text-primary-400 transition-colors duration-200"
                                        >
                                            <ContactIcon className="w-5 h-5" />
                                            <span>
                                                <EditableText section="footer" field={`contactLinks.${index}.label`} value={contact.label || safeHref || 'Kontakt'} />
                                            </span>
                                        </a>
                                    )
                                })
                            ) : (
                                <p className="text-sm text-gray-500">Keine Kontaktlinks verfügbar.</p>
                            )}
                        </div>
                    </div>
                </div>

                {/* Bottom Bar */}
                <div className="border-t border-gray-800 pt-8 flex flex-col md:flex-row justify-between items-center">
                    <p className="text-gray-400 text-sm mb-4 md:mb-0">
                        <EditableText
                            section="footer"
                            field="bottom.copyright"
                            value={footerContent?.bottom?.copyright || '© {year} IT Portal. Alle Rechte vorbehalten.'}
                        />
                    </p>
                    {footerContent?.bottom?.signature ? (
                        <p className="text-gray-400 text-sm text-center md:text-right">
                            <EditableText
                                section="footer"
                                field="bottom.signature"
                                value={footerContent.bottom.signature}
                            />
                        </p>
                    ) : (
                        <div className="flex items-center space-x-1 text-sm">
                            <span className="text-gray-400">Gemacht mit</span>
                            <Heart className="w-4 h-4 text-red-500 fill-red-500" />
                            <span className="text-gray-400">für IT Professionals</span>
                        </div>
                    )}
                </div>
            </div>
        </footer>
    )
}

export default Footer
