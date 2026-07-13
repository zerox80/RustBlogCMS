import { Link } from 'react-router-dom'
import { ArrowUpRight, Asterisk } from 'lucide-react'
import { useContent } from '../../context/ContentContext'
import EditableText from '../cms/EditableText'
import { renderIcon } from '../../utils/iconMap'
import { sanitizeExternalUrl } from '../../utils/urlValidation'

/** Editorial footer that closes the one-page layout without repeating a sitemap wall. */
const Footer = () => {
    const { getSection, navigation } = useContent()
    const footerContent = getSection('footer') ?? {}
    const contactLinks = Array.isArray(footerContent?.contactLinks) ? footerContent.contactLinks : []
    const pageLinks = (navigation?.dynamic || []).slice(0, 4)
    const copyright = (footerContent?.bottom?.copyright || '© {year} Zero Point. Alle Rechte vorbehalten.')
        .replace('{year}', new Date().getFullYear())

    return (
        <footer className="border-t border-white/15 bg-[#171713] px-5 py-10 text-[#f4f1ea] sm:px-8 lg:px-12">
            <div className="mx-auto max-w-[1480px]">
                <div className="grid gap-12 pb-16 pt-6 md:grid-cols-[1.3fr_0.7fr_0.7fr]">
                    <div>
                        <Link to="/" className="mb-6 inline-flex items-center gap-3">
                            <span className="grid h-10 w-10 place-items-center rounded-full bg-[#b9f227] text-[#171713]"><Asterisk className="h-5 w-5" /></span>
                            <span className="font-display text-xl font-bold uppercase">
                                <EditableText section="footer" field={footerContent?.brand?.title ? 'brand.title' : 'brand.name'} value={footerContent?.brand?.title || footerContent?.brand?.name || 'Zero Point'} />
                            </span>
                        </Link>
                        <p className="max-w-sm text-base leading-relaxed text-white/45">
                            <EditableText section="footer" field="brand.description" value={footerContent?.brand?.description || 'Ein unabhängiges Journal über Code, Systeme und digitale Kultur.'} multiline />
                        </p>
                    </div>

                    <div>
                        <p className="mb-5 font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-[#b9f227]">Explore</p>
                        <div className="flex flex-col gap-3 text-sm text-white/60">
                            <a href="/#stories" className="hover:text-white">Stories</a>
                            <a href="/#topics" className="hover:text-white">Themen</a>
                            <a href="/#manifest" className="hover:text-white">Manifest</a>
                            {pageLinks.map((link) => <Link key={link.id} to={link.path} className="hover:text-white">{link.label}</Link>)}
                        </div>
                    </div>

                    <div>
                        <p className="mb-5 font-mono text-[10px] font-bold uppercase tracking-[0.2em] text-[#b9f227]">Connect</p>
                        <div className="flex flex-col gap-3 text-sm text-white/60">
                            {contactLinks.map((contact, index) => {
                                const href = sanitizeExternalUrl(contact.href || contact.url)
                                if (!href) return null
                                const external = href.startsWith('http://') || href.startsWith('https://')
                                return (
                                    <a key={href || index} href={href} target={external ? '_blank' : undefined} rel={external ? 'noopener noreferrer' : undefined} className="group inline-flex items-center gap-2 hover:text-white">
                                        {renderIcon(contact.icon, 'ArrowUpRight', { className: 'h-4 w-4' })}
                                        <EditableText section="footer" field={`contactLinks.${index}.label`} value={contact.label || 'Kontakt'} />
                                    </a>
                                )
                            })}
                            <a href="https://github.com/zerox80/RustBlogCMS" target="_blank" rel="noreferrer" className="group inline-flex items-center gap-2 hover:text-white">GitHub <ArrowUpRight className="h-4 w-4 transition-transform group-hover:rotate-45" /></a>
                        </div>
                    </div>
                </div>

                <div className="flex flex-col gap-4 border-t border-white/15 pt-6 text-[10px] font-bold uppercase tracking-[0.16em] text-white/35 sm:flex-row sm:items-center sm:justify-between">
                    <p><EditableText section="footer" field="bottom.copyright" value={copyright} /></p>
                    <p>{footerContent?.bottom?.signature || 'Made for curious minds'}</p>
                </div>
            </div>
        </footer>
    )
}

export default Footer
