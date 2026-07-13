import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'
import ThemeToggle from '../ui/ThemeToggle'
import { ThemeProvider } from '../../context/ThemeContext'

describe('ThemeToggle', () => {
  it('renders nothing (null) as dark mode is enforced', () => {
    const { container } = render(
      <ThemeProvider>
        <ThemeToggle />
      </ThemeProvider>,
    )
    expect(container).toBeEmptyDOMElement()
  })
})
