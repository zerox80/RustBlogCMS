import React, { useState, useEffect } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Menu, X, Github } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useContent } from '../../context/ContentContext'
import { useEdit } from '../../context/EditContext'
import { useAuth } from '../../context/AuthContext'
import EditableText from '../cms/EditableText'
import { getIconComponent } from '../../utils/iconMap'

/**
 * The global site header with advanced scroll-aware behavior.
 * 
 * Features:
 * - Smart Visibility: Hides on scroll-down, reappears on significant scroll-up.
 * - Glassmorphism: Smoothly transitions from transparent to blurred slate background.
 * - CMS Integration: Loads brand identity and navigation items from `ContentContext`.
 * - Admin Controls: Provides an inline toggle for CMS "Edit Mode".
 */
const Header = () => {
    const { t } = useTranslation()
    const { navigation, getSection } = useContent()
    const { isEditing, toggleEditMode } = useEdit()
    const { isAuthenticated, user } = useAuth()
    const headerContent = getSection('header') ?? {}

    // Resolve dynamic brand icon
    const BrandIcon = React.useMemo(
        () => getIconComponent(headerContent?.brand?.icon, 'Terminal'),
        [headerContent?.brand?.icon]
    )
    const [isScrolled, setIsScrolled] = useState(false)
    const [isVisible, setIsVisible] = useState(true)
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)

    // Use refs for scroll tracking to avoid re-renders/dependency loops
    const lastScrollY = React.useRef(0)
    const scrollUpAccumulator = React.useRef(0)
    const SCROLL_UP_THRESHOLD = 200 // Pixels to scroll up before showing header

    const location = useLocation()

    const scrollTimeout = React.useRef(null)

    useEffect(() => {
        /**
         * State-of-the-art scroll handler that minimizes layout thrashing.
         * 
         * Implements "Intentional Scroll-Up": 
         * To prevent the header from flickering on accidental micro-adjustments, 
         * the user must scroll up by at least `SCROLL_UP_THRESHOLD` (200px) 
         * before the header slides back into view.
         */
        const handleScroll = () => {
            const currentScrollY = window.scrollY
            const lastY = lastScrollY.current
            const scrollDelta = lastY - currentScrollY

            if (scrollTimeout.current) {
                clearTimeout(scrollTimeout.current)
            }

            // Depth Check: Enable styling when not at the very top
            setIsScrolled(currentScrollY > 20)

            if (currentScrollY < 10) {
                // Top of Page: Always show the header
                setIsVisible(true)
                scrollUpAccumulator.current = 0
            } else if (currentScrollY > lastY) {
                // Direction: DOWN - Immediately hide for focus
                setIsVisible(false)
                scrollUpAccumulator.current = 0
            } else if (currentScrollY < lastY) {
                // Direction: UP - Accumulate delta for intentionality
                if (scrollDelta > 5) {
                    scrollUpAccumulator.current += scrollDelta
                }

                if (scrollUpAccumulator.current > SCROLL_UP_THRESHOLD) {
                    setIsVisible(true)
                }

                // Debounce directional change: Reset accumulator if scroll stops
                scrollTimeout.current = setTimeout(() => {
                    scrollUpAccumulator.current = 0
                }, 150)
            }

            lastScrollY.current = currentScrollY
        }

        window.addEventListener('scroll', handleScroll, { passive: true })
        return () => {
            window.removeEventListener('scroll', handleScroll)
            if (scrollTimeout.current) clearTimeout(scrollTimeout.current)
        }
    }, [])

    // Use centralized navigation data from ContentContext (CMS + Dynamic Pages)
    // This allows the user to edit ALL menu items via the Admin Panel
    const navLinks = (navigation?.items || []).map(item => {
        let path = item.path || '/'

        // Handle anchor/section links
        if (item.type === 'section') {
            const sectionId = item.value || item.id
            path = `/#${sectionId}`
        }

        return {
            name: item.label, // Translation can be handled here if keys are used in CMS
            path: path,
            isActive: location.pathname === path
        }
    })

    return (
        <header
            className={`
                fixed top-0 left-0 right-0 z-50 transition-all duration-500 flex justify-center px-4
                ${isScrolled ? 'pt-4' : 'pt-6'}
                ${isVisible ? 'translate-y-0 opacity-100' : '-translate-y-full opacity-0 pointer-events-none'}
            `}
        >
            <nav
                className={`
                    w-full max-w-5xl rounded-full transition-all duration-300
                    flex items-center justify-between px-6 py-3
                    ${isScrolled
                        ? 'bg-slate-900/80 backdrop-blur-xl border border-white/10 shadow-lg shadow-black/20'
                        : 'bg-transparent border border-transparent'
                    }
                `}
            >
                {/* Logo */}
                <Link to="/" className="flex items-center gap-2 group">
                    <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-neon-cyan to-neon-violet flex items-center justify-center text-white font-bold text-lg shadow-lg shadow-neon-cyan/20 group-hover:shadow-neon-cyan/40 transition-shadow">
                        <BrandIcon className="w-5 h-5" />
                    </div>
                    <span className="font-bold text-lg text-white tracking-tight group-hover:text-neon-cyan transition-colors hidden sm:block">
                        <EditableText section="header" field="brand.name" value={headerContent?.brand?.name || 'Zero Point'} />
                    </span>
                </Link>

                {/* Desktop Navigation */}
                <div className="hidden md:flex items-center gap-8">
                    {navLinks.map((link) => (
                        <Link
                            key={link.name}
                            to={link.path}
                            className="text-sm font-medium text-slate-300 hover:text-white transition-colors relative group"
                        >
                            {link.name}
                            <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-neon-cyan transition-all duration-300 group-hover:w-full"></span>
                        </Link>
                    ))}
                </div>

                {/* Action Buttons */}
                <div className="hidden md:flex items-center gap-4">

                    {/* [NEW] Edit Mode Toggle (only for admins) */}
                    {isAuthenticated && (
                        <button
                            onClick={toggleEditMode}
                            className={`px-3 py-1 rounded-full text-xs font-semibold border transition-all ${isEditing
                                ? 'bg-neon-cyan/20 border-neon-cyan text-neon-cyan shadow-[0_0_10px_rgba(6,182,212,0.3)]'
                                : 'bg-white/5 border-white/10 text-slate-400 hover:text-white'
                                }`}
                        >
                            {isEditing ? 'Editing On' : 'Edit Mode'}
                        </button>
                    )}

                    <a
                        href="https://github.com/zerox80/LinuxTutorialCMS"
                        target="_blank"
                        rel="noreferrer"
                        className="text-slate-400 hover:text-white transition-colors"
                    >
                        <Github className="w-5 h-5" />
                    </a>
                    <Link
                        to="/login"
                        className="px-5 py-2 rounded-full bg-white/10 hover:bg-white/20 text-white text-sm font-semibold transition-all border border-white/5"
                    >
                        {headerContent?.cta?.guestLabel || t('nav.login')}
                    </Link>
                </div>

                {/* Mobile Menu Button */}
                <button
                    className="md:hidden text-white p-2"
                    onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                >
                    {isMobileMenuOpen ? <X /> : <Menu />}
                </button>
            </nav>

            {/* Mobile Menu Overlay */}
            {
                isMobileMenuOpen && (
                    <div className="absolute top-24 left-4 right-4 bg-slate-900/95 backdrop-blur-xl rounded-2xl border border-white/10 p-6 flex flex-col gap-4 shadow-xl animate-fade-in md:hidden">
                        {navLinks.map((link) => (
                            <Link
                                key={link.name}
                                to={link.path}
                                className="text-lg font-medium text-slate-200 py-2 border-b border-white/5 last:border-0"
                                onClick={() => setIsMobileMenuOpen(false)}
                            >
                                {link.name}
                            </Link>
                        ))}
                        <div className="flex gap-4 mt-4 pt-4 border-t border-white/10">
                            <Link to="/login" className="flex-1 btn-primary text-center py-2">Sign In</Link>
                        </div>
                    </div>
                )
            }
        </header >
    )
}

export default Header
