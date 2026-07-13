import PropTypes from 'prop-types'

const TutorialBasicInfo = ({ title, description, onChange }) => {
  return (
    <>
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2">
          Titel *
        </label>
        <input
          type="text"
          value={title}
          onChange={(e) => onChange('title', e.target.value)}
          className={`w-full px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent`}
          placeholder="z.B. Grundlegende Befehle"
          maxLength={200}
          required
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-slate-400">{title.length}/200 Zeichen</p>
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2">
          Beschreibung *
        </label>
        <textarea
          value={description}
          onChange={(e) => onChange('description', e.target.value)}
          className={`w-full px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent`}
          rows="3"
          placeholder="Kurze Beschreibung des Tutorials"
          maxLength={1000}
          required
        />
        <p className="mt-1 text-xs text-gray-500 dark:text-slate-400">
          {description.length}/1000 Zeichen
        </p>
      </div>
    </>
  )
}

TutorialBasicInfo.propTypes = {
  title: PropTypes.string.isRequired,
  description: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
}

export default TutorialBasicInfo
