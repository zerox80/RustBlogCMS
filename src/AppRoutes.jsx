import { Navigate, Route, Routes } from 'react-router-dom'
import { lazy } from 'react'
import PropTypes from 'prop-types'
import ErrorBoundary from './components/ui/ErrorBoundary'
import Header from './components/layout/Header'
import Footer from './components/layout/Footer'
import ProtectedRoute from './components/ProtectedRoute'
import Home from './pages/Home'
import PostDetail from './pages/PostDetail'

const Login = lazy(() => import('./pages/Login'))
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'))

const PublicLayout = ({ children }) => (
  <ErrorBoundary>
    <Header />
    {children}
    <Footer />
  </ErrorBoundary>
)

PublicLayout.propTypes = {
  children: PropTypes.node.isRequired,
}

/** The public website is one personal blog, not a collection of CMS pages. */
const AppRoutes = () => {
  return (
    <Routes>
      <Route
        path="/"
        element={
          <PublicLayout>
            <Home />
          </PublicLayout>
        }
      />
      <Route path="/blog" element={<Navigate to="/" replace />} />
      <Route
        path="/posts/:pageSlug/:postSlug"
        element={
          <PublicLayout>
            <PostDetail />
          </PublicLayout>
        }
      />
      {/* Keep old article URLs working without exposing public CMS page views. */}
      <Route
        path="/pages/:pageSlug/posts/:postSlug"
        element={
          <PublicLayout>
            <PostDetail />
          </PublicLayout>
        }
      />
      <Route path="/pages/:slug" element={<Navigate to="/#stories" replace />} />
      <Route path="/tutorials/:id" element={<Navigate to="/#stories" replace />} />
      <Route
        path="/login"
        element={
          <ErrorBoundary>
            <Login />
          </ErrorBoundary>
        }
      />
      <Route
        path="/admin"
        element={
          <ProtectedRoute>
            <ErrorBoundary>
              <AdminDashboard />
            </ErrorBoundary>
          </ProtectedRoute>
        }
      />
      <Route
        path="*"
        element={
          <PublicLayout>
            <div className="grid min-h-[75vh] place-items-center bg-[#f4f1ea] px-6 pt-24">
              <div className="text-center text-[#171713]">
                <p className="font-serif text-8xl italic text-[#ff4f00]">404</p>
                <h1 className="mt-3 text-3xl font-semibold">Hier gibt es nichts zu lesen.</h1>
                <a
                  href="/#stories"
                  className={`mt-8 inline-flex rounded-full bg-[#171713] px-6 py-3 text-sm
font-bold uppercase tracking-[0.12em] text-white`}
                >
                  Zurück zu den Beiträgen
                </a>
              </div>
            </div>
          </PublicLayout>
        }
      />
    </Routes>
  )
}

export default AppRoutes
