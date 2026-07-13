import { Link } from 'react-router-dom'
import { ArrowUpRight, CalendarDays, Clock3 } from 'lucide-react'
import PropTypes from 'prop-types'
import { formatDate, normalizeSlug, buildPreviewText } from '../../utils/postUtils'

const estimateReadingTime = (text) => {
    const wordCount = String(text || '').trim().split(/\s+/).filter(Boolean).length
    return Math.max(2, Math.ceil(wordCount / 200))
}

const PostCard = ({ post, pageSlug, index = 0, featured = false }) => {
    const publishedDate = formatDate(post.published_at || post.created_at)
    const previewText = buildPreviewText(post)
    const postSlug = normalizeSlug(post?.slug)
    const href = postSlug ? `/pages/${pageSlug}/posts/${postSlug}` : null
    const cardNumber = String(index + 1).padStart(2, '0')

    const content = (
        <article className="group flex h-full min-h-[28rem] flex-col justify-between border-b border-r border-[#171713] bg-[#f4f1ea] p-6 transition-colors duration-300 hover:bg-[#fffdf7] sm:p-8">
            <div>
                <header className="flex items-start justify-between gap-5">
                    <div className="flex flex-wrap items-center gap-3 font-mono text-[10px] font-bold uppercase tracking-[0.16em] text-[#171713]/50">
                        <span className="rounded-full bg-[#b9f227] px-3 py-1.5 text-[#171713]">{post.pageTitle || 'Journal'}</span>
                        {publishedDate && (
                            <span className="inline-flex items-center gap-1.5"><CalendarDays className="h-3.5 w-3.5" />{publishedDate}</span>
                        )}
                    </div>
                    <span className="font-serif text-3xl italic text-[#171713]/20">{cardNumber}</span>
                </header>

                <div className={`mt-14 ${featured ? 'xl:mt-24' : ''}`}>
                    <h3 className={`font-display font-semibold leading-[0.98] tracking-[-0.045em] text-[#171713] transition-colors group-hover:text-[#ff4f00] ${featured ? 'text-4xl sm:text-5xl' : 'text-3xl sm:text-4xl'}`}>
                        {post.title}
                    </h3>
                    {previewText && (
                        <p className="mt-6 line-clamp-4 max-w-xl text-base leading-relaxed text-[#171713]/60">{previewText}</p>
                    )}
                </div>
            </div>

            <footer className="mt-12 flex items-end justify-between gap-4 border-t border-[#171713]/15 pt-5">
                <span className="inline-flex items-center gap-2 font-mono text-[10px] font-bold uppercase tracking-[0.14em] text-[#171713]/45">
                    <Clock3 className="h-3.5 w-3.5" /> {estimateReadingTime(post.content || previewText)} min read
                </span>
                {href && (
                    <span className="grid h-12 w-12 place-items-center rounded-full border border-[#171713] bg-transparent transition-all group-hover:rotate-45 group-hover:bg-[#171713] group-hover:text-white">
                        <ArrowUpRight className="h-5 w-5" />
                    </span>
                )}
            </footer>
        </article>
    )

    return href ? (
        <Link to={href} className={`block focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-[#ff4f00] ${featured ? 'md:col-span-2 xl:col-span-1 xl:row-span-1' : ''}`}>
            {content}
        </Link>
    ) : content
}

PostCard.propTypes = {
    post: PropTypes.object.isRequired,
    pageSlug: PropTypes.string.isRequired,
    index: PropTypes.number,
    featured: PropTypes.bool,
}

export default PostCard
