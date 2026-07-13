import PropTypes from 'prop-types'

const TutorialTopicsList = ({ topics }) => {
  if (!topics || topics.length === 0) return null

  return (
    <section>
      <h2 className="text-2xl font-semibold text-gray-900 dark:text-slate-100 mb-4">
        Was du lernen wirst
      </h2>
      <div className="grid gap-3 sm:grid-cols-2">
        {topics.map((topic, index) => (
          <div
            key={`${topic}-${index}`}
            className={`flex items-start gap-3 rounded-2xl border border-gray-200
dark:border-slate-700 bg-gray-50/60 dark:bg-slate-800/60 px-4 py-3`}
          >
            <span
              className={`inline-flex items-center justify-center h-6 w-6 shrink-0 rounded-full
bg-primary-600/10 text-primary-700 dark:text-primary-300 font-semibold
text-sm`}
            >
              {index + 1}
            </span>
            <span className="text-gray-700 dark:text-slate-200 leading-relaxed">{topic}</span>
          </div>
        ))}
      </div>
    </section>
  )
}

TutorialTopicsList.propTypes = {
  topics: PropTypes.arrayOf(PropTypes.string),
}

export default TutorialTopicsList
