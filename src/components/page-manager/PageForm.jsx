import { useMemo, useState } from 'react'
import PropTypes from 'prop-types'
import { AlertCircle, FilePlus, RefreshCw, X, ChevronDown, ChevronUp } from 'lucide-react'
import { normalizeTitle } from '../../utils/postUtils'
import { sanitizeSlug, isValidSlug } from '../../utils/slug'
import { parseJsonField, sanitizeInteger } from './formUtils'

const defaultHeroJson = JSON.stringify(
  {
    badge: 'Neue Seite',
    title: 'Titel der Seite',
    subtitle: 'Kurzbeschreibung deiner Seite',
    backgroundGradient: 'from-primary-600 to-primary-700',
  },
  null,
  2,
)
const defaultHeroTitle = normalizeTitle(JSON.parse(defaultHeroJson).title, '')
const defaultLayoutConfig = {
  aboutSection: {
    title: 'Über diese Seite',
  },
  postsSection: {
    title: 'Beiträge',
    emptyTitle: 'Keine Beiträge vorhanden',
    emptyMessage: 'Sobald für diese Seite Beiträge veröffentlicht werden, erscheinen sie hier.',
    countLabelSingular: '{count} veröffentlichter Beitrag',
    countLabelPlural: '{count} veröffentlichte Beiträge',
  },
}
const defaultLayoutJson = JSON.stringify(defaultLayoutConfig, null, 2)

const PageForm = ({ mode, initialData, onSubmit, onCancel, submitting }) => {
  const [title, setTitle] = useState(initialData?.title ?? '')
  const [slug, setSlug] = useState(initialData?.slug ?? '')
  const [description, setDescription] = useState(initialData?.description ?? '')
  const [navLabel, setNavLabel] = useState(initialData?.nav_label ?? '')
  const [showInNav, setShowInNav] = useState(Boolean(initialData?.show_in_nav))
  const [isPublished, setIsPublished] = useState(Boolean(initialData?.is_published))
  const [orderIndex, setOrderIndex] = useState(initialData?.order_index ?? 0)

  const [hero, setHero] = useState(
    initialData?.hero ? JSON.stringify(initialData.hero, null, 2) : defaultHeroJson,
  )
  const [layout, setLayout] = useState(
    initialData?.layout ? JSON.stringify(initialData.layout, null, 2) : defaultLayoutJson,
  )

  const [heroTitle, setHeroTitle] = useState(() => {
    if (initialData?.hero) {
      return normalizeTitle(initialData.hero.title ?? initialData.hero, initialData?.title ?? '')
    }
    return initialData?.title ?? defaultHeroTitle
  })
  const [heroTitleDirty, setHeroTitleDirty] = useState(false)

  const [showAdvanced, setShowAdvanced] = useState(false)
  const [error, setError] = useState(null)

  const formSanitizedSlug = useMemo(() => sanitizeSlug(slug), [slug])
  const slugHasInput = slug.trim().length > 0
  const slugHasInvalidCharacters = slugHasInput && !formSanitizedSlug

  // Helper to safely parse JSON state
  const getParsedState = (jsonString, fallback) => {
    try {
      return JSON.parse(jsonString) || fallback
    } catch {
      return fallback
    }
  }

  const heroState = getParsedState(hero, {})
  const layoutState = getParsedState(layout, defaultLayoutConfig)

  const updateHeroField = (field, value) => {
    const newHero = { ...heroState, [field]: value }
    setHero(JSON.stringify(newHero, null, 2))
    if (field === 'title') {
      setHeroTitle(value)
      setHeroTitleDirty(true)
    }
  }

  const updateLayoutField = (section, field, value) => {
    const newLayout = {
      ...layoutState,
      [section]: {
        ...(layoutState[section] || {}),
        [field]: value,
      },
    }
    setLayout(JSON.stringify(newLayout, null, 2))
  }

  const handleSubmit = async (event) => {
    event.preventDefault()
    setError(null)
    try {
      const trimmedTitle = title.trim()
      const trimmedDescription = description.trim()
      const trimmedNavLabel = navLabel.trim()
      const heroPayloadRaw = parseJsonField(hero, 'Hero JSON')
      const heroPayload =
        typeof heroPayloadRaw === 'object' && heroPayloadRaw !== null ? { ...heroPayloadRaw } : {}
      const trimmedHeroTitle = heroTitle.trim()
      const trimmedSlug = slug.trim()
      const sanitizedSlug = sanitizeSlug(trimmedSlug)

      if (heroTitleDirty) {
        if (trimmedHeroTitle) {
          heroPayload.title = trimmedHeroTitle
        } else {
          delete heroPayload.title
        }
      }
      if (!heroPayload.title) {
        heroPayload.title = trimmedTitle
      }

      if (!trimmedTitle) {
        throw new Error('Titel darf nicht leer sein.')
      }
      if (!sanitizedSlug) {
        throw new Error('Slug darf nur Kleinbuchstaben, Zahlen und Bindestriche enthalten.')
      }
      if (!isValidSlug(sanitizedSlug)) {
        throw new Error('Slug ist ungültig.')
      }

      setSlug(sanitizedSlug)

      const payload = {
        title: trimmedTitle,
        slug: sanitizedSlug,
        description: trimmedDescription,
        nav_label: trimmedNavLabel ? trimmedNavLabel : null,
        show_in_nav: showInNav,
        is_published: isPublished,
        order_index: sanitizeInteger(orderIndex),
        hero: heroPayload,
        layout: parseJsonField(layout, 'Layout JSON'),
      }
      await onSubmit(payload)
    } catch (err) {
      setError(err)
    }
  }

  return (
    <div
      className={`bg-white rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh]
overflow-y-auto dark:bg-slate-900`}
    >
      <div
        className={`flex items-center justify-between px-6 py-4 border-b border-gray-100
dark:border-slate-800`}
      >
        <div>
          <h3 className="text-xl font-semibold text-gray-900 dark:text-slate-100">
            {mode === 'edit' ? 'Seite bearbeiten' : 'Neue Seite erstellen'}
          </h3>
          <p className="text-sm text-gray-500 dark:text-slate-400">
            Konfiguriere die Darstellung und Inhalte deiner Seite.
          </p>
        </div>
        <button
          type="button"
          onClick={onCancel}
          className={`p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100
dark:text-slate-400 dark:hover:text-slate-200 dark:hover:bg-slate-800`}
        >
          <X className="w-5 h-5" />
        </button>
      </div>

      <form onSubmit={handleSubmit} className="space-y-8 px-6 py-6">
        {error && (
          <div
            className={`flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-3
text-sm text-red-700 dark:border-red-900/50 dark:bg-red-900/20
dark:text-red-300`}
          >
            <AlertCircle className="w-4 h-4 mt-0.5" />
            <div>
              <p className="font-medium">Speichern fehlgeschlagen</p>
              <p>{error.message}</p>
            </div>
          </div>
        )}

        {/* Basic Info Section */}
        <div className="space-y-4">
          <h4
            className={`text-lg font-medium text-gray-900 dark:text-slate-100 border-b
border-gray-100 pb-2 dark:border-slate-800`}
          >
            Allgemein
          </h4>
          <div className="grid gap-4 md:grid-cols-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Seitentitel (Intern)
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={title}
                onChange={(event) => setTitle(event.target.value)}
                required
              />
            </label>
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              URL Slug
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={slug}
                onChange={(event) => setSlug(event.target.value)}
                onBlur={() => setSlug(formSanitizedSlug)}
                required
              />
              {slugHasInvalidCharacters && (
                <p className="mt-1 text-xs text-red-600 dark:text-red-400">
                  Nur Kleinbuchstaben, Zahlen und Bindestriche erlaubt.
                </p>
              )}
            </label>
          </div>
          <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
            Beschreibung (Meta & Übersicht)
            <textarea
              className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
              rows={3}
              value={description}
              onChange={(event) => setDescription(event.target.value)}
              placeholder="Kurzbeschreibung der Seite"
            />
          </label>
        </div>

        {/* Navigation Section */}
        <div className="space-y-4">
          <h4
            className={`text-lg font-medium text-gray-900 dark:text-slate-100 border-b
border-gray-100 pb-2 dark:border-slate-800`}
          >
            Navigation
          </h4>
          <div className="grid gap-4 md:grid-cols-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Label im Menü
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={navLabel}
                onChange={(event) => setNavLabel(event.target.value)}
                placeholder={title}
              />
            </label>
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Reihenfolge
              <input
                type="number"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={orderIndex}
                onChange={(event) => setOrderIndex(event.target.value)}
              />
            </label>
          </div>
          <div className="flex flex-wrap items-center gap-6 pt-2">
            <label
              className={`inline-flex items-center gap-2 text-sm text-gray-700 dark:text-slate-200
cursor-pointer`}
            >
              <input
                type="checkbox"
                className={`h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500
dark:border-slate-600 dark:bg-slate-900`}
                checked={showInNav}
                onChange={(event) => setShowInNav(event.target.checked)}
              />
              In Navigation anzeigen
            </label>
            <label
              className={`inline-flex items-center gap-2 text-sm text-gray-700 dark:text-slate-200
cursor-pointer`}
            >
              <input
                type="checkbox"
                className={`h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500
dark:border-slate-600 dark:bg-slate-900`}
                checked={isPublished}
                onChange={(event) => setIsPublished(event.target.checked)}
              />
              Veröffentlicht
            </label>
          </div>
        </div>

        {/* Hero Configuration */}
        <div className="space-y-4">
          <h4
            className={`text-lg font-medium text-gray-900 dark:text-slate-100 border-b
border-gray-100 pb-2 dark:border-slate-800`}
          >
            Hero Bereich
          </h4>
          <div className="grid gap-4 md:grid-cols-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Badge Text
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={heroState.badge || ''}
                onChange={(e) => updateHeroField('badge', e.target.value)}
                placeholder="z.B. Neu"
              />
            </label>
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Überschrift
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={heroState.title || heroTitle}
                onChange={(e) => updateHeroField('title', e.target.value)}
                placeholder="Große Überschrift"
              />
            </label>
            <div className="md:col-span-2">
              <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
                Untertitel
                <textarea
                  className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                  rows={2}
                  value={heroState.subtitle || ''}
                  onChange={(e) => updateHeroField('subtitle', e.target.value)}
                  placeholder="Erklärender Text unter der Überschrift"
                />
              </label>
            </div>
          </div>
        </div>

        {/* Layout Configuration */}
        <div className="space-y-4">
          <h4
            className={`text-lg font-medium text-gray-900 dark:text-slate-100 border-b
border-gray-100 pb-2 dark:border-slate-800`}
          >
            Seiten-Layout
          </h4>
          <div className="grid gap-4 md:grid-cols-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Titel &quot;Über diese Seite&quot;
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={layoutState.aboutSection?.title || ''}
                onChange={(e) => updateLayoutField('aboutSection', 'title', e.target.value)}
                placeholder="Über diese Seite"
              />
            </label>
            <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
              Titel &quot;Beiträge&quot;
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                value={layoutState.postsSection?.title || ''}
                onChange={(e) => updateLayoutField('postsSection', 'title', e.target.value)}
                placeholder="Beiträge"
              />
            </label>
          </div>
        </div>

        {/* Advanced JSON Toggle */}
        <div className="pt-4 border-t border-gray-100 dark:border-slate-800">
          <button
            type="button"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className={`flex items-center gap-2 text-sm font-medium text-gray-500
hover:text-gray-700 dark:text-slate-400 dark:hover:text-slate-200`}
          >
            {showAdvanced ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            Erweiterte Einstellungen (JSON)
          </button>

          {showAdvanced && (
            <div className="mt-4 grid gap-4 lg:grid-cols-2 animate-fade-in-down">
              <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
                Hero JSON (Raw)
                <textarea
                  className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm font-mono
focus:border-primary-500 focus:outline-none focus:ring-2
focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900
dark:text-slate-100`}
                  rows={8}
                  value={hero}
                  onChange={(event) => {
                    const { value } = event.target
                    setHero(value)
                    try {
                      const parsed = JSON.parse(value)
                      if (!heroTitleDirty) {
                        const derivedTitle = normalizeTitle(parsed?.title ?? parsed, '').trim()
                        setHeroTitle((previous) =>
                          derivedTitle !== previous ? derivedTitle : previous,
                        )
                      }
                    } catch {
                      // Ignore invalid JSON while the user is editing the raw field.
                    }
                  }}
                />
              </label>
              <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
                Layout JSON (Raw)
                <textarea
                  className={`mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm font-mono
focus:border-primary-500 focus:outline-none focus:ring-primary-100
dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100`}
                  rows={8}
                  value={layout}
                  onChange={(event) => setLayout(event.target.value)}
                />
              </label>
            </div>
          )}
        </div>

        {/* Form Actions */}
        <div className="flex justify-end gap-3 pt-2">
          <button
            type="button"
            onClick={onCancel}
            className={`inline-flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2
text-sm font-medium text-gray-600 hover:bg-gray-50 dark:border-slate-700
dark:text-slate-200 dark:hover:bg-slate-800`}
          >
            Abbrechen
          </button>
          <button
            type="submit"
            className={`inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600
to-primary-700 px-5 py-2.5 text-sm font-semibold text-white shadow-lg
hover:from-primary-700 hover:to-primary-800`}
            disabled={submitting}
          >
            {submitting ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <FilePlus className="h-4 w-4" />
            )}
            <span>{mode === 'edit' ? 'Änderungen speichern' : 'Seite erstellen'}</span>
          </button>
        </div>
      </form>
    </div>
  )
}

PageForm.propTypes = {
  mode: PropTypes.oneOf(['create', 'edit']).isRequired,
  initialData: PropTypes.object,
  onSubmit: PropTypes.func.isRequired,
  onCancel: PropTypes.func.isRequired,
  submitting: PropTypes.bool,
}

export default PageForm
