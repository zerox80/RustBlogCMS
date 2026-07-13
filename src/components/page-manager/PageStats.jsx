import PropTypes from 'prop-types'
import { Eye, Layers, Navigation } from 'lucide-react'

const PageStats = ({ navigation, publishedSlugs, pages, selectedPage }) => {
  const dynamicPagesInNav = navigation?.dynamic?.length ?? 0
  const totalPublishedPages = publishedSlugs.length

  return (
    <div className="grid gap-6 lg:grid-cols-3">
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-5 shadow-sm
dark:border-gray-700 dark:bg-slate-900`}
      >
        <div
          className={`flex items-center gap-3 border-b border-gray-100 pb-4 mb-4
dark:border-gray-800`}
        >
          <Navigation className="h-5 w-5 text-primary-600" />
          <div>
            <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">Navigation</p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {dynamicPagesInNav} dynamische Seite{dynamicPagesInNav === 1 ? '' : 'n'} im Menü
            </p>
          </div>
        </div>
        <div className="space-y-3">
          {(navigation?.dynamic ?? []).map((item) => (
            <div
              key={item.id}
              className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300"
            >
              <span>{item.label}</span>
              <span className="text-xs text-gray-400 dark:text-gray-500">/pages/{item.slug}</span>
            </div>
          ))}
          {dynamicPagesInNav === 0 && (
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Noch keine dynamischen Seiten in der Navigation.
            </p>
          )}
        </div>
      </div>
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-5 shadow-sm
dark:border-gray-700 dark:bg-slate-900`}
      >
        <div
          className={`flex items-center gap-3 border-b border-gray-100 pb-4 mb-4
dark:border-gray-800`}
        >
          <Eye className="h-5 w-5 text-green-600" />
          <div>
            <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
              Veröffentlichungen
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {totalPublishedPages} veröffentlichte Seite{totalPublishedPages === 1 ? '' : 'n'}
            </p>
          </div>
        </div>
        <div className="space-y-3">
          {publishedSlugs.map((slugValue) => (
            <div
              key={slugValue}
              className="flex items-center justify-between text-sm text-gray-600 dark:text-gray-300"
            >
              <span>/pages/{slugValue}</span>
            </div>
          ))}
          {totalPublishedPages === 0 && (
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Noch keine Seite veröffentlicht.
            </p>
          )}
        </div>
      </div>
      <div
        className={`rounded-2xl border border-gray-200 bg-white p-5 shadow-sm
dark:border-gray-700 dark:bg-slate-900`}
      >
        <div
          className={`flex items-center gap-3 border-b border-gray-100 pb-4 mb-4
dark:border-gray-800`}
        >
          <Layers className="h-5 w-5 text-indigo-600" />
          <div>
            <p className="text-sm font-semibold text-gray-900 dark:text-gray-100">
              Seitenübersicht
            </p>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              {pages.length} Seite{pages.length === 1 ? '' : 'n'} insgesamt
            </p>
          </div>
        </div>
        <div className="space-y-3 text-sm text-gray-600 dark:text-gray-300">
          <p>
            <span className="font-semibold">Ausgewählt:</span>{' '}
            {selectedPage ? selectedPage.title : 'Keine Seite ausgewählt'}
          </p>
          <p>
            <span className="font-semibold">Entwürfe:</span>{' '}
            {pages.filter((page) => !page.is_published).length}
          </p>
        </div>
      </div>
    </div>
  )
}

PageStats.propTypes = {
  navigation: PropTypes.shape({
    dynamic: PropTypes.arrayOf(
      PropTypes.shape({
        id: PropTypes.oneOfType([PropTypes.string, PropTypes.number]),
        label: PropTypes.string,
        slug: PropTypes.string,
      }),
    ),
  }),
  publishedSlugs: PropTypes.arrayOf(PropTypes.string),
  pages: PropTypes.arrayOf(PropTypes.object),
  selectedPage: PropTypes.object,
}

PageStats.defaultProps = {
  navigation: null,
  publishedSlugs: [],
  pages: [],
  selectedPage: null,
}

export default PageStats
