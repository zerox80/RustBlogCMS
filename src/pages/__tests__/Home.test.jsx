import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import Home from '../Home'
import { api } from '../../api/client'

const contentSections = vi.hoisted(() => ({
  hero: {
    badgeText: 'CMS badge',
    title: { line1: 'CMS title', line2: 'CMS accent' },
    subtitle: 'CMS subtitle',
    subline: 'CMS subline',
    primaryCta: { label: 'CMS CTA', target: { type: 'section', value: 'stories' } },
  },
  about: {
    eyebrow: 'CMS about',
    lead: 'CMS lead',
    paragraphs: ['CMS paragraph one', 'CMS paragraph two'],
  },
  cta_section: { title: 'CMS newsletter', description: 'CMS newsletter description' },
}))

vi.mock('../../api/client', () => ({
  api: {
    listPublishedPages: vi.fn(),
    getPublishedPage: vi.fn(),
    subscribeNewsletter: vi.fn(),
  },
}))

vi.mock('../../context/ContentContext', () => ({
  useContent: () => ({ getSection: (section) => contentSections[section] }),
}))

vi.mock('../../components/cms/EditableText', () => ({
  default: ({ value }) => <>{value}</>,
}))

vi.mock('../../components/dynamic-page/PostCard', () => ({
  default: ({ post }) => <article>{post.title}</article>,
}))

const renderHome = () =>
  render(
    <MemoryRouter>
      <Home />
    </MemoryRouter>,
  )

describe('Home', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    api.listPublishedPages.mockResolvedValue([])
  })

  it('loads posts using the slug array returned by the public API', async () => {
    api.listPublishedPages.mockResolvedValue(['security'])
    api.getPublishedPage.mockResolvedValue({
      page: { title: 'Security' },
      posts: [{ id: 'post-1', title: 'Secure defaults', created_at: '2026-01-01T00:00:00Z' }],
    })

    renderHome()

    expect(await screen.findByText('Secure defaults')).toBeInTheDocument()
    expect(api.getPublishedPage).toHaveBeenCalledWith('security')
  })

  it('renders editorial copy from the CMS', async () => {
    renderHome()

    expect(screen.getByText('CMS title')).toBeInTheDocument()
    expect(screen.getByText('CMS lead')).toBeInTheDocument()
    expect(screen.getByText('CMS newsletter description')).toBeInTheDocument()
  })

  it('subscribes an email address and confirms success', async () => {
    const user = userEvent.setup()
    api.subscribeNewsletter.mockResolvedValue({ subscribed: true })
    renderHome()

    await user.type(screen.getByLabelText('E-Mail-Adresse'), 'reader@example.com')
    await user.click(screen.getByRole('button', { name: 'Newsletter abonnieren' }))

    expect(api.subscribeNewsletter).toHaveBeenCalledWith('reader@example.com')
    expect(await screen.findByRole('status')).toHaveTextContent('Danke!')
    expect(screen.getByLabelText('E-Mail-Adresse')).toHaveValue('')
  })
})
