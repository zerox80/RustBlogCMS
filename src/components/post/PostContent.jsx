import PropTypes from 'prop-types'
import MarkdownRenderer from '../markdown/MarkdownRenderer'
import PostHeader from './PostHeader'

const PostContent = ({ post }) => {
    return (
        <article id="post-content" className="bg-white dark:bg-slate-900/90 rounded-3xl shadow-xl border border-gray-200 dark:border-slate-700/60 overflow-hidden break-words">
            <div className="px-6 py-8 sm:px-10 sm:py-10">
                <PostHeader
                    title={post?.title}
                    publishedAt={post?.published_at}
                    excerpt={post?.excerpt}
                />

                {post?.content_markdown && (
                    <div className="mt-10">
                        <MarkdownRenderer
                            content={post.content_markdown}
                            withBreaks
                        />
                    </div>
                )}
            </div>
        </article>
    )
}

PostContent.propTypes = {
    post: PropTypes.object.isRequired,
}

export default PostContent
