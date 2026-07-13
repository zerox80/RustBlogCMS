import { useState } from 'react'
import { ArrowRight, Loader2, Mail } from 'lucide-react'
import PropTypes from 'prop-types'
import { api } from '../../api/client'
import EditableText from '../cms/EditableText'

const NewsletterSection = ({ content }) => {
  const [email, setEmail] = useState('')
  const [status, setStatus] = useState(null)
  const [submitting, setSubmitting] = useState(false)

  const handleSubmit = async (event) => {
    event.preventDefault()
    setSubmitting(true)
    setStatus(null)
    try {
      await api.subscribeNewsletter(email)
      setEmail('')
      setStatus({ type: 'success', message: 'Danke! Du bist jetzt eingetragen.' })
    } catch (error) {
      setStatus({
        type: 'error',
        message: error?.message || 'Die Anmeldung ist gerade nicht möglich.',
      })
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <section
      id="newsletter"
      className="bg-[#ff4f00] px-5 py-16 text-white sm:px-8 lg:px-12 lg:py-20"
    >
      <div className="mx-auto grid max-w-[1480px] gap-10 lg:grid-cols-[1fr_0.8fr] lg:items-end">
        <div>
          <p
            className={`mb-4 flex items-center gap-2 font-mono text-xs font-bold uppercase
tracking-[0.2em]`}
          >
            <Mail className="h-4 w-4" />{' '}
            <EditableText
              section="cta_section"
              field="title"
              value={content?.title || 'Neue Notizen per Mail'}
            />
          </p>
          <h2
            className={`max-w-4xl font-display text-5xl font-semibold leading-[0.92]
tracking-[-0.06em] text-white sm:text-7xl lg:text-8xl`}
          >
            <EditableText
              section="cta_section"
              field="description"
              value={
                content?.description ||
                'Ich melde mich, wenn es einen neuen Gedanken oder Beitrag zu teilen gibt.'
              }
              multiline
            />
          </h2>
        </div>
        <form className="border-b-2 border-white pb-3" onSubmit={handleSubmit}>
          <label htmlFor="newsletter-email" className="sr-only">
            E-Mail-Adresse
          </label>
          <div className="flex items-center gap-3">
            <input
              id="newsletter-email"
              name="email"
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@example.com"
              autoComplete="email"
              maxLength="254"
              required
              className={`min-w-0 flex-1 bg-transparent py-3 text-xl text-white outline-none
placeholder:text-white/55`}
            />
            <button
              type="submit"
              aria-label="Newsletter abonnieren"
              disabled={submitting}
              className={`grid h-12 w-12 shrink-0 place-items-center rounded-full bg-[#171713]
transition-transform hover:rotate-[-12deg] disabled:cursor-wait disabled:opacity-70`}
            >
              {submitting ? (
                <Loader2 className="h-5 w-5 animate-spin" />
              ) : (
                <ArrowRight className="h-5 w-5" />
              )}
            </button>
          </div>
          {status && (
            <p className="pt-3 text-sm font-semibold" role={status.type === 'error' ? 'alert' : 'status'}>
              {status.message}
            </p>
          )}
        </form>
      </div>
    </section>
  )
}

NewsletterSection.propTypes = {
  content: PropTypes.object,
}

export default NewsletterSection
