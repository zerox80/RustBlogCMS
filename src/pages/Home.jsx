import { useState, useEffect } from 'react'
import { Loader2 } from 'lucide-react'
import PostCard from '../components/dynamic-page/PostCard'
import { api } from '../api/client'

/**
 * Blog Home Page - Shows all published posts from ALL pages.
 * 
 * Uses listPublishedPages to get all pages (not just navigation items)
 * and then fetches posts from each page.
 */
const Home = () => {
  const [posts, setPosts] = useState([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    const fetchAllPosts = async () => {
      try {
        setLoading(true)

        // Get ALL published pages, not just navigation items
        const pagesData = await api.listPublishedPages()
        const allPages = pagesData?.items || []

        if (allPages.length === 0) {
          setPosts([])
          setLoading(false)
          return
        }

        // Fetch posts from each page with delay to avoid rate limiting
        const allPosts = []

        for (const page of allPages) {
          try {
            // Fetch the full page data which includes posts
            const pageData = await api.getPublishedPage(page.slug)
            if (pageData?.posts) {
              pageData.posts.forEach(post => {
                allPosts.push({
                  ...post,
                  pageSlug: page.slug,
                  pageTitle: page.title
                })
              })
            }
          } catch (e) {
            // Page might have no posts or fail, continue with others
            console.log(`Could not load posts for ${page.slug}:`, e.message)
          }

          // Small delay between requests to avoid rate limiting
          await new Promise(resolve => setTimeout(resolve, 150))
        }

        // Sort by creation date, newest first
        allPosts.sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
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

  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-100 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950">
      <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 pt-28 pb-16">
        {/* Page Header */}
        <div className="mb-12 text-center">
          <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-4">
            Blog
          </h1>
          <p className="text-lg text-gray-600 dark:text-slate-300">
            Alle veröffentlichten Artikel
          </p>
        </div>

        {/* Content */}
        {loading ? (
          <div className="flex flex-col items-center justify-center py-20 text-gray-500 dark:text-slate-400">
            <Loader2 className="w-10 h-10 animate-spin mb-4" />
            <p>Lade Artikel…</p>
          </div>
        ) : error ? (
          <div className="rounded-2xl border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20 p-6 text-red-700 dark:text-red-300">
            <h2 className="font-semibold mb-1">Fehler beim Laden</h2>
            <p className="text-sm">{error.message || 'Unbekannter Fehler'}</p>
          </div>
        ) : posts.length === 0 ? (
          <div className="rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/80 p-10 text-center">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
              Noch keine Artikel
            </h2>
            <p className="text-gray-500 dark:text-slate-400">
              Sobald Artikel veröffentlicht werden, erscheinen sie hier.
            </p>
          </div>
        ) : (
          <div className="space-y-8">
            <p className="text-sm text-gray-500 dark:text-slate-400">
              {posts.length} {posts.length === 1 ? 'Artikel' : 'Artikel'}
            </p>
            <div className="space-y-10">
              {posts.map((post) => (
                <PostCard key={`${post.pageSlug}-${post.id}`} post={post} pageSlug={post.pageSlug} />
              ))}
            </div>
          </div>
        )}
      </div>
    </main>
  )
}

export default Home
