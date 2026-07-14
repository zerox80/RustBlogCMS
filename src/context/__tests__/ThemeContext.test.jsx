import { describe, it, expect, vi } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { ThemeProvider, useTheme } from '../ThemeContext'

describe('ThemeContext', () => {
  it('provides "light" as the default theme', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(result.current.theme).toBe('light')
  })

  it('removes a stale "dark" class from document.documentElement on mount', () => {
    document.documentElement.classList.add('dark')
    const removeMock = vi.spyOn(document.documentElement.classList, 'remove')

    renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(removeMock).toHaveBeenCalledWith('dark')
    expect(document.documentElement).not.toHaveClass('dark')
    removeMock.mockRestore()
  })

  it('toggleTheme keeps the editorial light theme', () => {
    const { result } = renderHook(() => useTheme(), {
      wrapper: ThemeProvider,
    })

    expect(result.current.theme).toBe('light')

    act(() => {
      result.current.toggleTheme()
    })

    expect(result.current.theme).toBe('light')
  })
})
