import { useMemo } from 'react'
import { Sparkles, ArrowRight } from 'lucide-react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useContent } from '../context/ContentContext'
import { getIconComponent } from '../utils/iconMap'
import { scrollToSection } from '../utils/scrollToSection'
import { sanitizeExternalUrl } from '../utils/urlValidation'

/**
 * A generic Hero section component used for standard pages.
 * 
 * Unlike the specialized `LandingHero`, this component is designed for reusability
 * across different pages. It supports a configurable title, subtitle, CTA buttons,
 * and a grid of feature highlights.
 * 
 * Key Features:
 * - Dynamic Icon Resolution: Loads icons based on CMS string identifiers.
 * - Smart Navigation: Handles internal routes, external links, and anchor scrolling via `handleTarget`.
 * - Visuals: Features a background gradient with decorative blur blobs and a pattern overlay.
 * 
 * @component
 * @example
 * // Usage within a page component
 * <Hero />
 */
const Hero = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const { getSection } = useContent()
  const heroContent = getSection('hero') ?? {}
  const HeroIcon = useMemo(() => getIconComponent(heroContent.icon, 'Terminal'), [heroContent.icon])
  const features = Array.isArray(heroContent.features) ? heroContent.features : []

  /**
   * Unified handler for processing navigation actions.
   * 
   * Abstracts away the complexity of different link types:
   * - `section`: Scrolls to an ID on the home page (handling cross-page jumps if needed).
   * - `route`: Standard SPA client-side navigation.
   * - `external`: Opens in new tab with security attributes (noopener, noreferrer).
   * - `href`: Direct window location assignment (for special protocols or hard refreshes).
   * 
   * @param {Object} target - The target definition object from CMS.
   * @param {string} target.type - The type of navigation ('section', 'route', 'external', 'href').
   * @param {string} target.value - The destination path, URL, or ID.
   */
  const handleTarget = (target) => {
    if (!target || !target.type) {
      return
    }
    switch (target.type) {
      case 'section': {
        if (location.pathname !== '/') {
          navigate('/', { state: { scrollTo: target.value } })
        } else {
          scrollToSection(target.value)
        }
        break
      }
      case 'route': {
        if (typeof target.value === 'string') {
          navigate(target.value)
        }
        break
      }
      case 'external': {
        if (typeof window !== 'undefined' && target.value) {
          const safeUrl = sanitizeExternalUrl(target.value)
          if (!safeUrl) {
            console.warn('Blocked unsafe external hero link:', target.value)
            return
          }
          window.open(safeUrl, '_blank', 'noopener,noreferrer')
        }
        break
      }
      case 'href': {
        if (typeof window !== 'undefined' && target.value) {
          const safeUrl = sanitizeExternalUrl(target.value)
          if (!safeUrl) {
            console.warn('Blocked unsafe hero href:', target.value)
            return
          }
          if (safeUrl.startsWith('#')) {
            scrollToSection(safeUrl.slice(1))
            return
          }
          window.location.assign(safeUrl)
        }
        break
      }
      default:
        break
    }
  }
  return (
    <section className="relative isolate overflow-hidden bg-gradient-to-br from-slate-950 via-slate-900 to-primary-950 text-white">
      <div className="absolute inset-0 opacity-40">
        <div className="absolute -top-32 -right-20 h-96 w-96 rounded-full bg-primary-500/30 blur-3xl"></div>
        <div className="absolute -bottom-40 -left-16 h-96 w-96 rounded-full bg-cyan-500/20 blur-3xl"></div>
        <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNjAiIGhlaWdodD0iNjAiIHZpZXdCb3g9IjAgMCA2MCA2MCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48ZyBmaWxsPSJub25lIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxnIGZpbGw9IiNmZmYiIGZpbGwtb3BhY2l0eT0iMC4xIj48cGF0aCBkPSJNMzYgMzRjMC0yLjIxIDEuNzktNCA0LTRzNCAxLjc5IDQgNC0xLjc5IDQtNCA0LTQtMS43OS00LTR6TTAgMTRjMC0yLjIxIDEuNzktNCA0LTRzNCAxLjc5IDQgNC0xLjc5IDQtNCA0LTQtMS43OS00LTR6bTAgNDBjMC0yLjIxIDEuNzktNCA0LTRzNCAxLjc5IDQgNC0xLjc5IDQtNCA0LTQtMS43OS00LTR6Ii8+PC9nPjwvZz48L3N2Zz4=')] opacity-20"></div>
      </div>
      <div className="relative max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-24">
        <div className="text-center space-y-10">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white/10 border border-white/20 text-sm font-medium">
            <Sparkles className="w-4 h-4" />
            <span>{heroContent.badgeText}</span>
          </div>
          <div className="mx-auto flex h-24 w-24 items-center justify-center rounded-2xl border border-white/20 bg-white/10 backdrop-blur">
            <HeroIcon className="w-12 h-12 text-white" />
          </div>
          <div className="space-y-6">
            <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight text-white">
              <span>{heroContent?.title?.line1}</span>
              <span className="block text-primary-200">{heroContent?.title?.line2}</span>
            </h1>
            <p className="text-lg sm:text-xl text-slate-200 max-w-3xl mx-auto leading-relaxed">
              {heroContent?.subtitle}
              {heroContent?.subline && (
                <span className="block mt-2 text-slate-300">{heroContent.subline}</span>
              )}
            </p>
          </div>
          <div className="flex flex-col sm:flex-row gap-4 justify-center items-center">
            <button
              onClick={() => handleTarget(heroContent?.primaryCta?.target)}
              className="btn-primary"
              aria-label="Zu den Tutorials navigieren"
            >
              <span className="flex items-center gap-2">
                {heroContent?.primaryCta?.label || "Los geht's"}
                <ArrowRight className="w-5 h-5" />
              </span>
            </button>
            <button
              onClick={() => handleTarget(heroContent?.secondaryCta?.target)}
              className="btn-secondary-inverse"
              aria-label="Mehr über die Tutorials erfahren"
            >
              <span className="flex items-center gap-2">
                {heroContent?.secondaryCta?.label || 'Mehr erfahren'}
                <ArrowRight className="w-5 h-5" />
              </span>
            </button>
          </div>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 max-w-5xl mx-auto text-left">
            {features.map((feature, i) => {
              const FeatureIcon = getIconComponent(feature.icon, 'Terminal')
              return (
                <div
                  key={i}
                  className="rounded-2xl border border-white/15 bg-white/5 p-6 backdrop-blur transition-colors duration-300 hover:border-primary-400/40"
                >
                  <div className={`inline-flex p-3 rounded-xl bg-gradient-to-br ${feature.color || 'from-blue-500 to-cyan-500'} mb-4 shadow-md`}>
                    <FeatureIcon className="w-6 h-6 text-white" />
                  </div>
                  <h3 className="font-semibold text-lg text-white mb-2">{feature.title}</h3>
                  <p className="text-sm text-slate-200 leading-relaxed">{feature.description}</p>
                </div>
              )
            })}
          </div>
        </div>
      </div>
    </section>
  )
}
export default Hero
