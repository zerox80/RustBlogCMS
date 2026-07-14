import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { useContent } from '../context/ContentContext'
import SiteContentEditor from '../components/SiteContentEditor'
import AdminHeader from '../components/admin/AdminHeader'
import BlogPostManager from '../components/admin/BlogPostManager'
import DashboardTabs from '../components/admin/dashboard/DashboardTabs'

/**
 * The central management hub for site administrators.
 *
 * Orchestrates the unified post list and the editable one-page content.
 *
 * Restricted: This page should only be accessible to authenticated admin users.
 */
const AdminDashboard = () => {
  const [activeTab, setActiveTab] = useState('posts')
  const { loading: contentLoading } = useContent()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-slate-950 text-gray-900 dark:text-slate-100">
      <AdminHeader />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <DashboardTabs activeTab={activeTab} onTabChange={setActiveTab} />

        {activeTab === 'posts' && <BlogPostManager />}

        {/* Tab Content: Site Content */}
        {activeTab === 'content' && (
          <div className="space-y-6">
            {contentLoading && (
              <div
                className={`flex items-center gap-2 rounded-lg border border-gray-200 bg-gray-50 px-4
py-3 text-sm text-gray-600`}
              >
                <RefreshCw className="h-4 w-4 animate-spin" />
                Inhalte werden geladen…
              </div>
            )}
            <SiteContentEditor />
          </div>
        )}
      </main>
    </div>
  )
}

export default AdminDashboard
