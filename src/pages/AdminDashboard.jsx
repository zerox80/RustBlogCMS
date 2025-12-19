import { useState } from 'react'
import { RefreshCw } from 'lucide-react'
import { useContent } from '../context/ContentContext'
import SiteContentEditor from '../components/SiteContentEditor'
import PageManager from '../components/page-manager'
import AdminHeader from '../components/admin/AdminHeader'
import TutorialManagement from '../components/admin/TutorialManagement'
import DashboardTabs from '../components/admin/dashboard/DashboardTabs'
import SettingsEditor from '../components/admin/SettingsEditor'

/**
 * The central management hub for site administrators.
 * 
 * Orchestrates:
 * - Content Editing (SiteContentEditor)
 * - Page Creation & Deletion (PageManager)
 * - Tutorial/Course Management (TutorialManagement)
 * - Global Site Settings
 * 
 * Restricted: This page should only be accessible to authenticated admin users.
 */
const AdminDashboard = () => {
  const [activeTab, setActiveTab] = useState('tutorials')
  const { loading: contentLoading } = useContent()

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-slate-950 text-gray-900 dark:text-slate-100">
      <AdminHeader />
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <DashboardTabs activeTab={activeTab} onTabChange={setActiveTab} />

        {/* Tab Content: Tutorials */}
        {activeTab === 'tutorials' && <TutorialManagement />}

        {/* Tab Content: Site Content */}
        {activeTab === 'content' && (
          <div className="space-y-6">
            {contentLoading && (
              <div className="flex items-center gap-2 rounded-lg border border-gray-200 bg-gray-50 px-4 py-3 text-sm text-gray-600">
                <RefreshCw className="h-4 w-4 animate-spin" />
                Inhalte werden geladen…
              </div>
            )}
            <SiteContentEditor />
          </div>
        )}

        {/* Tab Content: Pages */}
        {activeTab === 'pages' && <PageManager />}

        {/* Tab Content: Settings */}
        {activeTab === 'settings' && <SettingsEditor />}
      </main>
    </div>
  )
}

export default AdminDashboard
