import { Component } from 'react'
import PropTypes from 'prop-types'
import { AlertCircle, RefreshCw } from 'lucide-react'

/**
 * A robust React Error Boundary to prevent application-wide crashes.
 * 
 * Features:
 * - Granular error catching and logging.
 * - Recovery UI with specific error details for development.
 * - "Reset and Return Home" logic that fixes the browser state after a crash.
 */
class ErrorBoundary extends Component {
  constructor(props) {
    super(props)
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    }
  }
  static getDerivedStateFromError(_error) {
    // Update state to show fallback UI on next render
    return { hasError: true }
  }
  componentDidCatch(error, errorInfo) {
    // Log detailed error information for debugging
    console.error('CMS: ErrorBoundary caught an error:', error, errorInfo)
    // Store error information in state for potential display
    this.setState({
      error: error,
      errorInfo: errorInfo,
    })
  }
  handleReset = () => {
    // Reset error state to allow re-rendering of children
    this.setState({ hasError: false, error: null, errorInfo: null })
    // Navigate to home page if browser APIs are available
    if (typeof window !== 'undefined' && window.history) {
      try {
        // Update URL without page reload
        window.history.pushState({}, '', '/')
        // Trigger React Router's navigation listener
        window.dispatchEvent(new PopStateEvent('popstate'))
      } catch (navError) {
        console.warn('CMS: Navigation during error reset failed:', navError)
      }
    }
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 flex items-center justify-center p-4">
          <div className="max-w-md w-full bg-white rounded-2xl shadow-2xl p-8">
            {/* Error icon with visual styling */}
            <div className="flex items-center justify-center w-16 h-16 bg-red-100 rounded-full mx-auto mb-4">
              <AlertCircle className="w-8 h-8 text-red-600" />
            </div>
            {/* Error title and description */}
            <h1 className="text-2xl font-bold text-gray-800 text-center mb-2">
              Oops! Etwas ist schiefgelaufen
            </h1>
            <p className="text-gray-600 text-center mb-6">
              Die Anwendung ist auf einen unerwarteten Fehler gestoßen.
              Bitte versuchen Sie es erneut oder kontaktieren Sie den Support.
            </p>
            {/* Technical error details (development mode) */}
            {this.state.error && (
              <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                <p className="text-sm font-mono text-gray-700 break-all">
                  {this.state.error.toString()}
                </p>
              </div>
            )}
            {/* Recovery button with interactive styling */}
            <button
              onClick={this.handleReset}
              className="w-full flex items-center justify-center space-x-2 px-6 py-3 bg-gradient-to-r from-primary-600 to-primary-700 text-white rounded-lg hover:from-primary-700 hover:to-primary-800 transition-all duration-200 shadow-lg hover:shadow-xl"
            >
              <RefreshCw className="w-5 h-5" />
              <span>Zur Startseite</span>
            </button>
          </div>
        </div>
      )
    }
    // No error - render children normally
    return this.props.children
  }
}
ErrorBoundary.propTypes = {
  children: PropTypes.node.isRequired,
}
ErrorBoundary.defaultProps = {
  children: null,
}
export default ErrorBoundary
