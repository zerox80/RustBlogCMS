import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'
import { useContent } from '../context/ContentContext'
import { getIconComponent } from '../utils/iconMap'
import { Terminal, Lock, User, AlertCircle, ArrowRight } from 'lucide-react'

/**
 * Secure Admin Login Page.
 * 
 * Security Features:
 * - Client-side progressive cooldown (10s after 3 failures, 60s after 5).
 * - Regex-based username validation to prevent injection attempts.
 * - Automatic redirection to `/admin` upon successful JWT acquisition.
 * - Animated glassmorphism UI with responsive design.
 */
const Login = () => {
  const { getSection } = useContent()
  const loginContent = getSection('login') || {}
  const IconComponent = getIconComponent(loginContent.icon, 'Terminal')
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [loginAttempts, setLoginAttempts] = useState(0)
  const [cooldownUntil, setCooldownUntil] = useState(null)
  const { login } = useAuth()
  const navigate = useNavigate()

  useEffect(() => {
    if (!cooldownUntil) {
      return
    }
    const intervalId = window.setInterval(() => {
      if (Date.now() >= cooldownUntil) {
        window.clearInterval(intervalId)
        setCooldownUntil(null)
        setError('')
      }
    }, 500)
    return () => {
      window.clearInterval(intervalId)
    }
  }, [cooldownUntil])

  /**
   * Processes the login submission with security checks.
   * 
   * Logic:
   * 1. Check client-side cooldown.
   * 2. Validate input patterns (Username regex, Password length).
   * 3. Send credentials to AuthContext.
   * 4. Handle failure with progressive penalty (cooldown levels).
   */
  const handleSubmit = async (e) => {
    e.preventDefault()
    const now = Date.now()

    // Enforcement: Reject if in cooldown period
    if (cooldownUntil && now < cooldownUntil) {
      const remainingSeconds = Math.ceil((cooldownUntil - now) / 1000)
      setError(`Zu viele Anmeldeversuche. Bitte warte ${remainingSeconds} Sekunde${remainingSeconds === 1 ? '' : 'n'}.`)
      return
    }

    if (isSubmitting) {
      return
    }

    // Input Sanitization: Strict allow-list for username characters
    const trimmedUsername = username.trim()
    if (!/^[a-zA-Z0-9_.-]{1,50}$/.test(trimmedUsername)) {
      setError('Benutzername darf nur Buchstaben, Zahlen sowie _ . - enthalten und max. 50 Zeichen lang sein.')
      return
    }

    // Length Check: Basic password validation before hitting server
    if (password.length === 0) {
      setError('Passwort darf nicht leer sein.')
      return
    }
    if (password.length > 128) {
      setError('Passwort darf maximal 128 Zeichen lang sein.')
      return
    }

    setError('')
    setIsSubmitting(true)

    try {
      const result = await login(trimmedUsername, password)
      if (result.success) {
        // Clear penalties on success
        setLoginAttempts(0)
        setCooldownUntil(null)
        navigate('/admin')
      } else {
        // Multi-tier Cooldown Logic:
        // Level 1: 3 attempts -> 10 seconds
        // Level 2: 5 attempts -> 60 seconds
        const nextAttempts = loginAttempts + 1
        setLoginAttempts(nextAttempts)
        setError(result.error)

        if (nextAttempts >= 5) {
          setCooldownUntil(now + 60000)
          setError('Zu viele fehlgeschlagene Versuche. Bitte warte 60 Sekunden.')
        } else if (nextAttempts >= 3) {
          setCooldownUntil(now + 10000)
          setError('Zu viele fehlgeschlagene Versuche. Bitte warte 10 Sekunden.')
        }
      }
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <div className="min-h-screen w-full flex items-center justify-center relative overflow-hidden bg-surface-950">
      {/* Animated Background Elements */}
      <div className="absolute inset-0 w-full h-full">
        <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] rounded-full bg-primary-600/20 blur-[120px] animate-float" />
        <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] rounded-full bg-accent-violet/20 blur-[120px] animate-float" style={{ animationDelay: '-3s' }} />
        <div className="absolute top-[20%] right-[20%] w-[20%] h-[20%] rounded-full bg-accent-cyan/20 blur-[80px] animate-pulse-slow" />
      </div>

      {/* Main Card */}
      <div className="relative w-full max-w-md p-4 animate-scale-in">
        <div className="relative bg-surface-900/60 backdrop-blur-xl border border-white/10 rounded-3xl shadow-card-xl overflow-hidden">

          {/* Decorative top line */}
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-primary-500 to-transparent opacity-50" />

          <div className="p-8 sm:p-10">
            {/* Header */}
            <div className="text-center mb-10">
              <div className="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-br from-primary-500/20 to-accent-violet/20 rounded-2xl mb-6 ring-1 ring-white/10 shadow-inner-glow animate-float">
                <IconComponent className="w-10 h-10 text-primary-400 drop-shadow-neon" />
              </div>
              <h1 className="text-3xl font-bold text-white mb-3 tracking-tight">
                {loginContent.title || 'Rust Blog'}
              </h1>
              <p className="text-surface-300 text-sm font-medium">
                {loginContent.subtitle || 'Admin Login'}
              </p>
            </div>

            {/* Error Message */}
            {error && (
              <div className="mb-6 p-4 bg-red-500/10 border border-red-500/20 rounded-xl flex items-start space-x-3 animate-slide-down">
                <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
                <p className="text-red-200 text-sm leading-relaxed">{error}</p>
              </div>
            )}

            {/* Form */}
            <form onSubmit={handleSubmit} className="space-y-6">
              <div className="space-y-2">
                <label className="block text-xs font-semibold text-surface-300 uppercase tracking-wider ml-1">
                  {loginContent.usernameLabel || 'Benutzername'}
                </label>
                <div className="relative group">
                  <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none transition-colors group-focus-within:text-primary-400 text-surface-400">
                    <User className="h-5 w-5" />
                  </div>
                  <input
                    type="text"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    className="block w-full pl-11 pr-4 py-3.5 bg-surface-800/50 border border-surface-700 rounded-xl focus:ring-2 focus:ring-primary-500/50 focus:border-primary-500 text-white placeholder-surface-500 transition-all duration-200 outline-none hover:bg-surface-800/80"
                    placeholder="admin"
                    required
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="block text-xs font-semibold text-surface-300 uppercase tracking-wider ml-1">
                  {loginContent.passwordLabel || 'Passwort'}
                </label>
                <div className="relative group">
                  <div className="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none transition-colors group-focus-within:text-primary-400 text-surface-400">
                    <Lock className="h-5 w-5" />
                  </div>
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="block w-full pl-11 pr-4 py-3.5 bg-surface-800/50 border border-surface-700 rounded-xl focus:ring-2 focus:ring-primary-500/50 focus:border-primary-500 text-white placeholder-surface-500 transition-all duration-200 outline-none hover:bg-surface-800/80"
                    placeholder="••••••••"
                    required
                  />
                </div>
              </div>

              <button
                type="submit"
                disabled={isSubmitting || (cooldownUntil && Date.now() < cooldownUntil)}
                className="group relative w-full flex items-center justify-center py-3.5 px-4 border border-transparent rounded-xl text-white font-semibold bg-gradient-to-r from-primary-600 to-accent-violet hover:from-primary-500 hover:to-accent-violet/90 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500 focus:ring-offset-surface-900 transition-all duration-200 shadow-lg hover:shadow-primary-500/25 disabled:opacity-50 disabled:cursor-not-allowed overflow-hidden"
              >
                <span className="relative z-10 flex items-center gap-2">
                  {isSubmitting ? 'Anmelden...' : (loginContent.buttonLabel || 'Anmelden')}
                  {!isSubmitting && <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />}
                </span>
                {/* Button Shine Effect */}
                <div className="absolute inset-0 -translate-x-full group-hover:animate-shimmer bg-gradient-to-r from-transparent via-white/20 to-transparent z-0" />
              </button>
            </form>
          </div>
        </div>

        {/* Footer Link */}
        <div className="text-center mt-8 animate-fade-in" style={{ animationDelay: '0.2s' }}>
          <button
            onClick={() => navigate('/')}
            className="text-surface-400 hover:text-white text-sm font-medium transition-colors duration-200 flex items-center justify-center gap-2 mx-auto group"
          >
            <span className="group-hover:-translate-x-1 transition-transform">←</span>
            {loginContent.backLinkText || 'Zurück zur Startseite'}
          </button>
        </div>
      </div>
    </div>
  )
}

export default Login
