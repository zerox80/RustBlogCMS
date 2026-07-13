import { useMemo } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import TutorialCard from './TutorialCard'
import { useTutorials } from '../context/TutorialContext'
import { AlertCircle } from 'lucide-react'
import { useContent } from '../context/ContentContext'
import { navigateContentTarget } from '../utils/contentNavigation'
const TutorialSection = () => {
  const { tutorials, getIconComponent, loading, error } = useTutorials()
  const navigate = useNavigate()
  const location = useLocation()
  const { getSection } = useContent()
  const sectionContent = getSection('tutorial_section') ?? {}
  const normalizedTutorials = useMemo(() => {
    return tutorials.map((tutorial) => ({
      ...tutorial,
      topics: Array.isArray(tutorial.topics) ? tutorial.topics : [],
    }))
  }, [tutorials])
  return (
    <section
      className="relative isolate overflow-hidden bg-slate-950 text-slate-100 py-24 sm:py-28"
      data-section="tutorials"
      id="tutorials"
    >
      <div className="absolute inset-0 -z-10">
        <div
          aria-hidden="true"
          className={`absolute inset-x-0 top-0 h-64 bg-gradient-to-b from-primary-500/15
via-primary-500/5 to-transparent`}
        ></div>
        <div
          aria-hidden="true"
          className="absolute -right-40 top-1/4 h-80 w-80 rounded-full bg-primary-400/20 blur-3xl"
        ></div>
        <div
          aria-hidden="true"
          className="absolute -left-32 bottom-0 h-72 w-72 rounded-full bg-cyan-400/10 blur-3xl"
        ></div>
      </div>
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="text-center mb-16">
          <h2 className="section-title">{sectionContent.title || 'Tutorial Inhalte'}</h2>
          <p className="mt-4 text-lg leading-relaxed text-slate-300 max-w-2xl mx-auto">
            {sectionContent.description ||
              'Umfassende Lernmodule für alle Erfahrungsstufen – vom Anfänger bis zum Profi'}
          </p>
        </div>
        {error && (
          <div
            className={`mb-10 flex items-start gap-3 rounded-xl border border-red-500/40
bg-red-500/10 p-4 text-red-100 backdrop-blur-sm`}
            role="alert"
          >
            <AlertCircle className="w-5 h-5 flex-shrink-0 text-red-200" aria-hidden="true" />
            <div>
              <p className="font-semibold text-red-50">Fehler beim Laden der Tutorials</p>
              <p className="text-sm text-red-100/80">{error?.message || String(error)}</p>
            </div>
          </div>
        )}
        {loading && normalizedTutorials.length === 0 ? (
          <div className="py-12 text-center text-neutral-300">Lade Tutorials…</div>
        ) : (
          <div className="grid grid-cols-1 gap-10 md:grid-cols-2 lg:grid-cols-3">
            {normalizedTutorials.map((tutorial) => (
              <TutorialCard
                key={tutorial.id}
                {...tutorial}
                icon={getIconComponent(tutorial.icon)}
                onSelect={() => navigate(`/tutorials/${tutorial.id}`)}
                buttonLabel={sectionContent.tutorialCardButton || 'Zum Tutorial'}
              />
            ))}
          </div>
        )}
        <div
          className={`mt-20 rounded-3xl border border-white/10 bg-slate-900/80 p-8 text-center
text-slate-100 shadow-card-xl md:p-12`}
        >
          <h3 className="mb-4 text-3xl font-semibold md:text-4xl text-white">
            {sectionContent.heading || 'Bereit anzufangen?'}
          </h3>
          <p className="mb-8 text-lg text-slate-300">
            {sectionContent.ctaDescription ||
              'Wähle ein Thema aus und starte deine Linux-Lernreise noch heute!'}
          </p>
          <div className="flex flex-col justify-center gap-4 sm:flex-row">
            <button
              onClick={() =>
                navigateContentTarget(sectionContent?.ctaPrimary?.target, { navigate, location })
              }
              className="btn-primary"
              aria-label="Tutorial starten und nach oben scrollen"
            >
              {sectionContent?.ctaPrimary?.label || 'Tutorial starten'}
            </button>
            <button
              onClick={() =>
                navigateContentTarget(sectionContent?.ctaSecondary?.target, { navigate, location })
              }
              className="btn-secondary-inverse"
              aria-label="Mehr über die Tutorials erfahren"
            >
              {sectionContent?.ctaSecondary?.label || 'Mehr erfahren'}
            </button>
          </div>
        </div>
      </div>
    </section>
  )
}
export default TutorialSection
