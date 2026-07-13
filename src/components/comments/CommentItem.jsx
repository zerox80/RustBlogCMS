import React from 'react'
import PropTypes from 'prop-types'
import { ShieldCheck, ThumbsUp, Trash2 } from 'lucide-react'

const CommentItem = ({ comment, canManageComments, onVote, onDelete }) => {
  return (
    <div
      className={`p-4 rounded-xl ${
        comment.is_admin
          ? 'bg-primary-50 dark:bg-primary-900/20 border border-primary-100 dark:border-primary-800'
          : 'bg-gray-50 dark:bg-gray-800'
      }`}
    >
      <div className="flex justify-between items-start mb-2">
        <div className="flex items-center gap-2">
          <span className="font-semibold text-gray-900 dark:text-gray-100">{comment.author}</span>
          {comment.is_admin && (
            <span
              className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium
bg-primary-100 text-primary-800 dark:bg-primary-900 dark:text-primary-200`}
            >
              <ShieldCheck className="w-3 h-3" />
              Admin
            </span>
          )}
          <span className="text-sm text-gray-500 dark:text-gray-400 ml-2">
            {new Date(comment.created_at).toLocaleDateString('de-DE')}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => onVote(comment.id)}
            className={`flex items-center gap-1 text-gray-500 hover:text-primary-600
dark:text-gray-400 dark:hover:text-primary-400 transition-colors`}
            title="Gefällt mir"
          >
            <ThumbsUp className="w-4 h-4" />
            <span className="text-sm font-medium">{comment.votes || 0}</span>
          </button>
          {canManageComments && (
            <button
              onClick={() => onDelete(comment.id)}
              className={`p-1 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded
transition-colors`}
              aria-label="Kommentar löschen"
            >
              <Trash2 className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>
      <p className="text-gray-700 dark:text-gray-300 whitespace-pre-wrap">{comment.content}</p>
    </div>
  )
}

CommentItem.propTypes = {
  comment: PropTypes.shape({
    id: PropTypes.string.isRequired,
    author: PropTypes.string.isRequired,
    content: PropTypes.string.isRequired,
    created_at: PropTypes.string.isRequired,
    is_admin: PropTypes.bool,
    votes: PropTypes.number,
  }).isRequired,
  canManageComments: PropTypes.bool.isRequired,
  onVote: PropTypes.func.isRequired,
  onDelete: PropTypes.func.isRequired,
}

export default CommentItem
