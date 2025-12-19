import { createContext, useContext, useState, useMemo } from 'react'
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
    const { isAuthenticated } = useAuth()
    const [isEditing, setIsEditing] = useState(false)

    /**
     * Toggles the interactive editing UI.
     * Prevents enabling if not logged in.
     */
    const toggleEditMode = () => {
        if (!isAuthenticated && !isEditing) {
            console.warn('CMS: Cannot enable edit mode: User not authenticated')
            return
        }
        setIsEditing((prev) => !prev)
    }

    // Security Lifecycle: Auto-disable edit mode if user session ends
    useMemo(() => {
        if (!isAuthenticated) {
            setIsEditing(false)
        }
    }, [isAuthenticated])

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
