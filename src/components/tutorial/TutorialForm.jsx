import { useState, useEffect } from 'react'
import PropTypes from 'prop-types'
import { useTutorials } from '../../context/TutorialContext'
import { AlertCircle } from 'lucide-react'
import TutorialFormHeader from './TutorialFormHeader'
import TutorialBasicInfo from './TutorialBasicInfo'
import TutorialAppearance from './TutorialAppearance'
import TutorialTopics from './TutorialTopics'
import TutorialContent from './TutorialContent'
import TutorialFormActions from './TutorialFormActions'

const TutorialForm = ({ tutorial, onClose }) => {
  const { addTutorial, updateTutorial } = useTutorials()
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    icon: 'Terminal',
    color: 'from-blue-500 to-cyan-500',
    topics: [''],
    content: '',
  })
  const [submitting, setSubmitting] = useState(false)
  const [formError, setFormError] = useState('')

  useEffect(() => {
    if (tutorial) {
      const validTopics = Array.isArray(tutorial.topics)
        ? tutorial.topics.filter((t) => t && t.trim() !== '')
        : []
      setFormData({
        title: tutorial.title || '',
        description: tutorial.description || '',
        icon: tutorial.icon || 'Terminal',
        color: tutorial.color || 'from-blue-500 to-cyan-500',
        topics: validTopics.length > 0 ? validTopics : [''],
        content: tutorial.content || '',
      })
    } else {
      setFormData({
        title: '',
        description: '',
        icon: 'Terminal',
        color: 'from-blue-500 to-cyan-500',
        topics: [''],
        content: '',
      })
    }
  }, [tutorial])

  const handleSubmit = async (e) => {
    e.preventDefault()
    if (submitting) {
      return
    }

    const cleanedData = {
      title: formData.title.trim(),
      description: formData.description.trim(),
      icon: formData.icon,
      color: formData.color,
      topics: formData.topics.map((t) => (t || '').trim()).filter((t) => t !== ''),
      content: formData.content,
    }

    if (!cleanedData.title) {
      setFormError('Der Titel darf nicht leer sein.')
      return
    }
    if (!cleanedData.description) {
      setFormError('Die Beschreibung darf nicht leer sein.')
      return
    }
    if (cleanedData.topics.length === 0) {
      setFormError('Füge mindestens ein Thema hinzu.')
      return
    }
    if (!cleanedData.content.trim()) {
      setFormError('Der Inhalt darf nicht leer sein.')
      return
    }

    setFormError('')
    setSubmitting(true)

    try {
      if (tutorial) {
        await updateTutorial(tutorial.id, cleanedData)
      } else {
        await addTutorial(cleanedData)
      }
      onClose()
    } catch (error) {
      console.error('Tutorial save error:', error)
      let errorMessage = 'Fehler beim Speichern: '
      if (error.status === 502) {
        errorMessage += 'Der Server antwortet nicht. Bitte versuche es in ein paar Sekunden erneut.'
      } else if (error.status === 504 || error.status === 408) {
        errorMessage +=
          'Die Anfrage dauert zu lange. Versuche, weniger Inhalt auf einmal zu speichern.'
      } else if (error.status === 413) {
        errorMessage += 'Der Inhalt ist zu groß. Bitte reduziere die Größe des Tutorials.'
      } else if (error.status === 409) {
        errorMessage += 'Das Tutorial wurde von jemand anderem geändert. Bitte lade die Seite neu.'
      } else if (error.status >= 500) {
        errorMessage += 'Serverfehler. Bitte kontaktiere den Administrator.'
      } else {
        errorMessage += error.message || 'Unbekannter Fehler'
      }
      setFormError(errorMessage)
    } finally {
      setSubmitting(false)
    }
  }

  const handleChange = (field, value) => {
    setFormData({ ...formData, [field]: value })
  }

  const handleTopicChange = (index, value) => {
    const newTopics = [...formData.topics]
    newTopics[index] = value
    setFormData({ ...formData, topics: newTopics })
  }

  const addTopic = () => {
    setFormData({ ...formData, topics: [...formData.topics, ''] })
  }

  const removeTopic = (index) => {
    const newTopics = formData.topics.filter((_, i) => i !== index)
    setFormData({ ...formData, topics: newTopics })
  }

  return (
    <div className="p-8 bg-white dark:bg-slate-900 text-gray-900 dark:text-slate-100">
      <TutorialFormHeader isEditing={!!tutorial} onClose={onClose} />

      <form onSubmit={handleSubmit} className="space-y-6">
        {formError && (
          <div
            className={`flex items-start gap-3 rounded-lg border border-red-200 bg-red-50 p-4
text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300`}
            role="alert"
          >
            <AlertCircle className="w-5 h-5 mt-0.5" aria-hidden="true" />
            <span className="text-sm">{formError}</span>
          </div>
        )}

        <TutorialBasicInfo
          title={formData.title}
          description={formData.description}
          onChange={handleChange}
        />

        <TutorialAppearance icon={formData.icon} color={formData.color} onChange={handleChange} />

        <TutorialTopics
          topics={formData.topics}
          onTopicChange={handleTopicChange}
          onAddTopic={addTopic}
          onRemoveTopic={removeTopic}
        />

        <TutorialContent content={formData.content} onChange={handleChange} />

        <TutorialFormActions submitting={submitting} isEditing={!!tutorial} onClose={onClose} />
      </form>
    </div>
  )
}

TutorialForm.propTypes = {
  tutorial: PropTypes.shape({
    id: PropTypes.string,
    title: PropTypes.string,
    description: PropTypes.string,
    icon: PropTypes.string,
    color: PropTypes.string,
    topics: PropTypes.arrayOf(PropTypes.string),
    content: PropTypes.string,
  }),
  onClose: PropTypes.func.isRequired,
}

export default TutorialForm
