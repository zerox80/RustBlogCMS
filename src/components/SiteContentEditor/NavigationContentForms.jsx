import PropTypes from 'prop-types'
import { Plus, Trash2 } from 'lucide-react'

export const HeaderForm = ({ content, onFieldChange }) => {
  const header = content || {}
  const brand = header.brand || {}
  const navItems = Array.isArray(header.navItems) ? header.navItems : []

  const handleNavChange = (index, field, value) => {
    const newItems = [...navItems]
    newItems[index] = { ...newItems[index], [field]: value }
    onFieldChange(['navItems'], newItems)
  }

  const handleAddNav = () => {
    onFieldChange(
      ['navItems'],
      [...navItems, { id: `nav-${Date.now()}`, label: 'Neuer Link', type: 'route', path: '/' }],
    )
  }

  const handleRemoveNav = (index) => {
    const newItems = navItems.filter((_, i) => i !== index)
    onFieldChange(['navItems'], newItems)
  }

  return (
    <div className="space-y-6">
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Marke & Logo
        </h3>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Markenname
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={brand.name || ''}
              onChange={(e) => onFieldChange(['brand', 'name'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Icon (Lucide Name)
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={brand.icon || ''}
              onChange={(e) => onFieldChange(['brand', 'icon'], e.target.value)}
            />
          </div>
        </div>
      </div>

      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Navigation</h3>
          <button
            type="button"
            onClick={handleAddNav}
            className={`inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white
px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200`}
          >
            <Plus className="h-3 w-3" />
            Link hinzufügen
          </button>
        </div>
        <div className="space-y-3">
          {navItems.map((item, index) => (
            <div
              key={item.id || `nav-${index}`}
              className={`flex items-start gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3
dark:border-slate-700 dark:bg-slate-800/50`}
            >
              <div className="flex-1 grid gap-3 sm:grid-cols-3">
                <div>
                  <label className="text-xs font-medium text-gray-500">Label</label>
                  <input
                    type="text"
                    className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                    value={item.label || ''}
                    onChange={(e) => handleNavChange(index, 'label', e.target.value)}
                  />
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Typ</label>
                  <select
                    className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
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
                    className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                    value={item.path || item.value || ''}
                    onChange={(e) => handleNavChange(index, 'path', e.target.value)}
                  />
                </div>
              </div>
              <button
                type="button"
                onClick={() => handleRemoveNav(index)}
                className={`mt-6 rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600
dark:hover:bg-red-900/20`}
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

export const FooterForm = ({ content, onFieldChange }) => {
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
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Footer Marke
        </h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Titel</label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={brand.title || ''}
              onChange={(e) => onFieldChange(['brand', 'title'], e.target.value)}
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
              value={brand.description || ''}
              onChange={(e) => onFieldChange(['brand', 'description'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Copyright Text
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={bottom.copyright || ''}
              onChange={(e) => onFieldChange(['bottom', 'copyright'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Signatur (rechts)
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={bottom.signature || ''}
              onChange={(e) => onFieldChange(['bottom', 'signature'], e.target.value)}
            />
          </div>
        </div>
      </div>

      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">Quick Links</h3>
          <button
            type="button"
            onClick={handleAddLink}
            className={`inline-flex items-center gap-1 rounded-lg border border-gray-200 bg-white
px-3 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-200`}
          >
            <Plus className="h-3 w-3" />
            Link hinzufügen
          </button>
        </div>
        <div className="space-y-3">
          {quickLinks.map((link, index) => (
            <div
              key={link.label || `link-${index}`}
              className={`flex items-center gap-3 rounded-lg border border-gray-100 bg-gray-50 p-3
dark:border-slate-700 dark:bg-slate-800/50`}
            >
              <div className="flex-1 grid gap-3 sm:grid-cols-2">
                <input
                  type="text"
                  placeholder="Label"
                  className={`w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={link.label || ''}
                  onChange={(e) => handleLinkChange(index, 'label', e.target.value)}
                />
                <input
                  type="text"
                  placeholder="Pfad / URL"
                  className={`w-full rounded-lg border border-gray-300 bg-white px-2 py-1.5 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
                  value={link.path || link.href || ''}
                  onChange={(e) => handleLinkChange(index, 'path', e.target.value)}
                />
              </div>
              <button
                type="button"
                onClick={() => handleRemoveLink(index)}
                className={`rounded-lg p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600
dark:hover:bg-red-900/20`}
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}
        </div>
      </div>

      <div
        className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
      >
        <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
          Copyright & Signatur
        </h3>
        <div className="grid grid-cols-1 gap-4">
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Copyright Text ({'{year}'} Platzhalter)
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
              value={bottom.copyright || ''}
              onChange={(e) => onFieldChange(['bottom', 'copyright'], e.target.value)}
            />
          </div>
          <div>
            <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Signatur (Rechts unten)
            </label>
            <input
              type="text"
              className={`mt-1 w-full rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm
dark:border-slate-600 dark:bg-slate-800 dark:text-slate-100`}
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
