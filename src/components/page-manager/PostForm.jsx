import { useMemo, useState, useRef } from 'react'
import PropTypes from 'prop-types'
import { AlertCircle, FileText, RefreshCw, X, Image as ImageIcon, Loader2 } from 'lucide-react'
import { sanitizeSlug, isValidSlug } from '../../utils/slug'
import { sanitizeInteger } from './formUtils'
import { api } from '../../api/client'

const formatDateTimeLocal = (value) => {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return ''
  }
  const pad = (unit) => String(unit).padStart(2, '0')
  const year = date.getFullYear()
  const month = pad(date.getMonth() + 1)
  const day = pad(date.getDate())
  const hours = pad(date.getHours())
  const minutes = pad(date.getMinutes())
  return `${year}-${month}-${day}T${hours}:${minutes}`
}

const parseDateTimeLocal = (value) => {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return ''
  }
  return date.toISOString()
}

const PostForm = ({ mode, initialData, onSubmit, onCancel, submitting }) => {
  const textareaRef = useRef(null)
  const [title, setTitle] = useState(initialData?.title ?? '')
  const [slug, setSlug] = useState(initialData?.slug ?? '')
  const [excerpt, setExcerpt] = useState(initialData?.excerpt ?? '')
  const [content, setContent] = useState(initialData?.content_markdown ?? '')
  const [orderIndex, setOrderIndex] = useState(initialData?.order_index ?? 0)
  const [isPublished, setIsPublished] = useState(Boolean(initialData?.is_published))
  const [allowComments, setAllowComments] = useState(initialData?.allow_comments ?? true)
  const [publishedAt, setPublishedAt] = useState(initialData?.published_at ?? '')
  const [error, setError] = useState(null)
  const [uploading, setUploading] = useState(false)
  const publishedAtInputValue = useMemo(() => formatDateTimeLocal(publishedAt), [publishedAt])
  const sanitizedPostSlug = useMemo(() => sanitizeSlug(slug), [slug])
  const postSlugHasInput = slug.trim().length > 0
  const postSlugInvalid = postSlugHasInput && !sanitizedPostSlug
  const postSlugDiffers =
    postSlugHasInput && sanitizedPostSlug && sanitizedPostSlug !== slug.trim()
  const handleSubmit = async (event) => {
    event.preventDefault()
    setError(null)
    try {
      if (!title.trim()) {
        throw new Error('Titel darf nicht leer sein.')
      }
      if (!slug.trim()) {
        throw new Error('Slug darf nicht leer sein.')
      }
      if (!content.trim()) {
        throw new Error('Inhalt darf nicht leer sein.')
      }
      const sanitizedSlug = sanitizeSlug(slug.trim())
      if (!sanitizedSlug) {
        throw new Error('Slug darf nur Kleinbuchstaben, Zahlen und Bindestriche enthalten.')
      }
      if (!isValidSlug(sanitizedSlug)) {
        throw new Error('Slug ist ungültig.')
      }
      setSlug(sanitizedSlug)
      const payload = {
        title: title.trim(),
        slug: sanitizedSlug,
        excerpt: excerpt.trim() || null,
        content_markdown: content,
        order_index: sanitizeInteger(orderIndex),
        is_published: isPublished,
        allow_comments: allowComments,
        published_at: isPublished && publishedAt ? publishedAt : null,
      }
      await onSubmit(payload)
    } catch (err) {
      setError(err)
    }
  }

  const handleImageUpload = async (event) => {
    const file = event.target.files?.[0]
    if (!file) return

    setUploading(true)
    setError(null)

    try {
      const data = await api.uploadImage(file)
      const imageUrl = data.url
      const markdownImage = `\n![${file.name}](${imageUrl})\n`

      const textarea = textareaRef.current
      if (textarea) {
        const start = textarea.selectionStart
        const end = textarea.selectionEnd
        const text = content
        const newText = text.substring(0, start) + markdownImage + text.substring(end)
        setContent(newText)

        // Restore cursor position after the inserted image
        setTimeout(() => {
          textarea.focus()
          textarea.selectionStart = textarea.selectionEnd = start + markdownImage.length
        }, 0)
      } else {
        setContent((prev) => prev + markdownImage)
      }
    } catch (err) {
      if (err.status === 401) {
        setError(new Error('Deine Sitzung ist abgelaufen. Bitte melde dich erneut an.'))
      } else {
        setError(err)
      }
    } finally {
      setUploading(false)
      // Reset file input
      event.target.value = ''
    }
  }

  return (
    <div className="bg-white rounded-2xl shadow-2xl max-w-3xl w-full max-h-[90vh] overflow-y-auto dark:bg-slate-900">
      <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100 dark:border-slate-800">
        <div>
          <h3 className="text-xl font-semibold text-gray-900 dark:text-slate-100">
            {mode === 'edit' ? 'Beitrag bearbeiten' : 'Neuen Beitrag erstellen'}
          </h3>
          <p className="text-sm text-gray-500 dark:text-slate-400">
            Inhalte werden als Markdown gespeichert und auf der Seite gerendert.
          </p>
        </div>
        <button
          type="button"
          onClick={onCancel}
          className="p-2 rounded-lg text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:text-slate-400 dark:hover:text-slate-200 dark:hover:bg-slate-800"
        >
          <X className="w-5 h-5" />
        </button>
      </div>
      <form onSubmit={handleSubmit} className="space-y-6 px-6 py-6">
        {error && (
          <div className="flex items-start gap-2 rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-900/50 dark:bg-red-900/20 dark:text-red-300">
            <AlertCircle className="w-4 h-4 mt-0.5" />
            <div>
              <p className="font-medium">Speichern fehlgeschlagen</p>
              <p>{error.message}</p>
            </div>
          </div>
        )}
        <div className="grid gap-4 md:grid-cols-2">
          <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
            Titel
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
              required
            />
          </label>
          <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
            Slug
            <input
              type="text"
              className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
              value={slug}
              onChange={(event) => setSlug(event.target.value)}
              onBlur={() => setSlug(sanitizedPostSlug)}
              required
            />
            {postSlugInvalid && (
              <p className="mt-1 text-xs text-red-600 dark:text-red-400">
                Nur Kleinbuchstaben, Zahlen und Bindestriche erlaubt.
              </p>
            )}
            {postSlugDiffers && !postSlugInvalid && (
              <p className="mt-1 text-xs text-gray-500 dark:text-slate-400">
                Gespeicherter Slug:{' '}
                <code className="rounded bg-gray-100 px-1 py-0.5 text-[11px] dark:bg-slate-800 dark:text-slate-200">{sanitizedPostSlug}</code>
              </p>
            )}
          </label>
        </div>
        <div className="grid gap-4 md:grid-cols-2">
          <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
            Reihenfolge
            <input
              type="number"
              className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
              value={orderIndex}
              onChange={(event) => setOrderIndex(event.target.value)}
            />
          </label>
          <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
            Veröffentlichungsdatum (optional)
            <input
              type="datetime-local"
              className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
              value={publishedAtInputValue}
              onChange={(event) => setPublishedAt(parseDateTimeLocal(event.target.value))}
              disabled={!isPublished}
            />
          </label>
        </div>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
          Auszug
          <textarea
            className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            rows={3}
            value={excerpt}
            onChange={(event) => setExcerpt(event.target.value)}
            placeholder="Kurze Zusammenfassung des Beitrags"
          />
        </label>
        <label className="block text-sm font-medium text-gray-700 dark:text-slate-200">
          Inhalt (Markdown)
          <textarea
            ref={textareaRef}
            className="mt-1 w-full rounded-lg border border-gray-200 px-3 py-2 text-sm font-mono focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-100 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100"
            rows={12}
            value={content}
            onChange={(event) => setContent(event.target.value)}
            required
          />
          <div className="mt-2">
            <label className="inline-flex items-center gap-2 px-4 py-2 rounded-lg border border-gray-200 bg-white text-sm font-medium text-gray-700 hover:bg-gray-50 cursor-pointer dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200 dark:hover:bg-slate-700">
              {uploading ? <Loader2 className="w-4 h-4 animate-spin" /> : <ImageIcon className="w-4 h-4" />}
              <span>Bild hochladen & einfügen</span>
              <input
                type="file"
                className="hidden"
                accept="image/*"
                onChange={handleImageUpload}
                disabled={uploading}
              />
            </label>
            <p className="mt-1 text-xs text-gray-500 dark:text-slate-400">
              Das Bild wird hochgeladen und der Markdown-Code an der Cursor-Position eingefügt.
            </p>
          </div>
        </label>
        <label className="inline-flex items-center gap-2 text-sm text-gray-700 dark:text-slate-200">
          <input
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500 dark:border-slate-600 dark:bg-slate-900"
            checked={isPublished}
            onChange={(event) => {
              const nextValue = event.target.checked
              setIsPublished(nextValue)
              if (!nextValue) {
                setPublishedAt('')
              }
            }}
          />
          Veröffentlicht
        </label>
        <label className="inline-flex items-center gap-2 text-sm text-gray-700 dark:text-slate-200">
          <input
            type="checkbox"
            className="h-4 w-4 rounded border-gray-300 text-primary-600 focus:ring-primary-500 dark:border-slate-600 dark:bg-slate-900"
            checked={allowComments}
            onChange={(event) => setAllowComments(event.target.checked)}
          />
          Kommentare erlauben
        </label>
        <div className="flex justify-end gap-3 pt-2">
          <button
            type="button"
            onClick={onCancel}
            className="inline-flex items-center gap-2 rounded-lg border border-gray-200 px-4 py-2 text-sm font-medium text-gray-600 hover:bg-gray-50 dark:border-slate-700 dark:text-slate-200 dark:hover:bg-slate-800"
          >
            Abbrechen
          </button>
          <button
            type="submit"
            className="inline-flex items-center gap-2 rounded-lg bg-gradient-to-r from-primary-600 to-primary-700 px-5 py-2.5 text-sm font-semibold text-white shadow-lg hover:from-primary-700 hover:to-primary-800"
            disabled={submitting}
          >
            {submitting ? (
              <RefreshCw className="h-4 w-4 animate-spin" />
            ) : (
              <FileText className="h-4 w-4" />
            )}
            <span>{mode === 'edit' ? 'Änderungen speichern' : 'Beitrag erstellen'}</span>
          </button>
        </div>
      </form>
    </div>
  )
}
PostForm.propTypes = {
  mode: PropTypes.oneOf(['create', 'edit']).isRequired,
  initialData: PropTypes.object,
  onSubmit: PropTypes.func.isRequired,
  onCancel: PropTypes.func.isRequired,
  submitting: PropTypes.bool,
}

export default PostForm
