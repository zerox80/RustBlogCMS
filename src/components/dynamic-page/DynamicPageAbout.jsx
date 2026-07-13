import PropTypes from 'prop-types'

const DynamicPageAbout = ({ title, description }) => {
  if (!description) return null

  return (
    <section
      className={`bg-white dark:bg-slate-900/80 rounded-3xl shadow-lg border border-gray-100
dark:border-slate-800 px-6 py-8 sm:px-10`}
    >
      <h2 className="text-2xl font-semibold text-gray-900 dark:text-slate-100 mb-3">{title}</h2>
      <p className="text-gray-700 dark:text-slate-200 leading-relaxed">{description}</p>
    </section>
  )
}

DynamicPageAbout.propTypes = {
  title: PropTypes.string.isRequired,
  description: PropTypes.string,
}

export default DynamicPageAbout
