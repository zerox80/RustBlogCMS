import React from 'react'
import { useNavigate } from 'react-router-dom'
import { Terminal, Home, LogOut } from 'lucide-react'
import { useAuth } from '../../context/AuthContext'

const AdminHeader = () => {
  const { user, logout } = useAuth()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
    navigate('/login')
  }

  return (
    <header className="bg-gray-900 shadow-md border-b border-gray-800">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
        <div className="flex justify-between items-center">
          {/* Left Side: Logo & Title */}
          <div className="flex items-center space-x-3">
            {/* Logo Icon */}
            <div
              className={`bg-gradient-to-r from-primary-600 to-primary-800 p-2 rounded-lg shadow-lg
shadow-primary-900/20`}
            >
              <Terminal className="w-6 h-6 text-white" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-slate-100">
                Admin Dashboard
              </h1>
              <p className="text-sm text-gray-600 dark:text-slate-400">
                Willkommen, {user?.username}
              </p>
            </div>
          </div>

          {/* Right Side: Actions */}
          <div className="flex space-x-3">
            {/* Home Button */}
            <button
              onClick={() => navigate('/')}
              className={`flex items-center space-x-2 px-4 py-2 bg-gray-100 text-gray-700 rounded-lg
hover:bg-gray-200 transition-colors duration-200 dark:bg-slate-800
dark:text-slate-200 dark:hover:bg-slate-700`}
            >
              <Home className="w-4 h-4" />
              <span>Startseite</span>
            </button>

            {/* Logout Button */}
            <button
              onClick={handleLogout}
              className={`flex items-center space-x-2 px-4 py-2 bg-red-100 text-red-700 rounded-lg
hover:bg-red-200 transition-colors duration-200 dark:bg-red-900/30
dark:text-red-300 dark:hover:bg-red-900/50`}
            >
              <LogOut className="w-4 h-4" />
              <span>Abmelden</span>
            </button>
          </div>
        </div>
      </div>
    </header>
  )
}

export default AdminHeader
