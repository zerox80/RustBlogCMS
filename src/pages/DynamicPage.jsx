import React, { useEffect, useState, useMemo } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Loader2, AlertCircle, ArrowLeft } from 'lucide-react'
import { useContent } from '../context/ContentContext'
import {
  normalizeTitle,
  normalizeText,
  normalizeSlug,
} from '../utils/postUtils'
import DynamicPageHero from '../components/dynamic-page/DynamicPageHero'
import DynamicPageAbout from '../components/dynamic-page/DynamicPageAbout'
import DynamicPagePostList from '../components/dynamic-page/DynamicPagePostList'

/**
 * The primary engine for rendering user-created CMS pages.
 * 
 * Features:
 * - Dynamic Slug Matching: Resolves URLs to backend page objects.
 * - Content Normalization: Uses `postUtils` to ensure titles/text have clean fallbacks.
 * - List Integration: Automatically renders posts associated with the specific page.
 * - Loading States: Handles transitions, 404s, and API errors gracefully.
 */
const DynamicPage = ({ slug: propSlug }) => {
  const { slug: paramSlug } = useParams()
  const navigate = useNavigate()
  const { pages } = useContent()
  
  // Use propSlug if available (e.g. when used as Home), otherwise use paramSlug from router
  const activeSlug = propSlug || paramSlug
  const normalizedSlug = useMemo(() => normalizeSlug(activeSlug), [activeSlug])
  
  const cachedPage = pages.cache?.[normalizedSlug]
  const [pageData, setPageData] = useState(cachedPage ?? null)
  const [loading, setLoading] = useState(!cachedPage)
  const [error, setError] = useState(null)

  useEffect(() => {
    /**
     * Life Cycle: Page Data Fetching.
     * 
     * Uses `AbortController` to cancel pending requests if the user navigates
     * away before the API responds, preventing state updates on unmounted components.
     */
    if (!normalizedSlug) {
      setError(new Error('Ungültige Seite'))
      setLoading(false)
      return
    }
    const controller = new AbortController()
    const load = async () => {
      setLoading(true)
      setError(null)
      try {
        const data = await pages.fetch(normalizedSlug, { signal: controller.signal })
        if (!controller.signal.aborted) {
          setPageData(data)
        }
      } catch (err) {
        if (controller.signal.aborted) {
          return
        }
        setError(err)
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      }
    }
    load()
    return () => {
      controller.abort()
    }
  }, [normalizedSlug, pages])

  const page = pageData?.page
  const posts = Array.isArray(pageData?.posts) ? pageData.posts : []
  const hero = page?.hero ?? {}
  const layout = page?.layout ?? {}
  const aboutSection = layout?.aboutSection ?? {}
  const postsSection = layout?.postsSection ?? {}

  const heroTitle = normalizeTitle(hero.title, page?.title)
  const heroSubtitle = normalizeText(hero.subtitle ?? hero.description, page?.description)
  const heroBadge = normalizeText(hero.badge ?? hero.badgeText, null)
  const heroGradient = hero.backgroundGradient || hero.gradient || 'from-primary-600 to-primary-700'

  const aboutTitle = normalizeText(aboutSection.title, 'Über diese Seite')

  const postsTitle = normalizeText(postsSection.title, 'Beiträge')
  const postsEmptyTitle = normalizeText(postsSection.emptyTitle, 'Keine Beiträge vorhanden')
  const postsEmptyMessage = normalizeText(
    postsSection.emptyMessage,
    'Sobald für diese Seite Beiträge veröffentlicht werden, erscheinen sie hier.',
  )
  const postsCountSingular = normalizeText(
    postsSection.countLabelSingular,
    '{count} veröffentlichter Beitrag',
  )
  const postsCountPlural = normalizeText(
    postsSection.countLabelPlural,
    '{count} veröffentlichte Beiträge',
  )

  const formatPostsCount = (count) => {
    const template = count === 1 ? postsCountSingular : postsCountPlural
    if (typeof template === 'string' && template.includes('{count}')) {
      return template.replace('{count}', count)
    }
    if (template && typeof template === 'string') {
      return template
    }
    return count === 1
      ? `${count} veröffentlichter Beitrag`
      : `${count} veröffentlichte Beiträge`
  }

  const hasContent = Boolean(page)

  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-100 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950 pb-16">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 pt-28 pb-10">
        <button
          onClick={() => navigate('/', { state: { scrollTo: null, from: 'dynamic-page' } })}
          className="group inline-flex items-center gap-2 text-primary-700 font-medium mb-6"
        >
          <ArrowLeft className="w-4 h-4 transition-transform duration-200 group-hover:-translate-x-1" />
          Zurück
        </button>

        {loading ? (
          <div className="flex flex-col items-center justify-center py-32 text-gray-500">
            <Loader2 className="w-10 h-10 animate-spin mb-4" />
            <p>Seite wird geladen…</p>
          </div>
        ) : error ? (
          <div className="rounded-3xl border border-red-200 bg-red-50 p-6 text-red-700 flex gap-3">
            <AlertCircle className="w-6 h-6 flex-shrink-0" />
            <div>
              <h2 className="font-semibold mb-1">Seite konnte nicht geladen werden</h2>
              <p className="text-sm">{error?.message || 'Unbekannter Fehler'}</p>
            </div>
          </div>
        ) : !hasContent ? (
          <div className="rounded-3xl border border-yellow-200 bg-yellow-50 p-6 text-yellow-800">
            Die angeforderte Seite wurde nicht gefunden oder ist nicht veröffentlicht.
          </div>
        ) : (
          <div className="space-y-12">
            <DynamicPageHero
              title={heroTitle}
              subtitle={heroSubtitle}
              badge={heroBadge}
              gradient={heroGradient}
            />

            <DynamicPageAbout
              title={aboutTitle}
              description={page.description}
            />

            <DynamicPagePostList
              posts={posts}
              title={postsTitle}
              emptyTitle={postsEmptyTitle}
              emptyMessage={postsEmptyMessage}
              countLabel={formatPostsCount(posts.length)}
              pageSlug={normalizedSlug}
            />
          </div>
        )}
      </div>
    </main>
  )
}

export default DynamicPage
