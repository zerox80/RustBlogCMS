import { useMemo } from 'react'
import PropTypes from 'prop-types'
import {
  AlertCircle,
  CalendarDays,
  Edit,
  Eye,
  EyeOff,
  Layers,
  Plus,
  RefreshCw,
  Trash2,
} from 'lucide-react'

const PostsPanel = ({ page, posts, onCreate, onEdit, onDelete, loading, error, onRefresh }) => {
  const publishedCount = useMemo(() => posts.filter((post) => post.is_published).length, [posts])
  return (
    <div
      className={`bg-white shadow-lg border border-gray-200 rounded-2xl p-6 space-y-6
dark:bg-slate-900 dark:border-slate-800`}
    >
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 className="text-xl font-semibold text-gray-900 dark:text-slate-100">
            Beiträge für „{page.title}“
          </h3>
          <p className="text-sm text-gray-600 dark:text-slate-300">
            {publishedCount} von {posts.length} Beiträgen veröffentlicht.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-3">
          <button
            onClick={onRefresh}
            className={`inline-flex items-center gap-2 rounded-lg border border-gray-200 px-3 py-2
text-sm text-gray-600 hover:bg-gray-50 disabled:opacity-60
dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800`}
            disabled={loading}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
            Aktualisieren
          </button>
          <button
            onClick={onCreate}
            className={`inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600
to-primary-700 px-4 py-2 text-sm font-semibold text-white shadow-lg
hover:from-primary-700 hover:to-primary-800`}
          >
            <Plus className="h-4 w-4" />
            Beitrag erstellen
          </button>
        </div>
      </div>
      {error && (
        <div
          className={`flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-3
text-sm text-red-700`}
        >
          <AlertCircle className="h-4 w-4" />
          <div>
            <p className="font-semibold">Beiträge konnten nicht geladen werden</p>
            <p>{error?.message || 'Unbekannter Fehler'}</p>
          </div>
        </div>
      )}
      {loading && posts.length === 0 ? (
        <div className="py-12 text-center text-gray-500 dark:text-slate-400">
          Beiträge werden geladen…
        </div>
      ) : posts.length === 0 ? (
        <div
          className={`rounded-2xl border border-gray-200 bg-gray-50 p-8 text-center text-gray-600
dark:border-slate-700 dark:bg-slate-900/60 dark:text-slate-300`}
        >
          Noch keine Beiträge für diese Seite vorhanden.
        </div>
      ) : (
        <div className="space-y-5">
          {posts.map((post) => (
            <article
              key={post.id}
              className={`rounded-2xl border border-gray-200 bg-white shadow-sm hover:shadow-md
transition-shadow duration-200 dark:border-slate-800 dark:bg-slate-900`}
            >
              <div className="px-5 py-5 space-y-4">
                <header className="space-y-2">
                  <div className="flex flex-wrap items-center gap-3 text-sm text-gray-500 dark:text-slate-400">
                    {post.is_published ? (
                      <span
                        className={`inline-flex items-center gap-1 rounded-full bg-green-50 px-2.5 py-1
text-green-700 text-xs font-medium`}
                      >
                        <Eye className="h-3.5 w-3.5" /> Veröffentlicht
                      </span>
                    ) : (
                      <span
                        className={`inline-flex items-center gap-1 rounded-full bg-gray-100 px-2.5 py-1
text-gray-600 text-xs font-medium`}
                      >
                        <EyeOff className="h-3.5 w-3.5" /> Entwurf
                      </span>
                    )}
                    {post.published_at && (
                      <span className="inline-flex items-center gap-1">
                        <CalendarDays className="h-4 w-4" />
                        {new Date(post.published_at).toLocaleString('de-DE')}
                      </span>
                    )}
                    <span className="inline-flex items-center gap-1">
                      <Layers className="h-4 w-4" /> Ordnung: {post.order_index ?? 0}
                    </span>
                  </div>
                  <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">
                    {post.title}
                  </h4>
                  {post.excerpt && (
                    <p className="text-sm text-gray-600 line-clamp-2 dark:text-slate-300">
                      {post.excerpt}
                    </p>
                  )}
                </header>
                <div
                  className={`flex flex-wrap items-center gap-3 pt-2 border-t border-gray-100
dark:border-slate-800`}
                >
                  <button
                    onClick={() => onEdit(post)}
                    className={`inline-flex items-center gap-2 rounded-lg border border-primary-100 px-3
py-1.5 text-sm text-primary-700 hover:bg-primary-50
dark:border-primary-900/50 dark:text-primary-200
dark:hover:bg-primary-900/40`}
                  >
                    <Edit className="h-4 w-4" /> Bearbeiten
                  </button>
                  <button
                    onClick={() => onDelete(post)}
                    className={`inline-flex items-center gap-2 rounded-lg border border-red-200 px-3 py-1.5
text-sm text-red-700 hover:bg-red-50 dark:border-red-900/40
dark:text-red-200 dark:hover:bg-red-900/30`}
                  >
                    <Trash2 className="h-4 w-4" /> Löschen
                  </button>
                </div>
              </div>
            </article>
          ))}
        </div>
      )}
    </div>
  )
}
PostsPanel.propTypes = {
  page: PropTypes.object.isRequired,
  posts: PropTypes.array.isRequired,
  onCreate: PropTypes.func.isRequired,
  onEdit: PropTypes.func.isRequired,
  onDelete: PropTypes.func.isRequired,
  loading: PropTypes.bool.isRequired,
  error: PropTypes.any,
  onRefresh: PropTypes.func.isRequired,
}

export default PostsPanel
