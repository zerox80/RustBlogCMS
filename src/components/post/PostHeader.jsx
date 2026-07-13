import { CalendarDays } from 'lucide-react'
import PropTypes from 'prop-types'
import { formatDate } from '../../utils/postUtils'

const PostHeader = ({ title, publishedAt, excerpt }) => {
  const publishedDate = formatDate(publishedAt)

  return (
    <header className="space-y-4 pb-6 border-b border-gray-200 dark:border-slate-700">
      {publishedDate && (
        <div className="inline-flex items-center gap-2 text-sm text-gray-500 dark:text-slate-400">
          <CalendarDays className="w-4 h-4" />
          {publishedDate}
        </div>
      )}
      <h1
        className={`text-3xl sm:text-4xl font-bold text-gray-900 dark:text-slate-100
leading-tight`}
      >
        {title}
      </h1>
      {excerpt && (
        <p className="text-lg text-gray-600 dark:text-slate-300 leading-relaxed">{excerpt}</p>
      )}
    </header>
  )
}

PostHeader.propTypes = {
  title: PropTypes.string,
  publishedAt: PropTypes.string,
  excerpt: PropTypes.string,
}

export default PostHeader
