import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react'
import PropTypes from 'prop-types'
import { api } from '../api/client'
const ContentContext = createContext(null)

/**
 * Global Fallback Data Configuration.
 *
 * Defines the initial structure and default content for the entire site.
 * This object serves as a blueprint for the backend data and also acts as
 * the reliable fallback if the API fails or returns incomplete sections.
 */
export const DEFAULT_CONTENT = {
  hero: {
    badgeText: 'Persönlicher Blog',
    icon: 'Terminal',
    title: {
      line1: 'Gedanken, Projekte',
      line2: '& Dinge dazwischen',
    },
    subtitle: 'Persönliche Notizen über Technik, Ideen und alles, was mich gerade beschäftigt.',
    subline: 'Ausprobiert, durchdacht und ehrlich aufgeschrieben.',
    heroImage: '', // URL to the hero image (can be set via CMS)
    primaryCta: {
      label: 'Beiträge lesen',
      target: { type: 'section', value: 'stories' },
    },
    secondaryCta: {
      label: 'Über diesen Blog',
      target: { type: 'section', value: 'about' },
    },
    features: [
      {
        icon: 'Shield',
        title: 'IT Security',
        description: 'Sicherheitskonzepte und Best Practices',
        color: 'from-blue-500 to-cyan-500',
      },
      {
        icon: 'Code',
        title: 'Programming',
        description: 'Softwareentwicklung und Coding',
        color: 'from-purple-500 to-pink-500',
      },
      {
        icon: 'Server',
        title: 'IT Administration',
        description: 'Systemverwaltung und DevOps',
        color: 'from-orange-500 to-red-500',
      },
    ],
  },
  stats: {
    items: [
      { label: 'Leser monatlich', value: '10k+' },
      { label: 'Artikel', value: '500+' },
      { label: 'Themenbereiche', value: '20+' },
      { label: 'Community', value: 'Active' },
    ],
  },
  cta_section: {
    title: 'Neue Notizen per Mail',
    description: 'Ich melde mich, wenn es einen neuen Gedanken oder Beitrag zu teilen gibt.',
  },
  about: {
    eyebrow: 'Warum ich schreibe',
    lead: 'Ich schreibe, um Dinge wirklich zu verstehen – und um meine Gedanken nicht zu verlieren.',
    paragraphs: [
      'Dieser Blog ist mein digitales Notizbuch. Ich teile, was ich lerne, woran ich arbeite ' +
        'und welche Fragen mich gerade begleiten.',
      'Die Themen dürfen wechseln. Was bleibt, ist eine persönliche Perspektive, ehrliche ' +
        'Neugier und der Wunsch, Gedanken sauber zu Ende zu denken.',
    ],
  },
  site_meta: {
    title: 'minos – Persönlicher Blog',
    description: 'Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen.',
  },

  header: {
    brand: {
      name: 'minos',
      tagline: 'Persönlicher Blog',
      icon: 'Terminal',
    },
    navItems: [
      { id: 'stories', label: 'Beiträge', type: 'section', value: 'stories' },
      { id: 'about', label: 'Über diesen Blog', type: 'section', value: 'about' },
    ],
    cta: {
      guestLabel: 'Login',
      authLabel: 'Admin',
      icon: 'Lock',
    },
  },
  footer: {
    brand: {
      title: 'minos',
      description: 'Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen.',
      icon: 'Terminal',
    },
    quickLinks: [
      { label: 'Beiträge', target: { type: 'section', value: 'stories' } },
      { label: 'Über diesen Blog', target: { type: 'section', value: 'about' } },
    ],
    contactLinks: [],
    bottom: {
      copyright: '© {year} minos.',
      signature: 'Persönlich notiert',
    },
  },
  login: {
    title: 'minos',
    subtitle: 'Admin Login',
    icon: 'Terminal',
    buttonLabel: 'Anmelden',
    usernameLabel: 'Benutzername',
    passwordLabel: 'Passwort',
    backLinkText: 'Zurück zur Startseite',
  },
}
const LEGACY_STARTER_BRANDS = new Set(['IT Portal', 'IT Wissensportal', 'Linux Tutorial'])
const normalizeLegacyBranding = (value) => {
  if (typeof value === 'string') {
    return value.replace(/zero[\s_-]+point/gi, 'minos')
  }
  if (Array.isArray(value)) {
    return value.map(normalizeLegacyBranding)
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.entries(value).map(([key, nestedValue]) => [key, normalizeLegacyBranding(nestedValue)]),
    )
  }
  return value
}
const normalizeStarterSection = (section, value) => {
  if (section === 'site_meta') {
    const legacyTitle = value?.title || ''
    if (legacyTitle.startsWith('IT Wissensportal') || legacyTitle.startsWith('Linux Tutorial')) {
      return DEFAULT_CONTENT.site_meta
    }
  }
  if (section === 'header' && LEGACY_STARTER_BRANDS.has(value?.brand?.name)) {
    return DEFAULT_CONTENT.header
  }
  if (section === 'footer' && LEGACY_STARTER_BRANDS.has(value?.brand?.title)) {
    return DEFAULT_CONTENT.footer
  }
  if (
    section === 'hero' &&
    (value?.title?.line1 === 'Lerne Linux' || value?.badgeText === 'Professionelles Linux Training')
  ) {
    return DEFAULT_CONTENT.hero
  }
  return normalizeLegacyBranding(value)
}
export const CONTENT_SECTIONS = Object.keys(DEFAULT_CONTENT)
export const ContentProvider = ({ children }) => {
  const [content, setContent] = useState(DEFAULT_CONTENT)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [savingSections, setSavingSections] = useState({})
  const [navLoading, setNavLoading] = useState(true)
  const [navError, setNavError] = useState(null)
  const [dynamicNavItems, setDynamicNavItems] = useState([])
  const [publishedPageSlugs, setPublishedPageSlugs] = useState([])
  const [publishedPagesLoading, setPublishedPagesLoading] = useState(true)
  const [publishedPagesError, setPublishedPagesError] = useState(null)
  const [pageCache, setPageCache] = useState({})
  const pageCacheRef = useRef({})
  const publishedPageSlugsRef = useRef([])
  /**
   * Loads global site content (Hero, Features, Stats, etc.) from the backend.
   *
   * Merges backend data on top of `DEFAULT_CONTENT` to ensure structural integrity
   * even if specific sections haven't been customized in the CMS yet.
   */
  const loadContent = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const data = await api.getSiteContent()
      const merged = { ...DEFAULT_CONTENT }
      if (data?.items?.length) {
        for (const item of data.items) {
          merged[item.section] = normalizeStarterSection(item.section, item.content)
        }
      }
      setContent(merged)
    } catch (err) {
      console.error('Failed to load site content:', err)
      const fallback = err?.status ? err : new Error('Inhalte konnten nicht geladen werden.')
      fallback.status = err?.status ?? 500
      setError(fallback)
    } finally {
      setLoading(false)
    }
  }, [])
  const loadNavigation = useCallback(async () => {
    try {
      setNavLoading(true)
      setNavError(null)
      const data = await api.getNavigation()
      const items = Array.isArray(data?.items) ? data.items : []
      setDynamicNavItems(items)
    } catch (err) {
      console.error('Failed to load dynamic navigation:', err)
      setNavError(err)
    } finally {
      setNavLoading(false)
    }
  }, [])
  /**
   * Fetches published page slugs to populate dynamic navigation.
   */
  const loadPublishedPages = useCallback(async () => {
    try {
      setPublishedPagesLoading(true)
      setPublishedPagesError(null)
      const data = await api.listPublishedPages()
      const items = Array.isArray(data) ? data : []
      setPublishedPageSlugs(items)
    } catch (err) {
      console.error('CMS: Failed to load published pages', err)
      setPublishedPagesError(err)
    } finally {
      setPublishedPagesLoading(false)
    }
  }, [])
  useEffect(() => {
    publishedPageSlugsRef.current = Array.isArray(publishedPageSlugs) ? publishedPageSlugs : []
  }, [publishedPageSlugs])

  const fetchPublishedPage = useCallback(
    async (slug, { force = false, signal } = {}) => {
      if (!slug || typeof slug !== 'string') {
        throw new Error('Slug is required')
      }
      const normalizedSlug = slug.trim().toLowerCase()
      if (!normalizedSlug) {
        throw new Error('Slug is required')
      }
      if (!force && pageCacheRef.current[normalizedSlug]) {
        return pageCacheRef.current[normalizedSlug]
      }
      const buildSlugSet = (values) =>
        new Set(
          (Array.isArray(values) ? values : [])
            .map((value) => (typeof value === 'string' ? value.trim().toLowerCase() : ''))
            .filter(Boolean),
        )

      if (!force) {
        const initialPublished = buildSlugSet(publishedPageSlugsRef.current)
        if (!initialPublished.has(normalizedSlug)) {
          await loadPublishedPages()
          await new Promise((resolve) => {
            // allow state updates triggered by loadPublishedPages to settle before checking again
            setTimeout(resolve, 0)
          })
          const refreshedPublished = buildSlugSet(publishedPageSlugsRef.current)
          if (!refreshedPublished.has(normalizedSlug)) {
            const error = new Error('Seite ist nicht veröffentlicht oder wurde entfernt.')
            error.status = 404
            throw error
          }
        }
      }
      try {
        const data = await api.getPublishedPage(normalizedSlug, { signal })
        const nextCache = { ...pageCacheRef.current, [normalizedSlug]: data }
        pageCacheRef.current = nextCache
        setPageCache(nextCache)
        return data
      } catch (err) {
        if (!force && pageCacheRef.current[normalizedSlug]) {
          return pageCacheRef.current[normalizedSlug]
        }
        const published = buildSlugSet(publishedPageSlugsRef.current)
        if (!published.has(normalizedSlug)) {
          const error = new Error('Seite ist nicht veröffentlicht oder wurde entfernt.')
          error.status = 404
          throw error
        }
        throw err
      }
    },
    [loadPublishedPages],
  )
  const invalidatePageCache = useCallback((slug) => {
    if (slug && typeof slug === 'string') {
      const normalizedSlug = slug.trim().toLowerCase()
      if (!normalizedSlug) {
        return
      }
      setPageCache((prev) => {
        if (!prev[normalizedSlug]) {
          return prev
        }
        const next = { ...prev }
        delete next[normalizedSlug]
        pageCacheRef.current = next
        return next
      })
      return
    }
    pageCacheRef.current = {}
    setPageCache({})
  }, [])
  useEffect(() => {
    loadContent()
  }, [loadContent])
  useEffect(() => {
    loadNavigation()
    loadPublishedPages()
  }, [loadNavigation, loadPublishedPages])
  const updateSection = useCallback(async (section, newContent) => {
    if (!section) {
      throw new Error('Section is required')
    }
    setSavingSections((prev) => ({ ...prev, [section]: true }))
    try {
      const response = await api.updateSiteContentSection(section, newContent)
      const updatedContent = normalizeLegacyBranding(response?.content ?? newContent)
      setContent((prev) => ({
        ...prev,
        [section]: updatedContent,
      }))
      return response
    } finally {
      setSavingSections((prev) => {
        const next = { ...prev }
        delete next[section]
        return next
      })
    }
  }, [])
  /**
   * Generates a unified navigation model combining static and dynamic items.
   *
   * static: Defined in the `header` section of DEFAULT_CONTENT/CMS.
   * dynamic: Automatically generated from published CMS pages.
   *
   * Items are normalized into a consistent shape { id, label, path, source }
   * for consumption by Navbar / Footer components.
   */
  const navigationData = useMemo(() => {
    const headerContent = content?.header ?? DEFAULT_CONTENT.header
    const staticNavItems = Array.isArray(headerContent?.navItems) ? headerContent.navItems : []
    const staticNormalized = staticNavItems.map((item, index) => ({
      ...item,
      id: item.id || item.slug || item.path || `static-${index}`,
      source: item.source || 'static',
    }))

    const normalizedPublishedSlugs = new Set(
      (Array.isArray(publishedPageSlugs) ? publishedPageSlugs : [])
        .map((slug) => (typeof slug === 'string' ? slug.trim().toLowerCase() : ''))
        .filter(Boolean),
    )
    const restrictToPublished = normalizedPublishedSlugs.size > 0

    const filteredDynamic = Array.isArray(dynamicNavItems)
      ? dynamicNavItems.filter((item) => {
          if (!item || typeof item.slug !== 'string') {
            return false
          }
          const normalizedSlug = item.slug.trim().toLowerCase()
          if (!normalizedSlug) {
            return false
          }
          if (restrictToPublished && !normalizedPublishedSlugs.has(normalizedSlug)) {
            return false
          }
          return true
        })
      : []
    const sortedDynamic = [...filteredDynamic].sort(
      (a, b) => (a?.order_index ?? 0) - (b?.order_index ?? 0),
    )
    const dynamicNormalized = sortedDynamic.map((item, index) => {
      const normalizedSlug = item.slug.trim().toLowerCase()
      return {
        id: item.id || `page-${normalizedSlug}-${index}`,
        label: item.label || item.slug || 'Seite',
        type: 'route',
        path: `/pages/${normalizedSlug}`,
        slug: normalizedSlug,
        source: 'dynamic',
        order_index: item.order_index ?? index,
      }
    })
    return {
      static: staticNormalized,
      dynamic: dynamicNormalized,
      items: [...staticNormalized, ...dynamicNormalized],
    }
  }, [content, dynamicNavItems, publishedPageSlugs])
  const value = useMemo(
    () => ({
      content,
      loading,
      error,
      refreshContent: loadContent,
      getSection: (section) => content[section] ?? DEFAULT_CONTENT[section],
      getDefaultSection: (section) => DEFAULT_CONTENT[section],
      getSiteMeta: () => content?.site_meta ?? DEFAULT_CONTENT.site_meta,
      updateSection,
      savingSections,
      navigation: {
        ...navigationData,
        loading: navLoading,
        error: navError,
        refresh: loadNavigation,
      },
      pages: {
        cache: pageCache,
        fetch: fetchPublishedPage,
        publishedSlugs: publishedPageSlugs,
        loading: publishedPagesLoading,
        error: publishedPagesError,
        refresh: loadPublishedPages,
        invalidate: invalidatePageCache,
        getPost: api.getPublishedPost.bind(api),
      },
    }),
    [
      content,
      loading,
      error,
      loadContent,
      updateSection,
      savingSections,
      navigationData,
      navLoading,
      navError,
      loadNavigation,
      pageCache,
      fetchPublishedPage,
      publishedPageSlugs,
      publishedPagesLoading,
      publishedPagesError,
      loadPublishedPages,
      invalidatePageCache,
    ],
  )
  return <ContentContext.Provider value={value}>{children}</ContentContext.Provider>
}
ContentProvider.propTypes = {
  children: PropTypes.node,
}
export const useContent = () => {
  const ctx = useContext(ContentContext)
  if (!ctx) {
    throw new Error('useContent must be used within a ContentProvider')
  }
  return ctx
}
