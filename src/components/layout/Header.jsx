import { useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Asterisk, Menu, X } from 'lucide-react'
import { useContent } from '../../context/ContentContext'
import { useEdit } from '../../context/EditContext'
import { useAuth } from '../../context/AuthContext'
import EditableText from '../cms/EditableText'
import { sanitizeExternalUrl } from '../../utils/urlValidation'

/** Compact global navigation for the personal one-page blog. */
const Header = () => {
  const { navigation, getSection } = useContent()
  const { isEditing, toggleEditMode } = useEdit()
  const { isAuthenticated } = useAuth()
  const headerContent = getSection('header') ?? {}
  const location = useLocation()
  const isEditorialHome = location.pathname === '/' || location.pathname === '/blog'
  const [isScrolled, setIsScrolled] = useState(false)
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
  const showSurface = isScrolled || !isEditorialHome

  useEffect(() => {
    const handleScroll = () => setIsScrolled(window.scrollY > 24)
    handleScroll()
    window.addEventListener('scroll', handleScroll, { passive: true })
    return () => window.removeEventListener('scroll', handleScroll)
  }, [])

  const onePageSections = new Set(['stories', 'about'])
  const navigationLinks = Array.isArray(navigation?.static)
    ? navigation.static.filter((item) => {
        const value = item?.value ?? item?.path ?? item?.href
        if (item?.type === 'section') return onePageSections.has(value)
        return item?.type === 'external' || item?.type === 'href'
      })
    : []
  const anchorPath = (hash) => (location.pathname === '/' ? `#${hash}` : `/#${hash}`)
  const authPath = isAuthenticated ? '/admin' : '/login'
  const authLabel = isAuthenticated
    ? headerContent?.cta?.authLabel || 'Admin'
    : headerContent?.cta?.guestLabel || 'Login'

  const renderNavigationLink = (link, index, mobile = false) => {
    const value = link?.value ?? link?.path ?? link?.href
    const isSection = link?.type === 'section'
    const isExternal = link?.type === 'external' || link?.type === 'href'
    const sectionId = typeof value === 'string' ? value.trim() : ''
    const destination = isSection ? (sectionId ? anchorPath(sectionId) : null) : value
    const key = link?.id || `${link?.label || 'navigation'}-${index}`
    const className = mobile
      ? `flex items-center justify-between border-b border-[#171713]/15 py-4 font-display
text-2xl font-semibold tracking-[-0.04em] text-[#171713]`
      : `shrink-0 text-xs font-bold uppercase tracking-[0.14em] text-[#171713]/65
transition-colors hover:text-[#ff4f00]`
    const content = (
      <>
        {link?.label || 'Seite'}
        {mobile && (
          <span className="font-serif text-sm italic text-[#171713]/35">
            {String(index + 1).padStart(2, '0')}
          </span>
        )}
      </>
    )

    if (!destination || typeof destination !== 'string') return null
    if (isSection) {
      return (
        <a
          key={key}
          href={destination}
          onClick={mobile ? () => setIsMobileMenuOpen(false) : undefined}
          className={className}
        >
          {content}
        </a>
      )
    }
    if (isExternal) {
      const safeDestination = sanitizeExternalUrl(destination)
      if (!safeDestination) return null
      const opensNewTab = /^https?:\/\//i.test(safeDestination)
      return (
        <a
          key={key}
          href={safeDestination}
          target={opensNewTab ? '_blank' : undefined}
          rel={opensNewTab ? 'noopener noreferrer' : undefined}
          onClick={mobile ? () => setIsMobileMenuOpen(false) : undefined}
          className={className}
        >
          {content}
        </a>
      )
    }
    return null
  }

  return (
    <header
      className={`fixed inset-x-0 top-0 z-50 px-4 transition-all duration-300 sm:px-6 ${isScrolled ? 'py-3' : 'py-5'}`}
    >
      <nav
        className={[
          'mx-auto flex max-w-[1480px] items-center justify-between border',
          'px-4 py-3 transition-all duration-300 sm:px-5',
          showSurface
            ? 'border-[#171713]/15 bg-[#f4f1ea]/90 shadow-[0_12px_35px_rgba(23,23,19,0.08)] backdrop-blur-xl'
            : 'border-transparent bg-transparent',
        ].join(' ')}
        aria-label="Hauptnavigation"
      >
        <Link to="/" className="group flex items-center gap-3 text-[#171713]">
          <span
            className={`grid h-9 w-9 place-items-center rounded-full bg-[#171713] text-[#b9f227]
transition-transform group-hover:rotate-45`}
          >
            <Asterisk className="h-5 w-5" />
          </span>
          <span className="font-display text-base font-bold uppercase tracking-[-0.02em] sm:text-lg">
            <EditableText
              section="header"
              field="brand.name"
              value={headerContent?.brand?.name || 'minos'}
            />
          </span>
        </Link>

        <div className="mx-6 hidden min-w-0 flex-1 items-center justify-center gap-7 overflow-x-auto lg:flex">
          {navigationLinks.map((link, index) => renderNavigationLink(link, index))}
        </div>

        <div className="hidden items-center gap-3 md:flex">
          {isAuthenticated && (
            <button
              type="button"
              onClick={toggleEditMode}
              className={[
                'rounded-full border px-4 py-2 text-[10px] font-black uppercase',
                'tracking-[0.13em]',
                isEditing
                  ? 'border-[#ff4f00] bg-[#ff4f00] text-white'
                  : 'border-[#171713]/25 text-[#171713]',
              ].join(' ')}
            >
              {isEditing ? 'Editing on' : 'Edit mode'}
            </button>
          )}
          <Link
            to={authPath}
            className={`rounded-full border border-[#171713]/25 px-4 py-2.5 text-xs font-bold
uppercase tracking-[0.12em] text-[#171713] transition-colors hover:border-[#ff4f00]
hover:text-[#ff4f00]`}
          >
            {authLabel}
          </Link>
        </div>

        <button
          type="button"
          className={`grid h-10 w-10 place-items-center rounded-full border border-[#171713]/20
text-[#171713] md:hidden`}
          onClick={() => setIsMobileMenuOpen((open) => !open)}
          aria-label={isMobileMenuOpen ? 'Menü schließen' : 'Menü öffnen'}
          aria-expanded={isMobileMenuOpen}
          aria-controls="mobile-navigation"
        >
          {isMobileMenuOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
        </button>
      </nav>

      {isMobileMenuOpen && (
        <div
          id="mobile-navigation"
          className={`mx-auto mt-2 max-w-[1480px] border border-[#171713] bg-[#f4f1ea] p-5
shadow-2xl md:hidden`}
        >
          <div className="flex flex-col">
            {navigationLinks.map((link, index) => renderNavigationLink(link, index, true))}
          </div>
          <div className="mt-5">
            <Link
              to={authPath}
              onClick={() => setIsMobileMenuOpen(false)}
              className={`block w-full border border-[#171713] px-4 py-3 text-center text-xs font-bold
uppercase tracking-[0.12em]`}
            >
              {authLabel}
            </Link>
          </div>
        </div>
      )}
    </header>
  )
}

export default Header
