import { describe, it, expect, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { ThemeProvider, useTheme } from '../ThemeContext'

describe('ThemeContext', () => {
  it('provides "dark" as the default theme', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(result.current.theme).toBe('dark')
  })

  it('adds "dark" class to document.documentElement on mount', () => {
    // Mock document.documentElement.classList.add
    const addMock = vi.spyOn(document.documentElement.classList, 'add')

    renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(addMock).toHaveBeenCalledWith('dark')
    addMock.mockRestore()
  })

  it('toggleTheme does not change the theme (enforced dark mode)', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(result.current.theme).toBe('dark')

    act(() => {
      result.current.toggleTheme()
    })

    expect(result.current.theme).toBe('dark')
  })
})
