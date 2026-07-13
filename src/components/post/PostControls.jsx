import { ArrowLeft, Loader2 } from 'lucide-react'
import PropTypes from 'prop-types'

const PostControls = ({ onBack, onDownload, loading }) => {
  return (
    <div className="flex items-center gap-4 mb-6">
      <button
        onClick={onBack}
        className="group inline-flex items-center gap-2 text-primary-700 font-medium"
      >
        <ArrowLeft className="w-4 h-4 transition-transform duration-200 group-hover:-translate-x-1" />
        Zurück zur Übersicht
      </button>
      <div className="flex-grow" />
      {onDownload && (
        <button
          onClick={onDownload}
          disabled={loading}
          className={`inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white
rounded-lg hover:bg-primary-700 transition-colors text-sm font-medium
disabled:opacity-50 disabled:cursor-not-allowed`}
        >
          {loading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="lucide lucide-download"
            >
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" x2="12" y1="15" y2="3" />
            </svg>
          )}
          {loading ? 'Wird erstellt…' : 'PDF herunterladen'}
        </button>
      )}
    </div>
  )
}

PostControls.propTypes = {
  onBack: PropTypes.func.isRequired,
  onDownload: PropTypes.func,
  loading: PropTypes.bool.isRequired,
}

export default PostControls
