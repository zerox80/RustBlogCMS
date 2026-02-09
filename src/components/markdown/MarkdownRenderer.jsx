import PropTypes from 'prop-types'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks'
import remarkMath from 'remark-math'
import rehypeHighlight from 'rehype-highlight'
import rehypeKatex from 'rehype-katex'
import 'katex/dist/katex.min.css'
import CodeBlock from '../ui/CodeBlock'
import remarkMergeInlineParagraphs from '../../utils/remarkMergeInlineParagraphs'
import remarkGithubAlerts from '../../utils/remarkGithubAlerts'
import { AlertCircle, AlertTriangle, Info, Lightbulb, MessageCircle } from 'lucide-react'
const mergeClassNames = (...classes) => classes.filter(Boolean).join(' ')
const headingClasses = {
  1: 'text-4xl sm:text-5xl font-bold text-gray-900 dark:text-slate-100 tracking-tight mt-16 first:mt-0 mb-8',
  2: 'text-3xl sm:text-4xl font-bold text-gray-900 dark:text-slate-100 mt-14 first:mt-0 mb-6',
  3: 'text-2xl sm:text-3xl font-semibold text-gray-900 dark:text-slate-100 mt-12 first:mt-0 mb-4',
  4: 'text-xl sm:text-2xl font-semibold text-gray-900 dark:text-slate-100 mt-10 first:mt-0 mb-4',
  5: 'text-lg font-semibold text-gray-800 dark:text-slate-300 mt-8 first:mt-0 mb-3 uppercase tracking-wide',
  6: 'text-base font-semibold text-gray-700 dark:text-slate-400 mt-8 first:mt-0 mb-3 uppercase tracking-wider',
}
const MarkdownRenderer = ({ content, className = '', withBreaks = false }) => {
  const remarkPlugins = withBreaks
    ? [remarkMath, remarkGfm, remarkMergeInlineParagraphs, remarkGithubAlerts, remarkBreaks]
    : [remarkMath, remarkGfm, remarkMergeInlineParagraphs, remarkGithubAlerts]
  return (
    <div className={mergeClassNames('markdown-renderer text-gray-700 dark:text-slate-200', className)}>
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={[rehypeKatex, rehypeHighlight]}
        components={{
          h1: ({ children, ...props }) => (
            <h1 className={headingClasses[1]} {...props}>
              {children}
            </h1>
          ),
          h2: ({ children, ...props }) => (
            <h2 className={headingClasses[2]} {...props}>
              {children}
            </h2>
          ),
          h3: ({ children, ...props }) => (
            <h3 className={headingClasses[3]} {...props}>
              {children}
            </h3>
          ),
          h4: ({ children, ...props }) => (
            <h4 className={headingClasses[4]} {...props}>
              {children}
            </h4>
          ),
          h5: ({ children, ...props }) => (
            <h5 className={headingClasses[5]} {...props}>
              {children}
            </h5>
          ),
          h6: ({ children, ...props }) => (
            <h6 className={headingClasses[6]} {...props}>
              {children}
            </h6>
          ),
          p: ({ children, ...props }) => (
            <p className="mt-8 first:mt-0 text-lg sm:text-xl leading-relaxed text-gray-700 dark:text-slate-300" {...props}>
              {children}
            </p>
          ),
          ul: ({ children, ...props }) => (
            <ul className="mt-8 first:mt-0 list-disc list-outside space-y-4 pl-6 text-lg sm:text-xl leading-relaxed text-gray-700 dark:text-slate-300" {...props}>
              {children}
            </ul>
          ),
          ol: ({ children, ...props }) => (
            <ol className="mt-8 first:mt-0 list-decimal list-outside space-y-4 pl-6 text-lg sm:text-xl leading-relaxed text-gray-700 dark:text-slate-300" {...props}>
              {children}
            </ol>
          ),
          li: ({ children, ...props }) => (
            <li className="leading-7 text-gray-700 dark:text-slate-200 marker:text-primary-600 dark:marker:text-primary-300 break-words" {...props}>
              {children}
            </li>
          ),
          blockquote: ({ children, ...props }) => {
            const alertType = props['data-alert-type']
            if (alertType) {
              const type = alertType.toUpperCase()
              let icon
              let title
              let classes = 'rounded-2xl border-l-4 px-6 py-4 my-8 shadow-sm text-base'

              switch (type) {
                case 'NOTE':
                  icon = <Info className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                  title = 'Note'
                  classes += ' bg-blue-50/50 border-blue-500/50 text-blue-900 dark:bg-blue-900/10 dark:text-blue-100 dark:border-blue-500'
                  break
                case 'TIP':
                  icon = <Lightbulb className="h-5 w-5 text-green-600 dark:text-green-400" />
                  title = 'Tip'
                  classes += ' bg-green-50/50 border-green-500/50 text-green-900 dark:bg-green-900/10 dark:text-green-100 dark:border-green-500'
                  break
                case 'IMPORTANT':
                  icon = <MessageCircle className="h-5 w-5 text-purple-600 dark:text-purple-400" />
                  title = 'Important'
                  classes += ' bg-purple-50/50 border-purple-500/50 text-purple-900 dark:bg-purple-900/10 dark:text-purple-100 dark:border-purple-500'
                  break
                case 'WARNING':
                  icon = <AlertTriangle className="h-5 w-5 text-yellow-600 dark:text-yellow-400" />
                  title = 'Warning'
                  classes += ' bg-yellow-50/50 border-yellow-500/50 text-yellow-900 dark:bg-yellow-900/10 dark:text-yellow-100 dark:border-yellow-500'
                  break
                case 'CAUTION':
                  icon = <AlertCircle className="h-5 w-5 text-red-600 dark:text-red-400" />
                  title = 'Caution'
                  classes += ' bg-red-50/50 border-red-500/50 text-red-900 dark:bg-red-900/10 dark:text-red-100 dark:border-red-500'
                  break
                default:
                  icon = <Info className="h-5 w-5" />
                  title = 'Note'
                  classes += ' bg-gray-50 border-gray-500 text-gray-900 dark:bg-gray-800 dark:text-gray-100 dark:border-gray-500'
              }

              return (
                <div className={classes} {...props}>
                  <div className="flex items-center gap-3 mb-2 font-semibold select-none">
                    {icon}
                    <span>{title}</span>
                  </div>
                  <div className="text-sm sm:text-base leading-relaxed opacity-90">
                    {children}
                  </div>
                </div>
              )
            }
            return (
              <blockquote
                className="mt-10 first:mt-0 rounded-2xl border-l-4 border-primary-500 bg-primary-50/40 dark:bg-slate-800/40 dark:border-primary-500/60 px-8 py-6 text-lg sm:text-xl italic text-gray-700 dark:text-slate-200 shadow-sm"
                {...props}
              >
                {children}
              </blockquote>
            )
          },
          a: ({ href, children, ...props }) => (
            <a
              href={href}
              className="font-semibold text-primary-700 dark:text-primary-300 underline underline-offset-4 transition-colors hover:text-primary-800 dark:hover:text-primary-200"
              target={href?.startsWith('#') ? undefined : '_blank'}
              rel={href?.startsWith('#') ? undefined : 'noopener noreferrer'}
              {...props}
            >
              {children}
            </a>
          ),
          hr: ({ ...props }) => (
            <hr className="my-10 border-t border-gray-200 dark:border-slate-700" {...props} />
          ),
          table: ({ children, ...props }) => (
            <div className="mt-6 overflow-x-auto rounded-2xl border border-gray-200 dark:border-slate-700/80">
              <table className="min-w-full divide-y divide-gray-200 dark:divide-slate-700" {...props}>
                {children}
              </table>
            </div>
          ),
          thead: ({ children, ...props }) => (
            <thead className="bg-gray-50 dark:bg-slate-800" {...props}>
              {children}
            </thead>
          ),
          tbody: ({ children, ...props }) => (
            <tbody className="divide-y divide-gray-100 dark:divide-slate-800" {...props}>
              {children}
            </tbody>
          ),
          th: ({ children, ...props }) => (
            <th className="px-4 py-3 text-left text-sm font-semibold uppercase tracking-wide text-gray-600 dark:text-slate-300" {...props}>
              {children}
            </th>
          ),
          td: ({ children, ...props }) => (
            <td className="px-4 py-3 text-sm text-gray-700 dark:text-slate-200" {...props}>
              {children}
            </td>
          ),
          code: ({ inline, className, children, ...props }) =>
            inline ? (
              <code
                className={mergeClassNames(
                  className,
                  'rounded-md bg-gray-100 dark:bg-slate-800/90 py-0.5 px-2 font-mono text-[0.9em] text-primary-700 dark:text-primary-300 whitespace-nowrap'
                )}
                {...props}
              >
                {children}
              </code>
            ) : (
              <code
                className={mergeClassNames(className, 'block font-mono text-sm leading-relaxed')}
                {...props}
              >
                {children}
              </code>
            ),
          pre: ({ className, children, ...props }) => (
            <CodeBlock
              className={mergeClassNames(
                className,
                'mt-10 mb-10 overflow-x-auto rounded-2xl bg-gray-900 dark:bg-slate-950 p-6 text-sm text-gray-100 shadow-xl border border-white/5'
              )}
              {...props}
            >
              {children}
            </CodeBlock>
          ),
          img: ({ alt, src, ...props }) => (
            <img
              src={src}
              alt={alt || ''}
              className="mt-6 w-full rounded-2xl border border-gray-200 dark:border-slate-700 object-contain"
              loading="lazy"
              {...props}
            />
          ),
        }}
      >
        {content || ''}
      </ReactMarkdown>
    </div>
  )
}
MarkdownRenderer.propTypes = {
  content: PropTypes.string,
  className: PropTypes.string,
  withBreaks: PropTypes.bool,
}
export default MarkdownRenderer
