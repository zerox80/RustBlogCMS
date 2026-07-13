import PropTypes from 'prop-types'

const DynamicPageHero = ({ title, subtitle, badge, gradient }) => {
  return (
    <section
      className={`relative overflow-hidden rounded-3xl border border-primary-100
dark:border-primary-900/40 bg-white dark:bg-slate-900/80 shadow-xl`}
    >
      <div
        className={`absolute inset-0 bg-gradient-to-br ${gradient} opacity-90`}
        aria-hidden="true"
      />
      <div className="relative px-6 py-12 sm:px-10 sm:py-14 text-white">
        <div className="max-w-3xl space-y-6">
          {badge && (
            <span
              className={`inline-flex items-center gap-2 rounded-full bg-white/15 px-4 py-1.5 text-sm
font-medium`}
            >
              {badge}
            </span>
          )}
          <h1 className="text-3xl sm:text-4xl font-bold leading-tight drop-shadow-sm">
            {title || 'Neue Seite'}
          </h1>
          {subtitle && <p className="text-lg text-white/90 leading-relaxed">{subtitle}</p>}
        </div>
      </div>
    </section>
  )
}

DynamicPageHero.propTypes = {
  title: PropTypes.string,
  subtitle: PropTypes.string,
  badge: PropTypes.string,
  gradient: PropTypes.string.isRequired,
}

export default DynamicPageHero
