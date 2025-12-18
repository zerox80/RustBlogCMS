import { useCallback, useEffect, useMemo, useState } from 'react'
import PropTypes from 'prop-types'
import { AlertCircle, ArrowLeft, Check, Loader2, RefreshCw, Plus, Trash2, Code } from 'lucide-react'
import {
  useContent,
  DEFAULT_CONTENT,
} from '../../context/ContentContext'

const sectionLabels = {
  hero: 'Hero-Bereich (Startseite)',
  stats: 'Statistiken (Startseite)',
  cta_section: 'CTA-Bereich (Startseite)',
  header: 'Navigation & Header',
  footer: 'Footer',
  site_meta: 'Seitentitel & Beschreibung',
  login: 'Login Seite',
}

const cloneContent = (value) => {
  if (value === undefined || value === null) {
    return {}
  }
  return JSON.parse(JSON.stringify(value))
}

const setNestedValue = (obj, path, value) => {
  if (!Array.isArray(path) || path.length === 0) {
    return obj
  }
  let cursor = obj
  for (let i = 0; i < path.length - 1; i += 1) {
    const key = path[i]
    if (typeof cursor[key] !== 'object' || cursor[key] === null) {
      cursor[key] = {}
    }
    cursor = cursor[key]
  }
  cursor[path[path.length - 1]] = value
  return obj
}

const SectionPicker = ({ sections, selected, onSelect }) => {
  return (
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
      {sections.map((section) => {
        const label = sectionLabels[section] || section
        const isActive = selected === section
        return (
          <button
            key={section}
            type="button"
            onClick={() => onSelect(section)}
            className={`rounded-xl border px-4 py-3 text-left transition-all ${isActive
              ? 'border-primary-600 bg-primary-50 text-primary-700 shadow-sm'
              : 'border-gray-200 text-gray-700 hover:border-primary-200 hover:bg-gray-50 dark:border-gray-600 dark:text-gray-200 dark:hover:bg-gray-700'
              }`}
          >
            <p className="text-sm font-semibold">{label}</p>
            <p className="text-xs text-gray-500 dark:text-gray-400">{section}</p>
          </button>
        )
      })}
    </div>
  )
}

SectionPicker.propTypes = {
  sections: PropTypes.arrayOf(PropTypes.string).isRequired,
  selected: PropTypes.string,
  onSelect: PropTypes.func.isRequired,
}

const SectionToolbar = ({ onBack, onReset, onSave, isSaving, hasChanges, showJson, onToggleJson }) => (
  <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <button
      type="button"
      onClick={onBack}
      className="inline-flex items-center gap-2 text-sm font-medium text-primary-600 hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300"
    >
      <ArrowLeft className="h-4 w-4" />
      Zurück zur Auswahl
    </button>
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onToggleJson}
        className={`inline-flex items-center gap-2 rounded-lg border px-4 py-2 text-sm transition-colors ${showJson
          ? 'border-primary-200 bg-primary-50 text-primary-700 dark:border-primary-800 dark:bg-primary-900/20 dark:text-primary-300'
          : 'border-gray-200 bg-white text-gray-700 hover:bg-gray-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200'
          }`}
      >
        <Code className="h-4 w-4" />
        {showJson ? 'Editor ausblenden' : 'JSON-Editor'}
      </button>
      <button
        type="button"
        onClick={onReset}
        className="rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm text-gray-700 transition-colors hover:bg-gray-100 disabled:cursor-not-allowed disabled:opacity-60 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700"
        disabled={!hasChanges || isSaving}
      >
        Verwerfen
      </button>
      <button
        type="button"
        onClick={onSave}
        className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600 to-primary-700 px-5 py-2 text-sm font-semibold text-white shadow-lg transition-all hover:from-primary-700 hover:to-primary-800 disabled:cursor-not-allowed disabled:opacity-60"
        disabled={!hasChanges || isSaving}
      >
        {isSaving ? <Loader2 className="h-4 w-4 animate-spin" /> : <Check className="h-4 w-4" />}
        Speichern
      </button>
    </div>
  </div>
)

SectionToolbar.propTypes = {
  onBack: PropTypes.func.isRequired,
  onReset: PropTypes.func.isRequired,
  onSave: PropTypes.func.isRequired,
  isSaving: PropTypes.bool,
  hasChanges: PropTypes.bool,
  showJson: PropTypes.bool,
  onToggleJson: PropTypes.func,
}

const ContentJsonEditor = ({ value, onChange, error, schemaHint }) => (
  <div className="space-y-3">
    <label className="block text-sm font-semibold text-gray-700 dark:text-slate-200">JSON-Inhalt (Erweitert)</label>
    <textarea
      className="min-h-[420px] w-full rounded-xl border border-gray-300 bg-white px-4 py-3 font-mono text-sm text-gray-900 shadow-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100 dark:placeholder:text-slate-400"
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
    <div className="rounded-xl border border-gray-200 bg-gray-50 p-4 text-xs text-gray-600">
      <p className="font-semibold">Strukturhinweis</p>
      <pre className="mt-2 max-h-48 overflow-auto whitespace-pre-wrap text-gray-500">
        {schemaHint}
      </pre>
    </div>
    {error && (
      <div className="flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-600">
        <AlertCircle className="h-4 w-4" />
        <span>JSON-Fehler: {error}</span>
      </div>
    )}
  </div>
)

ContentJsonEditor.propTypes = {
  value: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
  error: PropTypes.string,
  schemaHint: PropTypes.string,
}

const FeaturesEditor = ({ features, onChange }) => {
  const items = Array.isArray(features) ? features : []

  const handleAdd = () => {
    const newItem = {
      icon: 'Star',
      title: 'Neues Feature',
      description: 'Beschreibung hier eingeben',
      color: 'text-blue-500',
      bg: 'bg-blue-500/10',
      border: 'border-blue-500/20',
    }
    onChange([...items, newItem])
  }

  const handleRemove = (index) => {
    const newItems = items.filter((_, i) => i !== index)
    onChange(newItems)
  }

  const handleChange = (index, field, value) => {
    const newItems = [...items]
    newItems[index] = { ...newItems[index], [field]: value }
    onChange(newItems)
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">Feature-Karten</label>
        <button
          type="button"
          onClick={handleAdd}
          className="inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200"
        >
          <Plus className="h-3 w-3" />
          Karte hinzufügen
        </button>
      </div>
      <div className="grid gap-4">
        {items.map((item, index) => (
          <div key={item.id || `feature-${index}`} className="relative rounded-xl border border-gray-200 bg-gray-50 p-4 dark:border-slate-700 dark:bg-slate-800/50">
            <button
              type="button"
              onClick={() => handleRemove(index)}
              className="absolute right-2 top-2 rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
              title="Entfernen"
            >
              <Trash2 className="h-4 w-4" />
            </button>
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label className="text-xs font-medium text-gray-500">Titel</label>
                <input
                  type="text"
                  className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={item.title || ''}
                  onChange={(e) => handleChange(index, 'title', e.target.value)}
                />
              </div>
              <div>
                <label className="text-xs font-medium text-gray-500">Icon (Lucide Name)</label>
                <input
                  type="text"
                  className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={item.icon || ''}
                  onChange={(e) => handleChange(index, 'icon', e.target.value)}
                />
              </div>
              <div className="sm:col-span-2">
                <label className="text-xs font-medium text-gray-500">Beschreibung</label>
                <textarea
                  className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  rows="2"
                  value={item.description || ''}
                  onChange={(e) => handleChange(index, 'description', e.target.value)}
                />
              </div>
            </div>
          </div>
        ))}
        {items.length === 0 && (
          <div className="rounded-xl border border-dashed border-gray-300 p-8 text-center text-sm text-gray-500">
            Keine Feature-Karten vorhanden.
          </div>
        )}
      </div>
    </div>
  )
}

FeaturesEditor.propTypes = {
  features: PropTypes.array,
  onChange: PropTypes.func.isRequired,
}

const StatsForm = ({ content, onFieldChange }) => {
  const stats = content || {}
  const items = Array.isArray(stats.items) ? stats.items : []

  const handleAdd = () => {
    const newItem = { label: 'Neuer Wert', value: '100+' }
    onFieldChange(['items'], [...items, newItem])
  }

  const handleRemove = (index) => {
    const newItems = items.filter((_, i) => i !== index)
    onFieldChange(['items'], newItems)
  }

  const handleChange = (index, field, value) => {
    const newItems = [...items]
    newItems[index] = { ...newItems[index], [field]: value }
    onFieldChange(['items'], newItems)
  }

  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Statistiken</h3>
        <button
          type="button"
          onClick={handleAdd}
          className="inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200"
        >
          <Plus className="h-3 w-3" />
          Statistik hinzufügen
        </button>
      </div>
      <div className="space-y-3">
        {items.map((item, index) => (
          <div key={item.label || `stat-${index}`} className="flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-slate-700 dark:bg-slate-800/50">
            <div className="flex-1 grid gap-3 sm:grid-cols-2">
              <div>
                <label className="text-xs font-medium text-gray-500">Wert (z.B. 10k+)</label>
                <input
                  type="text"
                  className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={item.value || ''}
                  onChange={(e) => handleChange(index, 'value', e.target.value)}
                />
              </div>
              <div>
                <label className="text-xs font-medium text-gray-500">Label (z.B. Leser)</label>
                <input
                  type="text"
                  className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={item.label || ''}
                  onChange={(e) => handleChange(index, 'label', e.target.value)}
                />
              </div>
            </div>
            <button
              type="button"
              onClick={() => handleRemove(index)}
              className="mt-6 rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
            >
              <Trash2 className="h-4 w-4" />
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

StatsForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const CtaSectionForm = ({ content, onFieldChange }) => {
  const cta = content || {}
  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">CTA-Bereich (Startseite)</h3>
      <div className="grid grid-cols-1 gap-4">
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={cta.title || ''}
            onChange={(e) => onFieldChange(['title'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Beschreibung</label>
          <textarea
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            rows="2"
            value={cta.description || ''}
            onChange={(e) => onFieldChange(['description'], e.target.value)}
          />
        </div>
      </div>
    </div>
  )
}

CtaSectionForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const HeroContentForm = ({ content, onFieldChange }) => {
  const heroContent = content || {}
  const title = heroContent.title || {}
  const handleChange = (path) => (event) => {
    onFieldChange(path, event.target.value)
  }

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Hero-Inhalt bearbeiten</h3>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Badge Text</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={heroContent.badgeText || ''}
              onChange={handleChange(['badgeText'])}
              placeholder="z. B. Neu"
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel Zeile 1</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={title.line1 || ''}
              onChange={handleChange(['title', 'line1'])}
              placeholder="z. B. Lerne Linux"
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel Zeile 2</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={title.line2 || ''}
              onChange={handleChange(['title', 'line2'])}
              placeholder="z. B. von Grund auf"
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Untertitel</label>
            <textarea
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              rows="2"
              value={heroContent.subtitle || ''}
              onChange={handleChange(['subtitle'])}
              placeholder="Kurze Beschreibung"
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Subline</label>
            <textarea
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              rows="2"
              value={heroContent.subline || ''}
              onChange={handleChange(['subline'])}
              placeholder="Zusätzlicher Satz unter dem Untertitel"
            />
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Einstellungen</h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Hero Bild URL</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={heroContent.heroImage || ''}
              onChange={handleChange(['heroImage'])}
              placeholder="/hero-dashboard-v2.png"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Primary Button Label</label>
              <input
                type="text"
                className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                value={heroContent.primaryCta?.label || ''}
                onChange={handleChange(['primaryCta', 'label'])}
              />
            </div>
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Secondary Button Label</label>
              <input
                type="text"
                className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                value={heroContent.secondaryCta?.label || ''}
                onChange={handleChange(['secondaryCta', 'label'])}
              />
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <FeaturesEditor
          features={heroContent.features}
          onChange={(newFeatures) => onFieldChange(['features'], newFeatures)}
        />
      </div>
    </div>
  )
}

HeroContentForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const LoginForm = ({ content, onFieldChange }) => {
  const loginContent = content || {}
  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Login Seite</h3>
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.title || ''}
            onChange={(e) => onFieldChange(['title'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Untertitel</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.subtitle || ''}
            onChange={(e) => onFieldChange(['subtitle'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Icon (Lucide Name)</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.icon || ''}
            onChange={(e) => onFieldChange(['icon'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Button Label</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.buttonLabel || ''}
            onChange={(e) => onFieldChange(['buttonLabel'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Benutzername Label</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.usernameLabel || ''}
            onChange={(e) => onFieldChange(['usernameLabel'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Passwort Label</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.passwordLabel || ''}
            onChange={(e) => onFieldChange(['passwordLabel'], e.target.value)}
          />
        </div>
        <div className="md:col-span-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Zurück Link Text</label>
          <input
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
            value={loginContent.backLinkText || ''}
            onChange={(e) => onFieldChange(['backLinkText'], e.target.value)}
          />
        </div>
      </div>
    </div>
  )
}

LoginForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const SiteMetaForm = ({ content, onFieldChange }) => {
  const siteMeta = content || {}
  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Seitentitel & Beschreibung</h3>
      <div className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300" htmlFor="site-meta-title">
            Browser-Titel
          </label>
          <input
            id="site-meta-title"
            type="text"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100 dark:placeholder:text-slate-400"
            value={siteMeta.title || ''}
            onChange={(event) => onFieldChange(['title'], event.target.value)}
            placeholder="z. B. IT Portal"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300" htmlFor="site-meta-description">
            Meta-Beschreibung
          </label>
          <textarea
            id="site-meta-description"
            className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100 dark:placeholder:text-slate-400"
            rows="3"
            value={siteMeta.description || ''}
            onChange={(event) => onFieldChange(['description'], event.target.value)}
            placeholder="Kurze Beschreibung, die in Suchergebnissen angezeigt wird"
          />
          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
            Empfehlung: 50–160 Zeichen, enthält wichtige Schlüsselbegriffe.
          </p>
        </div>
      </div>
    </div>
  )
}

SiteMetaForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const HeaderForm = ({ content, onFieldChange }) => {
  const header = content || {}
  const brand = header.brand || {}
  const navItems = Array.isArray(header.navItems) ? header.navItems : []

  const handleNavChange = (index, field, value) => {
    const newItems = [...navItems]
    newItems[index] = { ...newItems[index], [field]: value }
    onFieldChange(['navItems'], newItems)
  }

  const handleAddNav = () => {
    onFieldChange(['navItems'], [...navItems, { id: `nav-${Date.now()}`, label: 'Neuer Link', type: 'route', path: '/' }])
  }

  const handleRemoveNav = (index) => {
    const newItems = navItems.filter((_, i) => i !== index)
    onFieldChange(['navItems'], newItems)
  }

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Marke & Logo</h3>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Markenname</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={brand.name || ''}
              onChange={(e) => onFieldChange(['brand', 'name'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Icon (Lucide Name)</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={brand.icon || ''}
              onChange={(e) => onFieldChange(['brand', 'icon'], e.target.value)}
            />
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Navigation</h3>
          <button
            type="button"
            onClick={handleAddNav}
            className="inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200"
          >
            <Plus className="h-3 w-3" />
            Link hinzufügen
          </button>
        </div>
        <div className="space-y-3">
          {navItems.map((item, index) => (
            <div key={item.id || `nav-${index}`} className="flex items-start gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-slate-700 dark:bg-slate-800/50">
              <div className="flex-1 grid gap-3 sm:grid-cols-3">
                <div>
                  <label className="text-xs font-medium text-gray-500">Label</label>
                  <input
                    type="text"
                    className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                    value={item.label || ''}
                    onChange={(e) => handleNavChange(index, 'label', e.target.value)}
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Typ</label>
                  <select
                    className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                    value={item.type || 'route'}
                    onChange={(e) => handleNavChange(index, 'type', e.target.value)}
                  >
                    <option value="route">Interne Route</option>
                    <option value="section">Scroll-Sektion</option>
                    <option value="external">Externer Link</option>
                  </select>
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Pfad / URL / ID</label>
                  <input
                    type="text"
                    className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                    value={item.path || item.value || ''}
                    onChange={(e) => handleNavChange(index, 'path', e.target.value)}
                  />
                </div>
              </div>
              <button
                type="button"
                onClick={() => handleRemoveNav(index)}
                className="mt-6 rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

HeaderForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}

const FooterForm = ({ content, onFieldChange }) => {
  const footer = content || {}
  const brand = footer.brand || {}
  const bottom = footer.bottom || {}
  const quickLinks = Array.isArray(footer.quickLinks) ? footer.quickLinks : []

  const handleLinkChange = (index, field, value) => {
    const newLinks = [...quickLinks]
    newLinks[index] = { ...newLinks[index], [field]: value }
    onFieldChange(['quickLinks'], newLinks)
  }

  const handleAddLink = () => {
    onFieldChange(['quickLinks'], [...quickLinks, { label: 'Neuer Link', path: '/' }])
  }

  const handleRemoveLink = (index) => {
    const newLinks = quickLinks.filter((_, i) => i !== index)
    onFieldChange(['quickLinks'], newLinks)
  }

  return (
    <div className="space-y-6">
      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Footer Marke</h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={brand.title || ''}
              onChange={(e) => onFieldChange(['brand', 'title'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Beschreibung</label>
            <textarea
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              rows="2"
              value={brand.description || ''}
              onChange={(e) => onFieldChange(['brand', 'description'], e.target.value)}
            />
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Quick Links</h3>
          <button
            type="button"
            onClick={handleAddLink}
            className="inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200"
          >
            <Plus className="h-3 w-3" />
            Link hinzufügen
          </button>
        </div>
        <div className="space-y-3">
          {quickLinks.map((link, index) => (
            <div key={link.label || `link-${index}`} className="flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3 dark:border-slate-700 dark:bg-slate-800/50">
              <div className="flex-1 grid gap-3 sm:grid-cols-2">
                <input
                  type="text"
                  placeholder="Label"
                  className="w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={link.label || ''}
                  onChange={(e) => handleLinkChange(index, 'label', e.target.value)}
                />
                <input
                  type="text"
                  placeholder="Pfad / URL"
                  className="w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
                  value={link.path || link.href || ''}
                  onChange={(e) => handleLinkChange(index, 'path', e.target.value)}
                />
              </div>
              <button
                type="button"
                onClick={() => handleRemoveLink(index)}
                className="rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-900/20"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      </div>

      <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">Copyright & Signatur</h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Copyright Text ({'{year}'} Platzhalter)</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={bottom.copyright || ''}
              onChange={(e) => onFieldChange(['bottom', 'copyright'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Signatur (Rechts unten)</label>
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100"
              value={bottom.signature || ''}
              onChange={(e) => onFieldChange(['bottom', 'signature'], e.target.value)}
            />
          </div>
        </div>
      </div>
    </div>
  )
}

FooterForm.propTypes = {
  content: PropTypes.object,
  onFieldChange: PropTypes.func.isRequired,
}





const HeroPreview = ({ content }) => {
  return (
    <div className="relative overflow-hidden rounded-2xl bg-slate-900 p-8 shadow-xl">
      <div className="absolute inset-0 bg-[linear-gradient(to_right,#4f4f4f2e_1px,transparent_1px),linear-gradient(to_bottom,#4f4f4f2e_1px,transparent_1px)] bg-[size:14px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]"></div>
      <div className="relative z-10 text-center">
        <div className="mb-6 flex justify-center">
          <span className="inline-flex items-center gap-2 rounded-full border border-primary-500/30 bg-primary-500/10 px-4 py-1.5 text-sm font-medium text-primary-400 backdrop-blur-sm">
            <span className="relative flex h-2 w-2">
              <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-primary-400 opacity-75"></span>
              <span className="relative inline-flex h-2 w-2 rounded-full bg-primary-500"></span>
            </span>
            {content.badgeText || 'IT Portal'}
          </span>
        </div>
        <h3 className="mb-6 text-3xl font-bold tracking-tight text-white sm:text-4xl">
          {content?.title?.line1}
          <span className="block bg-gradient-to-r from-primary-400 to-purple-400 bg-clip-text text-transparent">
            {content?.title?.line2}
          </span>
        </h3>
        <p className="mx-auto mb-8 max-w-2xl text-lg text-slate-400">
          {content.subtitle}
        </p>
        {content.subline && (
          <p className="text-sm text-slate-500">{content.subline}</p>
        )}
      </div>
    </div>
  )
}

HeroPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const StatsPreview = ({ content }) => {
  const items = Array.isArray(content.items) ? content.items : []
  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <div className="flex items-center justify-between mb-6">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">Statistiken Vorschau</h4>
        <span className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs font-medium text-primary-700">
          Vorschau
        </span>
      </div>
      <div className="rounded-xl bg-slate-950 p-8">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
          {items.map((stat, i) => (
            <div key={i}>
              <div className="text-2xl font-bold text-white mb-1">{stat.value}</div>
              <div className="text-sm text-slate-500 font-medium">{stat.label}</div>
            </div>
          ))}
          {items.length === 0 && (
            <div className="col-span-full text-slate-500">Keine Statistiken vorhanden.</div>
          )}
        </div>
      </div>
    </div>
  )
}

StatsPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const CtaSectionPreview = ({ content }) => {
  return (
    <div className="rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <div className="flex items-center justify-between mb-6">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">CTA Vorschau</h4>
        <span className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs font-medium text-primary-700">
          Vorschau
        </span>
      </div>
      <div className="rounded-xl bg-slate-950 p-8 text-center">
        <h2 className="text-2xl font-bold mb-4 text-white">{content.title || 'Wissen teilen & erweitern'}</h2>
        <p className="text-lg text-slate-400">
          {content.description || 'Bleib auf dem Laufenden mit den neuesten Entwicklungen in der IT-Welt.'}
        </p>
      </div>
    </div>
  )
}

CtaSectionPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const SiteMetaPreview = ({ content }) => {
  return (
    <div className="space-y-4 rounded-2xl border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-700 dark:bg-slate-900/80">
      <div className="flex items-center justify-between">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">Seitentitel (Tab)</h4>
        <span className="rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs font-medium text-primary-700">
          Vorschau
        </span>
      </div>
      <div className="rounded-xl border border-gray-100 bg-gray-50 p-4">
        <p className="text-sm font-semibold text-gray-800">{content?.title || 'IT Portal - Security, Programming & Admin'}</p>
        <p className="mt-2 text-sm text-gray-600 leading-relaxed">
          {content?.description || 'Dein Portal für IT Security, Programmierung und Administration.'}
        </p>
      </div>
      <p className="text-xs text-gray-500">
        Diese Angaben erscheinen als Browser-Titel und Meta-Beschreibung (z. B. in Suchmaschinen).
      </p>
    </div>
  )
}

SiteMetaPreview.propTypes = {
  content: PropTypes.object.isRequired,
}



const SectionPreview = ({ section, content }) => {
  switch (section) {
    case 'hero':
      return <HeroPreview content={content} />
    case 'stats':
      return <StatsPreview content={content} />
    case 'cta_section':
      return <CtaSectionPreview content={content} />
    case 'site_meta':
      return <SiteMetaPreview content={content} />
    default:
      return (
        <div className="rounded-2xl border border-gray-200 bg-white p-6 text-sm text-gray-600 shadow-sm">
          <p className="font-semibold text-gray-700">Vorschau nicht verfügbar</p>
          <p className="mt-2 text-gray-500">
            Nutze das Formular oben, um die Inhalte zu bearbeiten.
          </p>
        </div>
      )
  }
}

SectionPreview.propTypes = {
  section: PropTypes.string.isRequired,
  content: PropTypes.object.isRequired,
}

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
    return combined.filter(key => Object.prototype.hasOwnProperty.call(sectionLabels, key)).sort()
  }, [content])

  const [selectedSection, setSelectedSection] = useState(null)
  const [originalContent, setOriginalContent] = useState(null)
  const [draftContent, setDraftContent] = useState(null)
  const [editorValue, setEditorValue] = useState('')
  const [jsonError, setJsonError] = useState(null)
  const [status, setStatus] = useState(null)
  const [showJson, setShowJson] = useState(false)

  const activeContent = useMemo(() => {
    if (!selectedSection) {
      return null
    }
    if (content && content[selectedSection] !== undefined) {
      return content[selectedSection]
    }
    return getSection(selectedSection)
  }, [content, getSection, selectedSection])

  const handleSectionSelect = useCallback((section) => {
    setSelectedSection(section)
    setStatus(null)

    const sectionContent = (content && content[section]) ?? getSection(section)
    const current = cloneContent(sectionContent ?? getDefaultSection(section) ?? DEFAULT_CONTENT[section])
    setOriginalContent(current)
    setDraftContent(current)
    setEditorValue(JSON.stringify(current, null, 2))
    setJsonError(null)
    setShowJson(false)
  }, [content, getSection, getDefaultSection])

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
          <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Seiteninhalte verwalten</h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Bearbeite Texte, Navigation und weitere statische Inhalte.
          </p>
        </div>
        <button
          type="button"
          onClick={refreshContent}
          className="inline-flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium text-gray-700 transition-all hover:border-primary-200 hover:text-primary-700"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          {loading ? 'Aktualisiere…' : 'Inhalte neu laden'}
        </button>
      </div>

      {error && !selectedSection && (
        <div className="flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
          <AlertCircle className="h-4 w-4" />
          <div>
            <p className="font-semibold">Fehler beim Laden der Inhalte</p>
            <p>{error?.message || String(error)}</p>
          </div>
        </div>
      )}

      {!selectedSection && (
        <div className="space-y-4">
          <p className="text-sm font-medium text-gray-700 dark:text-gray-200">Wähle einen Inhaltsbereich aus:</p>
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
              className={`flex items-start gap-2 rounded-lg border p-3 text-sm ${status.type === 'success'
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

          {!showJson && (
            <SectionPreview section={selectedSection} content={draftContent || {}} />
          )}
        </div>
      )}
    </div>
  )
}

export default SiteContentEditor
