import { useCallback, useEffect, useMemo, useState } from 'react'
import { CalendarDays, Edit3, Eye, EyeOff, Loader2, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { api } from '../../api/client'
import { useContent } from '../../context/ContentContext'
import PostForm from '../page-manager/PostForm'

const BLOG_PAGE_PAYLOAD = {
  title: 'Blog',
  slug: 'blog',
  description: 'Interner Speicherbereich für die Beiträge des persönlichen Blogs.',
  nav_label: null,
  show_in_nav: false,
  is_published: true,
  order_index: 0,
  hero: { title: 'Blog' },
  layout: {},
}

/** One post manager across all legacy storage pages. */
const BlogPostManager = () => {
  const { pages: publicPages } = useContent()
  const [pages, setPages] = useState([])
  const [posts, setPosts] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)
  const [form, setForm] = useState(null)
  const [submitting, setSubmitting] = useState(false)

  const loadPosts = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const pagesData = await api.listPages()
      const pageItems = Array.isArray(pagesData?.items) ? pagesData.items : []
      const results = await Promise.all(
        pageItems.map(async (page) => {
          const postsData = await api.listPosts(page.id)
          return (Array.isArray(postsData?.items) ? postsData.items : []).map((post) => ({
            ...post,
            storagePageId: page.id,
            storagePageSlug: page.slug,
          }))
        }),
      )

      const allPosts = results.flat().sort((a, b) => {
        const left = new Date(a.published_at || a.created_at || 0)
        const right = new Date(b.published_at || b.created_at || 0)
        return right - left
      })
      setPages(pageItems)
      setPosts(allPosts)
    } catch (loadError) {
      setError(loadError)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadPosts()
  }, [loadPosts])

  const publishedCount = useMemo(
    () => posts.filter((post) => post.is_published).length,
    [posts],
  )

  const invalidatePublicFeed = useCallback(() => {
    publicPages?.invalidate?.()
    publicPages?.refresh?.()
  }, [publicPages])

  const ensureBlogPage = async () => {
    const existing = pages.find((page) => page.slug === 'blog') || pages[0]
    if (existing) return existing
    const created = await api.createPage(BLOG_PAGE_PAYLOAD)
    return created
  }

  const submitPost = async (payload) => {
    try {
      setSubmitting(true)
      if (form?.mode === 'edit' && form?.post?.id) {
        await api.updatePost(form.post.id, payload)
      } else {
        const blogPage = await ensureBlogPage()
        await api.createPost(blogPage.id, payload)
      }
      setForm(null)
      invalidatePublicFeed()
      await loadPosts()
    } finally {
      setSubmitting(false)
    }
  }

  const deletePost = async (post) => {
    if (!window.confirm('Soll dieser Beitrag wirklich gelöscht werden?')) return
    try {
      await api.deletePost(post.id)
      invalidatePublicFeed()
      await loadPosts()
    } catch (deleteError) {
      setError(deleteError)
    }
  }

  return (
    <section className="space-y-6 text-[#171713]">
      <div className="flex flex-col gap-5 border-b border-[#171713] pb-6 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="font-mono text-[11px] font-bold uppercase tracking-[0.18em] text-[#ff4f00]">
            Ein Blog · Ein Feed
          </p>
          <h1 className="mt-2 text-4xl font-semibold tracking-[-0.04em]">Beiträge</h1>
          <p className="mt-2 text-sm text-[#171713]/55">
            {publishedCount} von {posts.length} Beiträgen veröffentlicht. Alte Seitenzuordnungen
            bleiben intern erhalten, sind öffentlich aber nicht mehr sichtbar.
          </p>
        </div>
        <div className="flex gap-3">
          <button
            type="button"
            onClick={loadPosts}
            disabled={loading}
            className={`inline-flex items-center gap-2 rounded-full border border-[#171713]/25
px-4 py-2.5 text-xs font-bold uppercase tracking-[0.1em]`}
          >
            <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} /> Aktualisieren
          </button>
          <button
            type="button"
            onClick={() => setForm({ mode: 'create', post: null })}
            className={`inline-flex items-center gap-2 rounded-full bg-[#171713] px-5 py-2.5
text-xs font-bold uppercase tracking-[0.1em] text-white hover:bg-[#ff4f00]`}
          >
            <Plus className="h-4 w-4" /> Neuer Beitrag
          </button>
        </div>
      </div>

      {error && (
        <div className="border border-[#171713] bg-[#ff4f00] p-5 text-sm text-white">
          {error.message || 'Beiträge konnten nicht geladen werden.'}
        </div>
      )}

      {loading && posts.length === 0 ? (
        <div className="flex min-h-64 items-center justify-center gap-3 text-sm text-[#171713]/55">
          <Loader2 className="h-5 w-5 animate-spin text-[#ff4f00]" /> Beiträge werden geladen …
        </div>
      ) : posts.length === 0 ? (
        <div
          className={`grid min-h-64 place-items-center border border-dashed border-[#171713]/30
p-8 text-center`}
        >
          <div>
            <p className="font-serif text-4xl italic">Noch nichts veröffentlicht.</p>
            <p className="mt-2 text-sm text-[#171713]/55">
              Der erste Beitrag kann direkt hier entstehen.
            </p>
          </div>
        </div>
      ) : (
        <div className="grid border-l border-t border-[#171713] lg:grid-cols-2">
          {posts.map((post, index) => (
            <article
              key={post.id}
              className={`flex min-h-64 flex-col justify-between border-b border-r
border-[#171713] bg-[#f4f1ea] p-6`}
            >
              <div>
                <div className="flex items-start justify-between gap-4">
                  <div
                    className={`flex flex-wrap items-center gap-3 font-mono text-[10px] font-bold
uppercase tracking-[0.14em] text-[#171713]/50`}
                  >
                    <span
                      className={`inline-flex items-center gap-1 rounded-full bg-[#b9f227]
px-3 py-1.5 text-[#171713]`}
                    >
                      {post.is_published ? (
                        <Eye className="h-3.5 w-3.5" />
                      ) : (
                        <EyeOff className="h-3.5 w-3.5" />
                      )}
                      {post.is_published ? 'Veröffentlicht' : 'Entwurf'}
                    </span>
                    {post.published_at && (
                      <span className="inline-flex items-center gap-1.5">
                        <CalendarDays className="h-3.5 w-3.5" />
                        {new Date(post.published_at).toLocaleDateString('de-DE')}
                      </span>
                    )}
                  </div>
                  <span className="font-serif text-3xl italic text-[#171713]/20">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                </div>
                <h2 className="mt-10 text-3xl font-semibold leading-tight tracking-[-0.035em]">
                  {post.title}
                </h2>
                {post.excerpt && (
                  <p className="mt-4 line-clamp-3 text-sm leading-relaxed text-[#171713]/55">
                    {post.excerpt}
                  </p>
                )}
              </div>
              <div className="mt-8 flex gap-3 border-t border-[#171713]/15 pt-4">
                <button
                  type="button"
                  onClick={() => setForm({ mode: 'edit', post })}
                  className={`inline-flex items-center gap-2 text-xs font-bold uppercase
tracking-[0.1em] hover:text-[#ff4f00]`}
                >
                  <Edit3 className="h-4 w-4" /> Bearbeiten
                </button>
                <button
                  type="button"
                  onClick={() => deletePost(post)}
                  className={`ml-auto inline-flex items-center gap-2 text-xs font-bold uppercase
tracking-[0.1em] text-red-700`}
                >
                  <Trash2 className="h-4 w-4" /> Löschen
                </button>
              </div>
            </article>
          ))}
        </div>
      )}

      {form && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-4">
          <PostForm
            mode={form.mode}
            initialData={form.post}
            onSubmit={submitPost}
            onCancel={() => setForm(null)}
            submitting={submitting}
          />
        </div>
      )}
    </section>
  )
}

export default BlogPostManager
