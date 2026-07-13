import { useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { ArrowUpRight, Asterisk, Menu, X } from 'lucide-react'
import { useContent } from '../../context/ContentContext'
import { useEdit } from '../../context/EditContext'
import { useAuth } from '../../context/AuthContext'
import EditableText from '../cms/EditableText'

const personalLinks = [
  { label: 'Beiträge', hash: 'stories' },
  { label: 'Themen', hash: 'topics' },
  { label: 'Über diesen Blog', hash: 'about' },
]

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

  const dynamicLinks = (navigation?.dynamic || []).slice(0, 2)
  const anchorPath = (hash) => (location.pathname === '/' ? `#${hash}` : `/#${hash}`)

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
              value={headerContent?.brand?.name || 'Zero Point'}
            />
          </span>
        </Link>

        <div className="hidden items-center gap-7 lg:flex">
          {personalLinks.map((link) => (
            <a
              key={link.hash}
              href={anchorPath(link.hash)}
              className={`text-xs font-bold uppercase tracking-[0.14em] text-[#171713]/65
transition-colors hover:text-[#ff4f00]`}
            >
              {link.label}
            </a>
          ))}
          {dynamicLinks.map((link) => (
            <Link
              key={link.id}
              to={link.path}
              className={`text-xs font-bold uppercase tracking-[0.14em] text-[#171713]/65
transition-colors hover:text-[#ff4f00]`}
            >
              {link.label}
            </Link>
          ))}
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
          <a
            href="https://github.com/zerox80/RustBlogCMS"
            target="_blank"
            rel="noreferrer"
            className={`group inline-flex items-center gap-2 rounded-full bg-[#171713] px-5 py-2.5
text-xs font-bold uppercase tracking-[0.12em] text-white transition-colors
hover:bg-[#ff4f00]`}
          >
            GitHub <ArrowUpRight className="h-4 w-4 transition-transform group-hover:rotate-45" />
          </a>
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
            {personalLinks.map((link, index) => (
              <a
                key={link.hash}
                href={anchorPath(link.hash)}
                onClick={() => setIsMobileMenuOpen(false)}
                className={`flex items-center justify-between border-b border-[#171713]/15 py-4
font-display text-2xl font-semibold tracking-[-0.04em] text-[#171713]`}
              >
                {link.label}
                <span className="font-serif text-sm italic text-[#171713]/35">0{index + 1}</span>
              </a>
            ))}
            {dynamicLinks.map((link) => (
              <Link
                key={link.id}
                to={link.path}
                onClick={() => setIsMobileMenuOpen(false)}
                className={`border-b border-[#171713]/15 py-4 font-display text-2xl font-semibold
tracking-[-0.04em] text-[#171713]`}
              >
                {link.label}
              </Link>
            ))}
          </div>
          <div className="mt-5 flex gap-3">
            <Link
              to="/login"
              onClick={() => setIsMobileMenuOpen(false)}
              className={`flex-1 border border-[#171713] px-4 py-3 text-center text-xs font-bold
uppercase tracking-[0.12em]`}
            >
              Login
            </Link>
            <a
              href="https://github.com/zerox80/RustBlogCMS"
              target="_blank"
              rel="noreferrer"
              className={`flex flex-1 items-center justify-center gap-2 bg-[#171713] px-4 py-3 text-xs
font-bold uppercase tracking-[0.12em] text-white`}
            >
              GitHub <ArrowUpRight className="h-4 w-4" />
            </a>
          </div>
        </div>
      )}
    </header>
  )
}

export default Header
