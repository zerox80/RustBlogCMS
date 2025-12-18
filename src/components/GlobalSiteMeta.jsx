import { Helmet } from 'react-helmet-async'
import { useEffect } from 'react'
import { useContent } from '../context/ContentContext'

const FALLBACK_TITLE = 'Zero Point - IT & Rust Research'
const FALLBACK_DESCRIPTION = 'In-depth tutorials and research on Rust, Linux, and Server Infrastructure.'

const sanitize = (value) => {
  if (typeof value !== 'string') {
    return ''
  }
  return value.trim()
}

const GlobalSiteMeta = () => {
  const { getSiteMeta } = useContent()

  // Get fresh meta data directly
  const meta = getSiteMeta() || {}

  const title = sanitize(meta.title) || FALLBACK_TITLE
  const description = sanitize(meta.description) || FALLBACK_DESCRIPTION

  // Force update document title directly as fallback
  useEffect(() => {
    if (title) {
      document.title = title
    }
  }, [title])

  return (
    <Helmet>
      <title>{title}</title>
      {description && <meta name="description" content={description} />}

      <meta property="og:title" content={title} />
      {description && <meta property="og:description" content={description} />}
      <meta property="og:site_name" content={title} />

      {description && <meta name="twitter:description" content={description} />}
      <meta name="twitter:title" content={title} />
    </Helmet>
  )
}

export default GlobalSiteMeta
