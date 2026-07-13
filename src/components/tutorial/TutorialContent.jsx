import PropTypes from 'prop-types'

const TutorialContent = ({ content, onChange }) => {
  return (
    <div>
      <label className="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2">
        Inhalt (Markdown unterstützt)
      </label>
      <textarea
        value={content}
        onChange={(e) => onChange('content', e.target.value)}
        className={`w-full px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent font-mono text-sm`}
        rows="10"
        placeholder="Hier kannst du den vollständigen Tutorial-Inhalt eingeben..."
        maxLength={100000}
      />
      <p className="mt-1 text-xs text-gray-500 dark:text-slate-400">
        {content.length}/100000 Zeichen
      </p>
    </div>
  )
}

TutorialContent.propTypes = {
  content: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
}

export default TutorialContent
