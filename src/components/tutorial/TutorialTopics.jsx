import PropTypes from 'prop-types'
import { Plus, Trash2 } from 'lucide-react'

const TutorialTopics = ({ topics, onTopicChange, onAddTopic, onRemoveTopic }) => {
  return (
    <div>
      <div className="flex justify-between items-center mb-2">
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-300">
          Themen
        </label>
        <button
          type="button"
          onClick={onAddTopic}
          className="flex items-center space-x-1 text-sm text-primary-600 hover:text-primary-700"
        >
          <Plus className="w-4 h-4" />
          <span>Thema hinzufügen</span>
        </button>
      </div>
      <div className="space-y-2">
        {topics.map((topic, index) => (
          <div key={index} className="flex space-x-2">
            <input
              type="text"
              value={topic}
              onChange={(e) => onTopicChange(index, e.target.value)}
              className={`flex-1 px-4 py-2 border border-gray-300 dark:border-slate-700 rounded-lg
bg-white dark:bg-slate-800 text-gray-900 dark:text-slate-100 focus:ring-2
focus:ring-primary-500 focus:border-transparent`}
              placeholder={`Thema ${index + 1}`}
              maxLength={100}
            />
            {topics.length > 1 && (
              <button
                type="button"
                onClick={() => onRemoveTopic(index)}
                className={`p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg
transition-colors duration-200`}
              >
                <Trash2 className="w-5 h-5" />
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  )
}

TutorialTopics.propTypes = {
  topics: PropTypes.arrayOf(PropTypes.string).isRequired,
  onTopicChange: PropTypes.func.isRequired,
  onAddTopic: PropTypes.func.isRequired,
  onRemoveTopic: PropTypes.func.isRequired,
}

export default TutorialTopics
