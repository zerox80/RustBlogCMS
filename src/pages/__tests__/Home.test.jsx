import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import Home from '../Home'
import { api } from '../../api/client'

vi.mock('../../api/client', () => ({
  api: {
    listPublishedPages: vi.fn(),
    getPublishedPage: vi.fn(),
  },
}))

vi.mock('../../components/dynamic-page/PostCard', () => ({
  default: ({ post }) => <article>{post.title}</article>,
}))

describe('Home', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('loads posts using the slug array returned by the public API', async () => {
    api.listPublishedPages.mockResolvedValue(['security'])
    api.getPublishedPage.mockResolvedValue({
      page: { title: 'Security' },
      posts: [{ id: 'post-1', title: 'Secure defaults', created_at: '2026-01-01T00:00:00Z' }],
    })

    render(<Home />)

    expect(await screen.findByText('Secure defaults')).toBeInTheDocument()
    expect(api.getPublishedPage).toHaveBeenCalledWith('security')
  })
})
