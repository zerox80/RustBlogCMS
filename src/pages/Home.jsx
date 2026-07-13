import { useEffect, useMemo, useState } from 'react'
import { ArrowDownRight, ArrowRight, Asterisk, Loader2, Mail, Sparkles } from 'lucide-react'
import PostCard from '../components/dynamic-page/PostCard'
import { api } from '../api/client'

const ALL_TOPICS = 'Alle Beiträge'

/** Personal one-page blog that collects published posts from every CMS page. */
const Home = () => {
  const [posts, setPosts] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [activeTopic, setActiveTopic] = useState(ALL_TOPICS)

  useEffect(() => {
    const fetchAllPosts = async () => {
      try {
        setLoading(true)
        setError(null)

        const pagesData = await api.listPublishedPages()
        const publishedPages = Array.isArray(pagesData)
          ? pagesData
          : Array.isArray(pagesData?.items)
            ? pagesData.items
            : []

        const pageRequests = publishedPages
          .map((pageReference) => ({
            reference: pageReference,
            slug: typeof pageReference === 'string' ? pageReference : pageReference?.slug,
          }))
          .filter(({ slug }) => Boolean(slug))
          .map(async ({ reference, slug }) => {
            const pageData = await api.getPublishedPage(slug)
            return (pageData?.posts || []).map((post) => ({
              ...post,
              pageSlug: slug,
              pageTitle: pageData?.page?.title || reference?.title || slug,
            }))
          })

        const settledPages = await Promise.allSettled(pageRequests)
        const allPosts = settledPages
          .filter((result) => result.status === 'fulfilled')
          .flatMap((result) => result.value)
          .sort((a, b) => {
            const dateA = new Date(a.published_at || a.created_at || 0)
            const dateB = new Date(b.published_at || b.created_at || 0)
            return dateB - dateA
          })

        setPosts(allPosts)
      } catch (err) {
        console.error('Error fetching posts:', err)
        setError(err)
      } finally {
        setLoading(false)
      }
    }

    fetchAllPosts()
  }, [])

  const topics = useMemo(
    () => [ALL_TOPICS, ...new Set(posts.map((post) => post.pageTitle).filter(Boolean))],
    [posts],
  )

  const visiblePosts = useMemo(
    () =>
      activeTopic === ALL_TOPICS ? posts : posts.filter((post) => post.pageTitle === activeTopic),
    [activeTopic, posts],
  )

  const currentYear = new Date().getFullYear()

  return (
    <main className="blog-home overflow-hidden bg-[#f4f1ea] text-[#171713]">
      <section
        id="home"
        className={`relative min-h-[92vh] border-b border-[#171713]/15 px-5 pb-10 pt-32 sm:px-8
lg:px-12 lg:pb-12`}
      >
        <div className="editorial-grid pointer-events-none absolute inset-0 opacity-40" />
        <div
          className={[
            'relative mx-auto flex min-h-[calc(92vh-10rem)] max-w-[1480px] flex-col',
            'justify-between',
          ].join(' ')}
        >
          <div
            className={`flex items-center justify-between gap-6 border-y border-[#171713]/20 py-3
text-[11px] font-bold uppercase tracking-[0.2em] sm:text-xs`}
          >
            <span>Mein persönlicher Blog</span>
            <span className="hidden items-center gap-2 sm:flex">
              <span className="h-2 w-2 animate-pulse rounded-full bg-[#ff4f00]" />
              Persönliche Notizen · {currentYear}
            </span>
            <span>Gedanken · Projekte · Fundstücke</span>
          </div>

          <div
            className={`grid flex-1 items-center gap-10 py-12 lg:grid-cols-[minmax(0,1fr)_24rem]
lg:py-16 xl:grid-cols-[minmax(0,1fr)_29rem]`}
          >
            <div className="max-w-5xl">
              <div
                className={`mb-6 flex items-center gap-3 text-sm font-semibold uppercase
tracking-[0.18em] text-[#ff4f00]`}
              >
                <Sparkles className="h-4 w-4" />
                Dinge, die mich gerade beschäftigen
              </div>
              <h1
                className={`max-w-5xl font-display text-[clamp(4.2rem,10.5vw,10rem)] font-semibold
leading-[0.78] tracking-[-0.075em] text-[#171713]`}
              >
                Ich denke
                <span className="block font-serif font-normal italic tracking-[-0.055em] text-[#ff4f00]">
                  hier laut.
                </span>
              </h1>
              <p className="mt-8 max-w-xl text-lg leading-relaxed text-[#171713]/65 sm:text-xl">
                Hier schreibe ich über Technik, Projekte, Ideen und alles, was ich unterwegs besser
                verstehen möchte.
              </p>
            </div>

            <div className="relative mx-auto w-full max-w-md lg:mx-0">
              <div className="hero-orbit aspect-square rounded-full border border-[#171713]/20 p-7 sm:p-10">
                <div
                  className={`relative flex h-full flex-col justify-between overflow-hidden rounded-full
bg-[#171713] p-10 text-[#f4f1ea] shadow-[0_30px_80px_rgba(23,23,19,0.24)]
sm:p-12`}
                >
                  <Asterisk
                    className="h-14 w-14 animate-[spin_16s_linear_infinite] text-[#b9f227]"
                    strokeWidth={1.5}
                  />
                  <div>
                    <p className="mb-3 font-mono text-xs uppercase tracking-[0.2em] text-[#f4f1ea]/50">
                      Gerade im Kopf
                    </p>
                    <p className="font-serif text-3xl leading-tight sm:text-4xl">
                      Beobachten.
                      <br />
                      Ausprobieren. Teilen.
                    </p>
                  </div>
                  <a
                    href="#stories"
                    className={`group flex items-center justify-between border-t border-white/20 pt-5
text-sm font-bold uppercase tracking-[0.16em]`}
                  >
                    Beiträge lesen
                    <ArrowDownRight
                      className={`h-5 w-5 transition-transform group-hover:translate-x-1
group-hover:translate-y-1`}
                    />
                  </a>
                </div>
              </div>
              <div
                className={`absolute -right-2 top-8 rounded-full bg-[#b9f227] px-5 py-3 text-xs
font-black uppercase tracking-[0.16em] shadow-lg rotate-6`}
              >
                Persönlich notiert
              </div>
            </div>
          </div>

          <div
            className={`flex flex-col gap-4 border-t border-[#171713]/20 pt-5 text-sm sm:flex-row
sm:items-center sm:justify-between`}
          >
            <p className="max-w-xl text-[#171713]/55">
              Kein Redaktionsplan, keine feste Nische: einfach mein Platz für Gedanken, Erfahrungen
              und Dinge, die ich gelernt habe.
            </p>
            <a
              href="#stories"
              className={`inline-flex items-center gap-3 font-bold uppercase tracking-[0.14em]
hover:text-[#ff4f00]`}
            >
              Neueste Beiträge <ArrowRight className="h-4 w-4" />
            </a>
          </div>
        </div>
      </section>

      <section
        id="topics"
        aria-label="Themen"
        className="border-b border-[#171713] bg-[#b9f227] py-4"
      >
        <div
          className={`topic-marquee flex min-w-max items-center gap-8 whitespace-nowrap text-sm
font-black uppercase tracking-[0.16em]`}
        >
          {[
            'Rust',
            'Security',
            'Open Source',
            'DevOps',
            'Digital Culture',
            'Linux',
            'Web Engineering',
            'Rust',
            'Security',
            'Open Source',
            'DevOps',
            'Digital Culture',
          ].map((topic, index) => (
            <span key={`${topic}-${index}`} className="flex items-center gap-8">
              {topic}
              <Asterisk className="h-4 w-4" />
            </span>
          ))}
        </div>
      </section>

      <section id="stories" className="px-5 py-20 sm:px-8 lg:px-12 lg:py-28">
        <div className="mx-auto max-w-[1480px]">
          <div
            className={[
              'grid gap-10 border-b border-[#171713] pb-10 lg:grid-cols-[1fr_auto]',
              'lg:items-end',
            ].join(' ')}
          >
            <div>
              <p className="mb-4 font-mono text-xs font-bold uppercase tracking-[0.22em] text-[#ff4f00]">
                Zuletzt notiert / {String(visiblePosts.length).padStart(2, '0')}
              </p>
              <h2
                className={[
                  'font-display text-6xl font-semibold tracking-[-0.065em] sm:text-7xl',
                  'lg:text-8xl',
                ].join(' ')}
              >
                Was ich zuletzt
                <br />
                <span className="font-serif font-normal italic">aufgeschrieben habe.</span>
              </h2>
            </div>
            <p className="max-w-sm text-base leading-relaxed text-[#171713]/60 lg:pb-2">
              Mal Code, mal Alltag, mal eine Idee, die noch nicht ganz fertig ist. Alles aus meiner
              eigenen Perspektive.
            </p>
          </div>

          {topics.length > 1 && (
            <div
              className="flex gap-2 overflow-x-auto border-b border-[#171713]/15 py-6"
              aria-label="Artikel nach Thema filtern"
            >
              {topics.map((topic) => (
                <button
                  key={topic}
                  type="button"
                  onClick={() => setActiveTopic(topic)}
                  className={[
                    'shrink-0 rounded-full border px-5 py-2.5 text-xs font-bold uppercase',
                    'tracking-[0.12em] transition-colors',
                    activeTopic === topic
                      ? 'border-[#171713] bg-[#171713] text-white'
                      : 'border-[#171713]/20 hover:border-[#171713]',
                  ].join(' ')}
                >
                  {topic}
                </button>
              ))}
            </div>
          )}

          {loading ? (
            <div className="flex min-h-80 flex-col items-center justify-center gap-4 text-[#171713]/55">
              <Loader2 className="h-9 w-9 animate-spin text-[#ff4f00]" />
              <p className="font-mono text-xs font-bold uppercase tracking-[0.2em]">
                Beiträge werden geladen …
              </p>
            </div>
          ) : error ? (
            <div
              className={`my-10 grid gap-5 border border-[#171713] bg-[#ff4f00] p-8 text-white
sm:grid-cols-[auto_1fr] sm:items-center`}
            >
              <span className="font-serif text-6xl italic">Oops.</span>
              <div>
                <h3 className="text-xl font-bold text-white">Meine Beiträge machen kurz Pause.</h3>
                <p className="mt-1 text-white/75">
                  {error.message || 'Der Feed konnte gerade nicht geladen werden.'}
                </p>
              </div>
            </div>
          ) : visiblePosts.length === 0 ? (
            <div
              className={`my-10 grid min-h-72 place-items-center border border-dashed
border-[#171713]/40 bg-white/25 p-10 text-center`}
            >
              <div>
                <Asterisk className="mx-auto mb-5 h-10 w-10 text-[#ff4f00]" />
                <h3 className="font-serif text-4xl italic">Die erste Story ist in Arbeit.</h3>
                <p className="mx-auto mt-3 max-w-md text-[#171713]/55">
                  Sobald ich den ersten Beitrag veröffentliche, bekommt er hier seinen Platz.
                </p>
              </div>
            </div>
          ) : (
            <div className="grid border-l border-t border-[#171713] md:grid-cols-2 xl:grid-cols-3">
              {visiblePosts.map((post, index) => (
                <PostCard
                  key={`${post.pageSlug}-${post.id || post.slug}`}
                  post={post}
                  pageSlug={post.pageSlug}
                  index={index}
                  featured={index === 0}
                />
              ))}
            </div>
          )}
        </div>
      </section>

      <section
        id="about"
        className={`border-y border-[#171713] bg-[#171713] px-5 py-20 text-[#f4f1ea] sm:px-8
lg:px-12 lg:py-28`}
      >
        <div className="mx-auto grid max-w-[1480px] gap-14 lg:grid-cols-[0.8fr_1.2fr] lg:items-start">
          <div
            className={`flex items-center gap-3 font-mono text-xs font-bold uppercase
tracking-[0.2em] text-[#b9f227]`}
          >
            <Asterisk className="h-5 w-5" /> Warum ich schreibe
          </div>
          <div>
            <p
              className={[
                'font-serif text-[clamp(2.8rem,5.5vw,6.3rem)] leading-[0.98]',
                'tracking-[-0.045em]',
              ].join(' ')}
            >
              Ich schreibe, um Dinge{' '}
              <span className="text-[#b9f227] italic">wirklich zu verstehen</span> – und um meine
              Gedanken nicht zu verlieren.
            </p>
            <div className="mt-12 grid gap-8 border-t border-white/20 pt-8 sm:grid-cols-2">
              <p className="leading-relaxed text-white/55">
                Dieser Blog ist mein digitales Notizbuch. Ich teile, was ich lerne, woran ich
                arbeite und welche Fragen mich gerade begleiten.
              </p>
              <p className="leading-relaxed text-white/55">
                Die Themen dürfen wechseln. Was bleibt, ist eine persönliche Perspektive, ehrliche
                Neugier und der Wunsch, Gedanken sauber zu Ende zu denken.
              </p>
            </div>
          </div>
        </div>
      </section>

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
              <Mail className="h-4 w-4" /> Neue Notizen per Mail
            </p>
            <h2
              className={`max-w-4xl font-display text-5xl font-semibold leading-[0.92]
tracking-[-0.06em] text-white sm:text-7xl lg:text-8xl`}
            >
              Ich melde mich,
              <br />
              <span className="font-serif font-normal italic">wenn es etwas zu sagen gibt.</span>
            </h2>
          </div>
          <form
            className="border-b-2 border-white pb-3"
            onSubmit={(event) => event.preventDefault()}
          >
            <label htmlFor="newsletter-email" className="sr-only">
              E-Mail-Adresse
            </label>
            <div className="flex items-center gap-3">
              <input
                id="newsletter-email"
                type="email"
                placeholder="you@example.com"
                className={`min-w-0 flex-1 bg-transparent py-3 text-xl text-white outline-none
placeholder:text-white/55`}
              />
              <button
                type="submit"
                aria-label="Newsletter abonnieren"
                className={`grid h-12 w-12 shrink-0 place-items-center rounded-full bg-[#171713]
transition-transform hover:rotate-[-12deg]`}
              >
                <ArrowRight className="h-5 w-5" />
              </button>
            </div>
          </form>
        </div>
      </section>
    </main>
  )
}

export default Home
