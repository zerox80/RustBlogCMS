import { Navigate } from 'react-router-dom'
import PropTypes from 'prop-types'
import { useAuth } from '../context/AuthContext'
const ProtectedRoute = ({ children }) => {
  const { isAuthenticated, loading } = useAuth()
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div
            className={`inline-block animate-spin rounded-full h-12 w-12 border-b-2
border-primary-600`}
          ></div>
          <p className="mt-4 text-gray-600">Laden...</p>
        </div>
      </div>
    )
  }
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  return children
}
ProtectedRoute.propTypes = {
  children: PropTypes.node.isRequired,
}
export default ProtectedRoute
