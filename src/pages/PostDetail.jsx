import { useEffect, useMemo, useState } from 'react'
import { Helmet } from 'react-helmet-async'
import { ArrowLeft, Asterisk, CalendarDays, Clock3, Loader2, Share2 } from 'lucide-react'
import { Link, useParams } from 'react-router-dom'
import { api } from '../api/client'
import MarkdownRenderer from '../components/markdown/MarkdownRenderer'
import { formatDate } from '../utils/postUtils'

const readingTime = (content) => {
  const words = String(content || '')
    .trim()
    .split(/\s+/)
    .filter(Boolean).length
  return Math.max(2, Math.ceil(words / 200))
}

/** Editorial article view matching the public one-page blog design. */
const PostDetail = () => {
  const { pageSlug, postSlug } = useParams()
  const [post, setPost] = useState(null)
  const [error, setError] = useState(null)
  const [shareLabel, setShareLabel] = useState('Teilen')

  useEffect(() => {
    const controller = new AbortController()

    const loadPost = async () => {
      try {
        setError(null)
        const data = await api.getPublishedPost(pageSlug, postSlug, {
          signal: controller.signal,
        })
        if (!controller.signal.aborted) setPost(data?.post || data)
      } catch (loadError) {
        if (!controller.signal.aborted) setError(loadError)
      }
    }

    loadPost()
    return () => controller.abort()
  }, [pageSlug, postSlug])

  const minutes = useMemo(() => readingTime(post?.content_markdown), [post?.content_markdown])

  const handleShare = async () => {
    try {
      if (navigator.share) {
        await navigator.share({ title: post.title, url: window.location.href })
      } else {
        await navigator.clipboard.writeText(window.location.href)
        setShareLabel('Link kopiert')
        window.setTimeout(() => setShareLabel('Teilen'), 1800)
      }
    } catch (shareError) {
      if (shareError?.name !== 'AbortError') setShareLabel('Nicht möglich')
    }
  }

  if (error) {
    return (
      <main
        className={`grid min-h-[75vh] place-items-center bg-[#f4f1ea] px-6 pt-28
text-[#171713]`}
      >
        <div className="max-w-lg text-center">
          <p className="font-serif text-7xl italic text-[#ff4f00]">Oops.</p>
          <h1 className="mt-4 text-3xl font-semibold">Dieser Beitrag ist nicht verfügbar.</h1>
          <Link
            to="/#stories"
            className={`mt-8 inline-flex items-center gap-2 rounded-full bg-[#171713] px-6 py-3
text-sm font-bold uppercase tracking-[0.12em] text-white`}
          >
            <ArrowLeft className="h-4 w-4" /> Alle Beiträge
          </Link>
        </div>
      </main>
    )
  }

  if (!post) {
    return (
      <main className="grid min-h-[75vh] place-items-center bg-[#f4f1ea] pt-28 text-[#171713]">
        <div className="flex items-center gap-3 font-mono text-xs font-bold uppercase tracking-[0.18em]">
          <Loader2 className="h-5 w-5 animate-spin text-[#ff4f00]" /> Beitrag wird geladen
        </div>
      </main>
    )
  }

  const publishedDate = formatDate(post.published_at || post.created_at)

  return (
    <main className="bg-[#f4f1ea] pb-24 pt-28 text-[#171713] sm:pt-32">
      <Helmet>
        <title>{post.meta?.title || post.title}</title>
        {post.excerpt && <meta name="description" content={post.excerpt} />}
      </Helmet>

      <article>
        <header className="border-b border-[#171713] px-5 pb-14 sm:px-8 lg:px-12 lg:pb-20">
          <div className="mx-auto max-w-[1180px]">
            <Link
              to="/#stories"
              className={`inline-flex items-center gap-2 text-xs font-bold uppercase
tracking-[0.14em] text-[#171713]/60 transition-colors hover:text-[#ff4f00]`}
            >
              <ArrowLeft className="h-4 w-4" /> Zurück zum Blog
            </Link>

            <div className="mt-12 grid gap-10 lg:grid-cols-[minmax(0,1fr)_15rem] lg:items-end">
              <div>
                <div
                  className={`mb-6 flex items-center gap-3 font-mono text-[11px] font-bold
uppercase tracking-[0.18em] text-[#ff4f00]`}
                >
                  <Asterisk className="h-4 w-4" /> Persönlich notiert
                </div>
                <h1
                  className={`max-w-5xl font-display text-[clamp(3.4rem,8vw,7.8rem)]
font-semibold leading-[0.88] tracking-[-0.065em]`}
                >
                  {post.title}
                </h1>
                {post.excerpt && (
                  <p className="mt-8 max-w-3xl text-xl leading-relaxed text-[#171713]/60 sm:text-2xl">
                    {post.excerpt}
                  </p>
                )}
              </div>

              <div
                className={`border-t border-[#171713]/20 pt-5 font-mono text-[11px] font-bold
uppercase tracking-[0.13em] text-[#171713]/55 lg:border-l lg:border-t-0
lg:pl-6 lg:pt-0`}
              >
                {publishedDate && (
                  <p className="flex items-center gap-2 py-2">
                    <CalendarDays className="h-4 w-4 text-[#ff4f00]" /> {publishedDate}
                  </p>
                )}
                <p className="flex items-center gap-2 py-2">
                  <Clock3 className="h-4 w-4 text-[#ff4f00]" /> {minutes} Min. Lesezeit
                </p>
                <button
                  type="button"
                  onClick={handleShare}
                  className="flex items-center gap-2 py-2 transition-colors hover:text-[#ff4f00]"
                >
                  <Share2 className="h-4 w-4 text-[#ff4f00]" /> {shareLabel}
                </button>
              </div>
            </div>
          </div>
        </header>

        <div className="px-5 py-14 sm:px-8 lg:px-12 lg:py-20">
          <MarkdownRenderer
            content={post.content_markdown}
            withBreaks
            className="editorial-markdown mx-auto max-w-[780px]"
          />
        </div>

        <footer className="mx-auto max-w-[1180px] border-t border-[#171713] px-5 pt-10 sm:px-8">
          <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
            <p className="font-serif text-3xl italic">Danke fürs Lesen.</p>
            <Link
              to="/#stories"
              className={`inline-flex items-center gap-2 self-start rounded-full bg-[#171713]
px-6 py-3 text-xs font-bold uppercase tracking-[0.12em] text-white transition-colors
hover:bg-[#ff4f00]`}
            >
              <ArrowLeft className="h-4 w-4" /> Weitere Beiträge
            </Link>
          </div>
        </footer>
      </article>
    </main>
  )
}

export default PostDetail
