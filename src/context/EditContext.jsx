import { createContext, useContext, useState, useMemo } from 'react'
import PropTypes from 'prop-types'
import { useAuth } from './AuthContext'

const EditContext = createContext(null)

export const EditProvider = ({ children }) => {
    const { isAuthenticated } = useAuth()
    const [isEditing, setIsEditing] = useState(false)

    const toggleEditMode = () => {
        if (!isAuthenticated && !isEditing) {
            console.warn('Cannot enable edit mode: User not authenticated')
            return
        }
        setIsEditing((prev) => !prev)
    }

    // Auto-disable edit mode if user logs out
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
