import { Routes, Route } from 'react-router-dom'
import { lazy } from 'react'
import { useContent } from './context/ContentContext'
import ErrorBoundary from './components/ui/ErrorBoundary'
import Header from './components/layout/Header'
import Footer from './components/layout/Footer'
import ProtectedRoute from './components/ProtectedRoute'
import Home from './pages/Home' // Blog view
import PostDetail from './pages/PostDetail'
import DynamicPage from './pages/DynamicPage'

const Login = lazy(() => import('./pages/Login'))
const TutorialDetail = lazy(() => import('./pages/TutorialDetail'))
const AdminDashboard = lazy(() => import('./pages/AdminDashboard'))

/**
 * Blog-Only Routing Configuration.
 * 
 * The application defaults to a Blog view. The home page can be configured via
 * CMS settings to show:
 * - 'blog': The main blog listing (default)
 * - A specific page slug: A CMS-created dynamic page
 * - A specific post: homePagePost setting to show a featured blog article
 */
const AppRoutes = () => {
    const { getSection } = useContent()
    const settings = getSection('settings') || {}
    const homePageSlug = settings.homePageSlug || 'blog'
    const homePagePost = settings.homePagePost // Optional: specific blog post as start page

    // Determine the home page component based on settings
    let HomePageComponent

    if (homePagePost) {
        // Feature: Show a specific blog post as the start page
        HomePageComponent = () => (
            <>
                <Header />
                <PostDetail pageSlug="blog" postSlug={homePagePost} isHomePage />
                <Footer />
            </>
        )
    } else if (homePageSlug === 'blog' || !homePageSlug) {
        // Default: Show the blog listing
        HomePageComponent = () => (
            <>
                <Header />
                <Home />
                <Footer />
            </>
        )
    } else {
        // Show a specific CMS page
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
                        <HomePageComponent />
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
                        <Header />
                        <div className="min-h-screen flex items-center justify-center bg-slate-900">
                            <div className="text-center">
                                <h1 className="text-6xl font-bold text-white mb-4">404</h1>
                                <p className="text-slate-400 text-lg">Seite nicht gefunden</p>
                            </div>
                        </div>
                        <Footer />
                    </ErrorBoundary>
                }
            />
        </Routes>
    )
}

export default AppRoutes
