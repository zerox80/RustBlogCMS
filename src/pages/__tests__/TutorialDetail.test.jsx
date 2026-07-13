import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import TutorialDetail from '../TutorialDetail'
import { api } from '../../api/client'

vi.mock('../../api/client', () => ({
  api: {
    getTutorial: vi.fn(),
  },
}))

vi.mock('../../components/tutorial/TutorialHeader', () => ({
  default: ({ title }) => <h1>{title}</h1>,
}))

vi.mock('../../components/tutorial/TutorialTopicsList', () => ({
  default: () => null,
}))

vi.mock('../../components/tutorial/TutorialContentDisplay', () => ({
  default: ({ content }) => <div data-testid="tutorial-content">{content}</div>,
}))

const renderDetail = () =>
  render(
    <MemoryRouter initialEntries={['/tutorials/linux-basics']}>
      <Routes>
        <Route path="/tutorials/:id" element={<TutorialDetail />} />
      </Routes>
    </MemoryRouter>,
  )

describe('TutorialDetail', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('keeps the complete detail response instead of relying on list summaries', async () => {
    api.getTutorial.mockResolvedValue({
      id: 'linux-basics',
      title: 'Linux Basics',
      description: 'Learn the command line.',
      topics: ['Shell'],
      content: 'The complete tutorial body.',
    })

    const view = renderDetail()

    expect(await screen.findByTestId('tutorial-content')).toHaveTextContent(
      'The complete tutorial body.',
    )

    // A provider/list refresh only contains summary records. Re-rendering the
    // detail view must not replace its canonical API response with one.
    view.rerender(
      <MemoryRouter initialEntries={['/tutorials/linux-basics']}>
        <Routes>
          <Route path="/tutorials/:id" element={<TutorialDetail />} />
        </Routes>
      </MemoryRouter>,
    )

    expect(screen.getByTestId('tutorial-content')).toHaveTextContent('The complete tutorial body.')
  })
})
