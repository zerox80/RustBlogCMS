import { useState, useEffect, useRef } from 'react'
import { Search, X, Tag } from 'lucide-react'
import { useNavigate } from 'react-router-dom'
import PropTypes from 'prop-types'
import { api } from '../../api/client'
const SearchBar = ({ onClose }) => {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState([])
  const [topics, setTopics] = useState([])
  const [selectedTopic, setSelectedTopic] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const inputRef = useRef(null)
  const dialogRef = useRef(null)
  const previouslyFocusedElement = useRef(null)
  const navigate = useNavigate()
  useEffect(() => {
    previouslyFocusedElement.current = document.activeElement
    inputRef.current?.focus()
    api
      .request('/search/topics', { cacheBust: false })
      .then((data) => setTopics(Array.isArray(data) ? data : []))
      .catch((err) => {
        console.error('Failed to fetch topics:', err)
        setTopics([])
      })
  }, [])
  useEffect(() => {
    const handleKeyDown = (event) => {
      if (event.key === 'Escape') {
        event.preventDefault()
        if (onClose) {
          onClose()
        }
        return
      }
      if (event.key === 'Tab' && dialogRef.current) {
        const focusableSelectors = [
          'a[href]:not([tabindex="-1"])',
          'button:not([disabled]):not([tabindex="-1"])',
          'textarea:not([disabled]):not([tabindex="-1"])',
          'input:not([disabled]):not([tabindex="-1"])',
          'select:not([disabled]):not([tabindex="-1"])',
          '[tabindex]:not([tabindex="-1"])',
        ].join(',')
        const focusable = Array.from(dialogRef.current.querySelectorAll(focusableSelectors)).filter(
          (element) =>
            !element.hasAttribute('disabled') &&
            element.getAttribute('aria-hidden') !== 'true' &&
            (element.offsetWidth > 0 ||
              element.offsetHeight > 0 ||
              element.getClientRects().length > 0),
        )
        if (!focusable.length) {
          return
        }
        const first = focusable[0]
        const last = focusable[focusable.length - 1]
        const activeElement = document.activeElement
        if (!event.shiftKey && activeElement === last) {
          event.preventDefault()
          first.focus()
        } else if (event.shiftKey && activeElement === first) {
          event.preventDefault()
          last.focus()
        }
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
      previouslyFocusedElement.current?.focus?.()
    }
  }, [onClose])
  useEffect(() => {
    if (query.trim().length < 1) {
      setResults([])
      return
    }

    const controller = new AbortController()
    const timeoutId = setTimeout(async () => {
      setIsLoading(true)
      try {
        const params = new URLSearchParams({ q: query })
        if (selectedTopic) {
          params.append('topic', selectedTopic)
        }
        const data = await api.request(`/search/tutorials?${params.toString()}`, {
          cacheBust: false,
          signal: controller.signal,
        })
        setResults(Array.isArray(data) ? data : [])
      } catch (error) {
        if (error.name !== 'AbortError') {
          console.error('Search failed:', error)
          setResults([])
        }
      } finally {
        if (!controller.signal.aborted) {
          setIsLoading(false)
        }
      }
    }, 300)

    return () => {
      clearTimeout(timeoutId)
      controller.abort()
    }
  }, [query, selectedTopic])
  const handleResultClick = (id) => {
    navigate(`/tutorials/${id}`)
    if (onClose) onClose()
  }
  return (
    <div
      className={[
        'fixed inset-0 bg-black/50 dark:bg-black/70 z-50 flex items-start',
        'justify-center pt-20 px-4',
      ].join(' ')}
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget && onClose) {
          onClose()
        }
      }}
    >
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="search-dialog-title"
        className={`bg-white dark:bg-gray-800 rounded-2xl shadow-2xl w-full max-w-2xl
max-h-[80vh] flex flex-col`}
      >
        {}
        <div className="p-4 border-b dark:border-gray-700">
          <h2
            id="search-dialog-title"
            className="text-lg font-semibold text-gray-900 dark:text-gray-100"
          >
            Tutorialsuche
          </h2>
          <div className="relative">
            <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Tutorial suchen..."
              className={`w-full pl-12 pr-12 py-3 bg-gray-50 dark:bg-gray-900 border border-gray-200
dark:border-gray-700 rounded-xl focus:outline-none focus:ring-2
focus:ring-primary-500 dark:text-gray-100`}
            />
            {onClose && (
              <button
                onClick={onClose}
                className={`absolute right-4 top-1/2 -translate-y-1/2 p-1 hover:bg-gray-200
dark:hover:bg-gray-700 rounded-lg transition-colors`}
                aria-label="Suche schließen"
              >
                <X className="w-5 h-5 text-gray-500" />
              </button>
            )}
          </div>
          {}
          {topics.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-2">
              <button
                onClick={() => setSelectedTopic('')}
                className={`px-3 py-1 rounded-lg text-sm transition-colors ${
                  !selectedTopic
                    ? 'bg-primary-600 text-white'
                    : [
                        'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300',
                        'hover:bg-gray-200 dark:hover:bg-gray-600',
                      ].join(' ')
                }`}
              >
                Alle
              </button>
              {topics.slice(0, 10).map((topic) => (
                <button
                  key={topic}
                  onClick={() => setSelectedTopic(topic)}
                  className={`flex items-center gap-1 px-3 py-1 rounded-lg text-sm transition-colors ${
                    selectedTopic === topic
                      ? 'bg-primary-600 text-white'
                      : [
                          'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300',
                          'hover:bg-gray-200 dark:hover:bg-gray-600',
                        ].join(' ')
                  }`}
                >
                  <Tag className="w-3 h-3" />
                  {topic}
                </button>
              ))}
            </div>
          )}
        </div>
        {}
        <div className="flex-1 overflow-y-auto p-4">
          {isLoading && (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">Suche läuft...</div>
          )}
          {!isLoading && query.trim().length >= 1 && results.length === 0 && (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              Keine Ergebnisse gefunden
            </div>
          )}
          {!isLoading && query.trim().length < 1 && (
            <div className="text-center py-8 text-gray-500 dark:text-gray-400">
              Gib mindestens 1 Zeichen ein
            </div>
          )}
          {!isLoading && results.length > 0 && (
            <div className="space-y-2">
              {results.map((tutorial) => (
                <button
                  key={tutorial.id}
                  onClick={() => handleResultClick(tutorial.id)}
                  className={`w-full text-left p-4 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700
transition-colors group`}
                >
                  <h3
                    className={`font-semibold text-gray-900 dark:text-gray-100 group-hover:text-primary-600
dark:group-hover:text-primary-400`}
                  >
                    {tutorial.title}
                  </h3>
                  <p className="text-sm text-gray-600 dark:text-gray-400 mt-1 line-clamp-2">
                    {tutorial.description}
                  </p>
                  <div className="flex flex-wrap gap-1 mt-2">
                    {tutorial.topics.slice(0, 3).map((topic) => (
                      <span
                        key={topic}
                        className={`px-2 py-0.5 bg-primary-100 dark:bg-primary-900/30 text-primary-700
dark:text-primary-300 text-xs rounded`}
                      >
                        {topic}
                      </span>
                    ))}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
SearchBar.propTypes = {
  onClose: PropTypes.func,
}
export default SearchBar
