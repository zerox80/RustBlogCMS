import React, { useEffect, useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useContent } from '../context/ContentContext'
import { formatDate } from '../utils/postUtils'
import { Helmet } from 'react-helmet-async'
import MarkdownRenderer from '../components/markdown/MarkdownRenderer'
import { Calendar, Clock, User, ArrowLeft, Share2, Bookmark } from 'lucide-react'

const PostDetail = () => {
  const { pageSlug, postSlug } = useParams()
  const { pages } = useContent()
  const [post, setPost] = useState(null)
  const [activeSection, setActiveSection] = useState('')

  useEffect(() => {
    const fetchPost = async () => {
      try {
        const data = await pages.getPost(pageSlug, postSlug)
        setPost(data.post)
      } catch (err) {
        console.error("Failed to load post:", err)
      }
    }
    fetchPost()
  }, [pageSlug, postSlug, pages])

  // Scroll spy for Table of Contents
  useEffect(() => {
    const observer = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          setActiveSection(entry.target.id)
        }
      })
    }, { rootMargin: '-100px 0px -60% 0px' })

    document.querySelectorAll('h2, h3').forEach(heading => observer.observe(heading))
    return () => observer.disconnect()
  }, [post])

  if (!post) return <div className="min-h-screen flex items-center justify-center text-slate-400">Loading...</div>

  return (
    <div className="min-h-screen bg-slate-950 pt-24 pb-20">
      <Helmet>
        <title>{post.meta?.title || post.title} | RustCMS</title>
      </Helmet>

      {/* Hero Header */}
      <div className="relative w-full h-[40vh] md:h-[50vh] flex items-center justify-center mb-12">
        <div className="absolute inset-0 z-0">
          <div className="absolute inset-0 bg-gradient-to-t from-slate-950 via-slate-950/80 to-transparent z-10" />
          {post.image && (
            <img src={post.image} alt={post.title} className="w-full h-full object-cover opacity-60" />
          )}
          {/* Fallback pattern if no image */}
          {!post.image && (
            <div className="w-full h-full bg-slate-900 aurora-bg-animated opacity-30" />
          )}
        </div>

        <div className="container px-6 relative z-20 max-w-4xl text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-white/10 backdrop-blur-md border border-white/10 text-xs font-medium text-neon-cyan mb-6">
            {post.category || "Technology"}
          </div>
          <h1 className="text-4xl md:text-5xl lg:text-6xl font-serif font-bold text-white mb-6 leading-tight">
            {post.title}
          </h1>

          <div className="flex flex-wrap items-center justify-center gap-6 text-slate-300 text-sm">
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-neon-purple to-neon-blue p-0.5">
                <div className="w-full h-full rounded-full bg-slate-900 flex items-center justify-center">
                  <User className="w-4 h-4 text-white" />
                </div>
              </div>
              <span className="font-medium text-white">{post.author || "Admin"}</span>
            </div>
            <div className="flex items-center gap-2">
              <Calendar className="w-4 h-4" />
              <span>{formatDate(post.published_at || post.created_at) || "Recently"}</span>
            </div>
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4" />
              <span>{post.readTime || `${Math.ceil((post.content_markdown?.length || 0) / 1000) + 1} min read`}</span>
            </div>
          </div>
        </div>
      </div>

      <div className="container px-6 mx-auto relative max-w-7xl grid grid-cols-1 lg:grid-cols-12 gap-12">
        {/* Sidebar - Table of Contents */}
        <aside className="hidden lg:block lg:col-span-3">
          <div className="sticky top-32 glass-card p-6 rounded-2xl">
            <h4 className="text-xs font-bold text-slate-400 uppercase tracking-wider mb-4">Table of Contents</h4>
            <nav className="space-y-1">
              {/* Note: In a real app, generate these from markdown AST. For now static or simple regex */}
              <div className="text-sm text-slate-500 italic">Sections auto-generated</div>
            </nav>

            <div className="mt-8 pt-8 border-t border-white/5 flex gap-4">
              <button className="p-2 rounded-lg hover:bg-white/5 text-slate-400 hover:text-white transition-colors">
                <Share2 className="w-5 h-5" />
              </button>
              <button className="p-2 rounded-lg hover:bg-white/5 text-slate-400 hover:text-white transition-colors">
                <Bookmark className="w-5 h-5" />
              </button>
            </div>
          </div>
        </aside>

        {/* Main Content */}
        <main className="lg:col-span-8 lg:col-start-4">
          <article className="max-w-none">
            <MarkdownRenderer
              content={post.content_markdown}
              withBreaks
            />
          </article>

          {/* Footer / Comments Area */}
          <div className="mt-20 border-t border-white/10 pt-10">
            <div className="flex items-center justify-between mb-8">
              <h3 className="text-2xl font-bold text-white">Discussion</h3>
              <button className="btn-primary px-4 py-2 text-sm">Post Comment</button>
            </div>

            <div className="glass-card p-6 mb-6">
              <div className="flex gap-4">
                <div className="w-10 h-10 rounded-full bg-slate-800 flex-shrink-0" />
                <div className="flex-1">
                  <textarea
                    placeholder="What are your thoughts?"
                    className="w-full bg-transparent border-0 text-white placeholder-slate-500 focus:ring-0 resize-none h-24"
                  />
                </div>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  )
}

export default PostDetail
