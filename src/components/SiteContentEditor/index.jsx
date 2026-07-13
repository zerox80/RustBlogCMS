import { useCallback, useMemo, useState } from 'react'
import { AlertCircle, RefreshCw } from 'lucide-react'
import { useContent, DEFAULT_CONTENT } from '../../context/ContentContext'
import { LoginForm, SiteMetaForm } from './AccountContentForms'
import { CtaSectionForm, HeroContentForm, StatsForm } from './HomepageContentForms'
import { FooterForm, HeaderForm } from './NavigationContentForms'
import { AboutContentForm } from './EditorialContentForms'
import { SectionPreview } from './SectionPreview'
import {
  cloneContent,
  ContentJsonEditor,
  sectionLabels,
  SectionPicker,
  SectionToolbar,
  setNestedValue,
} from './editorShared'

const SiteContentEditor = () => {
  const {
    content,
    loading,
    error,
    refreshContent,
    getSection,
    getDefaultSection,
    updateSection,
    savingSections,
  } = useContent()

  const sectionOptions = useMemo(() => {
    const keys = Object.keys(content || {})
    const defaultKeys = Object.keys(DEFAULT_CONTENT)
    const combined = Array.from(new Set([...keys, ...defaultKeys]))
    // Only show sections that have a defined label (whitelist approach)
    return combined.filter((key) => Object.prototype.hasOwnProperty.call(sectionLabels, key)).sort()
  }, [content])

  const [selectedSection, setSelectedSection] = useState(null)
  const [originalContent, setOriginalContent] = useState(null)
  const [draftContent, setDraftContent] = useState(null)
  const [editorValue, setEditorValue] = useState('')
  const [jsonError, setJsonError] = useState(null)
  const [status, setStatus] = useState(null)
  const [showJson, setShowJson] = useState(false)

  const handleSectionSelect = useCallback(
    (section) => {
      setSelectedSection(section)
      setStatus(null)

      const sectionContent = (content && content[section]) ?? getSection(section)
      const current = cloneContent(
        sectionContent ?? getDefaultSection(section) ?? DEFAULT_CONTENT[section],
      )
      setOriginalContent(current)
      setDraftContent(current)
      setEditorValue(JSON.stringify(current, null, 2))
      setJsonError(null)
      setShowJson(false)
    },
    [content, getSection, getDefaultSection],
  )

  const handleEditorChange = useCallback((value) => {
    setEditorValue(value)
    try {
      const parsed = JSON.parse(value)
      setDraftContent(parsed)
      setJsonError(null)
    } catch (err) {
      setJsonError(err.message)
    }
  }, [])

  const handleReset = useCallback(() => {
    if (!originalContent) {
      return
    }
    const resetValue = cloneContent(originalContent)
    setDraftContent(resetValue)
    setEditorValue(JSON.stringify(resetValue, null, 2))
    setJsonError(null)
    setStatus(null)
  }, [originalContent])

  const handleBack = useCallback(() => {
    setSelectedSection(null)
    setOriginalContent(null)
    setDraftContent(null)
    setEditorValue('')
    setJsonError(null)
    setStatus(null)
  }, [])

  const handleSave = useCallback(async () => {
    if (!selectedSection || !draftContent || jsonError) {
      return
    }
    setStatus(null)
    try {
      const response = await updateSection(selectedSection, draftContent)
      const updated = cloneContent(response?.content ?? draftContent)
      setOriginalContent(updated)
      setDraftContent(updated)
      setEditorValue(JSON.stringify(updated, null, 2))
      setStatus({ type: 'success', message: 'Inhalt erfolgreich gespeichert.' })
    } catch (err) {
      setStatus({ type: 'error', message: err?.message || 'Speichern fehlgeschlagen.' })
    }
  }, [draftContent, jsonError, selectedSection, updateSection])

  const hasChanges = useMemo(() => {
    if (!selectedSection || !draftContent || !originalContent || jsonError) {
      return false
    }
    return JSON.stringify(draftContent) !== JSON.stringify(originalContent)
  }, [draftContent, jsonError, originalContent, selectedSection])

  const isSaving = selectedSection ? Boolean(savingSections?.[selectedSection]) : false

  const schemaHint = useMemo(() => {
    if (!selectedSection) {
      return ''
    }
    const base = getDefaultSection(selectedSection) ?? DEFAULT_CONTENT[selectedSection] ?? {}
    return JSON.stringify(base, null, 2)
  }, [getDefaultSection, selectedSection])

  const handleStructuredFieldChange = useCallback(
    (path, value) => {
      if (!selectedSection) {
        return
      }
      setDraftContent((prev) => {
        const fallback =
          getDefaultSection(selectedSection) ?? DEFAULT_CONTENT[selectedSection] ?? {}
        const base = cloneContent(prev ?? fallback)
        setNestedValue(base, path, value)
        setEditorValue(JSON.stringify(base, null, 2))
        setJsonError(null)
        return base
      })
    },
    [getDefaultSection, selectedSection],
  )

  return (
    <div className="space-y-8">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
            Seiteninhalte verwalten
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Bearbeite Texte, Navigation und weitere statische Inhalte.
          </p>
        </div>
        <button
          type="button"
          onClick={refreshContent}
          className={`inline-flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2
text-sm font-medium text-gray-700 transition-all hover:border-primary-200
hover:text-primary-700`}
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          {loading ? 'Aktualisiere…' : 'Inhalte neu laden'}
        </button>
      </div>

      {error && !selectedSection && (
        <div
          className={`flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-4
text-sm text-red-700`}
        >
          <AlertCircle className="h-4 w-4" />
          <div>
            <p className="font-semibold">Fehler beim Laden der Inhalte</p>
            <p>{error?.message || String(error)}</p>
          </div>
        </div>
      )}

      {!selectedSection && (
        <div className="space-y-4">
          <p className="text-sm font-medium text-gray-700 dark:text-gray-200">
            Wähle einen Inhaltsbereich aus:
          </p>
          <SectionPicker
            sections={sectionOptions}
            selected={selectedSection}
            onSelect={handleSectionSelect}
          />
        </div>
      )}

      {selectedSection && (
        <div className="space-y-6">
          <SectionToolbar
            onBack={handleBack}
            onReset={handleReset}
            onSave={handleSave}
            isSaving={isSaving}
            hasChanges={hasChanges}
            showJson={showJson}
            onToggleJson={() => setShowJson(!showJson)}
          />

          {status && (
            <div
              className={`flex items-start gap-2 rounded-lg border p-3 text-sm ${
                status.type === 'success'
                  ? 'border-green-200 bg-green-50 text-green-700'
                  : 'border-red-200 bg-red-50 text-red-700'
              }`}
            >
              <AlertCircle className="h-4 w-4" />
              <span>{status.message}</span>
            </div>
          )}

          {/* Structured Editors */}
          {selectedSection === 'hero' && (
            <HeroContentForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'stats' && (
            <StatsForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'cta_section' && (
            <CtaSectionForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'about' && (
            <AboutContentForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'login' && (
            <LoginForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'site_meta' && (
            <SiteMetaForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'header' && (
            <HeaderForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {selectedSection === 'footer' && (
            <FooterForm content={draftContent} onFieldChange={handleStructuredFieldChange} />
          )}

          {/* JSON Editor (Toggleable) */}
          {showJson && (
            <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
              <ContentJsonEditor
                value={editorValue}
                onChange={handleEditorChange}
                error={jsonError}
                schemaHint={schemaHint}
              />
              <SectionPreview section={selectedSection} content={draftContent || {}} />
            </div>
          )}

          {!showJson && <SectionPreview section={selectedSection} content={draftContent || {}} />}
        </div>
      )}
    </div>
  )
}

export default SiteContentEditor
