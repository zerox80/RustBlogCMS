import PropTypes from 'prop-types'
import { Plus, Trash2 } from 'lucide-react'

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
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Feature-Karten
        </label>
        <button
          type="button"
          onClick={handleAdd}
          className={`inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white
px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200`}
        >
          <Plus className="h-3 w-3" />
          Karte hinzufügen
        </button>
      </div>
      <div className="grid gap-4">
        {items.map((item, index) => (
          <div
            key={item.id || `feature-${index}`}
            className={`relative rounded-xl border border-gray-200 bg-gray-50 p-4
dark:border-slate-700 dark:bg-slate-800/50`}
          >
            <button
              type="button"
              onClick={() => handleRemove(index)}
              className={`absolute right-2 top-2 rounded-lg p-1.5 text-gray-400 hover:bg-red-50
hover:text-red-600 dark:hover:bg-red-900/20`}
              title="Entfernen"
            >
              <Trash2 className="h-4 w-4" />
            </button>
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <label className="text-xs font-medium text-gray-500">Titel</label>
                <input
                  type="text"
                  className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={item.title || ''}
                  onChange={(e) => handleChange(index, 'title', e.target.value)}
                />
              </div>
              <div>
                <label className="text-xs font-medium text-gray-500">Icon (Lucide Name)</label>
                <input
                  type="text"
                  className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={item.icon || ''}
                  onChange={(e) => handleChange(index, 'icon', e.target.value)}
                />
              </div>
              <div className="sm:col-span-2">
                <label className="text-xs font-medium text-gray-500">Beschreibung</label>
                <textarea
                  className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  rows="2"
                  value={item.description || ''}
                  onChange={(e) => handleChange(index, 'description', e.target.value)}
                />
              </div>
            </div>
          </div>
        ))}
        {items.length === 0 && (
          <div
            className={`rounded-xl border border-dashed border-gray-300 p-8 text-center text-sm
text-gray-500`}
          >
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

export const StatsForm = ({ content, onFieldChange }) => {
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
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Statistiken</h3>
        <button
          type="button"
          onClick={handleAdd}
          className={`inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white
px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200`}
        >
          <Plus className="h-3 w-3" />
          Statistik hinzufügen
        </button>
      </div>
      <div className="space-y-3">
        {items.map((item, index) => (
          <div
            key={item.label || `stat-${index}`}
            className={`flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3
dark:border-slate-700 dark:bg-slate-800/50`}
          >
            <div className="flex-1 grid gap-3 sm:grid-cols-2">
              <div>
                <label className="text-xs font-medium text-gray-500">Wert (z.B. 10k+)</label>
                <input
                  type="text"
                  className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={item.value || ''}
                  onChange={(e) => handleChange(index, 'value', e.target.value)}
                />
              </div>
              <div>
                <label className="text-xs font-medium text-gray-500">Label (z.B. Leser)</label>
                <input
                  type="text"
                  className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={item.label || ''}
                  onChange={(e) => handleChange(index, 'label', e.target.value)}
                />
              </div>
            </div>
            <button
              type="button"
              onClick={() => handleRemove(index)}
              className={`mt-6 rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600
dark:hover:bg-red-900/20`}
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

export const CtaSectionForm = ({ content, onFieldChange }) => {
  const cta = content || {}
  return (
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
        CTA-Bereich (Startseite)
      </h3>
      <div className="grid grid-cols-1 gap-4">
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={cta.title || ''}
            onChange={(e) => onFieldChange(['title'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Beschreibung
          </label>
          <textarea
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            rows="2"
            value={cta.description || ''}
            onChange={(e) => onFieldChange(['description'], e.target.value)}
          />
        </div>
        <div>
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Button Text
          </label>
          <input
            type="text"
            className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
            value={cta.buttonLabel || 'Get Started Now'}
            onChange={(e) => onFieldChange(['buttonLabel'], e.target.value)}
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

export const HeroContentForm = ({ content, onFieldChange }) => {
  const heroContent = content || {}
  const title = heroContent.title || {}
  const handleChange = (path) => (event) => {
    onFieldChange(path, event.target.value)
  }

  return (
    <div className="space-y-6">
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Hero-Inhalt bearbeiten
        </h3>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Badge Text
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={heroContent.badgeText || ''}
              onChange={handleChange(['badgeText'])}
              placeholder="z. B. Neu"
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Titel Zeile 1
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={title.line1 || ''}
              onChange={handleChange(['title', 'line1'])}
              placeholder="z. B. Lerne Linux"
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Titel Zeile 2
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={title.line2 || ''}
              onChange={handleChange(['title', 'line2'])}
              placeholder="z. B. von Grund auf"
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Untertitel
            </label>
            <textarea
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              rows="2"
              value={heroContent.subtitle || ''}
              onChange={handleChange(['subtitle'])}
              placeholder="Kurze Beschreibung"
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Features Sektion: Titel
            </label>
            <div className="grid grid-cols-2 gap-2">
              <input
                type="text"
                className={`w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                value={heroContent.features_title || ''}
                onChange={handleChange(['features_title'])}
                placeholder="Everything you need to"
              />
              <input
                type="text"
                className={`w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                value={heroContent.features_highlight || ''}
                onChange={handleChange(['features_highlight'])}
                placeholder="scale"
              />
            </div>
            <label className="mt-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
              Features Sektion: Untertitel
            </label>
            <textarea
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              rows="2"
              value={heroContent.features_subtitle || ''}
              onChange={handleChange(['features_subtitle'])}
              placeholder="Powerful features packaged in a beautiful interface."
            />
          </div>
          <div className="md:col-span-2">
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Subline</label>
            <textarea
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              rows="2"
              value={heroContent.subline || ''}
              onChange={handleChange(['subline'])}
              placeholder="Zusätzlicher Satz unter dem Untertitel"
            />
          </div>
        </div>
      </div>

      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Einstellungen
        </h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Hero Bild URL
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 focus:border-primary-500 focus:ring-2 focus:ring-primary-200
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={heroContent.heroImage || ''}
              onChange={handleChange(['heroImage'])}
              placeholder="/hero-dashboard-v2.png"
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Primary Button Label
              </label>
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                value={heroContent.primaryCta?.label || ''}
                onChange={handleChange(['primaryCta', 'label'])}
              />
            </div>
            <div>
              <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Secondary Button Label
              </label>
              <input
                type="text"
                className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
text-gray-900 dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                value={heroContent.secondaryCta?.label || ''}
                onChange={handleChange(['secondaryCta', 'label'])}
              />
            </div>
          </div>
        </div>
      </div>

      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
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
