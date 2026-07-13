import { ChevronRight, Sparkles } from 'lucide-react'
import PropTypes from 'prop-types'
import { scrollToSection } from '../utils/scrollToSection'
const TutorialCard = ({ icon: Icon, title, description, topics, color, onSelect, buttonLabel }) => {
  const handleSelect = () => {
    if (typeof onSelect === 'function') {
      onSelect()
      return
    }
    scrollToSection('tutorials')
  }
  return (
    <article className="tutorial-card group animate-fade-in">
      <div
        className={[
          'absolute inset-0 -z-10 rounded-2xl bg-gradient-to-br opacity-0 blur-2xl',
          'transition-opacity duration-700 group-hover:opacity-10',
          color,
        ].join(' ')}
        aria-hidden="true"
      ></div>
      <div className="absolute inset-0 rounded-2xl overflow-hidden">
        <div
          className={`absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity
duration-1000 shimmer`}
          aria-hidden="true"
        ></div>
      </div>
      <div className="relative mb-6">
        <div
          className={[
            'flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br shadow-2xl',
            'transition-all duration-500 group-hover:rotate-3 group-hover:scale-110',
            color,
          ].join(' ')}
        >
          <Icon className="w-8 h-8 text-white" />
        </div>
        <div
          className={[
            'absolute -right-1 -top-1 flex h-6 w-6 items-center justify-center rounded-full',
            'bg-gradient-to-br opacity-0 transition-opacity duration-500 group-hover:opacity-100',
            color,
          ].join(' ')}
          aria-hidden="true"
        >
          <Sparkles className="w-3 h-3 text-white" />
        </div>
      </div>
      <h3
        className={`text-2xl font-bold text-neutral-50 mb-3 group-hover:text-primary-300
transition-colors duration-300`}
      >
        {title}
      </h3>
      <p className="text-neutral-200 mb-6 leading-relaxed line-clamp-2">{description}</p>
      <div className="space-y-3 mb-8">
        {topics.map((topic, index) => (
          <div
            key={`${topic}-${index}`}
            className={`flex items-start text-sm text-neutral-200 group/item hover:text-primary-300
transition-colors duration-200`}
          >
            <div
              className={[
                'mr-3 mt-0.5 flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full',
                'bg-gradient-to-br transition-transform duration-200 group-hover/item:scale-110',
                color,
              ].join(' ')}
            >
              <ChevronRight className="w-3 h-3 text-white" />
            </div>
            <span className="flex-1">{topic}</span>
          </div>
        ))}
      </div>
      <button
        type="button"
        onClick={handleSelect}
        className={`relative w-full mt-auto py-3.5 px-6 rounded-xl font-semibold text-neutral-50
shadow-lg transition-all duration-300 flex items-center justify-center
overflow-hidden group/button bg-gradient-to-r from-primary-500
to-primary-600 hover:from-primary-400 hover:to-primary-500
hover:shadow-card-xl`}
      >
        <span className="relative z-10 flex items-center gap-2">
          {buttonLabel || 'Zum Tutorial'}
          <ChevronRight className="w-5 h-5 group-hover/button:translate-x-1 transition-transform duration-300" />
        </span>
        <div
          className={`absolute inset-0 bg-gradient-to-r from-transparent via-white/20
to-transparent -translate-x-full group-hover/button:translate-x-full
transition-transform duration-1000`}
          aria-hidden="true"
        ></div>
      </button>
    </article>
  )
}

TutorialCard.propTypes = {
  icon: PropTypes.elementType.isRequired,
  title: PropTypes.string.isRequired,
  description: PropTypes.string.isRequired,
  topics: PropTypes.arrayOf(PropTypes.string).isRequired,
  color: PropTypes.string.isRequired,
  onSelect: PropTypes.func,
  buttonLabel: PropTypes.string,
}
export default TutorialCard
