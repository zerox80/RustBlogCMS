import PropTypes from 'prop-types'
import { Edit, Eye, EyeOff, FileText, Layers, Navigation, Trash2 } from 'lucide-react'

const PageList = ({ pages, loading, selectedPageId, onSelect, onEdit, onDelete }) => {
  if (loading && pages.length === 0) {
    return (
      <div className="space-y-4">
        <div
          className={`rounded-2xl border border-gray-200 bg-white p-8 text-center text-gray-500
dark:border-gray-700 dark:bg-slate-900 dark:text-gray-400`}
        >
          Seiten werden geladen...
        </div>
      </div>
    )
  }

  if (pages.length === 0) {
    return (
      <div className="space-y-4">
        <div
          className={`rounded-2xl border border-dashed border-gray-300 bg-gray-50 p-10 text-center
text-gray-600 dark:border-gray-600 dark:bg-slate-900/50 dark:text-gray-300`}
        >
          Noch keine Seiten vorhanden. Erstelle deine erste Seite, um Beiträge zu veröffentlichen.
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      {pages.map((page) => {
        const isSelected = page.id === selectedPageId
        return (
          <div
            key={page.id}
            className={`rounded-2xl border ${
              isSelected
                ? 'border-primary-300 bg-primary-50/60 shadow-lg dark:border-primary-700 dark:bg-primary-900/40'
                : 'border-gray-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900'
            } transition-shadow duration-200`}
          >
            <div className="px-5 py-5 space-y-4">
              <div className="flex flex-wrap items-center gap-2 text-xs font-medium">
                <span
                  className={`inline-flex items-center gap-1 rounded-full px-2.5 py-1 ${
                    page.is_published ? 'bg-green-50 text-green-700' : 'bg-gray-100 text-gray-600'
                  }`}
                >
                  {page.is_published ? (
                    <Eye className="h-3.5 w-3.5" />
                  ) : (
                    <EyeOff className="h-3.5 w-3.5" />
                  )}
                  {page.is_published ? 'Veröffentlicht' : 'Entwurf'}
                </span>
                {page.show_in_nav && (
                  <span
                    className={`inline-flex items-center gap-1 rounded-full bg-blue-50 px-2.5 py-1
text-blue-700`}
                  >
                    <Navigation className="h-3.5 w-3.5" /> Navigation
                  </span>
                )}
                <span
                  className={`inline-flex items-center gap-1 rounded-full bg-gray-100 px-2.5 py-1
text-gray-600 dark:bg-slate-800 dark:text-slate-300`}
                >
                  <FileText className="h-3.5 w-3.5" /> /pages/{page.slug}
                </span>
              </div>
              <div>
                <h3 className="text-lg font-semibold text-gray-900 dark:text-slate-100">
                  {page.title}
                </h3>
                {page.description && (
                  <p className="mt-1 text-sm text-gray-600 line-clamp-2 dark:text-slate-300">
                    {page.description}
                  </p>
                )}
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <button
                  onClick={() => onSelect(page.id)}
                  className={`inline-flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm ${
                    isSelected
                      ? [
                          'border-primary-200 bg-primary-100 text-primary-800 dark:border-primary-600',
                          'dark:bg-primary-900/50 dark:text-primary-100',
                        ].join(' ')
                      : [
                          'border-gray-200 text-gray-600 hover:bg-gray-50 dark:border-slate-700',
                          'dark:text-slate-200 dark:hover:bg-slate-800',
                        ].join(' ')
                  }`}
                >
                  <Layers className="h-4 w-4" /> Beiträge ansehen
                </button>
                <button
                  onClick={() => onEdit(page)}
                  className={`inline-flex items-center gap-2 rounded-lg border border-primary-100 px-3
py-1.5 text-sm text-primary-700 hover:bg-primary-50
dark:border-primary-900/50 dark:text-primary-200
dark:hover:bg-primary-900/40`}
                >
                  <Edit className="h-4 w-4" /> Bearbeiten
                </button>
                <button
                  onClick={() => onDelete(page)}
                  className={`inline-flex items-center gap-2 rounded-lg border border-red-200 px-3 py-1.5
text-sm text-red-700 hover:bg-red-50 dark:border-red-900/40
dark:text-red-200 dark:hover:bg-red-900/30`}
                >
                  <Trash2 className="h-4 w-4" /> Löschen
                </button>
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}

PageList.propTypes = {
  pages: PropTypes.arrayOf(PropTypes.object).isRequired,
  loading: PropTypes.bool.isRequired,
  selectedPageId: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
  onSelect: PropTypes.func.isRequired,
  onEdit: PropTypes.func.isRequired,
  onDelete: PropTypes.func.isRequired,
}

PageList.defaultProps = {
  selectedPageId: null,
}

export default PageList
