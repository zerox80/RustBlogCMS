import { Link } from 'react-router-dom'
import { Asterisk } from 'lucide-react'
import { useContent } from '../../context/ContentContext'
import EditableText from '../cms/EditableText'
import { renderIcon } from '../../utils/iconMap'
import { sanitizeExternalUrl } from '../../utils/urlValidation'

const resolveFooterTarget = (target) => {
  if (!target || typeof target !== 'object') return null
  const rawValue = target.value ?? target.path ?? target.href
  if (typeof rawValue !== 'string' || !rawValue.trim()) return null
  const value = rawValue.trim()

  if (target.type === 'section') return { href: `/#${value}`, internal: false }
  if (target.type === 'route') return { href: value, internal: true }
  if (target.type === 'external' || target.type === 'href') {
    const href = sanitizeExternalUrl(value)
    return href ? { href, internal: false } : null
  }
  return null
}

/** Personal footer that closes the one-page layout without repeating a sitemap wall. */
const Footer = () => {
  const { getSection } = useContent()
  const footerContent = getSection('footer') ?? {}
  const contactLinks = Array.isArray(footerContent?.contactLinks) ? footerContent.contactLinks : []
  const quickLinks = Array.isArray(footerContent?.quickLinks) ? footerContent.quickLinks : []
  const copyright = (
    footerContent?.bottom?.copyright || '© {year} minos. Alle Rechte vorbehalten.'
  ).replace('{year}', new Date().getFullYear())

  return (
    <footer
      className={`border-t border-white/15 bg-[#171713] px-5 py-10 text-[#f4f1ea] sm:px-8
lg:px-12`}
    >
      <div className="mx-auto max-w-[1480px]">
        <div className="grid gap-12 pb-16 pt-6 md:grid-cols-[1.4fr_0.8fr_0.8fr]">
          <div>
            <Link to="/" className="mb-6 inline-flex items-center gap-3">
              <span className="grid h-10 w-10 place-items-center rounded-full bg-[#b9f227] text-[#171713]">
                <Asterisk className="h-5 w-5" />
              </span>
              <span className="font-display text-xl font-bold uppercase">
                <EditableText
                  section="footer"
                  field={footerContent?.brand?.title ? 'brand.title' : 'brand.name'}
                  value={footerContent?.brand?.title || footerContent?.brand?.name || 'minos'}
                />
              </span>
            </Link>
            <p className="max-w-sm text-base leading-relaxed text-white/45">
              <EditableText
                section="footer"
                field="brand.description"
                value={
                  footerContent?.brand?.description ||
                  'Mein persönlicher Blog über Technik, Projekte, Ideen und alles dazwischen.'
                }
                multiline
              />
            </p>
          </div>

          <div>
            <p
              className={[
                'mb-5 font-mono text-[10px] font-bold uppercase tracking-[0.2em]',
                'text-[#b9f227]',
              ].join(' ')}
            >
              Hier entlang
            </p>
            <div className="flex flex-col gap-3 text-sm text-white/60">
              {quickLinks.map((link, index) => {
                const destination = resolveFooterTarget(link?.target)
                if (!destination) return null
                const content = (
                  <EditableText
                    section="footer"
                    field={`quickLinks.${index}.label`}
                    value={link?.label || 'Link'}
                  />
                )
                if (destination.internal) {
                  return (
                    <Link key={`${destination.href}-${index}`} to={destination.href} className="hover:text-white">
                      {content}
                    </Link>
                  )
                }
                const external = /^https?:\/\//i.test(destination.href)
                return (
                  <a
                    key={`${destination.href}-${index}`}
                    href={destination.href}
                    target={external ? '_blank' : undefined}
                    rel={external ? 'noopener noreferrer' : undefined}
                    className="hover:text-white"
                  >
                    {content}
                  </a>
                )
              })}
            </div>
          </div>

          <div>
            <p
              className={[
                'mb-5 font-mono text-[10px] font-bold uppercase tracking-[0.2em]',
                'text-[#b9f227]',
              ].join(' ')}
            >
              Connect
            </p>
            <div className="flex flex-col gap-3 text-sm text-white/60">
              {contactLinks.map((contact, index) => {
                const href = sanitizeExternalUrl(contact.href || contact.url)
                if (!href) return null
                const external = href.startsWith('http://') || href.startsWith('https://')
                return (
                  <a
                    key={href || index}
                    href={href}
                    target={external ? '_blank' : undefined}
                    rel={external ? 'noopener noreferrer' : undefined}
                    className="group inline-flex items-center gap-2 hover:text-white"
                  >
                    {renderIcon(contact.icon, 'ArrowUpRight', { className: 'h-4 w-4' })}
                    <EditableText
                      section="footer"
                      field={`contactLinks.${index}.label`}
                      value={contact.label || 'Kontakt'}
                    />
                  </a>
                )
              })}
            </div>
          </div>
        </div>

        <div
          className={`flex flex-col gap-4 border-t border-white/15 pt-6 text-[10px] font-bold
uppercase tracking-[0.16em] text-white/35 sm:flex-row sm:items-center
sm:justify-between`}
        >
          <p>
            <EditableText section="footer" field="bottom.copyright" value={copyright} />
          </p>
          <p>{footerContent?.bottom?.signature || 'Persönlich notiert'}</p>
        </div>
      </div>
    </footer>
  )
}

export default Footer
