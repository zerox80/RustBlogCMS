import { createContext, useContext, useState } from 'react'
import PropTypes from 'prop-types'
import { useAuth } from './AuthContext'

const EditContext = createContext(null)

/**
 * Manages the global editing state for the CMS.
 * 
 * Secure by design: Edit mode cannot be enabled unless the user is authenticated.
 * It also handles automatic cleanup (disabling edit mode) upon logout.
 */
export const EditProvider = ({ children }) => {
    const { isAuthenticated, user } = useAuth()
    const activeSession = isAuthenticated ? user : null
    const [editingSession, setEditingSession] = useState(null)
    const isEditing = Boolean(activeSession && editingSession === activeSession)

    /**
     * Toggles the interactive editing UI.
     * Prevents enabling if not logged in.
     */
    const toggleEditMode = () => {
        if (!activeSession) {
            console.warn('CMS: Cannot enable edit mode: User not authenticated')
            return
        }
        setEditingSession((currentSession) => (
            currentSession === activeSession ? null : activeSession
        ))
    }

    return (
        <EditContext.Provider value={{ isEditing, toggleEditMode }}>
            {children}
        </EditContext.Provider>
    )
}

EditProvider.propTypes = {
    children: PropTypes.node,
}

export const useEdit = () => {
    const context = useContext(EditContext)
    if (!context) {
        throw new Error('useEdit must be used within an EditProvider')
    }
    return context
}
