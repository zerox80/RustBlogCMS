import { Routes, Route } from 'react-router-dom'
import { lazy } from 'react'
import { useContent } from './context/ContentContext'
import ErrorBoundary from './components/ui/ErrorBoundary'
import Header from './components/layout/Header'
import Footer from './components/layout/Footer'
import ProtectedRoute from './components/ProtectedRoute'
import Home from './pages/Home' // Blog view
import LandingPage from './pages/LandingPage' // Default Landing Page
import PostDetail from './pages/PostDetail'
import DynamicPage from './pages/DynamicPage'

const Login = lazy(() => import('./pages/Login'))
const TutorialDetail = lazy(() => import('./pages/TutorialDetail'))
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'))

const AppRoutes = () => {
    const { contentType, getSection } = useContent()
    const settings = getSection('settings') || {}
    const homePageSlug = settings.homePageSlug

    let HomePageComponent = LandingPage
    let homePageProps = {}

    if (homePageSlug === 'blog') {
        HomePageComponent = () => (
            <>
                <Header />
                <Home />
                <Footer />
            </>
        )
    } else if (homePageSlug) {
        HomePageComponent = () => (
            <>
                <Header />
                <DynamicPage slug={homePageSlug} />
                <Footer />
            </>
        )
    }

    return (
        <Routes>
            <Route
                path="/"
                element={
                    <ErrorBoundary>
                        <HomePageComponent {...homePageProps} />
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
    )
}

export default AppRoutes
