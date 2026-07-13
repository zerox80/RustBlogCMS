import PropTypes from 'prop-types'

const iconOptions = [
  'Terminal',
  'FolderTree',
  'FileText',
  'Settings',
  'Shield',
  'Network',
  'Database',
  'Server',
]

const colorOptions = [
  { value: 'from-blue-500 to-cyan-500', label: 'Blau' },
  { value: 'from-green-500 to-emerald-500', label: 'Grün' },
  { value: 'from-purple-500 to-pink-500', label: 'Lila' },
  { value: 'from-orange-500 to-red-500', label: 'Orange' },
  { value: 'from-indigo-500 to-blue-500', label: 'Indigo' },
  { value: 'from-teal-500 to-green-500', label: 'Türkis' },
  { value: 'from-yellow-500 to-orange-500', label: 'Gelb' },
  { value: 'from-red-500 to-pink-500', label: 'Rot' },
]

const TutorialAppearance = ({ icon, color, onChange }) => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2">
          Icon
        </label>
        <select
          value={icon}
          onChange={(e) => onChange('icon', e.target.value)}
          className={`w-full px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent`}
        >
          {iconOptions.map((opt) => (
            <option key={opt} value={opt}>
              {opt}
            </option>
          ))}
        </select>
      </div>
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-300 mb-2">
          Farbe
        </label>
        <select
          value={color}
          onChange={(e) => onChange('color', e.target.value)}
          className={`w-full px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent`}
        >
          {colorOptions.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  )
}

TutorialAppearance.propTypes = {
  icon: PropTypes.string.isRequired,
  color: PropTypes.string.isRequired,
  onChange: PropTypes.func.isRequired,
}

export default TutorialAppearance
