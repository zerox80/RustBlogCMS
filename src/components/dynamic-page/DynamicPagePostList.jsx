import PropTypes from 'prop-types'
import PostCard from './PostCard'

const DynamicPagePostList = ({ posts, title, emptyTitle, emptyMessage, countLabel, pageSlug }) => {
  return (
    <section className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="text-2xl font-semibold text-gray-900 dark:text-slate-100">
          {posts.length > 0 ? title : emptyTitle}
        </h2>
        {posts.length > 0 && (
          <span className="text-sm text-gray-500 dark:text-slate-400">{countLabel}</span>
        )}
      </div>

      {posts.length === 0 ? (
        <div
          className={`rounded-2xl border border-gray-200 dark:border-slate-800 bg-white
dark:bg-slate-900/80 p-8 text-center text-gray-600 dark:text-slate-300`}
        >
          {emptyMessage}
        </div>
      ) : (
        <div className="space-y-10">
          {posts.map((post) => (
            <PostCard key={post.id} post={post} pageSlug={pageSlug} />
          ))}
        </div>
      )}
    </section>
  )
}

DynamicPagePostList.propTypes = {
  posts: PropTypes.array.isRequired,
  title: PropTypes.string.isRequired,
  emptyTitle: PropTypes.string.isRequired,
  emptyMessage: PropTypes.string.isRequired,
  countLabel: PropTypes.string.isRequired,
  pageSlug: PropTypes.string.isRequired,
}

export default DynamicPagePostList
