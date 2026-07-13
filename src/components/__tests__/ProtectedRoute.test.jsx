import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import ProtectedRoute from '../ProtectedRoute'
import { useAuth } from '../../context/AuthContext'

// Mock useAuth
vi.mock('../../context/AuthContext', () => ({
  useAuth: vi.fn(),
}))

describe('ProtectedRoute', () => {
  it('renders loading state when loading is true', () => {
    useAuth.mockReturnValue({ loading: true, isAuthenticated: false })

    render(
      <MemoryRouter>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </MemoryRouter>,
    )

    expect(screen.getByText('Laden...')).toBeInTheDocument()
  })

  it('renders children when authenticated', () => {
    useAuth.mockReturnValue({ loading: false, isAuthenticated: true })

    render(
      <MemoryRouter>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </MemoryRouter>,
    )

    expect(screen.getByText('Protected Content')).toBeInTheDocument()
  })

  it('redirects to /login when not authenticated', () => {
    useAuth.mockReturnValue({ loading: false, isAuthenticated: false })

    render(
      <MemoryRouter initialEntries={['/protected']}>
        <Routes>
          <Route
            path="/protected"
            element={
              <ProtectedRoute>
                <div>Protected Content</div>
              </ProtectedRoute>
            }
          />
          <Route path="/login" element={<div>Login Page</div>} />
        </Routes>
      </MemoryRouter>,
    )

    expect(screen.getByText('Login Page')).toBeInTheDocument()
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument()
  })
})
