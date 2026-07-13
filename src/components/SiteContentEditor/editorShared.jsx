import PropTypes from 'prop-types'
import { AlertCircle, ArrowLeft, Check, Code, Loader2 } from 'lucide-react'

export const sectionLabels = {
  hero: 'Startseite: Hauptbereich (Hero)',
  stats: 'Startseite: Statistiken',
  cta_section: 'Startseite: Aufruf (CTA)',
  header: 'Navigation & Header',
  footer: 'Footer',
  site_meta: 'globale Meta-Daten (SEO)',
  login: 'Login Seite',
}

export const cloneContent = (value) => {
  if (value === undefined || value === null) {
    return {}
  }
  return JSON.parse(JSON.stringify(value))
}

export const setNestedValue = (obj, path, value) => {
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

export const SectionPicker = ({ sections, selected, onSelect }) => {
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
            className={`rounded-xl border px-4 py-3 text-left transition-all ${
              isActive
                ? 'border-primary-600 bg-primary-50 text-primary-700 shadow-sm'
                : [
                    'border-gray-200 text-gray-700 hover:border-primary-200 hover:bg-gray-50',
                    'dark:border-gray-600 dark:text-gray-200 dark:hover:bg-gray-700',
                  ].join(' ')
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

export const SectionToolbar = ({
  onBack,
  onReset,
  onSave,
  isSaving,
  hasChanges,
  showJson,
  onToggleJson,
}) => (
  <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <button
      type="button"
      onClick={onBack}
      className={`inline-flex items-center gap-2 text-sm font-medium text-primary-600
hover:text-primary-700 dark:text-primary-400 dark:hover:text-primary-300`}
    >
      <ArrowLeft className="h-4 w-4" />
      Zurück zur Auswahl
    </button>
    <div className="flex items-center gap-2">
      <button
        type="button"
        onClick={onToggleJson}
        className={`inline-flex items-center gap-2 rounded-lg border px-4 py-2 text-sm transition-colors ${
          showJson
            ? [
                'border-primary-200 bg-primary-50 text-primary-700 dark:border-primary-800',
                'dark:bg-primary-900/20 dark:text-primary-300',
              ].join(' ')
            : [
                'border-gray-200 bg-white text-gray-700 hover:bg-gray-50 dark:border-slate-600',
                'dark:bg-slate-800 dark:text-slate-200',
              ].join(' ')
        }`}
      >
        <Code className="h-4 w-4" />
        {showJson ? 'Editor ausblenden' : 'JSON-Editor'}
      </button>
      <button
        type="button"
        onClick={onReset}
        className={`rounded-lg border border-gray-200 bg-white px-4 py-2 text-sm text-gray-700
transition-colors hover:bg-gray-100 disabled:cursor-not-allowed
disabled:opacity-60 dark:border-slate-600 dark:bg-slate-800
dark:text-slate-200 dark:hover:bg-slate-700`}
        disabled={!hasChanges || isSaving}
      >
        Verwerfen
      </button>
      <button
        type="button"
        onClick={onSave}
        className={`inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600
to-primary-700 px-5 py-2 text-sm font-semibold text-white shadow-lg
transition-all hover:from-primary-700 hover:to-primary-800
disabled:cursor-not-allowed disabled:opacity-60`}
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

export const ContentJsonEditor = ({ value, onChange, error, schemaHint }) => (
  <div className="space-y-3">
    <label className="block text-sm font-semibold text-gray-700 dark:text-slate-200">
      JSON-Inhalt (Erweitert)
    </label>
    <textarea
      className={`min-h-[420px] w-full rounded-xl border border-gray-300 bg-white px-4 py-3
font-mono text-sm text-gray-900 shadow-sm focus:border-primary-500
focus:outline-none focus:ring-2 focus:ring-primary-200 dark:border-slate-600
dark:bg-slate-800 dark:text-slate-100 dark:placeholder:text-slate-400`}
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
      <div
        className={`flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-3
text-sm text-red-600`}
      >
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
