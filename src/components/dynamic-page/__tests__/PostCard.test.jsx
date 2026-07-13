import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it } from 'vitest'
import PostCard from '../PostCard'

describe('PostCard', () => {
  it('calculates reading time from the published Markdown body', () => {
    const content = Array.from({ length: 600 }, (_, index) => `word${index}`).join(' ')

    render(
      <MemoryRouter>
        <PostCard
          post={{
            id: 'post-1',
            slug: 'reading-time',
            title: 'Reading time',
            excerpt: 'Short preview',
            content_markdown: content,
          }}
          pageSlug="notes"
        />
      </MemoryRouter>,
    )

    expect(screen.getByText(/3 Min\. Lesezeit/)).toBeInTheDocument()
  })
})
