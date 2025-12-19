import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { ArrowLeft, Loader2, AlertCircle } from 'lucide-react'
import { useTutorials } from '../context/TutorialContext'
import { api } from '../api/client'
import TutorialHeader from '../components/tutorial/TutorialHeader'
import TutorialTopicsList from '../components/tutorial/TutorialTopicsList'
import TutorialContentDisplay from '../components/tutorial/TutorialContentDisplay'

/**
 * Detailed Tutorial Viewer.
 * 
 * Features:
 * - Hybrid Loading: Attempts to use cached context data before fetching from API.
 * - Modular Rendering: Decouples header, topics, and content for scalability.
 * - Nav Logic: Intelligent "Go Back" that stays within app history when possible.
 */
const TutorialDetail = () => {
  const { id } = useParams()
  const navigate = useNavigate()
  const { getTutorial, tutorials } = useTutorials()
  const [tutorial, setTutorial] = useState(() => getTutorial(id))
  const [loading, setLoading] = useState(!getTutorial(id))
  const [error, setError] = useState(null)

  useEffect(() => {
    const controller = new AbortController()
    const fetchTutorial = async () => {
      try {
        setLoading(true)
        const data = await api.getTutorial(id, { signal: controller.signal })
        if (!controller.signal.aborted) {
          setTutorial(data)
          setError(null)
        }
      } catch (err) {
        if (!controller.signal.aborted) {
          setError(err)
        }
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      }
    }
    fetchTutorial()
    return () => {
      controller.abort()
    }
  }, [id])

  useEffect(() => {
    if (!Array.isArray(tutorials)) {
      setTutorial(null)
      return
    }
    const cached = tutorials.find((item) => item.id === id)
    setTutorial(cached || null)
  }, [id, tutorials])

  const topics = useMemo(() => {
    if (!tutorial?.topics) {
      return []
    }
    return Array.isArray(tutorial.topics) ? tutorial.topics : []
  }, [tutorial])

  const handleBack = () => {
    if (window.history.length > 1) {
      navigate(-1)
      return
    }
    navigate('/')
  }

  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-100 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950 pt-28 pb-16">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
        <button
          onClick={handleBack}
          className="group inline-flex items-center gap-2 text-primary-700 font-medium mb-8"
        >
          <ArrowLeft className="w-4 h-4 transition-transform duration-200 group-hover:-translate-x-1" />
          Zurück
        </button>

        {loading ? (
          <div className="flex flex-col items-center justify-center py-24 text-gray-500">
            <Loader2 className="w-10 h-10 animate-spin mb-4" />
            <p>Inhalt wird geladen…</p>
          </div>
        ) : error ? (
          <div className="rounded-2xl border border-red-200 bg-red-50 p-6 text-red-700 flex gap-3">
            <AlertCircle className="w-6 h-6 flex-shrink-0" />
            <div>
              <h2 className="font-semibold mb-1">Tutorial konnte nicht geladen werden</h2>
              <p className="text-sm">{error?.message || 'Unbekannter Fehler'}</p>
            </div>
          </div>
        ) : tutorial ? (
          <article className="bg-white dark:bg-slate-900/90 rounded-3xl shadow-xl border border-gray-200 dark:border-slate-800/70 overflow-hidden">
            <TutorialHeader
              title={tutorial.title}
              description={tutorial.description}
            />

            <div className="px-8 py-10 space-y-12">
              <TutorialTopicsList topics={topics} />
              <TutorialContentDisplay content={tutorial.content} />
            </div>
          </article>
        ) : (
          <div className="rounded-2xl border border-yellow-200 bg-yellow-50 p-6 text-yellow-800">
            Das gewünschte Tutorial wurde nicht gefunden.
          </div>
        )}
      </div>
    </main>
  )
}

export default TutorialDetail
