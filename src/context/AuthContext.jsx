import { createContext, useContext, useState, useEffect } from "react"
import PropTypes from "prop-types"
import { api } from "../api/client"

const AuthContext = createContext(null)

/**
 * Global Authentication Provider.
 * 
 * Manages the user's session state, including login, logout, and token validation.
 * 
 * Architecture:
 * - **Initialization**: Checks `/api/me` on mount to re-hydrate session from HTTP-only cookie.
 * - **Security**: Uses `AbortController` to prevent memory leaks during async auth checks.
 * - **Error Handling**: fails gracefully to "Unauthenticated" state on API errors (401/500).
 */
export const AuthProvider = ({ children }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(false)
  const [user, setUser] = useState(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState(null)

  useEffect(() => {
    const controller = new AbortController()

    const checkAuth = async () => {
      try {
        const userData = await api.me({ signal: controller.signal });
        setUser(userData);
        setIsAuthenticated(true);
      } catch (err) {
        // If any error occurs (401, network error, 500, etc.), we assume the user is not authenticated
        // or the session is invalid. This prevents stale "logged in" state.
        console.error('Auth check failed:', err);
        setUser(null);
        setIsAuthenticated(false);
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      }
    }

    checkAuth()

    return () => {
      controller.abort()
    }
  }, [])

  const login = async (username, password) => {
    try {
      setError(null)
      setLoading(true)

      const sanitizedUsername = username.trim()
      const response = await api.login(sanitizedUsername, password)

      if (!response?.user) {
        throw new Error('Ungueltige Antwort vom Server')
      }

      setIsAuthenticated(true)
      setUser(response.user)

      return { success: true }
    } catch (err) {
      setIsAuthenticated(false)
      setUser(null)

      const message = err.message || 'Ungueltige Anmeldedaten'
      setError(message)

      return { success: false, error: message }
    } finally {
      setLoading(false)
    }
  }

  const logout = async () => {
    try {
      await api.logout()
    } catch (err) {
      console.error('Logout failed:', err)
    } finally {
      setIsAuthenticated(false)
      setUser(null)
      setError(null)
    }
  }

  return (
    <AuthContext.Provider value={{ isAuthenticated, user, login, logout, loading, error }}>
      {children}
    </AuthContext.Provider>
  )
}

AuthProvider.propTypes = {
  children: PropTypes.node.isRequired,
}

export const useAuth = () => {
  const context = useContext(AuthContext)

  if (!context) {
    throw new Error('useAuth must be used within AuthProvider')
  }

  return context
}
