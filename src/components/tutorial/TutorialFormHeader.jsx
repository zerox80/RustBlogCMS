import { X } from 'lucide-react'
import PropTypes from 'prop-types'

const TutorialFormHeader = ({ isEditing, onClose }) => {
  return (
    <div className="flex justify-between items-center mb-6">
      <h2 id="modal-title" className="text-2xl font-bold text-gray-800 dark:text-slate-100">
        {isEditing ? 'Tutorial bearbeiten' : 'Neues Tutorial erstellen'}
      </h2>
      <button
        onClick={onClose}
        className={[
          'p-2 hover:bg-gray-100 dark:hover:bg-slate-800 rounded-lg transition-colors',
          'duration-200',
        ].join(' ')}
      >
        <X className="w-6 h-6 text-gray-600 dark:text-slate-300" />
      </button>
    </div>
  )
}

TutorialFormHeader.propTypes = {
  isEditing: PropTypes.bool.isRequired,
  onClose: PropTypes.func.isRequired,
}

export default TutorialFormHeader
