import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { Suspense, lazy } from 'react'
import { HelmetProvider } from 'react-helmet-async'
import { AuthProvider } from './context/AuthContext'
import { ContentProvider } from './context/ContentContext'
import { TutorialProvider } from './context/TutorialContext'
import { ThemeProvider } from './context/ThemeContext'
import ErrorBoundary from './components/ui/ErrorBoundary'
import Header from './components/layout/Header'
import Footer from './components/layout/Footer'
import ProtectedRoute from './components/ProtectedRoute'
import GlobalSiteMeta from './components/GlobalSiteMeta'
import Home from './pages/Home' // Landing/home page
import LandingPage from './pages/LandingPage' // New IT Landing Page
import PostDetail from './pages/PostDetail' // Individual blog post view

const Login = lazy(() => import('./pages/Login')) // User authentication page
const TutorialDetail = lazy(() => import('./pages/TutorialDetail')) // Individual tutorial view
const DynamicPage = lazy(() => import('./pages/DynamicPage')) // CMS-driven dynamic pages
const AdminDashboard = lazy(() => import('./pages/AdminDashboard')) // Admin control panel

const LoadingSpinner = () => (
  <div className="flex items-center justify-center min-h-[50vh]">
    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
  </div>
)

function App() {
  return (
    <ErrorBoundary>
      <HelmetProvider>
        <ThemeProvider>
          <Router>
            <AuthProvider>
              <ContentProvider>
                <TutorialProvider>
                  <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 transition-colors">
                    <GlobalSiteMeta />
                    <Suspense fallback={<LoadingSpinner />}>
                      <Routes>
                        <Route
                          path="/"
                          element={
                            <ErrorBoundary>
                              <LandingPage />
                            </ErrorBoundary>
                          }
                        />
                        <Route
                          path="/blog"
                          element={
                            <ErrorBoundary>
                              <Header />
                              <Home />
                              <Footer />
                            </ErrorBoundary>
                          }
                        />
                        <Route
                          path="/tutorials/:id"
                          element={
                            <ErrorBoundary>
                              <Header />
                              <TutorialDetail />
                              <Footer />
                            </ErrorBoundary>
                          }
                        />
                        <Route
                          path="/pages/:pageSlug/posts/:postSlug"
                          element={
                            <ErrorBoundary>
                              <Header />
                              <PostDetail />
                              <Footer />
                            </ErrorBoundary>
                          }
                        />
                        <Route
                          path="/pages/:slug"
                          element={
                            <ErrorBoundary>
                              <Header />
                              <DynamicPage />
                              <Footer />
                            </ErrorBoundary>
                          }
                        />
                        <Route path="/login" element={<ErrorBoundary><Login /></ErrorBoundary>} />
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
                            <ErrorBoundary>
                              <LandingPage />
                            </ErrorBoundary>
                          }
                        />
                      </Routes>
                    </Suspense>
                  </div>
                </TutorialProvider>
              </ContentProvider>
            </AuthProvider>
          </Router>
        </ThemeProvider>
      </HelmetProvider>
    </ErrorBoundary>
  )
}
export default App
