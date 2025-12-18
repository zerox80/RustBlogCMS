import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import PropTypes from 'prop-types'
import { api } from '../api/client'
const ContentContext = createContext(null)
export const DEFAULT_CONTENT = {
  hero: {
    badgeText: 'Professionelles IT Wissen',
    icon: 'Terminal',
    title: {
      line1: 'IT Security, Programming',
      line2: '& Administration',
    },
    subtitle: 'Dein Wissensportal für IT-Themen – von Security bis Systemadministration.',
    subline: 'Aktuell, praxisnah und verständlich.',
    heroImage: '', // URL to the hero image (can be set via CMS)
    primaryCta: {
      label: "Themen entdecken",
      target: { type: 'section', value: 'features' },
    },
    secondaryCta: {
      label: 'Blog lesen',
      target: { type: 'route', value: '/blog' },
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
    title: 'Wissen teilen & erweitern',
    description: 'Bleib auf dem Laufenden mit den neuesten Entwicklungen in der IT-Welt.',
  },
  site_meta: {
    title: 'IT Wissensportal - Security, Programming & Admin',
    description: 'Dein Portal für IT Security, Programmierung und Administration.',
  },

  header: {
    brand: {
      name: 'IT Portal',
      tagline: '',
      icon: 'Terminal',
    },
    navItems: [
      { id: 'features', label: 'Features', type: 'section', value: 'features' },
      { id: 'tutorial', label: 'Tutorial', type: 'route', path: '/tutorial/getting-started' },
      { id: 'blog', label: 'Blog', type: 'route', path: '/blog' },
      { id: 'about', label: 'Über', type: 'route', path: '/about' },
    ],
    cta: {
      guestLabel: 'Login',
      authLabel: 'Admin',
      icon: 'Lock',
    },
  },
  footer: {
    brand: {
      title: 'IT Wissensportal',
      description: 'Dein Portal für IT Security, Programmierung und Administration.',
      icon: 'Terminal',
    },
    quickLinks: [
      { label: 'Home', target: { type: 'section', value: 'home' } },
      { label: 'Blog', target: { type: 'route', value: '/blog' } },
    ],
    contactLinks: [
      { label: 'GitHub', href: 'https://github.com', icon: 'Github' },
      { label: 'E-Mail', href: 'mailto:info@example.com', icon: 'Mail' },
    ],
    bottom: {
      copyright: '© {year} IT Wissensportal. Alle Rechte vorbehalten.',
      signature: 'Made for IT Professionals',
    },
  },
  login: {
    title: 'Linux Tutorial',
    subtitle: 'Admin Login',
    icon: 'Terminal',
    buttonLabel: 'Anmelden',
    usernameLabel: 'Benutzername',
    passwordLabel: 'Passwort',
    backLinkText: 'Zurück zur Startseite',
  },
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
  const loadContent = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const data = await api.getSiteContent()
      const merged = { ...DEFAULT_CONTENT }
      if (data?.items?.length) {
        for (const item of data.items) {
          merged[item.section] = item.content
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
  const loadPublishedPages = useCallback(async () => {
    try {
      setPublishedPagesLoading(true)
      setPublishedPagesError(null)
      const data = await api.listPublishedPages()
      const items = Array.isArray(data) ? data : []
      setPublishedPageSlugs(items)
    } catch (err) {
      console.error('Failed to load published pages:', err)
      setPublishedPagesError(err)
    } finally {
      setPublishedPagesLoading(false)
    }
  }, [])
  useEffect(() => {
    publishedPageSlugsRef.current = Array.isArray(publishedPageSlugs)
      ? publishedPageSlugs
      : []
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
      const updatedContent = response?.content ?? newContent
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
  const value = useMemo(() => ({
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
    },
  }), [
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
  ]);
  return <ContentContext.Provider value={value}>{children}</ContentContext.Provider>;
};
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
