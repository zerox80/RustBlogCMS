const TEXT_FILE_TYPE = 'text/markdown;charset=utf-8'

const normalizeText = (value) => {
  if (value === null || value === undefined) {
    return ''
  }
  return String(value).replace(/\r\n?/g, '\n').trim()
}

const escapeFrontMatterValue = (value) =>
  normalizeText(value).replace(/\\/g, '\\\\').replace(/"/g, '\\"')

const escapeTableCell = (value) =>
  normalizeText(value).replace(/\|/g, '\\|').replace(/\n+/g, '<br>')

const formatStatus = (published) => (published ? 'Veroeffentlicht' : 'Entwurf')

const formatCommentsStatus = (allowComments) => (allowComments ? 'Aktiv' : 'Deaktiviert')

const formatDateValue = (value) => {
  const normalized = normalizeText(value)
  if (!normalized) {
    return ''
  }
  const date = new Date(normalized)
  if (Number.isNaN(date.getTime())) {
    return normalized
  }
  return date.toISOString()
}

const compareByOrderAndTitle = (left, right) => {
  const leftOrder = Number.isFinite(Number(left?.order_index)) ? Number(left.order_index) : 0
  const rightOrder = Number.isFinite(Number(right?.order_index)) ? Number(right.order_index) : 0
  if (leftOrder !== rightOrder) {
    return leftOrder - rightOrder
  }
  return normalizeText(left?.title || left?.slug).localeCompare(
    normalizeText(right?.title || right?.slug),
    'de',
    {
      sensitivity: 'base',
    },
  )
}

const stringifyJson = (value) => {
  if (!value || typeof value !== 'object') {
    return ''
  }
  return JSON.stringify(value, null, 2)
}

const metadataTable = (rows) => {
  const visibleRows = rows.filter(([, value]) => normalizeText(value))
  if (visibleRows.length === 0) {
    return []
  }
  return [
    '| Feld | Wert |',
    '| --- | --- |',
    ...visibleRows.map(
      ([label, value]) => `| ${escapeTableCell(label)} | ${escapeTableCell(value)} |`,
    ),
  ]
}

const appendSection = (lines, sectionLines) => {
  const cleanLines = sectionLines.filter((line) => line !== null && line !== undefined)
  if (cleanLines.length === 0) {
    return
  }
  if (lines.length > 0 && lines[lines.length - 1] !== '') {
    lines.push('')
  }
  lines.push(...cleanLines)
}

const appendJsonBlock = (lines, title, value) => {
  const json = stringifyJson(value)
  if (!json) {
    return
  }
  appendSection(lines, [`### ${title}`, '', '```json', json, '```'])
}

const formatPost = (post, pageSlug, index) => {
  const title = normalizeText(post?.title) || normalizeText(post?.slug) || `Beitrag ${index + 1}`
  const postSlug = normalizeText(post?.slug)
  const lines = [`#### ${title}`]
  const rows = [
    ['Slug', postSlug ? `\`${postSlug}\`` : ''],
    ['URL', pageSlug && postSlug ? `\`/pages/${pageSlug}/posts/${postSlug}\`` : ''],
    ['Status', formatStatus(Boolean(post?.is_published))],
    ['Kommentare', formatCommentsStatus(post?.allow_comments !== false)],
    ['Reihenfolge', post?.order_index ?? 0],
    ['Veroeffentlicht am', formatDateValue(post?.published_at)],
    ['Erstellt am', formatDateValue(post?.created_at)],
    ['Aktualisiert am', formatDateValue(post?.updated_at)],
  ]

  appendSection(lines, metadataTable(rows))

  const excerpt = normalizeText(post?.excerpt)
  if (excerpt) {
    appendSection(lines, ['**Auszug**', '', ...excerpt.split('\n').map((line) => `> ${line}`)])
  }

  const content = normalizeText(post?.content_markdown)
  if (content) {
    appendSection(lines, [content])
  }

  return lines
}

const formatPage = ({ page, posts = [] }, index) => {
  const title = normalizeText(page?.title) || normalizeText(page?.slug) || `Seite ${index + 1}`
  const slug = normalizeText(page?.slug)
  const sortedPosts = [...posts].sort(compareByOrderAndTitle)
  const lines = ['---', '', `## ${title}`]
  const rows = [
    ['Slug', slug ? `\`${slug}\`` : ''],
    ['URL', slug ? `\`/pages/${slug}\`` : ''],
    ['Status', formatStatus(Boolean(page?.is_published))],
    ['Navigation', page?.show_in_nav ? 'Sichtbar' : 'Ausgeblendet'],
    ['Navigationslabel', page?.nav_label],
    ['Reihenfolge', page?.order_index ?? 0],
    ['Erstellt am', formatDateValue(page?.created_at)],
    ['Aktualisiert am', formatDateValue(page?.updated_at)],
  ]

  appendSection(lines, metadataTable(rows))

  const description = normalizeText(page?.description)
  if (description) {
    appendSection(lines, ['**Beschreibung**', '', description])
  }

  if (page?.hero && typeof page.hero === 'object') {
    const heroLines = ['### Hero']
    const heroRows = [
      ['Badge', page.hero.badge],
      ['Titel', page.hero.title],
      ['Untertitel', page.hero.subtitle],
    ]
    appendSection(heroLines, metadataTable(heroRows))
    appendSection(lines, heroLines)
  }

  appendJsonBlock(lines, 'Layout JSON', page?.layout)

  appendSection(lines, ['### Beitraege'])
  if (sortedPosts.length === 0) {
    lines.push('', 'Keine Beitraege vorhanden.')
  } else {
    sortedPosts.forEach((post, postIndex) => {
      appendSection(lines, formatPost(post, slug, postIndex))
    })
  }

  return lines
}

export const buildPagesMarkdownExport = (pagesWithPosts, options = {}) => {
  const generatedAt = options.generatedAt instanceof Date ? options.generatedAt : new Date()
  const normalizedPages = Array.isArray(pagesWithPosts) ? pagesWithPosts : []
  const sortedPages = [...normalizedPages].sort((left, right) =>
    compareByOrderAndTitle(left?.page, right?.page),
  )
  const pageCount = sortedPages.length
  const postCount = sortedPages.reduce(
    (total, entry) => total + (Array.isArray(entry?.posts) ? entry.posts.length : 0),
    0,
  )
  const lines = [
    '---',
    'title: "minos Seiten Export"',
    `generated_at: "${escapeFrontMatterValue(generatedAt.toISOString())}"`,
    `page_count: ${pageCount}`,
    `post_count: ${postCount}`,
    '---',
    '',
    '# minos Seiten Export',
    '',
    `Erstellt am: ${generatedAt.toISOString()}`,
  ]

  if (sortedPages.length === 0) {
    appendSection(lines, ['Keine Seiten vorhanden.'])
  } else {
    sortedPages.forEach((entry, index) => {
      appendSection(lines, formatPage(entry, index))
    })
  }

  return `${lines.join('\n')}\n`
}

export const buildPagesMarkdownFilename = (date = new Date()) => {
  const stamp = date.toISOString().slice(0, 10)
  return `minos-seiten-export-${stamp}.md`
}

export const downloadMarkdownFile = (markdown, filename = buildPagesMarkdownFilename()) => {
  const blob = new Blob([markdown], { type: TEXT_FILE_TYPE })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.rel = 'noopener'
  document.body.appendChild(link)
  link.click()
  link.remove()
  setTimeout(() => URL.revokeObjectURL(url), 0)
}
