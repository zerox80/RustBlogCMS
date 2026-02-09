import { BrowserRouter as Router } from 'react-router-dom'
import { Suspense } from 'react'
import { HelmetProvider } from 'react-helmet-async'
import { AuthProvider } from './context/AuthContext'
import { ContentProvider } from './context/ContentContext'
import { EditProvider } from './context/EditContext'
import { TutorialProvider } from './context/TutorialContext'
import { ThemeProvider } from './context/ThemeContext'
import ErrorBoundary from './components/ui/ErrorBoundary'
import GlobalSiteMeta from './components/GlobalSiteMeta'
import AppRoutes from './AppRoutes'

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
                <EditProvider>
                  <TutorialProvider>
                    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 transition-colors">
                      <GlobalSiteMeta />
                      <Suspense fallback={<LoadingSpinner />}>
                        <AppRoutes />
                      </Suspense>
                    </div>
                  </TutorialProvider>
                </EditProvider>
              </ContentProvider>
            </AuthProvider>
          </Router>
        </ThemeProvider>
      </HelmetProvider>
    </ErrorBoundary>
  )
}
export default App
