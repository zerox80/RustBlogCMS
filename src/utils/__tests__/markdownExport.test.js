import { describe, expect, it } from 'vitest'
import { buildPagesMarkdownExport, buildPagesMarkdownFilename } from '../markdownExport'

describe('markdownExport', () => {
  it('builds a complete markdown document for pages and posts', () => {
    const markdown = buildPagesMarkdownExport(
      [
        {
          page: {
            title: 'Startseite',
            slug: 'start',
            description: 'Willkommen im CMS.',
            nav_label: 'Start',
            show_in_nav: true,
            is_published: true,
            order_index: 2,
            created_at: '2026-06-20T08:00:00Z',
            updated_at: '2026-06-22T09:30:00Z',
            hero: {
              badge: 'Neu',
              title: 'Willkommen',
              subtitle: 'Alles an einem Ort',
            },
            layout: {
              postsSection: {
                title: 'Artikel',
              },
            },
          },
          posts: [
            {
              title: 'Zweiter Beitrag',
              slug: 'zweiter-beitrag',
              excerpt: 'Kurzer Auszug',
              content_markdown: '## Details\n\nMehr Inhalt.',
              is_published: false,
              allow_comments: false,
              order_index: 2,
            },
            {
              title: 'Erster Beitrag',
              slug: 'erster-beitrag',
              excerpt: '',
              content_markdown: '# Inhalt\n\nAbsatz eins.\n\n\nAbsatz zwei.',
              is_published: true,
              allow_comments: true,
              published_at: '2026-06-21T10:00:00Z',
              order_index: 1,
            },
          ],
        },
      ],
      { generatedAt: new Date('2026-06-23T12:00:00Z') },
    )

    expect(markdown).toContain('title: "RustBlogCMS Seiten Export"')
    expect(markdown).toContain('generated_at: "2026-06-23T12:00:00.000Z"')
    expect(markdown).toContain('page_count: 1')
    expect(markdown).toContain('post_count: 2')
    expect(markdown).toContain('## Startseite')
    expect(markdown).toContain('| URL | `/pages/start` |')
    expect(markdown).toContain('| Navigation | Sichtbar |')
    expect(markdown).toContain('### Hero')
    expect(markdown).toContain('| Untertitel | Alles an einem Ort |')
    expect(markdown).toContain(
      '```json\n{\n  "postsSection": {\n    "title": "Artikel"\n  }\n}\n```',
    )
    expect(markdown.indexOf('#### Erster Beitrag')).toBeLessThan(
      markdown.indexOf('#### Zweiter Beitrag'),
    )
    expect(markdown).toContain('| Kommentare | Deaktiviert |')
    expect(markdown).toContain('# Inhalt\n\nAbsatz eins.\n\n\nAbsatz zwei.')
  })

  it('handles empty exports gracefully', () => {
    const markdown = buildPagesMarkdownExport([], {
      generatedAt: new Date('2026-06-23T12:00:00Z'),
    })

    expect(markdown).toContain('page_count: 0')
    expect(markdown).toContain('post_count: 0')
    expect(markdown).toContain('Keine Seiten vorhanden.')
  })

  it('builds stable dated markdown filenames', () => {
    expect(buildPagesMarkdownFilename(new Date('2026-06-23T12:00:00Z'))).toBe(
      'rustblogcms-seiten-export-2026-06-23.md',
    )
  })
})
