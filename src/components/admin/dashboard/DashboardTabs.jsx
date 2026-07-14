import PropTypes from 'prop-types'
import { FileText, Paintbrush } from 'lucide-react'

const DashboardTabs = ({ activeTab, onTabChange }) => {
  return (
    <div
      className={`mb-8 flex flex-col gap-3 border-b border-gray-200 pb-3 sm:flex-row
sm:items-center sm:justify-between dark:border-slate-800`}
    >
      <div className="flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => onTabChange('posts')}
          className={`inline-flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all ${
            activeTab === 'posts'
              ? 'bg-primary-600 text-white shadow-lg shadow-primary-900/20'
              : [
                  'bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-slate-800',
                  'dark:text-slate-200 dark:hover:bg-slate-700',
                ].join(' ')
          }`}
        >
          <FileText className="h-4 w-4" />
          Beiträge
        </button>
        <button
          type="button"
          onClick={() => onTabChange('content')}
          className={`inline-flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-all ${
            activeTab === 'content'
              ? 'bg-primary-600 text-white shadow-lg shadow-primary-900/20'
              : [
                  'bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-slate-800',
                  'dark:text-slate-200 dark:hover:bg-slate-700',
                ].join(' ')
          }`}
        >
          <Paintbrush className="h-4 w-4" />
          Seiteninhalte
        </button>
      </div>
    </div>
  )
}

DashboardTabs.propTypes = {
  activeTab: PropTypes.string.isRequired,
  onTabChange: PropTypes.func.isRequired,
}

export default DashboardTabs
