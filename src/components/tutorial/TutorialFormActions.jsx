import PropTypes from 'prop-types'
import { Save } from 'lucide-react'

const TutorialFormActions = ({ submitting, isEditing, onClose }) => {
  return (
    <div className="flex space-x-4 pt-4 border-t border-gray-200 dark:border-slate-800">
      <button
        type="submit"
        className={`flex-1 flex items-center justify-center space-x-2 px-6 py-3 bg-gradient-to-r
from-primary-600 to-primary-700 text-white rounded-lg hover:from-primary-700
hover:to-primary-800 transition-all duration-200 shadow-lg hover:shadow-xl
disabled:opacity-60`}
        disabled={submitting}
      >
        <Save className="w-5 h-5" />
        <span>
          {submitting ? 'Speichere…' : isEditing ? 'Änderungen speichern' : 'Tutorial erstellen'}
        </span>
      </button>
      <button
        type="button"
        onClick={onClose}
        className={`px-6 py-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200
transition-colors duration-200 dark:bg-slate-800 dark:text-slate-200
dark:hover:bg-slate-700`}
      >
        Abbrechen
      </button>
    </div>
  )
}

TutorialFormActions.propTypes = {
  submitting: PropTypes.bool.isRequired,
  isEditing: PropTypes.bool.isRequired,
  onClose: PropTypes.func.isRequired,
}

export default TutorialFormActions
