import PropTypes from 'prop-types'
import { Download, Plus, RefreshCw } from 'lucide-react'

const PageManagerHeader = ({ onRefresh, refreshing, onExportMarkdown, markdownExporting, onCreate }) => (
  <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
    <div>
      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Seiten & Beitraege</h2>
      <p className="text-sm text-gray-600 dark:text-gray-300">
        Verwalte dynamische Seiten, Navigationseintraege und veroeffentlichte Beitraege.
      </p>
    </div>
    <div className="flex flex-wrap items-center gap-3">
      <button
        onClick={onRefresh}
        className="inline-flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2 text-sm text-gray-600 hover:bg-gray-50 disabled:opacity-60 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800"
        disabled={refreshing}
      >
        <RefreshCw className={`h-4 w-4 ${refreshing ? 'animate-spin' : ''}`} />
        Seiten aktualisieren
      </button>
      <button
        onClick={onExportMarkdown}
        className="inline-flex items-center gap-2 rounded-lg border border-primary-200 px-4 py-2 text-sm font-medium text-primary-700 hover:bg-primary-50 disabled:opacity-60 dark:border-primary-900/50 dark:text-primary-200 dark:hover:bg-primary-900/40"
        disabled={markdownExporting}
      >
        {markdownExporting ? (
          <RefreshCw className="h-4 w-4 animate-spin" />
        ) : (
          <Download className="h-4 w-4" />
        )}
        {markdownExporting ? 'Markdown wird erstellt...' : 'Alle Seiten als .md'}
      </button>
      <button
        onClick={onCreate}
        className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600 to-primary-700 px-5 py-2.5 text-sm font-semibold text-white shadow-lg hover:from-primary-700 hover:to-primary-800"
      >
        <Plus className="h-4 w-4" /> Neue Seite
      </button>
    </div>
  </div>
)

PageManagerHeader.propTypes = {
  onRefresh: PropTypes.func.isRequired,
  onExportMarkdown: PropTypes.func.isRequired,
  onCreate: PropTypes.func.isRequired,
  refreshing: PropTypes.bool,
  markdownExporting: PropTypes.bool,
}

PageManagerHeader.defaultProps = {
  refreshing: false,
  markdownExporting: false,
}

export default PageManagerHeader
