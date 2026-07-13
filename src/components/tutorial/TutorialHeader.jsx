import PropTypes from 'prop-types'

const TutorialHeader = ({ title, description }) => {
  return (
    <header className="bg-gradient-to-r from-primary-600 to-primary-700 text-white px-8 py-10">
      <div className="flex flex-col gap-4">
        <span
          className={`inline-flex items-center gap-2 bg-white/15 px-4 py-2 rounded-full text-sm
font-medium w-fit`}
        >
          Linux Tutorial
        </span>
        <h1 className="text-3xl sm:text-4xl font-bold leading-tight">{title}</h1>
        <p className="text-primary-100 text-lg max-w-2xl">{description}</p>
      </div>
    </header>
  )
}

TutorialHeader.propTypes = {
  title: PropTypes.string.isRequired,
  description: PropTypes.string.isRequired,
}

export default TutorialHeader
