import React, { useState, useEffect } from 'react'
import { useEdit } from '../../context/EditContext'
import { useContent } from '../../context/ContentContext'
import { Image } from 'lucide-react'

/**
 * A CMS-integrated component for inline image editing.
 * 
 * Displays an image with a hover overlay when site editing is enabled.
 * Standard implementation uses a simple URL prompt, but is designed to be
 * extended with a full media picker.
 * 
 * @param {Object} props
 * @param {string} props.section - The CMS section containing this image data.
 * @param {string} props.field - Path to the image URL field within the section.
 * @param {string} props.src - Initial image source URL.
 * @param {string} props.alt - Accessibility alt text.
 * @param {string} [props.className=''] - CSS classes for the image element itself.
 * @param {string} [props.containerClassName=''] - CSS classes for the wrapper div.
 */
const EditableImage = ({
    section,
    field,
    src,
    alt,
    className = '',
    containerClassName = '',
    ...props
}) => {
    const { isEditing } = useEdit()
    const { updateSection, getSection } = useContent()
    const [currentSrc, setCurrentSrc] = useState(src)

    useEffect(() => {
        setCurrentSrc(src)
    }, [src])

    /**
     * Triggers the image update flow.
     * 
     * Handles deep cloning and traversal logic identical to EditableText,
     * ensuring that image paths at any depth are correctly updated and synced.
     */
    const handleEdit = async () => {
        // UI Interaction: Request new image URL from the user
        const newUrl = window.prompt("Enter new image URL:", currentSrc)

        if (newUrl && newUrl !== currentSrc) {
            // Optimistic update of the local image source
            setCurrentSrc(newUrl)

            try {
                // Step 1: Deep clone the section state
                const sectionData = getSection(section)
                const updatedSection = JSON.parse(JSON.stringify(sectionData))

                // Step 2: Traverse keys to reach the specific image field
                const keys = field.split('.')
                let target = updatedSection
                for (let i = 0; i < keys.length - 1; i++) {
                    if (!target[keys[i]]) target[keys[i]] = {}
                    target = target[keys[i]]
                }

                // Step 3: Set new URL at the target path
                target[keys[keys.length - 1]] = newUrl

                // Step 4: Commit changes to global state and backend
                await updateSection(section, updatedSection)

            } catch (err) {
                console.error('CMS: Failed to update image', err)
                alert('Failed to save image update. Please check console.')
                // Revert UI to previous state on error
                setCurrentSrc(src)
            }
        }
    }

    return (
        <div className={`relative group ${containerClassName}`}>
            <img
                src={currentSrc}
                alt={alt}
                className={className}
                {...props}
            />
            {isEditing && (
                <button
                    onClick={(e) => {
                        e.preventDefault()
                        e.stopPropagation()
                        handleEdit()
                    }}
                    className="absolute inset-0 bg-black/50 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer border-2 border-neon-cyan"
                >
                    <div className="bg-slate-900 p-2 rounded-full text-neon-cyan">
                        <Image className="w-6 h-6" />
                    </div>
                </button>
            )}
        </div>
    )
}

export default EditableImage
