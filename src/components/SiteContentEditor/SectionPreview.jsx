import PropTypes from 'prop-types'

const HeroPreview = ({ content }) => {
  return (
    <div className="relative overflow-hidden rounded-2xl bg-slate-900 p-8 shadow-xl">
      <div
        className={`absolute inset-0
bg-[linear-gradient(to_right,#4f4f4f2e_1px,transparent_1px),linear-gradient(to_bottom,#4f4f4f2e_1px,transparent_1px)]
bg-[size:14px_24px]
[mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]`}
      ></div>
      <div className="relative z-10 text-center">
        <div className="mb-6 flex justify-center">
          <span
            className={`inline-flex items-center gap-2 rounded-full border border-primary-500/30
bg-primary-500/10 px-4 py-1.5 text-sm font-medium text-primary-400
backdrop-blur-sm`}
          >
            <span className="relative flex h-2 w-2">
              <span
                className={`absolute inline-flex h-full w-full animate-ping rounded-full bg-primary-400
opacity-75`}
              ></span>
              <span className="relative inline-flex h-2 w-2 rounded-full bg-primary-500"></span>
            </span>
            {content.badgeText || 'Persönlicher Blog'}
          </span>
        </div>
        <h3 className="mb-6 text-3xl font-bold tracking-tight text-white sm:text-4xl">
          {content?.title?.line1}
          <span
            className={[
              'block bg-gradient-to-r from-primary-400 to-purple-400 bg-clip-text',
              'text-transparent',
            ].join(' ')}
          >
            {content?.title?.line2}
          </span>
        </h3>
        <p className="mx-auto mb-8 max-w-2xl text-lg text-slate-400">{content.subtitle}</p>
        {content.subline && <p className="text-sm text-slate-500">{content.subline}</p>}
      </div>
    </div>
  )
}

HeroPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const StatsPreview = ({ content }) => {
  const items = Array.isArray(content.items) ? content.items : []
  return (
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <div className="flex items-center justify-between mb-6">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">
          Statistiken Vorschau
        </h4>
        <span
          className={`rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs
font-medium text-primary-700`}
        >
          Vorschau
        </span>
      </div>
      <div className="rounded-xl bg-slate-950 p-8">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
          {items.map((stat, i) => (
            <div key={i}>
              <div className="text-2xl font-bold text-white mb-1">{stat.value}</div>
              <div className="text-sm text-slate-500 font-medium">{stat.label}</div>
            </div>
          ))}
          {items.length === 0 && (
            <div className="col-span-full text-slate-500">Keine Statistiken vorhanden.</div>
          )}
        </div>
      </div>
    </div>
  )
}

StatsPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const CtaSectionPreview = ({ content }) => {
  return (
    <div
      className={`rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <div className="flex items-center justify-between mb-6">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">CTA Vorschau</h4>
        <span
          className={`rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs
font-medium text-primary-700`}
        >
          Vorschau
        </span>
      </div>
      <div className="rounded-xl bg-slate-950 p-8 text-center">
        <h2 className="text-2xl font-bold mb-4 text-white">
          {content.title || 'Neue Notizen per Mail'}
        </h2>
        <p className="text-lg text-slate-400">
          {content.description || 'Ich melde mich, wenn es etwas Neues zu teilen gibt.'}
        </p>
      </div>
    </div>
  )
}

CtaSectionPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

const SiteMetaPreview = ({ content }) => {
  return (
    <div
      className={`space-y-4 rounded-2xl border border-gray-200 bg-white p-6 shadow-sm
dark:border-slate-700 dark:bg-slate-900/80`}
    >
      <div className="flex items-center justify-between">
        <h4 className="text-lg font-semibold text-gray-900 dark:text-slate-100">
          Seitentitel (Tab)
        </h4>
        <span
          className={`rounded-full border border-primary-200 bg-primary-50 px-3 py-1 text-xs
font-medium text-primary-700`}
        >
          Vorschau
        </span>
      </div>
      <div className="rounded-xl border border-gray-100 bg-gray-50 p-4">
        <p className="text-sm font-semibold text-gray-800">
          {content?.title || 'minos – Persönlicher Blog'}
        </p>
        <p className="mt-2 text-sm text-gray-600 leading-relaxed">
          {content?.description ||
            'Persönliche Notizen über Technik, Projekte, Ideen und alles dazwischen.'}
        </p>
      </div>
      <p className="text-xs text-gray-500">
        Diese Angaben erscheinen als Browser-Titel und Meta-Beschreibung (z. B. in Suchmaschinen).
      </p>
    </div>
  )
}

const AboutPreview = ({ content }) => (
  <div className="rounded-2xl bg-[#171713] p-8 text-[#f4f1ea] shadow-xl">
    <p className="text-xs font-bold uppercase tracking-[0.2em] text-[#b9f227]">
      {content?.eyebrow || 'Warum ich schreibe'}
    </p>
    <p className="mt-5 font-serif text-3xl leading-tight">
      {content?.lead || 'Ein persönlicher Ort für Gedanken und Erfahrungen.'}
    </p>
    <div className="mt-6 grid gap-4 border-t border-white/20 pt-5 sm:grid-cols-2">
      {(Array.isArray(content?.paragraphs) ? content.paragraphs : []).slice(0, 2).map((text, index) => (
        <p key={`about-preview-${index}`} className="text-sm leading-relaxed text-white/60">
          {text}
        </p>
      ))}
    </div>
  </div>
)

AboutPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

SiteMetaPreview.propTypes = {
  content: PropTypes.object.isRequired,
}

export const SectionPreview = ({ section, content }) => {
  switch (section) {
    case 'hero':
      return <HeroPreview content={content} />
    case 'stats':
      return <StatsPreview content={content} />
    case 'cta_section':
      return <CtaSectionPreview content={content} />
    case 'about':
      return <AboutPreview content={content} />
    case 'site_meta':
      return <SiteMetaPreview content={content} />
    default:
      return (
        <div
          className={`rounded-2xl border border-gray-200 bg-white p-6 text-sm text-gray-600
shadow-sm`}
        >
          <p className="font-semibold text-gray-700">Vorschau nicht verfügbar</p>
          <p className="mt-2 text-gray-500">
            Nutze das Formular oben, um die Inhalte zu bearbeiten.
          </p>
        </div>
      )
  }
}

SectionPreview.propTypes = {
  section: PropTypes.string.isRequired,
  content: PropTypes.object.isRequired,
}
