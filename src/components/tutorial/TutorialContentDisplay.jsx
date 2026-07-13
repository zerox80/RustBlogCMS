import PropTypes from 'prop-types'
import MarkdownRenderer from '../markdown/MarkdownRenderer'

const TutorialContentDisplay = ({ content }) => {
  return (
    <section>
      <h2 className="text-2xl font-semibold text-gray-900 dark:text-slate-100 mb-4">Inhalt</h2>
      <MarkdownRenderer content={content || 'Für dieses Tutorial liegt noch kein Inhalt vor.'} />
    </section>
  )
}

TutorialContentDisplay.propTypes = {
  content: PropTypes.string,
}

export default TutorialContentDisplay
